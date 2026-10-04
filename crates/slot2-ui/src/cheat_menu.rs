//! The cheats submenu: one row per cheat the game's `.cht` holds, with the state of each one
//! beside it.
//!
//! Reached from the in-game menu's Cheats row (a later task wires that row up). Mirrors the
//! in-game and Display menus deliberately: the same panel, spacing, palette and safe-area
//! maths, and no menu framework of its own.
//!
//! The menu owns only what it takes to walk a list: how long it is, which row is highlighted,
//! and which row is first on screen. The entries belong to the caller and are handed in at
//! draw time, so the session stays the one place that knows what is enabled — the menu never
//! copies a cheat or holds a second opinion about one.
//!
//! Draw order: full-panel dim (BLACK at alpha 0.6), box centred in the safe area (BACKDROP),
//! title `ingame-cheats` at PX_BODY in INK_DIM centred with top at box_y + PAD, up to
//! [`MAX_ROWS`] rows each ROW_H tall starting at box_y + PAD + [`TITLE_AREA_H`] (the selected
//! row highlighted with INK at alpha 0.15; the description at PX_BODY in INK on the left, the
//! enabled/disabled word at PX_BODY on the right, dimmed while it is off), a short bar above or
//! below the window while rows are hidden that way, then the hints at PX_HINT in INK_DIM on one
//! centred line at the bottom of the box. An empty list draws the `cheat-empty` line in the
//! middle of the row area instead: no highlight, no bars, and only the back hint.
//!
//! draw overlays the paused game's last frame and does not clear.

use slot2_gfx::{Canvas, Color};
use slot2_store::Cheat;

use crate::UiCtx;

/// How many rows fit on one screen. Five to seven is what the safe area holds under a title
/// and a hint line; six is what the other menus' rhythm works out to.
pub const MAX_ROWS: usize = 6;

pub const BOX_W: f32 = 420.0;
// PAD + title area + MAX_ROWS rows + hint area, the same proportions the Display menu uses.
pub const BOX_H: f32 = PAD + TITLE_AREA_H + MAX_ROWS as f32 * ROW_H + PAD + crate::PX_HINT + 26.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 36.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

/// The band the title line owns above the rows: the title's own line box, the room the
/// "rows are hidden above" bar needs, and a pixel of clearance on either side of that bar.
///
/// The tallest body line box the shipped fonts produce is Noto Sans KR's at PX_BODY (24 px),
/// so 30 px holds it and still leaves the bar (MORE_H) a real band on both sides. A font with
/// metrics larger than this would need the whole menu laid out from the measured line box;
/// that is not what this constant is for.
const TITLE_AREA_H: f32 = 30.0;

/// Space between a row's edge and the text inside it.
const ROW_PAD: f32 = 8.0;
/// Space between a description and the state column, so a full description never touches it.
pub const STATE_GAP: f32 = 10.0;
/// The "there are more rows this way" bar: short, centred, and clear of the rows.
pub const MORE_W: f32 = 24.0;
pub const MORE_H: f32 = 4.0;
const MORE_CLEAR: f32 = 8.0;

/// What a description too long for its column is cut with.
const ELLIPSIS: &str = "…";

const KEY_ENABLED: &str = "cheat-enabled";
const KEY_DISABLED: &str = "cheat-disabled";
const KEY_EMPTY: &str = "cheat-empty";
const KEY_TOGGLE_HINT: &str = "hint-cheat-toggle";
const KEY_BACK_HINT: &str = "hint-back";

/// Which row of the game's cheats is highlighted, and which row is on screen first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheatMenu {
    /// The number of cheats the session had when the menu opened. Navigation and the window
    /// are sized by this, not by whatever slice a draw is handed.
    len: usize,
    /// `None` for an empty list: nothing to highlight, and no row to move to.
    selected: Option<usize>,
    /// The first row on screen, kept so the highlighted row is always inside the window.
    first: usize,
}

impl CheatMenu {
    /// The menu for a game with `len` cheats.
    pub fn new(len: usize) -> Self {
        CheatMenu {
            len,
            selected: (len > 0).then_some(0),
            first: 0,
        }
    }

    /// The highlighted row, or `None` when there is nothing to highlight.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// The first row on screen. Always low enough for the highlighted row to be visible.
    pub fn first_visible(&self) -> usize {
        self.first
    }

