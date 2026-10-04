use crate::framing::{ChannelType, Frame};

pub trait ControlChannel {
    fn heartbeat_frame(&self) -> Frame;
    fn negotiation_frame(&self, session_id: &str) -> Frame;
}

pub struct BasicControlChannel;

impl ControlChannel for BasicControlChannel {
    fn heartbeat_frame(&self) -> Frame {
        Frame {
            channel: ChannelType::Control,
            payload: b"heartbeat".to_vec(),
        }
    }

    fn negotiation_frame(&self, session_id: &str) -> Frame {
        Frame {
            channel: ChannelType::Control,
            payload: format!("negotiate:{session_id}").into_bytes(),
        }
    }
}
