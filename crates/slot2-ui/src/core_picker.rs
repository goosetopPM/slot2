//! The platform's core picker: which of the cores this frontend ships runs this game.
//!
//! Reached from the in-game menu's Core row (task 61 wires that). Mirrors the in-game and
//! display menus deliberately: the same overlay language, panel, spacing, text sizes and
//! palette, and no menu framework of its own.
//!
//! The model has no file system, no session and no settings. A caller hands it the platform,
//! the official cores whose libraries are on the card, and which core is running; the order,
//! the highlight and the words are decided here. Which library actually exists is task 61's
//! question — a UI that read the card would be a second, worse answer to it.
//!
//! Draw order: full-panel dim (BLACK at alpha 0.6), box centred in the safe area (BACKDROP),
//! title at PX_BODY in INK_DIM centred with top at box_y + PAD, the candidate rows each ROW_H
//! tall starting at box_y + PAD + 28 (only the highlighted row is backed with INK at alpha
//! 0.15, labels PX_TITLE in INK centred, and the row of the core that is running carries a
//! `Current` badge at its right end in INK_DIM), the restart notice at PX_HINT in INK_DIM
//! centred below the rows, then the hint-select and hint-back spans at PX_HINT in INK_DIM on
//! one centred line at the bottom of the box. An empty candidate list draws the empty message
//! in the first row's place and no highlight at all.
//!
//! draw overlays the paused game's last frame and does not clear.

use slot2_gfx::{Canvas, Color};
use slot2_retro::{supported_cores, CoreId, Platform};

use crate::UiCtx;

/// The panel's title: the in-game row the player pressed to get here, so it is that message.
pub const TITLE_KEY: &str = "ingame-core";
/// The badge on the row of the core that is running now.
pub const CURRENT_KEY: &str = "core-current";
/// What is said when no candidate's library is on the card.
pub const EMPTY_KEY: &str = "core-picker-empty";
/// The notice under the rows: this choice is not free, it starts the game again.
pub const RESTART_KEY: &str = "core-picker-restart";

pub const BOX_W: f32 = 360.0;
// PAD + title area + the longest candidate list + the notice line + the hint line, so
// everything fits 640x480 with room to spare. The Korean restart notice is the widest thing
// in the box and is what sets the width.
pub const BOX_H: f32 = 190.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 36.0;
/// The length of the longest list [`supported_cores`] returns — GB, GBC and GBA, at two. The
/// box is laid out for that case, so the notice and hints do not move between platforms.
pub const MAX_ROWS: usize = 2;
/// Between the last row and the restart notice.
pub const NOTICE_GAP: f32 = 10.0;
/// The `Current` badge's inset from the right edge of its row.
pub const BADGE_INSET: f32 = 6.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

/// The platform's core picker, holding the highlight on one candidate.
///
/// `current` is the core the running session opened, and is only kept when it is one of the
/// candidates: a session on an external library has no row here, and neither does a core whose
/// library is not on the card. There is nothing to point at in either case, and pointing at
/// the wrong row would be worse than pointing at nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorePicker {
    // Private so a caller cannot highlight a row that is not there: `up`/`down` keep the
    // invariant. Read the answer back with `highlighted`.
    rows: Vec<CoreId>,
    selected: usize,
    current: Option<CoreId>,
}

impl CorePicker {
    /// The candidates for `platform`, in the registry's order.
    ///
    /// `installed` is the official cores whose libraries are on the card, in whatever order
    /// and with whatever repeats the caller found them: the registry decides the order, and a
    /// core that cannot run this platform is not a candidate however the card is arranged.
    pub fn new(platform: Platform, installed: &[CoreId], current: Option<CoreId>) -> Self {
        let rows: Vec<CoreId> = supported_cores(platform)
            .iter()
            .copied()
            .filter(|core| installed.contains(core))
            .collect();
        // The core that is running starts highlighted, so opening this shows what the game is
        // on. Anything else — no session, an external core, a core that is not on the card —
        // starts on the first row and badges nothing.
        let selected = current
            .and_then(|core| rows.iter().position(|row| *row == core))
            .unwrap_or(0);
        CorePicker {
            current: current.filter(|core| rows.contains(core)),
            rows,
            selected,
        }
    }

    /// The candidates in the order they are drawn. Read-only: the highlight is moved with
    /// `up`/`down` rather than by rearranging a list the caller holds a reference to.
    pub fn rows(&self) -> &[CoreId] {
        &self.rows
    }

