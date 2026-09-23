//! A line that says what happened, and then goes away.
//!
//! It holds a message *key*, never a sentence. DESIGN §8: whole sentences only, never a
//! fragment the code glues together — the language decides the word order, and a toast built
//! by concatenation in Rust is a toast that reads as machine translation in every language
//! but the one it was written in.

use slot2_gfx::{Canvas, Color};
use slot2_i18n::Arg;

use crate::layout::SafeArea;
use crate::shelf::MOUTH_H;
use crate::UiCtx;

/// How long a toast stays up, fade included.
///
/// Three seconds is about twice as long as a short line takes to read, which is the margin a
/// player who was looking at the cartridge rather than the words needs.
pub const TOAST_S: f32 = 3.0;

/// The fade at each end. Long enough not to be a cut, short enough that the message is at
/// full strength for most of its life.
pub const TOAST_FADE_S: f32 = 0.35;

/// How far above the slot the toast sits.
///
/// In the gap between the feet of the row and the band, which is empty on every panel
/// because the row is placed against the slot rather than the middle of the screen. Measured
/// from the slot for the same reason the row is: a taller panel grows that gap and the toast
/// should stay with the machine, not drift up with the extra space.
pub const TOAST_ABOVE_SLOT: f32 = 46.0;

/// Padding around the text, and the height of the plate it sits on.
const PAD_X: f32 = 14.0;
const PAD_Y: f32 = 7.0;

const PLATE: Color = Color::from_rgb8(0x1E, 0x21, 0x26);

/// One message on screen, decaying on its own clock.
pub struct Toast {
    key: String,
    args: Vec<(String, Arg)>,
    age: f32,
}

impl Toast {
    /// A toast for `key`, starting now. `args` are the message's variables, by name.
    pub fn new(key: impl Into<String>, args: Vec<(String, Arg)>) -> Toast {
        todo!()
    }

    /// Advance it. `dt` in seconds.
    pub fn tick(&mut self, dt: f32) {
        todo!()
    }

    /// True once it has faded out and the caller should drop it.
    pub fn done(&self) -> bool {
        todo!()
    }

    /// How lit it is: up through the fade in, one for most of its life, down at the end.
    pub fn alpha(&self) -> f32 {
        todo!()
    }

    /// Draw it, plate and all. Does nothing once it is `done`.
    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, safe: &SafeArea) {
        todo!()
    }
}
