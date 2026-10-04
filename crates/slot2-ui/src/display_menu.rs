//! The Display submenu: which of the four scale settings a game is drawn with, and the rows
//! that open the shader, overscan and overlay screens.
//!
//! Reached from the in-game menu's Display row (task 46 wires that). Mirrors the in-game and
//! power menus deliberately: the same panel, spacing, text sizes and palette, and no menu
//! framework of its own. Three of the rows are ways into other screens rather than settings: the
//! shader choice is a screen of its own ([`crate::shader_menu`]) because it answers a different
//! question, and the overscan and overlay choices are screens of their own
//! ([`crate::overscan_menu`], [`crate::overlay_menu`]) for the same reason.
//!
//! The overscan row exists only where the platform's registry entry actually crops something:
//! a screen where every choice draws the same picture is that same lie in a longer form. The
//! overlay row is offered everywhere, because it is not an asset browser: it chooses whether this
//! game wants the picture for its own platform and panel, and a card the player drops the PNG
//! onto later must still find the stored yes. Which platforms crop is the App's answer — this
//! crate is handed `overscan_available` and knows nothing about platforms, NES included.
//!
//! The box grows by one row when that row is there, so the hints stay at the bottom of the box
//! rather than the box staying put and the list running into them.
//!
//! Draw order: full-panel dim (BLACK at alpha 0.6), box centred in the safe area (BACKDROP),
//! title at PX_BODY in INK_DIM centred with top at box_y + PAD, the rows each ROW_H tall
//! starting at box_y + PAD + 28 (the selected row highlighted with INK at alpha 0.15, labels
//! PX_TITLE in INK centred), then the hint-select and hint-back spans at PX_HINT in INK_DIM on
//! one centred line at the bottom of the box.
//!
//! draw overlays the paused game's last frame and does not clear.

use slot2_gfx::{Canvas, Color};
use slot2_store::ScaleMode;

use crate::UiCtx;

/// What a row of the Display menu chooses: one of the four scale settings, or one of the three
/// submenus.
///
/// The scale rows carry the card's own value (`None` is the platform default), and the submenu
/// rows carry nothing: which shader, crop or overlay a game is on is a screen of its own, opened
/// with A.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayChoice {
    Scale(Option<ScaleMode>),
    Shader,
    Overscan,
    Overlay,
}

/// The rows for a platform with nothing to crop: the four scale settings a game can be on, then
/// the shader submenu, then the overlay submenu. `Scale(None)` is the platform default, which is
/// what a game nobody has overridden is on.
pub const ROWS: [DisplayChoice; 6] = [
    DisplayChoice::Scale(None),
    DisplayChoice::Scale(Some(ScaleMode::Integer)),
    DisplayChoice::Scale(Some(ScaleMode::AspectFit)),
    DisplayChoice::Scale(Some(ScaleMode::Fill)),
    DisplayChoice::Shader,
    DisplayChoice::Overlay,
];

/// The same choices with the overscan submenu inserted before the final overlay row, for a
/// platform whose registry entry crops something. The four scale rows and Shader stay in place;
/// Overlay remains last in either set.
pub const CROPPING_ROWS: [DisplayChoice; 7] = [
    DisplayChoice::Scale(None),
    DisplayChoice::Scale(Some(ScaleMode::Integer)),
    DisplayChoice::Scale(Some(ScaleMode::AspectFit)),
    DisplayChoice::Scale(Some(ScaleMode::Fill)),
    DisplayChoice::Shader,
    DisplayChoice::Overscan,
    DisplayChoice::Overlay,
];

pub const BOX_W: f32 = 340.0;
// PAD + title area + 6 rows + hint area, so everything fits 640x480 with room to spare.
pub const BOX_H: f32 = 316.0;
/// The box with the overscan row: one more row, so the same title and hint areas hold it.
pub const BOX_H_CROPPING: f32 = BOX_H + ROW_H;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 36.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

