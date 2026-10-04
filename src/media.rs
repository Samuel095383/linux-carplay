use thiserror::Error;

use crate::channels::audio::AudioFrame;
use crate::channels::video::VideoFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    Aac,
    Pcm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedVideoPacket {
    pub codec: VideoCodec,
    pub pts_ms: u64,
    pub keyframe: bool,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedAudioPacket {
    pub codec: AudioCodec,
    pub pts_ms: u64,
    pub channels: u8,
    pub sample_rate_hz: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicrophoneFrame {
    pub pts_ms: u64,
    pub sample_rate_hz: u32,
    pub channels: u8,
    pub pcm: Vec<u8>,
}

pub trait VideoDecoder {
    fn decode(&mut self, packet: &EncodedVideoPacket) -> Result<VideoFrame, MediaError>;
}

pub trait AudioDecoder {
    fn decode(&mut self, packet: &EncodedAudioPacket) -> Result<AudioFrame, MediaError>;
}

#[derive(Debug, Default)]
pub struct MockVideoDecoder;

impl VideoDecoder for MockVideoDecoder {
    fn decode(&mut self, packet: &EncodedVideoPacket) -> Result<VideoFrame, MediaError> {
        if packet.payload.is_empty() {
            return Err(MediaError::EmptyPayload("video"));
        }

        Ok(VideoFrame {
            width: 800,
            height: 480,
            data: packet.payload.clone(),
        })
    }
}

#[derive(Debug, Default)]
pub struct MockAudioDecoder;

impl AudioDecoder for MockAudioDecoder {
    fn decode(&mut self, packet: &EncodedAudioPacket) -> Result<AudioFrame, MediaError> {
        if packet.payload.is_empty() {
            return Err(MediaError::EmptyPayload("audio"));
        }

        Ok(AudioFrame {
            sample_rate_hz: packet.sample_rate_hz,
            channels: packet.channels,
            pcm: packet.payload.clone(),
        })
    }
}

#[derive(Debug)]
pub struct MediaPipeline<DV, DA>
where
    DV: VideoDecoder,
    DA: AudioDecoder,
{
    pub video_decoder: DV,
    pub audio_decoder: DA,
}

impl<DV, DA> MediaPipeline<DV, DA>
where
    DV: VideoDecoder,
    DA: AudioDecoder,
{
    pub fn new(video_decoder: DV, audio_decoder: DA) -> Self {
        Self {
            video_decoder,
            audio_decoder,
        }
    }

    pub fn decode_video(&mut self, packet: &EncodedVideoPacket) -> Result<VideoFrame, MediaError> {
        self.video_decoder.decode(packet)
    }

    pub fn decode_audio(&mut self, packet: &EncodedAudioPacket) -> Result<AudioFrame, MediaError> {
        self.audio_decoder.decode(packet)
    }
}

#[derive(Debug, Default)]
pub struct MicrophoneUplink;

impl MicrophoneUplink {
    pub fn packetize(&self, frame: &MicrophoneFrame) -> EncodedAudioPacket {
        EncodedAudioPacket {
            codec: AudioCodec::Pcm,
            pts_ms: frame.pts_ms,
            channels: frame.channels,
            sample_rate_hz: frame.sample_rate_hz,
            payload: frame.pcm.clone(),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MediaError {
    #[error("{0} payload is empty")]
    EmptyPayload(&'static str),
}

#[cfg(test)]
mod tests {
    use super::{
        AudioCodec, EncodedAudioPacket, EncodedVideoPacket, MediaPipeline, MicrophoneFrame,
        MicrophoneUplink, MockAudioDecoder, MockVideoDecoder, VideoCodec,
    };

    #[test]
    fn decodes_video_and_audio_packets() {
        let mut pipeline = MediaPipeline::new(MockVideoDecoder, MockAudioDecoder);
        let video_packet = EncodedVideoPacket {
            codec: VideoCodec::H264,
            pts_ms: 0,
            keyframe: true,
            payload: vec![0, 0, 1, 103],
        };
        let audio_packet = EncodedAudioPacket {
            codec: AudioCodec::Pcm,
            pts_ms: 0,
            channels: 2,
            sample_rate_hz: 48_000,
            payload: vec![1, 2, 3, 4],
        };

        let frame = pipeline.decode_video(&video_packet).expect("decode video");
        assert_eq!(frame.width, 800);
        let pcm = pipeline.decode_audio(&audio_packet).expect("decode audio");
        assert_eq!(pcm.sample_rate_hz, 48_000);
    }

    #[test]
    fn packetizes_microphone_frame() {
        let uplink = MicrophoneUplink;
        let mic = MicrophoneFrame {
            pts_ms: 42,
            sample_rate_hz: 16_000,
            channels: 1,
            pcm: vec![7, 8, 9],
        };
        let packet = uplink.packetize(&mic);
        assert_eq!(packet.sample_rate_hz, 16_000);
        assert_eq!(packet.payload, vec![7, 8, 9]);
    }
}