    /// Move the highlight up one row, wrapping to the last from the first. No-op when the
    /// list is empty.
    pub fn up(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };
        let next = (selected + self.len - 1) % self.len;
        self.selected = Some(next);
        self.window(next);
    }

    /// Move the highlight down one row, wrapping to the first from the last. No-op when the
    /// list is empty.
    pub fn down(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };
        let next = (selected + 1) % self.len;
        self.selected = Some(next);
        self.window(next);
    }

    /// Slide the window the least that puts `index` inside it.
    fn window(&mut self, index: usize) {
        if index < self.first {
            self.first = index;
        } else if index >= self.first + MAX_ROWS {
            self.first = index + 1 - MAX_ROWS;
        }
    }

    /// `(first row drawn, how many)` for a list of `rows` entries.
    ///
    /// The window is `len` rows of the session's list; a slice of another length is drawn as
    /// far as it agrees with that, rather than reading past its end.
    fn window_of(&self, rows: usize) -> (usize, usize) {
        let len = rows.min(self.len);
        let first = self.first.min(len);
        (first, (len - first).min(MAX_ROWS))
    }

    /// Whether rows are hidden above the window, for a list of `rows` entries.
    pub fn more_above(&self, rows: usize) -> bool {
        self.window_of(rows).0 > 0
    }

    /// Whether rows are hidden below the window, for a list of `rows` entries.
    pub fn more_below(&self, rows: usize) -> bool {
        let (first, shown) = self.window_of(rows);
        first + shown < rows.min(self.len)
    }

    /// Top-left of the box on the panel, centred in the safe area.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    /// Panel y of row `i`'s top, counting from the first row on screen.
    pub fn row_y(ctx: &UiCtx, i: usize) -> f32 {
        Self::box_origin(ctx).1 + PAD + TITLE_AREA_H + i as f32 * ROW_H
    }

    /// Panel x where a row's description starts.
    pub fn row_text_x(ctx: &UiCtx) -> f32 {
        Self::box_origin(ctx).0 + PAD + ROW_PAD
    }

    /// Panel x of a state word `w` wide, right-aligned inside the row.
    pub fn state_x(ctx: &UiCtx, w: f32) -> f32 {
        Self::box_origin(ctx).0 + BOX_W - PAD - ROW_PAD - w
    }

    /// Panel y just below the title line's box.
    pub fn title_bottom(ctx: &mut UiCtx) -> f32 {
        let line_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
        Self::box_origin(ctx).1 + PAD + line_h
    }

    /// Panel y of the "rows are hidden above" bar.
    ///
    /// It goes in the empty band between the title line's box and the first row, centred in
    /// what is there. "A little above the first row" is not good enough: the title's line box
    /// reaches most of the way down to the rows, so a bar placed that way crosses the words
    /// over it — which is exactly what it used to do.
    pub fn more_above_y(ctx: &mut UiCtx) -> f32 {
        let title_bottom = Self::title_bottom(ctx);
        let row_top = Self::row_y(ctx, 0);
        // Centred in the band, and never closer than a pixel to either side of it.
        (title_bottom + (row_top - title_bottom - MORE_H) / 2.0)
            .max(title_bottom + 1.0)
            .min(row_top - MORE_H - 1.0)
    }

    /// Panel y of the "rows are hidden below" bar.
    pub fn more_below_y(ctx: &UiCtx) -> f32 {
        Self::row_y(ctx, MAX_ROWS - 1) + ROW_H + MORE_CLEAR
    }

    /// Panel y of the empty-list line's top, centred in the row area.
    pub fn empty_y(ctx: &mut UiCtx) -> f32 {
        let line_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
        Self::row_y(ctx, 0) + (MAX_ROWS as f32 * ROW_H - line_h) / 2.0
    }

    /// Panel y of the hint line's top.
    pub fn hint_y(ctx: &UiCtx) -> f32 {
        Self::box_origin(ctx).1 + BOX_H - PAD - crate::PX_HINT
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, cheats: &[Cheat]) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        // The title is the in-game row the player pressed to get here, so it is that message.
        let title_spans = ctx.i18n.spans("ingame-cheats", &[]);
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

        let line_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
        let (first, shown) = self.window_of(cheats.len());

        for row in 0..shown {
            let index = first + row;
            let Some(cheat) = cheats.get(index) else {
                break;
            };
            let row_y = Self::row_y(ctx, row);
            if self.selected == Some(index) {
                canvas.rect(
                    bx + PAD,
                    row_y,
                    BOX_W - 2.0 * PAD,
                    ROW_H,
                    crate::splash::INK.with_alpha(0.15),
                );
            }
            let text_y = row_y + (ROW_H - line_h) / 2.0;

            // The state word is what the description has to leave room for, so it is measured
            // first. It is always words, never only a colour: a player who cannot tell two
            // shades apart still has to read whether a cheat is on.
            let key = if cheat.enabled {
                KEY_ENABLED
            } else {
                KEY_DISABLED
            };
            let state = ctx.i18n.spans(key, &[]);
            let state_w = crate::face::spans_width(ctx, &state, crate::PX_BODY);
            let state_x = Self::state_x(ctx, state_w);
            let state_color = if cheat.enabled {
                crate::splash::INK
            } else {
                crate::splash::INK_DIM
            };
            crate::draw_spans(
                canvas,
                ctx,
                &state,
                crate::PX_BODY,
                state_x,
                text_y,
                state_color,
            );

            let room = state_x - STATE_GAP - Self::row_text_x(ctx);
            let description = fit(ctx, &cheat.description, crate::PX_BODY, room);
            crate::face::draw_text(
                canvas,
                ctx,
                &description,
                crate::PX_BODY,
                Self::row_text_x(ctx),
                text_y,
                crate::splash::INK,
            );
        }

        // A bar just inside the window on the side there are rows on: the list is longer than
        // the screen, and this is which way to look. A glyph would be a font question; this is
        // two rects' worth of arithmetic the fonts cannot get wrong.
        if self.more_above(cheats.len()) {
            canvas.rect(
                bx + (BOX_W - MORE_W) / 2.0,
                Self::more_above_y(ctx),
                MORE_W,
                MORE_H,
                crate::splash::INK_DIM,
            );
        }
        if self.more_below(cheats.len()) {
            canvas.rect(
                bx + (BOX_W - MORE_W) / 2.0,
                Self::more_below_y(ctx),
                MORE_W,
                MORE_H,
                crate::splash::INK_DIM,
            );
        }

        if shown == 0 {
            let spans = ctx.i18n.spans(KEY_EMPTY, &[]);
            let w = crate::face::spans_width(ctx, &spans, crate::PX_BODY);
            let y = Self::empty_y(ctx);
            crate::draw_spans(
                canvas,
                ctx,
                &spans,
                crate::PX_BODY,
                bx + (BOX_W - w) / 2.0,
                y,
                crate::splash::INK_DIM,
            );
        }

        let back = ctx.i18n.spans(KEY_BACK_HINT, &[]);
        let w_back = crate::face::spans_width(ctx, &back, crate::PX_HINT);
        let hint_y = Self::hint_y(ctx);
        if shown == 0 {
            // Nothing to toggle: the way out is the whole hint line.
            crate::draw_spans(
                canvas,
                ctx,
                &back,
                crate::PX_HINT,
                bx + (BOX_W - w_back) / 2.0,
                hint_y,
                crate::splash::INK_DIM,
            );
            return;
        }

        let toggle = ctx.i18n.spans(KEY_TOGGLE_HINT, &[]);
        let w_toggle = crate::face::spans_width(ctx, &toggle, crate::PX_HINT);
        let total = w_toggle + crate::PX_HINT + w_back;
        let hint_x = bx + (BOX_W - total) / 2.0;
        crate::draw_spans(
            canvas,
            ctx,
            &toggle,
            crate::PX_HINT,
            hint_x,
            hint_y,
            crate::splash::INK_DIM,
        );
        crate::draw_spans(
            canvas,
            ctx,
            &back,
            crate::PX_HINT,
            hint_x + w_toggle + crate::PX_HINT,
            hint_y,
            crate::splash::INK_DIM,
        );
    }
}

