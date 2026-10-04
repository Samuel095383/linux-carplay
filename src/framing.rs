use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Control = 1,
    Video = 2,
    Audio = 3,
    Input = 4,
}

impl TryFrom<u8> for ChannelType {
    type Error = FramingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Control),
            2 => Ok(Self::Video),
            3 => Ok(Self::Audio),
            4 => Ok(Self::Input),
            _ => Err(FramingError::UnknownChannel(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub channel: ChannelType,
    pub payload: Vec<u8>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FramingError {
    #[error("frame too short")]
    TooShort,
    #[error("unknown channel id {0}")]
    UnknownChannel(u8),
    #[error("declared length does not match payload")]
    LengthMismatch,
}

pub struct FrameCodec;

impl FrameCodec {
    pub fn encode(frame: &Frame) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(1 + 4 + frame.payload.len());
        encoded.push(frame.channel as u8);
        encoded.extend_from_slice(&(frame.payload.len() as u32).to_be_bytes());
        encoded.extend_from_slice(&frame.payload);
        encoded
    }

    pub fn decode(buf: &[u8]) -> Result<Frame, FramingError> {
        if buf.len() < 5 {
            return Err(FramingError::TooShort);
        }

        let channel = ChannelType::try_from(buf[0])?;
        let mut len_bytes = [0_u8; 4];
        len_bytes.copy_from_slice(&buf[1..5]);
        let declared_len = u32::from_be_bytes(len_bytes) as usize;
        let payload = &buf[5..];

        if payload.len() != declared_len {
            return Err(FramingError::LengthMismatch);
        }

        Ok(Frame {
            channel,
            payload: payload.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ChannelType, Frame, FrameCodec, FramingError};

    #[test]
    fn roundtrip_frame_codec() {
        let frame = Frame {
            channel: ChannelType::Control,
            payload: vec![1, 2, 3, 4],
        };
        let encoded = FrameCodec::encode(&frame);
        let decoded = FrameCodec::decode(&encoded).expect("decode");
        assert_eq!(decoded, frame);
    }

    #[test]
    fn decode_rejects_invalid_length() {
        let encoded = vec![2, 0, 0, 0, 5, 1, 2, 3];
        let err = FrameCodec::decode(&encoded).expect_err("must fail");
        assert_eq!(err, FramingError::LengthMismatch);
    }
}
