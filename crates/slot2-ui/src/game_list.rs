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
        if len == 0 {
            self.selected = 0;
        } else if self.selected + 1 < len {
            self.selected += 1;
        }
        self.follow(len);
    }

    pub fn up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
    }

    /// Clamp the selection when the list changes under it (platform switch).
    pub fn clamp(&mut self, len: usize) {
        self.selected = self.selected.min(len.saturating_sub(1));
        self.follow(len);
    }

    /// Keep the selection inside the visible window.
    fn follow(&mut self, len: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + VISIBLE {
            self.scroll = self.selected + 1 - VISIBLE;
        }
        let max_scroll = len.saturating_sub(VISIBLE);
        self.scroll = self.scroll.min(max_scroll);
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
        canvas.clear(crate::splash::BACKDROP);

        let heading = format!("{platform_folder} ({})", carts.len());
        crate::face::draw_text(
            canvas,
            ctx,
            &heading,
            crate::PX_TITLE,
            ctx.safe.px(32.0),
            ctx.safe.py(24.0),
            crate::splash::INK_DIM,
        );

        if carts.is_empty() {
            let text = ctx.i18n.t("list-empty");
            let w = crate::face::measure(ctx, &text, crate::PX_BODY);
            crate::face::draw_text(
                canvas,
                ctx,
                &text,
                crate::PX_BODY,
                ctx.safe.centre_x(w),
                ctx.safe.py(200.0),
                crate::splash::INK_DIM,
            );
        } else {
            for (row, cart) in carts.iter().skip(self.scroll).take(VISIBLE).enumerate() {
                let index = self.scroll + row;
                let y = ctx.safe.py(72.0 + row as f32 * ROW_H);
                let ink = if index == self.selected {
                    canvas.rect(
                        ctx.safe.px(32.0),
                        y - 4.0,
                        crate::SAFE_W as f32 - 64.0,
                        ROW_H,
                        crate::splash::INK.with_alpha(0.15),
                    );
                    crate::splash::INK
                } else {
                    crate::splash::INK_DIM
                };
                crate::face::draw_text(
                    canvas,
                    ctx,
                    &cart.title,
                    crate::PX_BODY,
                    ctx.safe.px(48.0),
                    y,
                    ink,
                );
            }
        }

        let play = ctx.i18n.spans("hint-play", &[]);
        let switch = ctx.i18n.spans("hint-switch", &[]);
        let w_play = crate::face::spans_width(ctx, &play, crate::PX_HINT);
        let w_switch = crate::face::spans_width(ctx, &switch, crate::PX_HINT);
        let x = ctx.safe.centre_x(w_play + crate::PX_HINT + w_switch);
        let y = ctx.safe.py(440.0);
        crate::draw_spans(
            canvas,
            ctx,
            &play,
            crate::PX_HINT,
            x,
            y,
            crate::splash::INK_DIM,
        );
        crate::draw_spans(
            canvas,
            ctx,
            &switch,
            crate::PX_HINT,
            x + w_play + crate::PX_HINT,
            y,
            crate::splash::INK_DIM,
        );
    }
}
