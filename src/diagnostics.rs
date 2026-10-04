use anyhow::Result;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::config::LogFormat;
use crate::framing::Frame;

pub fn init_logging(format: LogFormat) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let builder = tracing_subscriber::fmt().with_env_filter(env_filter);

    match format {
        LogFormat::Text => builder
            .compact()
            .try_init()
            .map_err(|err| anyhow::anyhow!(err.to_string()))?,
        LogFormat::Json => builder
            .json()
            .try_init()
            .map_err(|err| anyhow::anyhow!(err.to_string()))?,
    }

    Ok(())
}

pub fn trace_state_transition(from: &str, to: &str) {
    info!(from, to, "session state transition");
}

pub fn trace_packet(stage: &str, frame: &Frame) {
    debug!(
        stage,
        channel = ?frame.channel,
        bytes = frame.payload.len(),
        "packet trace"
    );
}
