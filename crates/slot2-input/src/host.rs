//! Keyboard → buttons for the PC window. A fixed layout; nothing to configure yet.
//!
//! | Key                | Button |
//! |--------------------|--------|
//! | Arrows             | D-pad  |
//! | X / Z              | A / B  |
//! | S / A              | X / Y  |
//! | Q / W              | L1 / R1|
//! | 1 / 2              | L2 / R2|
//! | Enter              | Start  |
//! | Right Shift        | Select |
//! | M or Backspace     | Menu   |
//! | Page Up / Page Down| Vol+ / Vol− |
//! | P                  | Power  |
//!
//! Escape is not mapped: the app loop uses it to quit. Task 05 implements `host_map`.

pub use winit::keyboard::KeyCode;

use crate::Button;

pub fn host_map(code: KeyCode) -> Option<Button> {
    match code {
        KeyCode::ArrowUp => Some(Button::Up),
        KeyCode::ArrowDown => Some(Button::Down),
        KeyCode::ArrowLeft => Some(Button::Left),
        KeyCode::ArrowRight => Some(Button::Right),
        KeyCode::KeyX => Some(Button::A),
        KeyCode::KeyZ => Some(Button::B),
        KeyCode::KeyS => Some(Button::X),
        KeyCode::KeyA => Some(Button::Y),
        KeyCode::KeyQ => Some(Button::L1),
        KeyCode::KeyW => Some(Button::R1),
        KeyCode::Digit1 => Some(Button::L2),
        KeyCode::Digit2 => Some(Button::R2),
        KeyCode::Enter => Some(Button::Start),
        KeyCode::ShiftRight => Some(Button::Select),
        KeyCode::KeyM | KeyCode::Backspace => Some(Button::Menu),
        KeyCode::PageUp => Some(Button::VolUp),
        KeyCode::PageDown => Some(Button::VolDown),
        KeyCode::KeyP => Some(Button::Power),
        _ => None,
    }
}
