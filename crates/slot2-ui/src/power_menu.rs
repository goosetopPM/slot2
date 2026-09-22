//! The power menu: three choices over a dimmed screen. Implementation notes for task 05
//! (see tasks/05-input.md):
//!
//! ```text
//!   dim        canvas.rect over the WHOLE panel, Color::BLACK.with_alpha(0.6)
//!   box        centred in the safe area: BOX_W x BOX_H, BACKDROP colour (splash::BACKDROP)
//!   title      i18n `power-menu-title`, PX_BODY, INK_DIM, centred in the box, top at box_y + PAD
//!   items      ITEMS in order, each a row ROW_H tall starting at box_y + PAD + 28:
//!                selected row: rect (box_x + PAD, row_y, BOX_W - 2*PAD, ROW_H) in INK.with_alpha(0.15)
//!                label: i18n key from `PowerChoice::key`, PX_TITLE, INK, centred in the box,
//!                       top at row_y + (ROW_H - line_h) / 2 where line_h is the PX_TITLE line height
//!   hints      i18n spans `hint-select` then `hint-back`, PX_HINT, INK_DIM, on one line centred
//!              in the box, top at box_y + BOX_H - PAD - line_h(PX_HINT), with a gap of PX_HINT
//!              between the two
//! ```
//! Draw nothing else. `draw` does not clear: it is drawn over whatever screen was showing.

use slot2_gfx::{Canvas, Color};

use crate::UiCtx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerChoice {
    Resume,
    Restart,
    PowerOff,
}

impl PowerChoice {
    /// The message key for the row's label.
    pub const fn key(self) -> &'static str {
        match self {
            PowerChoice::Resume => "resume",
            PowerChoice::Restart => "power-restart",
            PowerChoice::PowerOff => "power-off",
        }
    }
}

pub const ITEMS: [PowerChoice; 3] = [
    PowerChoice::Resume,
    PowerChoice::Restart,
    PowerChoice::PowerOff,
];
pub const BOX_W: f32 = 300.0;
pub const BOX_H: f32 = 220.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 44.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PowerMenu {
    pub selected: usize,
}

impl PowerMenu {
    pub fn up(&mut self) {
        self.selected = (self.selected + ITEMS.len() - 1) % ITEMS.len();
    }

    pub fn down(&mut self) {
        self.selected = (self.selected + 1) % ITEMS.len();
    }

    pub fn choice(&self) -> PowerChoice {
        ITEMS[self.selected]
    }

    /// Top-left of the box on the panel.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    /// Panel y of row `i`'s top.
    pub fn row_y(ctx: &UiCtx, i: usize) -> f32 {
        Self::box_origin(ctx).1 + PAD + 28.0 + i as f32 * ROW_H
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        let title_spans = ctx.i18n.spans("power-menu-title", &[]);
        let title_w = crate::face::spans_width(ctx, &title_spans, crate::PX_BODY);
        crate::draw_spans(
            canvas,
            ctx,
            &title_spans,
            crate::PX_BODY,
            bx + (BOX_W - title_w) / 2.0,
            by + PAD,
            crate::splash::INK_DIM,
        );

        for (i, choice) in ITEMS.iter().enumerate() {
            let row_y = Self::row_y(ctx, i);
            if i == self.selected {
                canvas.rect(
                    bx + PAD,
                    row_y,
                    BOX_W - 2.0 * PAD,
                    ROW_H,
                    crate::splash::INK.with_alpha(0.15),
                );
            }
            let label_spans = ctx.i18n.spans(choice.key(), &[]);
            let label_w = crate::face::spans_width(ctx, &label_spans, crate::PX_TITLE);
            let line_h = ctx.fonts.measure("", crate::PX_TITLE).line_height as f32;
            crate::draw_spans(
                canvas,
                ctx,
                &label_spans,
                crate::PX_TITLE,
                bx + (BOX_W - label_w) / 2.0,
                row_y + (ROW_H - line_h) / 2.0,
                crate::splash::INK,
            );
        }

        let hint_select = ctx.i18n.spans("hint-select", &[]);
        let hint_back = ctx.i18n.spans("hint-back", &[]);
        let w_select = crate::face::spans_width(ctx, &hint_select, crate::PX_HINT);
        let w_back = crate::face::spans_width(ctx, &hint_back, crate::PX_HINT);
        let total_hint_w = w_select + crate::PX_HINT + w_back;
        let hint_x = bx + (BOX_W - total_hint_w) / 2.0;
        let hint_y = by + BOX_H - PAD - crate::PX_HINT;

        crate::draw_spans(
            canvas,
            ctx,
            &hint_select,
            crate::PX_HINT,
            hint_x,
            hint_y,
            crate::splash::INK_DIM,
        );
        crate::draw_spans(
            canvas,
            ctx,
            &hint_back,
            crate::PX_HINT,
            hint_x + w_select + crate::PX_HINT,
            hint_y,
            crate::splash::INK_DIM,
        );
    }
}
