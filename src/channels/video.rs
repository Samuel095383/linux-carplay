use crate::framing::{ChannelType, Frame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

pub trait VideoChannel {
    fn wrap_frame(&self, frame: &VideoFrame) -> Frame;
}

pub struct BasicVideoChannel;

impl VideoChannel for BasicVideoChannel {
    fn wrap_frame(&self, frame: &VideoFrame) -> Frame {
        let mut payload = Vec::with_capacity(4 + frame.data.len());
        payload.extend_from_slice(&frame.width.to_be_bytes());
        payload.extend_from_slice(&frame.height.to_be_bytes());
        payload.extend_from_slice(&frame.data);

        Frame {
            channel: ChannelType::Video,
            payload,
        }
    }
}