/// The Display submenu, open on one of its rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayMenu {
    // Private so callers cannot construct a row that is not there; the modulo in up/down keeps
    // the invariant. Read it back with `choice`.
    selected: usize,
    /// Whether this platform's registry entry crops anything, which is the whole of what decides
    /// the row set. Kept rather than derived: the row set is the menu's, and a second answer
    /// computed per frame would be a second answer.
    overscan_available: bool,
}

impl DisplayMenu {
    /// The menu for a game drawn with `scale`, which is `None` for the platform's default:
    /// that row starts selected, so opening the menu shows the game's scale as it is now.
    ///
    /// `overscan_available` adds the overscan row; the scale rows are reached by stored value
    /// alone, never by walking, because the submenu rows are ways in and not settings.
    pub fn new(scale: Option<ScaleMode>, overscan_available: bool) -> Self {
        let want = DisplayChoice::Scale(scale);
        let menu = DisplayMenu {
            selected: 0,
            overscan_available,
        };
        let selected = menu
            .choices()
            .iter()
            .position(|row| *row == want)
            .unwrap_or(0);
        DisplayMenu { selected, ..menu }
    }

    /// The rows this menu offers, in draw order. The one place the row set is decided: `choice`,
    /// `up`, `down` and `draw` all read this and nothing else, so a row cannot be drawn that
    /// navigation refuses to reach or reached without being drawn.
    pub fn choices(&self) -> &'static [DisplayChoice] {
        if self.overscan_available {
            &CROPPING_ROWS
        } else {
            &ROWS
        }
    }

    /// The row the highlight is on.
    pub fn choice(&self) -> DisplayChoice {
        self.choices()[self.selected]
    }

    /// The message key for a row's label.
    ///
    /// An exhaustive match rather than the enum's order or its name: a new choice stops
    /// compiling here, where somebody has to decide what it is called on screen. Each submenu
    /// row wears that screen's own title — one feature, one word — rather than a second
    /// spelling of it.
    pub const fn key(choice: DisplayChoice) -> &'static str {
        match choice {
            DisplayChoice::Scale(None) => "display-platform-default",
            DisplayChoice::Scale(Some(ScaleMode::Integer)) => "display-integer",
            DisplayChoice::Scale(Some(ScaleMode::AspectFit)) => "display-aspect-fit",
            DisplayChoice::Scale(Some(ScaleMode::Fill)) => "display-fill",
            DisplayChoice::Shader => "shader-title",
            DisplayChoice::Overscan => "overscan-title",
            // The screen the row opens wears its own title: one feature, one word. The overlay
            // screen's title is the same word the row is, and the overlay menu is measured on
            // the same panel the picture is, so there is nothing to spell differently here.
            DisplayChoice::Overlay => "overlay-title",
        }
    }

    pub fn up(&mut self) {
        let n = self.choices().len();
        self.selected = (self.selected + n - 1) % n;
    }

    pub fn down(&mut self) {
        let n = self.choices().len();
        self.selected = (self.selected + 1) % n;
    }

    /// How tall this menu's box is. Grows by one row when the overscan row is there.
    pub fn box_h(&self) -> f32 {
        if self.overscan_available {
            BOX_H_CROPPING
        } else {
            BOX_H
        }
    }

    /// Top-left of the box on the panel, centred in the safe area.
    pub fn box_origin(&self, ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - self.box_h()) / 2.0),
        )
    }

    /// Panel y of row `i`'s top.
    pub fn row_y(&self, ctx: &UiCtx, i: usize) -> f32 {
        self.box_origin(ctx).1 + PAD + 28.0 + i as f32 * ROW_H
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = self.box_origin(ctx);
        canvas.rect(bx, by, BOX_W, self.box_h(), crate::splash::BACKDROP);

        // The title is the row the player pressed to get here: `Display` stays `Display`.
        let title_spans = ctx.i18n.spans("ingame-display", &[]);
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

        for (i, choice) in self.choices().iter().enumerate() {
            let row_y = self.row_y(ctx, i);
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
        let hint_y = by + self.box_h() - PAD - crate::PX_HINT;

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
