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
//!
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

/// Hat axes. Most handhelds report their D-pad as a hat rather than as keys, so these are
/// translated into `Button::Left/Right/Up/Down` press and release pairs.
pub const ABS_HAT0X: u16 = 16;
pub const ABS_HAT0Y: u16 = 17;

/// One decoded `input_event`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawEvent {
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

/// Decode one 24-byte little-endian `input_event`. `None` if `bytes` is too short.
pub fn parse_input_event(bytes: &[u8]) -> Option<RawEvent> {
    if bytes.len() < 24 {
        return None;
    }
    let type_ = u16::from_le_bytes([bytes[16], bytes[17]]);
    let code = u16::from_le_bytes([bytes[18], bytes[19]]);
    let value = i32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Some(RawEvent { type_, code, value })
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
    keymap: KeyMap,
    devices: Vec<std::fs::File>,
    /// Last value seen on each hat axis, so a move produces a release of the old direction
    /// and a press of the new one.
    hat: (i32, i32),
    /// Every raw event since the last `take_raw_log`, when logging is on.
    raw_log: Option<Vec<(usize, RawEvent)>>,
}

/// Which buttons a hat value means: negative is left/up, positive right/down.
fn hat_buttons(axis: u16) -> (Button, Button) {
    if axis == ABS_HAT0X {
        (Button::Left, Button::Right)
    } else {
        (Button::Up, Button::Down)
    }
}

impl EvdevSource {
    pub fn open_all(keymap: KeyMap) -> Self {
        #[allow(unused_mut)]
        let mut devices = Vec::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            for i in 0..32 {
                let path = format!("/dev/input/event{}", i);
                let file = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(0o4000) // O_NONBLOCK
                    .open(&path);
                match file {
                    Ok(f) => devices.push(f),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => eprintln!("slot2: failed to open {}: {}", path, e),
                }
            }
        }
        EvdevSource {
            keymap,
            devices,
            hat: (0, 0),
            raw_log: None,
        }
    }

    /// How many device nodes are open.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Everything pending, stamped `now`. Never blocks.
    pub fn poll(&mut self, now: Instant) -> Vec<Event> {
        let mut events = Vec::new();
        let mut buf = [0u8; 24 * 64];
        for (index, file) in self.devices.iter_mut().enumerate() {
            use std::io::Read;
            loop {
                match file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        for chunk in buf[..n].as_chunks::<24>().0 {
                            if let Some(raw) = parse_input_event(chunk) {
                                if let Some(log) = self.raw_log.as_mut() {
                                    if raw.type_ != EV_SYN {
                                        log.push((index, raw));
                                    }
                                }
                                // A D-pad reported as a hat becomes press and release
                                // pairs; `map_raw` cannot do this because it needs the
                                // previous value.
                                if raw.type_ == EV_ABS
                                    && (raw.code == ABS_HAT0X || raw.code == ABS_HAT0Y)
                                {
                                    let (neg, pos) = hat_buttons(raw.code);
                                    let last = if raw.code == ABS_HAT0X {
                                        std::mem::replace(&mut self.hat.0, raw.value)
                                    } else {
                                        std::mem::replace(&mut self.hat.1, raw.value)
                                    };
                                    if last < 0 {
                                        events.push(Event::Button {
                                            button: neg,
                                            pressed: false,
                                            at: now,
                                        });
                                    } else if last > 0 {
                                        events.push(Event::Button {
                                            button: pos,
                                            pressed: false,
                                            at: now,
                                        });
                                    }
                                    if raw.value < 0 {
                                        events.push(Event::Button {
                                            button: neg,
                                            pressed: true,
                                            at: now,
                                        });
                                    } else if raw.value > 0 {
                                        events.push(Event::Button {
                                            button: pos,
                                            pressed: true,
                                            at: now,
                                        });
                                    }
                                    continue;
                                }
                                if let Some(ev) = Self::map_raw(&self.keymap, raw, now) {
                                    events.push(ev);
                                }
                            }
                        }
                        if n < buf.len() {
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
        }
        events
    }

    /// Start recording every raw event (except `EV_SYN`) with the index of the device it
    /// came from. What the on-device input probe reads to learn a board's real codes.
    pub fn log_raw(&mut self, on: bool) {
        self.raw_log = if on { Some(Vec::new()) } else { None };
    }

    /// Take what has been recorded since the last call.
    pub fn take_raw_log(&mut self) -> Vec<(usize, RawEvent)> {
        match self.raw_log.as_mut() {
            Some(v) => std::mem::take(v),
            None => Vec::new(),
        }
    }

    /// Map one raw event with this source's keymap — the pure core of `poll`, exposed so a
    /// test can drive it without a device.
    pub fn map_raw(keymap: &KeyMap, raw: RawEvent, now: Instant) -> Option<Event> {
        match raw.type_ {
            EV_KEY if raw.value == 0 || raw.value == 1 => {
                if let Some(button) = keymap.button(raw.code) {
                    return Some(Event::Button {
                        button,
                        pressed: raw.value == 1,
                        at: now,
                    });
                }
            }
            EV_ABS => {
                let (axis, range) = match raw.code {
                    0 => (Some(crate::Axis::LeftX), keymap.stick_range),
                    1 => (Some(crate::Axis::LeftY), keymap.stick_range),
                    3 => (Some(crate::Axis::RightX), keymap.stick_range),
                    4 => (Some(crate::Axis::RightY), keymap.stick_range),
                    2 => (Some(crate::Axis::L2), keymap.trigger_range),
                    5 => (Some(crate::Axis::R2), keymap.trigger_range),
                    _ => (None, (0, 0)),
                };
                if let Some(axis) = axis {
                    let (min, max) = range;
                    if max != min {
                        let mut v = (raw.value - min) as f32 / (max - min) as f32;
                        v = v.clamp(0.0, 1.0);
                        let value = if raw.code == 2 || raw.code == 5 {
                            v
                        } else {
                            v * 2.0 - 1.0
                        };
                        return Some(Event::Axis {
                            axis,
                            value,
                            at: now,
                        });
                    }
                }
            }
            EV_SW if raw.code == SW_LID => {
                return Some(Event::Lid {
                    closed: raw.value == 1,
                    at: now,
                });
            }
            _ => {}
        }
        None
    }
}
