//! The overlay submenu: the platform's own default, an explicit on, or an explicit off.
//!
//! A screen of its own, like the shader and overscan submenus and for the same reason: the
//! rows answer a question the display rows do not. App wiring (the Display row that opens
//! this, instant preview and saving) is a later task; this is the component and nothing else.
//!
//! The rows hold the card's own meanings, not the resolved picture. `None` is the *absence*
//! of a setting, which inherits whatever the platform's default decides later; `Some(true)`
//! asks for the overlay on this game and `Some(false)` turns it off. `None` draws nothing
//! while the platform default is "no overlay"; `Some(true)` draws a valid card or built-in
//! picture when one exists. The rows stay separate so an inherited default never becomes a
//! per-game override the player did not make.
//!
//! Nothing here looks at whether a picture exists. The overlay is one PNG per platform and
//! geometry with no name or path to choose, so the menu cannot offer one, and a game whose
//! picture is missing still gets to keep the choice it made.
//!
//! Draw order, mirroring the display, in-game and power menus (same panel, spacing, text sizes
//! and palette): full-panel dim (BLACK at alpha 0.6), box centred in the safe area (BACKDROP),
//! title at PX_BODY in INK_DIM centred with top at box_y + PAD, the three ROWS each ROW_H tall
//! starting at box_y + PAD + 28 (the selected row highlighted with INK at alpha 0.15, labels
//! PX_TITLE in INK centred), then the hint-select and hint-back spans at PX_HINT in INK_DIM on
//! one centred line at the bottom of the box.
//!
//! draw overlays the paused game's last frame and does not clear.

use slot2_gfx::{Canvas, Color};

use crate::UiCtx;

/// The three choices, in the order the rows are drawn. `None` is the platform default, which
/// is what a game nobody has overridden is on; `Some(true)` asks for the overlay on this game
/// and `Some(false)` turns it off for this game.
pub const ROWS: [Option<bool>; 3] = [None, Some(true), Some(false)];

pub const BOX_W: f32 = 360.0;
// PAD + title area + 3 rows + hint area, so everything fits 640x480 with room to spare.
pub const BOX_H: f32 = 208.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 36.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

/// The overlay submenu, open on one of its three rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayMenu {
    // Private so callers cannot construct a row that is not there; the modulo in up/down keeps
    // the invariant. Read it back with `selected`.
    selected: usize,
}

impl OverlayMenu {
    /// The menu for a game whose setting is `overlay`, which is `None` when the key is absent:
    /// that row starts selected, so opening the menu shows what the card says now.
    pub fn new(overlay: Option<bool>) -> Self {
        OverlayMenu {
            selected: ROWS.iter().position(|row| *row == overlay).unwrap_or(0),
        }
    }

    /// The setting the highlighted row would choose. `None` is the platform default.
    pub fn selected(&self) -> Option<bool> {
        ROWS[self.selected]
    }

    /// The message key for a row's label.
    ///
    /// An exhaustive match on the two-valued setting rather than its index: a third state
    /// stops compiling here, where somebody has to decide what it is called on screen. The
    /// platform default has no choice of its own to name, so it reuses the display menu's word
    /// for the same meaning.
    pub const fn key(choice: Option<bool>) -> &'static str {
        match choice {
            None => "display-platform-default",
            Some(true) => "overlay-on",
            Some(false) => "overlay-off",
        }
    }

    pub fn up(&mut self) {
        self.selected = (self.selected + ROWS.len() - 1) % ROWS.len();
    }

    pub fn down(&mut self) {
        self.selected = (self.selected + 1) % ROWS.len();
    }

    /// Top-left of the box on the panel, centred in the safe area.
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

        // The title is the setting the player is choosing, not the row they came through.
        let title_spans = ctx.i18n.spans("overlay-title", &[]);
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

        for (i, choice) in ROWS.iter().enumerate() {
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
            let label_spans = ctx.i18n.spans(Self::key(*choice), &[]);
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
