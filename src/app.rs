use anyhow::{Context, Result, bail};
use tracing::{info, warn};

use crate::channels::audio::{AudioChannel, AudioFrame, BasicAudioChannel};
use crate::channels::control::{BasicControlChannel, ControlChannel};
use crate::channels::input::{BasicInputChannel, InputChannel, InputEvent, InputMapper};
use crate::channels::video::{BasicVideoChannel, VideoChannel};
use crate::config::{Cli, RendererKind, TransportMode, UsbBackend};
use crate::diagnostics;
use crate::framing::FrameCodec;
use crate::media::{
    AudioCodec, EncodedAudioPacket, EncodedVideoPacket, MediaPipeline, MicrophoneFrame,
    MicrophoneUplink, MockAudioDecoder, MockVideoDecoder, VideoCodec,
};
use crate::protocol::{
    BasicNegotiator, CapabilitySet, CarPlayAuthenticator, CarPlayNegotiator,
    SharedTokenAuthenticator,
};
use crate::renderer::{Renderer, StubRenderer, TerminalRenderer};
use crate::session::{Session, SessionEvent, SessionState};
use crate::transport::{LibusbDiscovery, MockUsbDiscovery, UsbDiscoveryBackend};
use crate::wireless::{PairingRecord, PairingStore, WirelessDiscovery, device_id};

pub struct Application {
    cli: Cli,
}

impl Application {
    pub fn new(cli: Cli) -> Self {
        Self { cli }
    }

    pub fn run(self) -> Result<()> {
        if self.cli.demo {
            self.run_lifecycle(true)
        } else {
            self.run_lifecycle(false)
        }
    }

