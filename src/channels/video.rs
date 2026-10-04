use crate::framing::{ChannelType, Frame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

pub trait VideoChannel {
    fn wrap_frame(&self, frame: &VideoFrame) -> Frame;
    fn unwrap_frame(&self, frame: &Frame) -> Option<VideoFrame>;
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

    fn unwrap_frame(&self, frame: &Frame) -> Option<VideoFrame> {
        if frame.channel != ChannelType::Video || frame.payload.len() < 4 {
            return None;
        }

        let mut width = [0_u8; 2];
        width.copy_from_slice(&frame.payload[0..2]);
        let mut height = [0_u8; 2];
        height.copy_from_slice(&frame.payload[2..4]);

        Some(VideoFrame {
            width: u16::from_be_bytes(width),
            height: u16::from_be_bytes(height),
            data: frame.payload[4..].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{BasicVideoChannel, VideoChannel, VideoFrame};

    #[test]
    fn wraps_and_unwraps_video_frame() {
        let channel = BasicVideoChannel;
        let frame = VideoFrame {
            width: 800,
            height: 480,
            data: vec![0, 0, 1, 103],
        };

        let wrapped = channel.wrap_frame(&frame);
        let decoded = channel.unwrap_frame(&wrapped).expect("unwrap frame");
        assert_eq!(decoded, frame);
    }
}
