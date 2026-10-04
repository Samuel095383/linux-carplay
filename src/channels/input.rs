use crate::framing::{ChannelType, Frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareButton {
    Home,
    Back,
    VolumeUp,
    VolumeDown,
    PushToTalk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    TouchDown {
        x: u16,
        y: u16,
    },
    TouchUp {
        x: u16,
        y: u16,
    },
    TouchMove {
        x: u16,
        y: u16,
    },
    Key {
        code: u16,
        pressed: bool,
    },
    Rotary {
        delta: i16,
    },
    Button {
        button: HardwareButton,
        pressed: bool,
    },
}

pub trait InputChannel {
    fn encode_event(&self, event: &InputEvent) -> Frame;
}

pub struct BasicInputChannel;

impl InputChannel for BasicInputChannel {
    fn encode_event(&self, event: &InputEvent) -> Frame {
        let payload = match event {
            InputEvent::TouchDown { x, y } => touch_payload(0x01, *x, *y),
            InputEvent::TouchUp { x, y } => touch_payload(0x02, *x, *y),
            InputEvent::TouchMove { x, y } => touch_payload(0x03, *x, *y),
            InputEvent::Key { code, pressed } => {
                let mut payload = Vec::with_capacity(4);
                payload.push(0x04);
                payload.push(u8::from(*pressed));
                payload.extend_from_slice(&code.to_be_bytes());
                payload
            }
            InputEvent::Rotary { delta } => {
                let mut payload = Vec::with_capacity(3);
                payload.push(0x05);
                payload.extend_from_slice(&delta.to_be_bytes());
                payload
            }
            InputEvent::Button { button, pressed } => {
                vec![0x06, button_code(*button), u8::from(*pressed)]
            }
        };

        Frame {
            channel: ChannelType::Input,
            payload,
        }
    }
}

fn touch_payload(kind: u8, x: u16, y: u16) -> Vec<u8> {
    let mut payload = Vec::with_capacity(5);
    payload.push(kind);
    payload.extend_from_slice(&x.to_be_bytes());
    payload.extend_from_slice(&y.to_be_bytes());
    payload
}

fn button_code(button: HardwareButton) -> u8 {
    match button {
        HardwareButton::Home => 0x01,
        HardwareButton::Back => 0x02,
        HardwareButton::VolumeUp => 0x03,
        HardwareButton::VolumeDown => 0x04,
        HardwareButton::PushToTalk => 0x05,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputMapper {
    pub source_width: u16,
    pub source_height: u16,
    pub target_width: u16,
    pub target_height: u16,
}

impl InputMapper {
    pub fn map_touch(&self, x: u16, y: u16) -> (u16, u16) {
        let source_w = self.source_width.max(1) as u32;
        let source_h = self.source_height.max(1) as u32;
        let target_w = self.target_width as u32;
        let target_h = self.target_height as u32;

        let scaled_x =
            ((x as u32).saturating_mul(target_w) / source_w).min(target_w.saturating_sub(1));
        let scaled_y =
            ((y as u32).saturating_mul(target_h) / source_h).min(target_h.saturating_sub(1));
        (scaled_x as u16, scaled_y as u16)
    }

    pub fn map_event(&self, event: InputEvent) -> InputEvent {
        match event {
            InputEvent::TouchDown { x, y } => {
                let (mx, my) = self.map_touch(x, y);
                InputEvent::TouchDown { x: mx, y: my }
            }
            InputEvent::TouchUp { x, y } => {
                let (mx, my) = self.map_touch(x, y);
                InputEvent::TouchUp { x: mx, y: my }
            }
            InputEvent::TouchMove { x, y } => {
                let (mx, my) = self.map_touch(x, y);
                InputEvent::TouchMove { x: mx, y: my }
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BasicInputChannel, HardwareButton, InputChannel, InputEvent, InputMapper};

    #[test]
    fn maps_touch_coordinates() {
        let mapper = InputMapper {
            source_width: 1920,
            source_height: 1080,
            target_width: 800,
            target_height: 480,
        };

        let mapped = mapper.map_event(InputEvent::TouchMove { x: 960, y: 540 });
        assert_eq!(mapped, InputEvent::TouchMove { x: 400, y: 240 });
    }

    #[test]
    fn encodes_button_event() {
        let channel = BasicInputChannel;
        let frame = channel.encode_event(&InputEvent::Button {
            button: HardwareButton::PushToTalk,
            pressed: true,
        });

        assert_eq!(frame.payload, vec![0x06, 0x05, 0x01]);
    }
}
