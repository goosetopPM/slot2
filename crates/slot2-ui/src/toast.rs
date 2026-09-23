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

/// The gutter the toast keeps either side of it, per DESIGN §5.
const MARGIN: f32 = 16.0;

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
        Toast {
            key: key.into(),
            args,
            age: 0.0,
        }
    }

    /// Advance it. `dt` in seconds.
    pub fn tick(&mut self, dt: f32) {
        self.age += dt;
    }
    pub fn key(&self) -> &str {
        &self.key
    }

    /// True once it has faded out and the caller should drop it.
    pub fn done(&self) -> bool {
        self.age >= TOAST_S
    }

    /// How lit it is: up through the fade in, one for most of its life, down at the end.
    pub fn alpha(&self) -> f32 {
        if self.age < TOAST_FADE_S {
            self.age / TOAST_FADE_S
        } else if self.age > TOAST_S - TOAST_FADE_S {
            ((TOAST_S - self.age) / TOAST_FADE_S).max(0.0)
        } else {
            1.0
        }
    }

    /// Draw it, plate and all. Does nothing once it is `done`.
    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, safe: &SafeArea) {
        if self.done() {
            return;
        }

        let args: Vec<(&str, slot2_i18n::Arg)> = self
            .args
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        let spans = ctx.i18n.spans(&self.key, &args);

        // Shrink to fit rather than run off the sides.
        //
        // A game's name comes from a filename and has no length anyone agreed to — "Shin
        // Megami Tensei Devil Survivor Overclocked Special Edition" is 586 px at the body
        // size, which with the padding is already wider than the usable part of a 640 panel.
        // Centring a plate wider than the screen puts the first and last words off both
        // edges, so the one part of the message that is always cut is the part that says
        // which game.
        //
        // Scaled rather than wrapped or clipped: a title long enough to need this was never
        // going to be read at a glance, and small text beats text that leaves the screen.
        // Measured again afterwards because a glyph's advance is not exactly linear in the
        // size it is laid out at, and "nearly fits" is the same bug.
        let max_w = safe.panel_w as f32 - 2.0 * MARGIN - 2.0 * PAD_X;
        let mut px = crate::PX_BODY;
        let mut sw = crate::face::spans_width(ctx, &spans, px);
        if sw > max_w {
            px *= max_w / sw;
            sw = crate::face::spans_width(ctx, &spans, px);
        }

        let w = sw + 2.0 * PAD_X;
        let h = px + 2.0 * PAD_Y;

        let x = ((safe.panel_w as f32 - w) / 2.0).max(MARGIN);
        let y = safe.panel_h as f32 - MOUTH_H - TOAST_ABOVE_SLOT - h;

        let alpha = self.alpha();
        let color = slot2_gfx::Color::WHITE.with_alpha(alpha);
        let plate = PLATE.with_alpha(alpha);

        canvas.rect(x, y, w, h, plate);
        crate::draw_spans(canvas, ctx, &spans, px, x + PAD_X, y + PAD_Y, color);
    }
}
