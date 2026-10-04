use anyhow::Result;
use linux_carplay::app::Application;
use linux_carplay::config::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    linux_carplay::diagnostics::init_logging(cli.log_format)?;
    let app = Application::new(cli);
    app.run()
}
