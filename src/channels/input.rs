use crate::framing::{ChannelType, Frame};

pub trait InputChannel {
    fn touch_event(&self, x: u16, y: u16) -> Frame;
}

pub struct BasicInputChannel;

impl InputChannel for BasicInputChannel {
    fn touch_event(&self, x: u16, y: u16) -> Frame {
        let mut payload = Vec::with_capacity(4);
        payload.extend_from_slice(&x.to_be_bytes());
        payload.extend_from_slice(&y.to_be_bytes());

        Frame {
            channel: ChannelType::Input,
            payload,
        }
    }
}