    /// The row the highlight is on, which is the core A would choose.
    pub fn highlighted(&self) -> Option<CoreId> {
        self.rows.get(self.selected).copied()
    }

    /// The core of the row wearing the `Current` badge, which is `None` when the running core
    /// is not one of the candidates. It does not move when the highlight does.
    pub fn current(&self) -> Option<CoreId> {
        self.current
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// The message key for a core's display name.
    ///
    /// The registry's own names are library files — `mgba_libretro`, `genesis_plus_gx_libretro`
    /// — and a player is looking at consoles, not at the frontend's file names. The match is
    /// exhaustive on purpose: a core added to the registry without a name here fails to build
    /// rather than showing up on screen as a file name.
    pub const fn key(core: CoreId) -> &'static str {
        match core {
            CoreId::Mgba => "core-name-mgba",
            CoreId::Gambatte => "core-name-gambatte",
            CoreId::Gpsp => "core-name-gpsp",
            CoreId::Fceumm => "core-name-fceumm",
            CoreId::Snes9x => "core-name-snes9x",
            CoreId::GenesisPlusGx => "core-name-genesis-plus-gx",
        }
    }

    /// Move the highlight up one row, wrapping. Nothing to walk stays where it is.
    pub fn up(&mut self) {
        if self.rows.len() > 1 {
            self.selected = (self.selected + self.rows.len() - 1) % self.rows.len();
        }
    }

    /// Move the highlight down one row, wrapping. Nothing to walk stays where it is.
    pub fn down(&mut self) {
        if self.rows.len() > 1 {
            self.selected = (self.selected + 1) % self.rows.len();
        }
    }

    /// Top-left of the box on the panel, centred in the safe area.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    /// Panel y of candidate row `i`'s top. A row's place does not depend on how many rows
    /// there are, so the notice and hints below never move.
    pub fn row_y(ctx: &UiCtx, i: usize) -> f32 {
        Self::box_origin(ctx).1 + PAD + 28.0 + i as f32 * ROW_H
    }

    /// Panel y of the restart notice's top.
    pub fn notice_y(ctx: &UiCtx) -> f32 {
        Self::row_y(ctx, MAX_ROWS) + NOTICE_GAP
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        let title_spans = ctx.i18n.spans(TITLE_KEY, &[]);
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

        if self.rows.is_empty() {
            // The first row's place, so an empty picker is a panel that has nothing to offer
            // rather than a panel with a hole in it.
            let empty_spans = ctx.i18n.spans(EMPTY_KEY, &[]);
            let empty_w = crate::face::spans_width(ctx, &empty_spans, crate::PX_BODY);
            let row_y = Self::row_y(ctx, 0);
            let line_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
            crate::draw_spans(
                canvas,
                ctx,
                &empty_spans,
                crate::PX_BODY,
                bx + (BOX_W - empty_w) / 2.0,
                row_y + (ROW_H - line_h) / 2.0,
                crate::splash::INK_DIM,
            );
        } else {
            for (i, core) in self.rows.iter().enumerate() {
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
                let label_spans = ctx.i18n.spans(Self::key(*core), &[]);
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

                // The badge follows the running core, not the highlight: a player walking the
                // list must always be able to see which one they are on right now.
                if self.current == Some(*core) {
                    let badge_spans = ctx.i18n.spans(CURRENT_KEY, &[]);
                    let badge_w = crate::face::spans_width(ctx, &badge_spans, crate::PX_BODY);
                    let badge_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
                    crate::draw_spans(
                        canvas,
                        ctx,
                        &badge_spans,
                        crate::PX_BODY,
                        bx + BOX_W - PAD - BADGE_INSET - badge_w,
                        row_y + (ROW_H - badge_h) / 2.0,
                        crate::splash::INK_DIM,
                    );
                }
            }
        }

        // What the choice costs, said before it is made: the core owns the state format, so
        // this is not a setting that can take effect while the game runs.
        let notice_spans = ctx.i18n.spans(RESTART_KEY, &[]);
        let notice_w = crate::face::spans_width(ctx, &notice_spans, crate::PX_HINT);
        crate::draw_spans(
            canvas,
            ctx,
            &notice_spans,
            crate::PX_HINT,
            bx + (BOX_W - notice_w) / 2.0,
            Self::notice_y(ctx),
            crate::splash::INK_DIM,
        );

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
