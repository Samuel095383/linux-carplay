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

#[derive(Debug, Clone, Parser)]
#[command(
    name = "linux-carplay",
    author,
    version,
    about = "Linux CarPlay receiver MVP foundation with demo mode"
)]
pub struct Cli {
    #[arg(long, help = "Run without real iPhone hardware")]
    pub demo: bool,

    #[arg(long, value_enum, default_value_t = RendererKind::Terminal)]
    pub renderer: RendererKind,

    #[arg(long, value_enum, default_value_t = UsbBackend::Auto)]
    pub usb_backend: UsbBackend,

    #[arg(long, value_enum, default_value_t = LogFormat::Text)]
    pub log_format: LogFormat,
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
    use super::{Cli, LogFormat, RendererKind, UsbBackend};

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["linux-carplay"]);
        assert!(!cli.demo);
        assert_eq!(cli.renderer, RendererKind::Terminal);
        assert_eq!(cli.usb_backend, UsbBackend::Auto);
        assert_eq!(cli.log_format, LogFormat::Text);
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
        ]);
        assert!(cli.demo);
        assert_eq!(cli.renderer, RendererKind::Stub);
        assert_eq!(cli.usb_backend, UsbBackend::Mock);
        assert_eq!(cli.log_format, LogFormat::Json);
    }
}