/// `text` as much of it as fits in `max_w` at `px`, with [`ELLIPSIS`] when it does not fit.
///
/// A description is arbitrary UTF-8 out of a file somebody else wrote, so the cut is made on
/// character boundaries: a byte-wise cut would be a panic or half a glyph. A string that
/// already fits is handed back untouched.
///
/// Re-measuring every prefix would shape the whole description once per character, and a `.cht`
/// description has no length limit: a long one would stall the menu, on every row, on every
/// draw. So the character boundaries are collected once and the longest prefix that fits is
/// found by bisection over them — a prefix only gets wider as it grows, which is what makes
/// bisection the right search — and the whole thing costs O(log N) measurements.
pub fn fit(ctx: &mut UiCtx, text: &str, px: f32, max_w: f32) -> String {
    if crate::face::measure(ctx, text, px) <= max_w {
        return text.to_owned();
    }
    let ell_w = crate::face::measure(ctx, ELLIPSIS, px);
    // Not even the ellipsis fits: there is nothing honest to draw, and an empty string is
    // still narrower than the room it was given.
    if ell_w > max_w {
        return String::new();
    }

    let mut bounds: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    bounds.push(text.len());
    // `lo` is a boundary whose prefix fits with the ellipsis; `hi` is one that does not. The
    // whole string does not fit (measured above), so the two start apart.
    let mut lo = 0;
    let mut hi = bounds.len() - 1;
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        if crate::face::measure(ctx, &text[..bounds[mid]], px) + ell_w <= max_w {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let mut out = String::with_capacity(bounds[lo] + ELLIPSIS.len());
    out.push_str(&text[..bounds[lo]]);
    out.push_str(ELLIPSIS);
    out
}
