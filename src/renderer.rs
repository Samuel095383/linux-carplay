use thiserror::Error;
use tracing::info;

use crate::channels::video::VideoFrame;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("renderer is unavailable in this build")]
    Unavailable,
}

pub trait Renderer {
    fn render(&mut self, frame: &VideoFrame) -> Result<(), RenderError>;
}

pub struct StubRenderer;

impl Renderer for StubRenderer {
    fn render(&mut self, _frame: &VideoFrame) -> Result<(), RenderError> {
        Err(RenderError::Unavailable)
    }
}

pub struct TerminalRenderer;

impl Renderer for TerminalRenderer {
    fn render(&mut self, frame: &VideoFrame) -> Result<(), RenderError> {
        info!(
            width = frame.width,
            height = frame.height,
            bytes = frame.data.len(),
            "rendered demo frame metadata"
        );
        Ok(())
    }
}
