use anyhow::Result;
use tracing_subscriber::EnvFilter;

use crate::config::LogFormat;

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
