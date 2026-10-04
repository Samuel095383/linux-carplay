use crate::framing::{ChannelType, Frame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioFrame {
    pub sample_rate_hz: u32,
    pub channels: u8,
    pub pcm: Vec<u8>,
}

pub trait AudioChannel {
    fn pcm_frame(&self, frame: &AudioFrame) -> Frame;
    fn decode_pcm_frame(&self, frame: &Frame) -> Option<AudioFrame>;
}

pub struct BasicAudioChannel;

impl AudioChannel for BasicAudioChannel {
    fn pcm_frame(&self, frame: &AudioFrame) -> Frame {
        let mut payload = Vec::with_capacity(5 + frame.pcm.len());
        payload.extend_from_slice(&frame.sample_rate_hz.to_be_bytes());
        payload.push(frame.channels);
        payload.extend_from_slice(&frame.pcm);

        Frame {
            channel: ChannelType::Audio,
            payload,
        }
    }

    fn decode_pcm_frame(&self, frame: &Frame) -> Option<AudioFrame> {
        if frame.channel != ChannelType::Audio || frame.payload.len() < 5 {
            return None;
        }

        let mut sample_rate = [0_u8; 4];
        sample_rate.copy_from_slice(&frame.payload[0..4]);
        Some(AudioFrame {
            sample_rate_hz: u32::from_be_bytes(sample_rate),
            channels: frame.payload[4],
            pcm: frame.payload[5..].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioChannel, AudioFrame, BasicAudioChannel};

    #[test]
    fn wraps_and_decodes_pcm() {
        let channel = BasicAudioChannel;
        let frame = AudioFrame {
            sample_rate_hz: 48_000,
            channels: 2,
            pcm: vec![1, 2, 3],
        };

        let wrapped = channel.pcm_frame(&frame);
        let decoded = channel
            .decode_pcm_frame(&wrapped)
            .expect("decode audio frame");
        assert_eq!(decoded, frame);
    }
}
