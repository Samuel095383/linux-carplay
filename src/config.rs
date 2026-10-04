use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RendererKind {
    Stub,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum UsbBackend {
    Auto,
    Mock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TransportMode {
    Wired,
    Wireless,
    Auto,
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "linux-carplay",
    author,
    version,
    about = "Linux CarPlay receiver foundation with demo and integration scaffolding"
)]
pub struct Cli {
    #[arg(long, help = "Run lifecycle without iPhone hardware")]
    pub demo: bool,

    #[arg(long, value_enum, default_value_t = RendererKind::Terminal)]
    pub renderer: RendererKind,

    #[arg(long, value_enum, default_value_t = UsbBackend::Auto)]
    pub usb_backend: UsbBackend,

    #[arg(long, value_enum, default_value_t = LogFormat::Text)]
    pub log_format: LogFormat,

    #[arg(long, value_enum, default_value_t = TransportMode::Auto)]
    pub transport: TransportMode,

    #[arg(long, default_value_t = 3)]
    pub max_reconnect_attempts: usize,

    #[arg(long, help = "Optional shared token used by scaffold authenticator")]
    pub auth_token: Option<String>,

    #[arg(
        long,
        default_value = "./carplay_pairings.db",
        help = "Pairing metadata store path for wireless mode"
    )]
    pub pairing_store: PathBuf,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    pub fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        <Self as Parser>::parse_from(itr)
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, LogFormat, RendererKind, TransportMode, UsbBackend};

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["linux-carplay"]);
        assert!(!cli.demo);
        assert_eq!(cli.renderer, RendererKind::Terminal);
        assert_eq!(cli.usb_backend, UsbBackend::Auto);
        assert_eq!(cli.log_format, LogFormat::Text);
        assert_eq!(cli.transport, TransportMode::Auto);
        assert_eq!(cli.max_reconnect_attempts, 3);
        assert!(cli.auth_token.is_none());
        assert!(cli.pairing_store.ends_with("carplay_pairings.db"));
    }

    #[test]
    fn cli_parses_explicit_values() {
        let cli = Cli::parse_from([
            "linux-carplay",
            "--demo",
            "--renderer",
            "stub",
            "--usb-backend",
            "mock",
            "--log-format",
            "json",
            "--transport",
            "wireless",
            "--max-reconnect-attempts",
            "7",
            "--auth-token",
            "abc123",
            "--pairing-store",
            "/tmp/test-pairings.db",
        ]);
        assert!(cli.demo);
        assert_eq!(cli.renderer, RendererKind::Stub);
        assert_eq!(cli.usb_backend, UsbBackend::Mock);
        assert_eq!(cli.log_format, LogFormat::Json);
        assert_eq!(cli.transport, TransportMode::Wireless);
        assert_eq!(cli.max_reconnect_attempts, 7);
        assert_eq!(cli.auth_token.as_deref(), Some("abc123"));
        assert_eq!(
            cli.pairing_store
                .to_str()
                .expect("pairing store path should be valid UTF-8"),
            "/tmp/test-pairings.db"
        );
    }
}
