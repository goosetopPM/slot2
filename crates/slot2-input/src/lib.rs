//! Logical buttons and axes, evdev reader, host key map, gestures.
//!
//! Three layers, bottom up:
//!
//! 1. **Sources** turn hardware into [`Event`]s with timestamps: [`EvdevSource`] on the
//!    device (raw `/dev/input/event*`, mapped through a [`KeyMap`]), and on the host a
//!    winit key code mapped by [`host_map`].
//! 2. [`State`] is the current pressed set and axis values — what an emulator core polls.
//! 3. [`Gestures`] turns button events into UI [`Action`]s: tap, hold, double-tap, chord.
//!    Raw presses still flow through so a game and the UI can see the same event.
//!
//! Design: docs/DESIGN.md §10, decisions D-15 (axes are read now, mapped later), D-23
//! (MENU tap = menu, MENU hold = eject; no double-tap on MENU).

pub mod button;
pub mod evdev;
pub mod gestures;
#[cfg(feature = "host")]
pub mod host;

pub use button::Button;
pub use evdev::{parse_input_event, EvdevSource, KeyMap, RawEvent, DEFAULT_H700_KEYMAP};
pub use gestures::{Action, GestureConfig, Gestures};
#[cfg(feature = "host")]
pub use host::host_map;

use std::time::Instant;

/// Analogue inputs. Values are -1.0..=1.0 (sticks) or 0.0..=1.0 (triggers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    L2,
    R2,
}

/// One thing that happened, with when it happened (so gestures are testable without a
/// clock).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Button {
        button: Button,
        pressed: bool,
        at: Instant,
    },
    Axis {
        axis: Axis,
        value: f32,
        at: Instant,
    },
    /// SP-style lid switch. `closed == true` when shut.
    Lid {
        closed: bool,
        at: Instant,
    },
}

impl Event {
    pub fn at(&self) -> Instant {
        match self {
            Event::Button { at, .. } | Event::Axis { at, .. } | Event::Lid { at, .. } => *at,
        }
    }
}

/// Current input state, updated by feeding events. What a core polls each frame.
#[derive(Clone, Debug, Default)]
pub struct State {
    pressed: u32,
    axes: [f32; 6],
    lid_closed: bool,
}

impl State {
    pub fn feed(&mut self, e: &Event) {
        match *e {
            Event::Button {
                button, pressed, ..
            } => {
                let bit = 1u32 << button as u32;
                if pressed {
                    self.pressed |= bit;
                } else {
                    self.pressed &= !bit;
                }
            }
            Event::Axis { axis, value, .. } => self.axes[axis as usize] = value.clamp(-1.0, 1.0),
            Event::Lid { closed, .. } => self.lid_closed = closed,
        }
    }

    pub fn pressed(&self, b: Button) -> bool {
        self.pressed & (1 << b as u32) != 0
    }

    pub fn axis(&self, a: Axis) -> f32 {
        self.axes[a as usize]
    }

    pub fn lid_closed(&self) -> bool {
        self.lid_closed
    }

    /// Every pressed button, in `Button::ALL` order.
    pub fn held(&self) -> Vec<Button> {
        Button::ALL
            .into_iter()
            .filter(|b| self.pressed(*b))
            .collect()
    }
}
