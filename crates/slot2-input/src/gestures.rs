//! Button events → UI actions. Implementation notes for task 05 (see tasks/05-input.md).
//!
//! Semantics, all driven by the `Instant`s on the events (never by the wall clock, so
//! tests can feed arbitrary times):
//!
//! - **Tap**: press then release within `hold` → `Action::Tap(b)` on release.
//! - **Hold**: pressed for `hold` or longer → `Action::Hold(b)` fires ONCE, from whichever
//!   of `feed`/`tick` first observes a time ≥ press + hold. The eventual release then
//!   produces nothing (no tap). `hold_progress(b, now)` is `Some(0..=1)` while the button
//!   is down and the hold has not fired yet, `None` otherwise — the UI draws a ring with it.
//! - **Double-tap**: only for buttons in `config.double_tap_buttons`. A tap followed by a
//!   second press within `double_tap` of the first release → `Action::DoubleTap(b)` on the
//!   second press; that second press's own tap/hold is suppressed. A first tap on such a
//!   button is delayed: it is emitted from `tick`/`feed` once `double_tap` has passed
//!   without a second press. Buttons not in the list emit `Tap` immediately on release.
//! - **Chord**: while `config.modifier` (SELECT) is held, pressing another button `b` emits
//!   `Action::Chord(b)` immediately on `b`'s press, suppresses `b`'s tap/hold, and marks the
//!   modifier as "used" so its own release produces no tap. A modifier pressed and released
//!   alone is a normal tap (SELECT alone may still mean something).
//! - **Raw**: every press/release is also emitted as `Action::Down(b)` / `Action::Up(b)`
//!   before any gesture action from the same event, so a game sees keys exactly as pressed.
//! - Axis and lid events are ignored by `Gestures` (they are for `State`).
//! - Releases for buttons never seen pressed are ignored.

use std::time::{Duration, Instant};

use crate::{Button, Event};

#[derive(Clone, Debug, PartialEq)]
pub struct GestureConfig {
    pub hold: Duration,
    pub double_tap: Duration,
    pub modifier: Button,
    pub double_tap_buttons: Vec<Button>,
}

impl Default for GestureConfig {
    /// MENU hold = eject, SELECT chords, R2 double-tap = lock fast-forward (D-23).
    fn default() -> Self {
        GestureConfig {
            hold: Duration::from_millis(600),
            double_tap: Duration::from_millis(250),
            modifier: Button::Select,
            double_tap_buttons: vec![Button::R2],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Down(Button),
    Up(Button),
    Tap(Button),
    Hold(Button),
    DoubleTap(Button),
    /// `modifier + button`.
    Chord(Button),
}

#[derive(Clone, Copy, Default)]
struct ButtonState {
    press_time: Option<Instant>,
    release_time: Option<Instant>,
    hold_fired: bool,
    is_chorded: bool,
    pending_tap: bool,
}

pub struct Gestures {
    config: GestureConfig,
    states: [ButtonState; 18],
    modifier_used: bool,
}

impl Gestures {
    pub fn new(config: GestureConfig) -> Self {
        Gestures {
            config,
            states: [ButtonState::default(); 18],
            modifier_used: false,
        }
    }

    pub fn config(&self) -> &GestureConfig {
        &self.config
    }

    fn check_matures(&mut self, now: Instant) -> Vec<Action> {
        let mut actions = Vec::new();
        for b in Button::ALL {
            let state = &mut self.states[b as usize];
            if let Some(press_time) = state.press_time {
                if !state.hold_fired && !state.is_chorded && now >= press_time + self.config.hold {
                    state.hold_fired = true;
                    actions.push(Action::Hold(b));
                }
            }
        }
        for b in Button::ALL {
            let state = &mut self.states[b as usize];
            if state.pending_tap {
                if let Some(release_time) = state.release_time {
                    if now > release_time + self.config.double_tap {
                        state.pending_tap = false;
                        actions.push(Action::Tap(b));
                    }
                }
            }
        }
        actions
    }

    /// Feed one event; returns the actions it produced, in order.
    pub fn feed(&mut self, event: &Event) -> Vec<Action> {
        let (b, pressed, at) = match event {
            Event::Button {
                button,
                pressed,
                at,
            } => (*button, *pressed, *at),
            _ => return Vec::new(),
        };

        let mut actions = Vec::new();
        if pressed {
            actions.push(Action::Down(b));
        } else {
            if self.states[b as usize].press_time.is_none() {
                return Vec::new();
            }
            actions.push(Action::Up(b));
        }

        actions.extend(self.check_matures(at));

        let modifier_pressed = self.states[self.config.modifier as usize]
            .press_time
            .is_some();
        let state = &mut self.states[b as usize];
        if pressed {
            if state.pending_tap {
                state.pending_tap = false;
                actions.push(Action::DoubleTap(b));
                state.press_time = Some(at);
                state.is_chorded = true;
                state.hold_fired = false;
            } else {
                if modifier_pressed && b != self.config.modifier {
                    actions.push(Action::Chord(b));
                    state.press_time = Some(at);
                    state.is_chorded = true;
                    state.hold_fired = false;
                    self.modifier_used = true;
                } else {
                    state.press_time = Some(at);
                    state.is_chorded = false;
                    state.hold_fired = false;
                    if b == self.config.modifier {
                        self.modifier_used = false;
                    }
                }
            }
        } else {
            if !state.is_chorded && !state.hold_fired {
                if b == self.config.modifier && self.modifier_used {
                } else if self.config.double_tap_buttons.contains(&b) {
                    state.pending_tap = true;
                    state.release_time = Some(at);
                } else {
                    actions.push(Action::Tap(b));
                }
            }
            state.press_time = None;
        }
        actions
    }

    /// Advance time without an event: fires holds that have matured and delayed taps
    /// whose double-tap window closed.
    pub fn tick(&mut self, now: Instant) -> Vec<Action> {
        self.check_matures(now)
    }

    /// 0.0 at press, 1.0 at the moment the hold fires; `None` when not applicable.
    pub fn hold_progress(&self, button: Button, now: Instant) -> Option<f32> {
        let state = &self.states[button as usize];
        if let Some(press_time) = state.press_time {
            if !state.hold_fired && !state.is_chorded {
                let elapsed = now.saturating_duration_since(press_time);
                let progress = elapsed.as_secs_f32() / self.config.hold.as_secs_f32();
                return Some(progress.min(1.0));
            }
        }
        None
    }
}
