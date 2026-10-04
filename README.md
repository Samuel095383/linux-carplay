# linux-carplay (MVP foundation)

`linux-carplay` is an **early Linux CarPlay receiver MVP foundation** inspired by Android-side projects such as DiPlay, but implemented from scratch for Linux with an honest scope.

> ⚠️ Current status: this repository now includes end-to-end **scaffolding** for all major CarPlay subsystems, but it is still not Apple-certified and does not provide full production CarPlay compatibility.

## What is implemented now

- Rust project structure with modular components:
  - USB discovery abstraction (`transport`), with:
    - mock backend (always available)
    - optional `libusb` backend via `rusb` feature
  - session state machine (`session`) with explicit lifecycle transitions for discovery, pairing, negotiation, auth, channel setup, streaming, reconnect and teardown
  - channel abstractions (`channels`) for control/video/audio/input with richer event/frame structures
  - framing codec (`framing`) for simple channel-tagged frame exchange
  - protocol scaffolding (`protocol`) for capability negotiation, channel setup decisions and challenge/proof auth flow
  - media scaffolding (`media`) for video/audio packet decoding and microphone uplink packetization
  - wireless pairing/discovery persistence scaffolding (`wireless`) for reconnect metadata
  - renderer abstraction (`renderer`) with:
    - `TerminalRenderer` practical Linux implementation (logs frame metadata)
    - `StubRenderer` fallback when display output is unavailable
  - structured diagnostics (`tracing` + `tracing-subscriber`)
- Demo and integration scaffold modes that exercise discovery → pairing → negotiation → auth → channel setup → media/input/uplink handling → reconnect lifecycle behavior.
- Tests for:
  - session state machine transitions
  - framing encode/decode behavior
  - transport backend behavior
  - CLI/config parsing
- Linux CI for format, lint, test, and build.

## What is still not production-ready

- Apple/MFi-certified proprietary protocol and credential handling
- Hardware-verified H.264/AAC decode/render/audio output integration against real iPhone sessions
- Full Linux device integration for microphone capture, low-latency playback, and complete input device matrix
- Production-grade wireless discovery/advertising/security handshakes across networks

## Safety and legal notice

- This project is **not Apple-certified**.
- Do **not** add Apple/MFi private keys, certificates, identities, or copied proprietary credentials.
- No source code, keys, certificates, or proprietary identities are copied from DiPlay.

## Build and run (Ubuntu/Debian)

### Prerequisites

- Rust toolchain (stable):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

- Optional native dependency for `libusb` backend:

```bash
sudo apt-get update
sudo apt-get install -y libusb-1.0-0-dev pkg-config
```

### Build

Default (no optional libusb backend):

```bash
cargo build
```

With optional libusb backend:

```bash
cargo build --features libusb
```

### Run demo lifecycle

```bash
cargo run -- --demo
```

Useful options:

```bash
cargo run -- --help
cargo run -- --version
cargo run -- --demo --renderer terminal --log-format json
cargo run -- --transport wireless --pairing-store /tmp/carplay_pairings.db
```

## USB permissions / udev guidance (for future real-device mode)

When using real USB device access, non-root users commonly need udev rules. Example placeholder rule:

```udev
SUBSYSTEM=="usb", ATTR{idVendor}=="05ac", MODE="0660", GROUP="plugdev"
```

Then reload rules and reconnect device:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Roadmap

1. Wired CarPlay session transport handshake foundation
2. Control channel semantics and protocol negotiation scaffolding
3. Video path (decode/render), likely via GStreamer and/or SDL2/Wayland paths
4. Audio path (PipeWire/PulseAudio integration)
5. Input forwarding (touch/keys/buttons)
6. Wireless discovery/pairing/reconnect support
7. Better Linux UI integration (GTK/Qt/embedded compositor variants)
