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
    let _ = code;
    todo!("task 05")
}
