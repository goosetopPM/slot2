//! Raw Linux input events, and the map from their key codes to [`Button`]s.
//!
//! Implementation notes for task 05 (see tasks/05-input.md):
//!
//! - `struct input_event` on 64-bit Linux is 24 bytes: `tv_sec: i64`, `tv_usec: i64`,
//!   `type_: u16`, `code: u16`, `value: i32`, little-endian on aarch64. `parse_input_event`
//!   decodes exactly that; anything shorter than 24 bytes is `None`.
//! - Types: `EV_SYN = 0`, `EV_KEY = 1`, `EV_ABS = 3`, `EV_SW = 5`. Key values: 0 release,
//!   1 press, 2 auto-repeat (ignored). Switch `SW_LID = 0`, value 1 = closed.
//! - `EvdevSource::open_all(keymap)` opens every `/dev/input/event*` it can (non-blocking:
//!   `O_NONBLOCK` via `OpenOptions` + `custom_flags`, unix only) and keeps them. Devices
//!   that cannot be opened are skipped with one `eprintln!`. `poll(now)` reads whatever is
//!   pending from each fd (a 24-byte-multiple buffer, EAGAIN = nothing) and maps:
//!   - `EV_KEY` with a code in the keymap → `Event::Button { at: now }`
//!   - `EV_ABS` codes `ABS_X=0, ABS_Y=1` → `Axis::LeftX/LeftY`, `ABS_RX=3, ABS_RY=4` →
//!     `RightX/RightY`, `ABS_Z=2` → `L2`, `ABS_RZ=5` → `R2`, scaled by the axis range from
//!     `keymap.abs_range` (default -32768..=32767 for sticks, 0..=255 for triggers)
//!   - `EV_SW` `SW_LID` → `Event::Lid`
//!   Everything else is dropped. Unix-only code sits behind `#[cfg(unix)]`; on other
//!   targets `open_all` returns a source with no devices and `poll` returns nothing, so the
//!   crate builds and its tests run on Windows.
//! - No `libc` crate: `open`/`read` go through `std::fs::File` + `std::io::Read`, and a
//!   `WouldBlock` error means "nothing pending".

use std::collections::HashMap;
use std::time::Instant;

use crate::{Button, Event};

pub const EV_SYN: u16 = 0;
pub const EV_KEY: u16 = 1;
pub const EV_ABS: u16 = 3;
pub const EV_SW: u16 = 5;
pub const SW_LID: u16 = 0;

/// One decoded `input_event`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawEvent {
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

/// Decode one 24-byte little-endian `input_event`. `None` if `bytes` is too short.
pub fn parse_input_event(bytes: &[u8]) -> Option<RawEvent> {
    let _ = bytes;
    todo!("task 05")
}

/// Key code → button, plus axis ranges. Data, so a device profile (or a user file later)
/// can replace it without code.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyMap {
    pub keys: HashMap<u16, Button>,
    /// `(min, max)` for stick axes and for trigger axes.
    pub stick_range: (i32, i32),
    pub trigger_range: (i32, i32),
}

impl KeyMap {
    pub fn from_pairs(pairs: &[(u16, Button)]) -> Self {
        KeyMap {
            keys: pairs.iter().copied().collect(),
            stick_range: (-32768, 32767),
            trigger_range: (0, 255),
        }
    }

    pub fn button(&self, code: u16) -> Option<Button> {
        self.keys.get(&code).copied()
    }
}

/// Linux `input-event-codes.h` values the default map uses.
pub mod codes {
    pub const KEY_UP: u16 = 103;
    pub const KEY_LEFT: u16 = 105;
    pub const KEY_RIGHT: u16 = 106;
    pub const KEY_DOWN: u16 = 108;
    pub const KEY_VOLUMEDOWN: u16 = 114;
    pub const KEY_VOLUMEUP: u16 = 115;
    pub const KEY_POWER: u16 = 116;
    pub const BTN_SOUTH: u16 = 0x130; // A on a gamepad
    pub const BTN_EAST: u16 = 0x131; // B
    pub const BTN_NORTH: u16 = 0x133; // X
    pub const BTN_WEST: u16 = 0x134; // Y
    pub const BTN_TL: u16 = 0x136; // L1
    pub const BTN_TR: u16 = 0x137; // R1
    pub const BTN_TL2: u16 = 0x138; // L2
    pub const BTN_TR2: u16 = 0x139; // R2
    pub const BTN_SELECT: u16 = 0x13a;
    pub const BTN_START: u16 = 0x13b;
    pub const BTN_MODE: u16 = 0x13c; // MENU / guide
    pub const BTN_DPAD_UP: u16 = 0x220;
    pub const BTN_DPAD_DOWN: u16 = 0x221;
    pub const BTN_DPAD_LEFT: u16 = 0x222;
    pub const BTN_DPAD_RIGHT: u16 = 0x223;
}

/// A first guess at the H700 gamepad node's codes, to be corrected from the first-boot
/// survey (V-1). Face buttons follow the Linux gamepad convention; the D-pad is listed
/// both as KEY_* and BTN_DPAD_* since drivers differ.
pub const DEFAULT_H700_KEYMAP: &[(u16, Button)] = &[
    (codes::BTN_SOUTH, Button::A),
    (codes::BTN_EAST, Button::B),
    (codes::BTN_NORTH, Button::X),
    (codes::BTN_WEST, Button::Y),
    (codes::BTN_TL, Button::L1),
    (codes::BTN_TR, Button::R1),
    (codes::BTN_TL2, Button::L2),
    (codes::BTN_TR2, Button::R2),
    (codes::BTN_SELECT, Button::Select),
    (codes::BTN_START, Button::Start),
    (codes::BTN_MODE, Button::Menu),
    (codes::KEY_UP, Button::Up),
    (codes::KEY_DOWN, Button::Down),
    (codes::KEY_LEFT, Button::Left),
    (codes::KEY_RIGHT, Button::Right),
    (codes::BTN_DPAD_UP, Button::Up),
    (codes::BTN_DPAD_DOWN, Button::Down),
    (codes::BTN_DPAD_LEFT, Button::Left),
    (codes::BTN_DPAD_RIGHT, Button::Right),
    (codes::KEY_VOLUMEUP, Button::VolUp),
    (codes::KEY_VOLUMEDOWN, Button::VolDown),
    (codes::KEY_POWER, Button::Power),
];

/// All readable `/dev/input/event*` nodes, mapped through one `KeyMap`.
pub struct EvdevSource {
    _todo: (),
}

impl EvdevSource {
    pub fn open_all(keymap: KeyMap) -> Self {
        let _ = keymap;
        todo!("task 05")
    }

    /// How many device nodes are open.
    pub fn device_count(&self) -> usize {
        todo!("task 05")
    }

    /// Everything pending, stamped `now`. Never blocks.
    pub fn poll(&mut self, now: Instant) -> Vec<Event> {
        let _ = now;
        todo!("task 05")
    }

    /// Map one raw event with this source's keymap — the pure core of `poll`, exposed so a
    /// test can drive it without a device.
    pub fn map_raw(keymap: &KeyMap, raw: RawEvent, now: Instant) -> Option<Event> {
        let _ = (keymap, raw, now);
        todo!("task 05")
    }
}
