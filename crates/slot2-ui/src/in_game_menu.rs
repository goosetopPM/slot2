//! The in-game menu: seven choices over a dimmed game frame. Mirrors the layout and
//! drawing style of the power menu on purpose: no generic menu framework, only the
//! shared UiCtx / draw_spans / Canvas pieces.
//!
//! Draw order: full-panel dim (BLACK at alpha 0.6), box centred in the safe area
//! (BACKDROP), title ingame-menu-title at PX_BODY in INK_DIM centred with top at
//! box_y + PAD, the seven INGAME_ITEMS rows each ROW_H tall starting at box_y + PAD + 28
//! (selected row highlighted with INK at alpha 0.15, labels PX_TITLE in INK centred),
//! then the hint-select and hint-back spans at PX_HINT in INK_DIM on one centred line at
//! the bottom of the box.
//!
//! draw overlays the current game frame and does not clear.

use slot2_gfx::{Canvas, Color};

use crate::UiCtx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InGameChoice {
    Continue,
    SaveState,
    Cheats,
    Display,
    Core,
    Device,
    Eject,
}

impl InGameChoice {
    /// The message key for the row's label.
    pub const fn key(self) -> &'static str {
        match self {
            InGameChoice::Continue => "ingame-continue",
            InGameChoice::SaveState => "ingame-save-state",
            InGameChoice::Cheats => "ingame-cheats",
            InGameChoice::Display => "ingame-display",
            InGameChoice::Core => "ingame-core",
            InGameChoice::Device => "ingame-device",
            InGameChoice::Eject => "ingame-eject",
        }
    }
}

pub const INGAME_ITEMS: [InGameChoice; 7] = [
    InGameChoice::Continue,
    InGameChoice::SaveState,
    InGameChoice::Cheats,
    InGameChoice::Display,
    InGameChoice::Core,
    InGameChoice::Device,
    InGameChoice::Eject,
];
pub const BOX_W: f32 = 340.0;
// PAD + title area + 7 rows + hint area, so everything fits 640x480 with room to spare.
pub const BOX_H: f32 = 344.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 36.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InGameMenu {
    // Private so callers cannot construct an out-of-range index; the modulo in up/down
    // keeps the invariant. Read it back with selected_index.
    selected: usize,
}

impl InGameMenu {
    /// Index of the highlighted row; always less than INGAME_ITEMS.len().
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn up(&mut self) {
        self.selected = (self.selected + INGAME_ITEMS.len() - 1) % INGAME_ITEMS.len();
    }

    pub fn down(&mut self) {
        self.selected = (self.selected + 1) % INGAME_ITEMS.len();
    }

    pub fn choice(&self) -> InGameChoice {
        INGAME_ITEMS[self.selected]
    }

    /// Top-left of the box on the panel, centred in the safe area.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    /// Panel y of row i's top.
    pub fn row_y(ctx: &UiCtx, i: usize) -> f32 {
        Self::box_origin(ctx).1 + PAD + 28.0 + i as f32 * ROW_H
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        let title_spans = ctx.i18n.spans("ingame-menu-title", &[]);
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

        for (i, choice) in INGAME_ITEMS.iter().enumerate() {
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
