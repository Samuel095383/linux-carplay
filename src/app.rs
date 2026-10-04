use anyhow::{Context, Result, bail};
use tracing::{info, warn};

use crate::channels::control::{BasicControlChannel, ControlChannel};
use crate::channels::video::{BasicVideoChannel, VideoChannel, VideoFrame};
use crate::config::{Cli, RendererKind, UsbBackend};
use crate::framing::FrameCodec;
use crate::renderer::{Renderer, StubRenderer, TerminalRenderer};
use crate::session::{Session, SessionEvent};
use crate::transport::{LibusbDiscovery, MockUsbDiscovery, NullUsbDiscovery, UsbDiscoveryBackend};

pub struct Application {
    cli: Cli,
}

impl Application {
    pub fn new(cli: Cli) -> Self {
        Self { cli }
    }

    pub fn run(self) -> Result<()> {
        if self.cli.demo {
            self.run_demo()
        } else {
            bail!(
                "real CarPlay receiver mode is not yet implemented. Re-run with --demo to exercise lifecycle"
            );
        }
    }

    fn run_demo(self) -> Result<()> {
        info!("starting demo lifecycle");
        let mut session = Session::new();
        session.transition(SessionEvent::Start)?;

        let discovery = self.discovery_backend();
        let devices = discovery
            .discover()
            .context("USB discovery failed while running demo")?;

        if devices.is_empty() {
            bail!("no USB devices discovered in demo backend");
        }

        info!(devices = devices.len(), "demo discovery completed");
        session.transition(SessionEvent::DeviceDiscovered)?;
        session.transition(SessionEvent::NegotiationSucceeded)?;

        let mut renderer = self.renderer();
        let control = BasicControlChannel;
        let video = BasicVideoChannel;

        let heartbeat = control.heartbeat_frame();
        let encoded = FrameCodec::encode(&heartbeat);
        let decoded = FrameCodec::decode(&encoded)?;
        info!(
            payload_size = decoded.payload.len(),
            "control frame roundtrip successful"
        );

        let frame = VideoFrame {
            width: 800,
            height: 480,
            data: vec![0_u8; 800],
        };
        let wrapped = video.wrap_frame(&frame);
        info!(wrapped_size = wrapped.payload.len(), "video frame wrapped");

        if let Err(err) = renderer.render(&frame) {
            warn!(error = %err, "renderer unavailable; continuing demo without display output");
        }

        session.transition(SessionEvent::Stop)?;
        session.transition(SessionEvent::TeardownComplete)?;
        info!("demo lifecycle finished");
        Ok(())
    }

    fn renderer(&self) -> Box<dyn Renderer> {
        match self.cli.renderer {
            RendererKind::Terminal => Box::new(TerminalRenderer),
            RendererKind::Stub => Box::new(StubRenderer),
        }
    }

    fn discovery_backend(&self) -> Box<dyn UsbDiscoveryBackend> {
        if self.cli.demo || self.cli.usb_backend == UsbBackend::Mock {
            return Box::new(MockUsbDiscovery);
        }

        match self.cli.usb_backend {
            UsbBackend::Auto => Box::new(LibusbDiscovery),
            UsbBackend::Mock => Box::new(MockUsbDiscovery),
        }
    }

    #[allow(dead_code)]
    fn fallback_discovery_backend(&self) -> Box<dyn UsbDiscoveryBackend> {
        Box::new(NullUsbDiscovery)
    }
}
