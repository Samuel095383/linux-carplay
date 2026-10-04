use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use thiserror::Error;

use crate::config::TransportMode;
use crate::transport::UsbDeviceInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySet {
    pub video_h264: bool,
    pub audio_aac: bool,
    pub audio_pcm: bool,
    pub microphone_uplink: bool,
    pub input_events: bool,
    pub wireless: bool,
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self {
            video_h264: true,
            audio_aac: true,
            audio_pcm: true,
            microphone_uplink: true,
            input_events: true,
            wireless: true,
        }
    }
}

impl CapabilitySet {
    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            video_h264: self.video_h264 && other.video_h264,
            audio_aac: self.audio_aac && other.audio_aac,
            audio_pcm: self.audio_pcm && other.audio_pcm,
            microphone_uplink: self.microphone_uplink && other.microphone_uplink,
            input_events: self.input_events && other.input_events,
            wireless: self.wireless && other.wireless,
        }
    }

    pub fn is_minimum_supported(&self) -> bool {
        self.video_h264 && self.audio_pcm && self.input_events
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarPlayChannel {
    Control,
    Video,
    Audio,
    Input,
    VoiceUplink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationRequest {
    pub local_capabilities: CapabilitySet,
    pub requested_transport: TransportMode,
    pub protocol_version: u16,
    pub device: UsbDeviceInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationResult {
    pub session_id: String,
    pub transport: TransportMode,
    pub capabilities: CapabilitySet,
    pub channels: Vec<CarPlayChannel>,
}

pub trait CarPlayNegotiator {
    fn negotiate(&self, request: NegotiationRequest) -> Result<NegotiationResult, ProtocolError>;
}

#[derive(Debug, Default)]
pub struct BasicNegotiator;

impl CarPlayNegotiator for BasicNegotiator {
    fn negotiate(&self, request: NegotiationRequest) -> Result<NegotiationResult, ProtocolError> {
        if request.protocol_version < 2 {
            return Err(ProtocolError::VersionUnsupported(request.protocol_version));
        }

        if request.device.vendor_id != 0x05ac {
            return Err(ProtocolError::UnsupportedDevice {
                vendor_id: request.device.vendor_id,
                product_id: request.device.product_id,
            });
        }

        let phone_caps = CapabilitySet::default();
        let capabilities = request.local_capabilities.intersect(&phone_caps);
        if !capabilities.is_minimum_supported() {
            return Err(ProtocolError::IncompatibleCapabilities);
        }

        let mut channels = vec![
            CarPlayChannel::Control,
            CarPlayChannel::Video,
            CarPlayChannel::Audio,
            CarPlayChannel::Input,
        ];
        if capabilities.microphone_uplink {
            channels.push(CarPlayChannel::VoiceUplink);
        }

        let transport = match request.requested_transport {
            TransportMode::Auto => TransportMode::Wired,
            TransportMode::Wireless if capabilities.wireless => TransportMode::Wireless,
            TransportMode::Wireless => return Err(ProtocolError::WirelessNotSupported),
            TransportMode::Wired => TransportMode::Wired,
        };

        Ok(NegotiationResult {
            session_id: format!(
                "{:04x}:{:04x}:{:02x}:{:02x}",
                request.device.vendor_id,
                request.device.product_id,
                request.device.bus_number.unwrap_or_default(),
                request.device.address.unwrap_or_default()
            ),
            transport,
            capabilities,
            channels,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthChallenge {
    pub session_id: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthProof {
    pub digest: u64,
}

pub trait CarPlayAuthenticator {
    fn issue_challenge(&self, session_id: &str) -> AuthChallenge;
    fn prove(&self, challenge: &AuthChallenge) -> AuthProof;
    fn verify(&self, challenge: &AuthChallenge, proof: &AuthProof) -> Result<(), ProtocolError>;
}

#[derive(Debug, Clone)]
pub struct SharedTokenAuthenticator {
    token: String,
}

impl SharedTokenAuthenticator {
    pub fn new(token: Option<String>) -> Self {
        Self {
            token: token.unwrap_or_else(|| "linux-carplay-default-auth".to_string()),
        }
    }

    fn digest(&self, challenge: &AuthChallenge) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.token.hash(&mut hasher);
        challenge.session_id.hash(&mut hasher);
        challenge.nonce.hash(&mut hasher);
        hasher.finish()
    }
}

impl CarPlayAuthenticator for SharedTokenAuthenticator {
    fn issue_challenge(&self, session_id: &str) -> AuthChallenge {
        let mut hasher = DefaultHasher::new();
        session_id.hash(&mut hasher);
        self.token.hash(&mut hasher);

        AuthChallenge {
            session_id: session_id.to_string(),
            nonce: hasher.finish(),
        }
    }

    fn prove(&self, challenge: &AuthChallenge) -> AuthProof {
        AuthProof {
            digest: self.digest(challenge),
        }
    }

    fn verify(&self, challenge: &AuthChallenge, proof: &AuthProof) -> Result<(), ProtocolError> {
        let expected = self.digest(challenge);
        if subtle_eq(expected, proof.digest) {
            Ok(())
        } else {
            Err(ProtocolError::AuthenticationFailed)
        }
    }
}

fn subtle_eq(lhs: u64, rhs: u64) -> bool {
    let mut diff = 0_u64;
    diff |= lhs ^ rhs;
    diff == 0
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("device vendor/product is unsupported: {vendor_id:04x}:{product_id:04x}")]
    UnsupportedDevice { vendor_id: u16, product_id: u16 },
    #[error("protocol version {0} is unsupported")]
    VersionUnsupported(u16),
    #[error("requested capabilities are incompatible")]
    IncompatibleCapabilities,
    #[error("wireless transport was requested but not supported")]
    WirelessNotSupported,
    #[error("authentication failed")]
    AuthenticationFailed,
}

#[cfg(test)]
mod tests {
    use super::{
        BasicNegotiator, CapabilitySet, CarPlayAuthenticator, CarPlayNegotiator, NegotiationRequest,
        ProtocolError, SharedTokenAuthenticator,
    };
    use crate::config::TransportMode;
    use crate::transport::UsbDeviceInfo;

    fn apple_device() -> UsbDeviceInfo {
        UsbDeviceInfo {
            vendor_id: 0x05ac,
            product_id: 0x12a8,
            bus_number: Some(1),
            address: Some(4),
        }
    }

    #[test]
    fn negotiates_channels_and_capabilities() {
        let negotiator = BasicNegotiator;
        let request = NegotiationRequest {
            local_capabilities: CapabilitySet::default(),
            requested_transport: TransportMode::Wired,
            protocol_version: 2,
            device: apple_device(),
        };

        let result = negotiator.negotiate(request).expect("negotiate should work");
        assert_eq!(result.transport, TransportMode::Wired);
        assert!(result.channels.len() >= 4);
    }

    #[test]
    fn rejects_non_apple_vendor() {
        let negotiator = BasicNegotiator;
        let request = NegotiationRequest {
            local_capabilities: CapabilitySet::default(),
            requested_transport: TransportMode::Auto,
            protocol_version: 2,
            device: UsbDeviceInfo {
                vendor_id: 0x1234,
                product_id: 0x0001,
                bus_number: None,
                address: None,
            },
        };

        let err = negotiator
            .negotiate(request)
            .expect_err("non-apple vendor should fail");
        assert!(matches!(err, ProtocolError::UnsupportedDevice { .. }));
    }

    #[test]
    fn shared_token_authenticator_roundtrip() {
        let auth = SharedTokenAuthenticator::new(Some("s3cr3t".to_string()));
        let challenge = auth.issue_challenge("session-x");
        let proof = auth.prove(&challenge);
        auth.verify(&challenge, &proof)
            .expect("proof should be accepted");
    }

    #[test]
    fn shared_token_authenticator_rejects_mismatch() {
        let server = SharedTokenAuthenticator::new(Some("server".to_string()));
        let client = SharedTokenAuthenticator::new(Some("client".to_string()));
        let challenge = server.issue_challenge("session-y");
        let proof = client.prove(&challenge);
        let err = server
            .verify(&challenge, &proof)
            .expect_err("proof should be rejected");
        assert_eq!(err, ProtocolError::AuthenticationFailed);
    }
}
