use crate::framing::{ChannelType, Frame};

pub trait ControlChannel {
    fn heartbeat_frame(&self) -> Frame;
}

pub struct BasicControlChannel;

impl ControlChannel for BasicControlChannel {
    fn heartbeat_frame(&self) -> Frame {
        Frame {
            channel: ChannelType::Control,
            payload: b"heartbeat".to_vec(),
        }
    }
}
