//! A plain list of carts, until the shelf replaces it in M3. Implementation notes for
//! task 09 (see tasks/09-play.md):
//!
//! ```text
//!   clear      splash::BACKDROP
//!   heading    the platform's folder name + " (n)" e.g. "GBA (12)", PX_TITLE, INK_DIM,
//!              safe-area left at x = 32, top at 24
//!   rows       up to VISIBLE titles, PX_BODY, from safe y = 72, ROW_H apart, x = 48.
//!              The selected row gets a rect (safe x 32, row y - 4, SAFE_W - 64, ROW_H)
//!              in INK.with_alpha(0.15) drawn BEFORE its label; labels are INK, or
//!              INK_DIM for the rest.
//!   scrolling  keep the selection visible: the window starts at `scroll()`, which moves
//!              so that `selected` is always within it (scroll to selected - VISIBLE + 1
//!              when below, to selected when above).
//!   empty      when there are no carts: the message key `list-empty`, PX_BODY, INK_DIM,
//!              centred in the safe area at y = 200.
//!   hints      i18n spans `hint-play` then `hint-switch`, PX_HINT, INK_DIM, one line
//!              centred in the safe area at y = 440, PX_HINT of gap between them.
//! ```
//! Selection and platform switching are the caller's (`App`); this type only holds the
//! indices and draws.

use slot2_gfx::Canvas;
use slot2_store::Cart;

use crate::UiCtx;

pub const ROW_H: f32 = 28.0;
pub const VISIBLE: usize = 12;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameList {
    pub selected: usize,
    scroll: usize,
}

impl GameList {
    /// Move down one, stopping at the end. No wrap: a long list should not jump.
    pub fn down(&mut self, len: usize) {
        let _ = len;
        todo!("task 09")
    }

    pub fn up(&mut self) {
        todo!("task 09")
    }

    /// Clamp the selection when the list changes under it (platform switch).
    pub fn clamp(&mut self, len: usize) {
        let _ = len;
        todo!("task 09")
    }

    /// First visible row.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn draw(
        &self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        platform_folder: &str,
        carts: &[Cart],
    ) {
        let _ = (canvas, ctx, platform_folder, carts);
        todo!("task 09")
    }
}
