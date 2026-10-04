use crate::framing::{ChannelType, Frame};

pub trait AudioChannel {
    fn pcm_frame(&self, pcm_bytes: &[u8]) -> Frame;
}

pub struct BasicAudioChannel;

impl AudioChannel for BasicAudioChannel {
    fn pcm_frame(&self, pcm_bytes: &[u8]) -> Frame {
        Frame {
            channel: ChannelType::Audio,
            payload: pcm_bytes.to_vec(),
        }
    }
}