    fn run_lifecycle(self, demo_mode: bool) -> Result<()> {
        let mode = if demo_mode { "demo" } else { "integration" };
        info!(mode, "starting CarPlay lifecycle");

        let mut session = Session::new();
        Self::apply_event(&mut session, SessionEvent::Start)?;

        let discovery = self.discovery_backend();
        let devices = discovery
            .discover()
            .context("transport discovery failed while running lifecycle")?;
        if devices.is_empty() {
            bail!("no iPhone-like devices discovered");
        }

        Self::apply_event(&mut session, SessionEvent::DeviceDiscovered)?;

        let mut pairing_store = PairingStore::load(&self.cli.pairing_store)
            .context("failed to load wireless pairing store")?;
        let selected = devices
            .iter()
            .find(|device| device.vendor_id == 0x05ac)
            .unwrap_or(&devices[0]);
        let selected_device_id = device_id(selected);

        let discovery = WirelessDiscovery;
        if matches!(self.cli.transport, TransportMode::Wireless | TransportMode::Auto) {
            if let Some(candidate) = discovery.discover_candidate(&devices) {
                info!(candidate, "wireless candidate discovered");
            }
        }

        let pairing = pairing_store
            .find(&selected_device_id)
            .cloned()
            .unwrap_or_else(|| PairingRecord {
                device_id: selected_device_id.clone(),
                host_id: "linux-host".to_string(),
                reconnect_token: format!("reconnect:{}", selected_device_id),
                last_seen_unix: 0,
            });
        pairing_store.upsert(pairing);
        pairing_store
            .save(&self.cli.pairing_store)
            .context("failed to persist wireless pairing store")?;
        Self::apply_event(&mut session, SessionEvent::PairingComplete)?;

        let negotiator = BasicNegotiator;
        let negotiation = negotiator.negotiate(crate::protocol::NegotiationRequest {
            local_capabilities: CapabilitySet::default(),
            requested_transport: self.cli.transport,
            protocol_version: 2,
            device: selected.clone(),
        })?;
        info!(
            session_id = negotiation.session_id,
            transport = ?negotiation.transport,
            channels = negotiation.channels.len(),
            "negotiation completed"
        );
        Self::apply_event(&mut session, SessionEvent::NegotiationSucceeded)?;

        let authenticator = SharedTokenAuthenticator::new(self.cli.auth_token.clone());
        let challenge = authenticator.issue_challenge(&negotiation.session_id);
        let proof = authenticator.prove(&challenge);
        authenticator.verify(&challenge, &proof)?;
        Self::apply_event(&mut session, SessionEvent::AuthenticationSucceeded)?;

        Self::apply_event(&mut session, SessionEvent::ChannelsReady)?;

        let mut renderer = self.renderer();
        let control = BasicControlChannel;
        let video = BasicVideoChannel;
        let audio = BasicAudioChannel;
        let input = BasicInputChannel;

        let heartbeat = control.heartbeat_frame();
        diagnostics::trace_packet("tx-control", &heartbeat);
        let encoded = FrameCodec::encode(&heartbeat);
        let decoded = FrameCodec::decode(&encoded)?;
        diagnostics::trace_packet("rx-control", &decoded);

        let negotiation_control = control.negotiation_frame(&negotiation.session_id);
        diagnostics::trace_packet("tx-negotiation", &negotiation_control);

        let mut media_pipeline = MediaPipeline::new(MockVideoDecoder, MockAudioDecoder);
        let encoded_video = EncodedVideoPacket {
            codec: VideoCodec::H264,
            pts_ms: 0,
            keyframe: true,
            payload: vec![0, 0, 1, 103, 103, 66],
        };
        let decoded_video = media_pipeline.decode_video(&encoded_video)?;
        let wrapped_video = video.wrap_frame(&decoded_video);
        diagnostics::trace_packet("tx-video", &wrapped_video);
        let restored_video = video
            .unwrap_frame(&wrapped_video)
            .context("video frame unwrap failed")?;
        if let Err(err) = renderer.render(&restored_video) {
            warn!(error = %err, "renderer unavailable; continuing without visual output");
        }

        let encoded_audio = EncodedAudioPacket {
            codec: AudioCodec::Pcm,
            pts_ms: 0,
            channels: 2,
            sample_rate_hz: 48_000,
            payload: vec![1, 2, 3, 4, 5, 6],
        };
        let decoded_audio = media_pipeline.decode_audio(&encoded_audio)?;
        let wrapped_audio = audio.pcm_frame(&decoded_audio);
        diagnostics::trace_packet("tx-audio", &wrapped_audio);
        let _restored_audio = audio
            .decode_pcm_frame(&wrapped_audio)
            .context("audio frame decode failed")?;

        let microphone = MicrophoneUplink;
        let uplink_packet = microphone.packetize(&MicrophoneFrame {
            pts_ms: 10,
            sample_rate_hz: 16_000,
            channels: 1,
            pcm: vec![9, 8, 7, 6],
        });
        let uplink_frame = audio.pcm_frame(&AudioFrame {
            sample_rate_hz: uplink_packet.sample_rate_hz,
            channels: uplink_packet.channels,
            pcm: uplink_packet.payload,
        });
        diagnostics::trace_packet("tx-voice-uplink", &uplink_frame);

        let mapper = InputMapper {
            source_width: 1920,
            source_height: 1080,
            target_width: restored_video.width,
            target_height: restored_video.height,
        };
        let mapped_input = mapper.map_event(InputEvent::TouchDown { x: 960, y: 540 });
        let input_frame = input.encode_event(&mapped_input);
        diagnostics::trace_packet("tx-input", &input_frame);

        if !demo_mode {
            warn!("running integration scaffold mode; protocol/media paths are non-MFi placeholders");
        }

        Self::apply_event(&mut session, SessionEvent::LinkLost)?;
        let reconnect_succeeded = self.attempt_reconnect(&mut session, control, &negotiation.session_id)?;
        if !reconnect_succeeded {
            Self::apply_event(&mut session, SessionEvent::ReconnectFailed)?;
        }

        if session.state() != SessionState::Failed {
            Self::apply_event(&mut session, SessionEvent::Stop)?;
            Self::apply_event(&mut session, SessionEvent::TeardownComplete)?;
        }

        info!(state = ?session.state(), mode, "CarPlay lifecycle finished");
        Ok(())
    }

    fn attempt_reconnect(
        &self,
        session: &mut Session,
        control: BasicControlChannel,
        session_id: &str,
    ) -> Result<bool> {
        for attempt in 1..=self.cli.max_reconnect_attempts.max(1) {
            info!(attempt, "attempting reconnect");
            let frame = control.negotiation_frame(session_id);
            diagnostics::trace_packet("tx-reconnect", &frame);

            if attempt == 1 {
                Self::apply_event(session, SessionEvent::ReconnectSucceeded)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn apply_event(session: &mut Session, event: SessionEvent) -> Result<()> {
        let from = format!("{:?}", session.state());
        let next = session.transition(event)?;
        let to = format!("{:?}", next);
        diagnostics::trace_state_transition(&from, &to);
        Ok(())
    }

    fn renderer(&self) -> Box<dyn Renderer> {
        match self.cli.renderer {
            RendererKind::Terminal => Box::new(TerminalRenderer),
            RendererKind::Stub => Box::new(StubRenderer),
        }
    }

    fn discovery_backend(&self) -> Box<dyn UsbDiscoveryBackend> {
        if self.cli.demo || self.cli.usb_backend == UsbBackend::Mock {
            return Box::new(MockUsbDiscovery);
        }

        match self.cli.usb_backend {
            UsbBackend::Auto => Box::new(LibusbDiscovery),
            UsbBackend::Mock => Box::new(MockUsbDiscovery),
        }
    }
}
