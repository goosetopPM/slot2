//! The language picker: which language the frontend speaks, out of the packs that loaded.
//!
//! A screen of its own, in the style of the other menus. What it holds is what the caller handed
//! over — the codes and the names of the packs that actually loaded — and nothing else: no file
//! system, no `System/Lang`, no `I18n::available`, no environment, no `Card`, no saving. Which
//! languages a card can speak is the frontend's question, answered once before this is opened;
//! a screen that went looking for packs would be a second, worse answer to it, and one that could
//! offer a language nothing can load.
//!
//! Each row's name is the pack's own `lang-name`, which is why a Korean pack says `한국어` on an
//! English screen: a language names itself, in itself. The caller's order is kept — nothing here
//! sorts by name or collates by locale — and codes are compared exactly, as the strings they are.
//!
//! Draw order, mirroring the other menus (same panel, palette and text sizes): full-panel dim
//! (BLACK at alpha 0.6), box centred in the safe area (BACKDROP), title at PX_BODY in INK_DIM
//! centred with its top at box_y + PAD, the visible rows ROW_H tall starting at box_y + PAD + 28
//! (the highlighted row backed with INK at alpha 0.15, the language's own name at PX_BODY in INK
//! on the left and its code at PX_HINT in INK_DIM on the right, the code wearing the `· Current`
//! sentence on the row that is running now), the 1-based position line at PX_HINT in INK_DIM
//! under the rows, then hint-select and hint-back at PX_HINT in INK_DIM on one centred line at
//! the bottom of the box. An empty list draws the empty message in the first row's place, with no
//! highlight and no position.
//!
//! draw overlays the shelf underneath and does not clear.

use slot2_gfx::{Canvas, Color};
use slot2_i18n::Arg;

use crate::UiCtx;

/// The panel's title: the settings row the player pressed to get here, so it is that message.
pub const TITLE_KEY: &str = "shelf-language";
/// A language's code, as the pack spells it.
pub const CODE_KEY: &str = "language-code";
/// The same, on the row of the language that is running now.
pub const CODE_CURRENT_KEY: &str = "language-code-current";
/// What is said when no pack loaded at all.
pub const EMPTY_KEY: &str = "language-picker-empty";
/// Where the highlight is in the list, 1-based.
pub const POSITION_KEY: &str = "language-picker-position";
/// What A does on the highlighted row.
pub const HINT_SELECT_KEY: &str = "hint-select";
/// Leaving the picker. The other menus' own back hint.
pub const HINT_BACK_KEY: &str = "hint-back";

pub const BOX_W: f32 = 440.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 36.0;
/// How many rows the panel shows at once.
///
/// Fixed, so the rows, the position line and the hints keep their places however many packs a card
/// turned out to have: a menu whose rows move because a language was added is a menu nobody can
/// aim at. A longer list scrolls under the highlight instead.
pub const MAX_VISIBLE_ROWS: usize = 6;
/// Inset of a row's name and code from the row's own edges.
pub const INSET: f32 = 6.0;
/// The room the right-hand code column gets, and the gap before it. The name column is whatever
/// is left, so the two can never be laid out over each other.
pub const CODE_W: f32 = 160.0;
pub const COLUMN_GAP: f32 = 12.0;
pub const NAME_W: f32 = BOX_W - 2.0 * (PAD + INSET) - COLUMN_GAP - CODE_W;
/// Between the title's line and the first row.
pub const TITLE_GAP: f32 = 28.0;
/// Between the last visible row and the position line.
pub const POSITION_GAP: f32 = 10.0;
/// A line of PX_HINT text, with a little room over it: the position line and the hints are both
/// this tall, and the panel is laid out for both of them whatever the list holds.
pub const HINT_LINE_H: f32 = 19.0;
/// PAD + the title + the visible rows + the position line + a gap + the hint line + PAD.
pub const BOX_H: f32 = PAD
    + TITLE_GAP
    + MAX_VISIBLE_ROWS as f32 * ROW_H
    + POSITION_GAP
    + HINT_LINE_H
    + 12.0
    + HINT_LINE_H
    + PAD;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);
/// The one highlighted row's backing.
pub const HIGHLIGHT_ALPHA: f32 = 0.15;

/// One language the caller knows how to speak: the code its pack is stored under, and the name
/// that pack gives itself.
///
/// Both are kept exactly as they were handed over. The name is what a person reads, so it is the
/// pack's own words; the code is what a settings file is written with, so it is the spelling the
/// caller will compare and store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageOption {
    code: String,
    name: String,
}

impl LanguageOption {
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self {
        LanguageOption {
            code: code.into(),
            name: name.into(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// The picker, holding one highlight and the fixed row of the language that is running.
///
/// The list is the caller's, minus exact repeat codes, and in the caller's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguagePicker {
    options: Vec<LanguageOption>,
    selected: usize,
    /// The row wearing the `Current` badge, as a position: the language that is running, when it
    /// is one of the candidates. A code nothing can load is not a row, and pointing at the wrong
    /// one would be worse than pointing at nothing.
    current: Option<usize>,
    /// The first row the window shows. Keeps the highlight visible.
    first: usize,
}

impl LanguagePicker {
    /// The picker for the languages a caller can speak, starting on `current` when it is one of
    /// them.
    ///
    /// An exact repeat code is one language, and the first entry is the caller's own answer for
    /// it; two different codes that share a display name are two languages and both stay.
    pub fn new(options: Vec<LanguageOption>, current: &str) -> Self {
        let mut kept: Vec<LanguageOption> = Vec::with_capacity(options.len());
        for option in options {
            if !kept.iter().any(|other| other.code() == option.code()) {
                kept.push(option);
            }
        }
        let current = kept.iter().position(|option| option.code() == current);
        let mut picker = LanguagePicker {
            options: kept,
            selected: current.unwrap_or(0),
            current,
            first: 0,
        };
        picker.follow();
        picker
    }

    /// The candidates in the order they are drawn, exactly as the caller gave them.
    pub fn options(&self) -> &[LanguageOption] {
        &self.options
    }

    /// The row the highlight is on, which is the language A would choose. `None` on an empty
    /// menu.
    pub fn highlighted(&self) -> Option<&LanguageOption> {
        self.options.get(self.selected)
    }

    /// The language that is running, which is the row wearing the `Current` badge. `None` when
    /// the running language is not one of the candidates. It does not move with the highlight.
    pub fn current(&self) -> Option<&LanguageOption> {
        self.options.get(self.current?)
    }

    /// Whether the highlighted language is a change: either the running one is not on the list at
    /// all — so anything picked is new — or the highlight is on a different row from the badge.
    /// An empty menu changes nothing.
    pub fn changed(&self) -> bool {
        let Some(row) = self.highlighted() else {
            return false;
        };
        match self.current() {
            Some(current) => row.code() != current.code(),
            None => true,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    pub fn len(&self) -> usize {
        self.options.len()
    }

    /// The first row the window shows. Always a row that exists, while there are any.
    pub fn first_visible(&self) -> usize {
        self.first
    }

    /// Move the highlight up one row, wrapping. A menu with nothing to walk, or one row, stays
    /// where it is.
    pub fn up(&mut self) {
        if self.options.len() > 1 {
            self.selected = (self.selected + self.options.len() - 1) % self.options.len();
            self.follow();
        }
    }

    /// Move the highlight down one row, wrapping.
    pub fn down(&mut self) {
        if self.options.len() > 1 {
            self.selected = (self.selected + 1) % self.options.len();
            self.follow();
        }
    }

    /// How many rows one screen shows: the whole list when it is short enough to fit.
    fn visible(&self) -> usize {
        MAX_VISIBLE_ROWS.min(self.options.len())
    }

    /// Slide the window as far as the new highlight needs and no further: the list keeps its
    /// place while the selection moves inside it, and follows it to the other end when the
    /// selection wraps.
    fn follow(&mut self) {
        let last = self.options.len().saturating_sub(self.visible());
        self.first = self.first.min(last);
        if self.selected < self.first {
            self.first = self.selected;
        } else if self.selected >= self.first + self.visible() {
            self.first = self.selected + 1 - self.visible();
        }
        self.first = self.first.min(last);
    }

    /// Top-left of the box on the panel, centred in the safe area.
    ///
    /// The safe area and not the panel: the same safe-area coordinates on every geometry, which
    /// is what keeps the three panels' menus in the same place.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    /// Panel y of visible slot `i`'s top, where slot 0 is the top of the window. A row's place
    /// does not depend on how many candidates there are.
    pub fn row_y(ctx: &UiCtx, i: usize) -> f32 {
        Self::box_origin(ctx).1 + PAD + TITLE_GAP + i as f32 * ROW_H
    }

    /// Panel y of the position line's top.
    pub fn position_y(ctx: &UiCtx) -> f32 {
        Self::row_y(ctx, MAX_VISIBLE_ROWS) + POSITION_GAP
    }

    /// Panel y of the hint line's top.
    pub fn hint_y(ctx: &UiCtx) -> f32 {
        Self::box_origin(ctx).1 + BOX_H - PAD - crate::PX_HINT
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        let title = ctx.i18n.spans(TITLE_KEY, &[]);
        let title_w = crate::face::spans_width(ctx, &title, crate::PX_BODY);
        crate::draw_spans(
            canvas,
            ctx,
            &title,
            crate::PX_BODY,
            bx + (BOX_W - title_w) / 2.0,
            by + PAD,
            crate::splash::INK_DIM,
        );

        if self.options.is_empty() {
            // The first row's place, so an empty picker is a panel with nothing to offer rather
            // than a panel with a hole in it. Nothing is highlighted and no position is shown:
            // there is no row to be at.
            let empty = ctx.i18n.spans(EMPTY_KEY, &[]);
            let empty_w = crate::face::spans_width(ctx, &empty, crate::PX_BODY);
            let line_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
            crate::draw_spans(
                canvas,
                ctx,
                &empty,
                crate::PX_BODY,
                bx + (BOX_W - empty_w) / 2.0,
                Self::row_y(ctx, 0) + (ROW_H - line_h) / 2.0,
                crate::splash::INK_DIM,
            );
        } else {
            let name_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
            let code_h = ctx.fonts.measure("", crate::PX_HINT).line_height as f32;
            for (slot, index) in (self.first..self.first + self.visible()).enumerate() {
                let option = &self.options[index];
                let row_y = Self::row_y(ctx, slot);
                if index == self.selected {
                    canvas.rect(
                        bx + PAD,
                        row_y,
                        BOX_W - 2.0 * PAD,
                        ROW_H,
                        crate::splash::INK.with_alpha(HIGHLIGHT_ALPHA),
                    );
                }

                // The name is the pack's own words for itself, or the code when the caller had no
                // name to give: a blank name would be a gap where a language should be. Neither
                // column trusts its text to fit — a card pack is somebody else's file — so both
                // are cut on character boundaries to the room they actually have.
                let name =
                    crate::cheat_menu::fit(ctx, display_name(option), crate::PX_BODY, NAME_W);
                if !name.is_empty() {
                    crate::face::draw_text(
                        canvas,
                        ctx,
                        &name,
                        crate::PX_BODY,
                        bx + PAD + INSET,
                        row_y + (ROW_H - name_h) / 2.0,
                        crate::splash::INK,
                    );
                }

                // The code, in the pack's own sentence: the words around it are the message's,
                // and the `· Current` one belongs to the row that is running now rather than to
                // the highlight, so a player walking the list can still see where they are.
                let key = if self.current == Some(index) {
                    CODE_CURRENT_KEY
                } else {
                    CODE_KEY
                };
                let text = ctx.i18n.t_args(key, &[("code", Arg::from(option.code()))]);
                let text = crate::cheat_menu::fit(ctx, &text, crate::PX_HINT, CODE_W);
                if !text.is_empty() {
                    let w = crate::face::measure(ctx, &text, crate::PX_HINT);
                    crate::face::draw_text(
                        canvas,
                        ctx,
                        &text,
                        crate::PX_HINT,
                        bx + BOX_W - PAD - INSET - w,
                        row_y + (ROW_H - code_h) / 2.0,
                        crate::splash::INK_DIM,
                    );
                }
            }

            // Where the highlight is, 1-based, out of how many there are.
            let position = ctx.i18n.spans(
                POSITION_KEY,
                &[
                    ("current", Arg::from((self.selected + 1) as i64)),
                    ("total", Arg::from(self.options.len() as i64)),
                ],
            );
            let position_w = crate::face::spans_width(ctx, &position, crate::PX_HINT);
            crate::draw_spans(
                canvas,
                ctx,
                &position,
                crate::PX_HINT,
                bx + (BOX_W - position_w) / 2.0,
                Self::position_y(ctx),
                crate::splash::INK_DIM,
            );
        }

        let hint_select = ctx.i18n.spans(HINT_SELECT_KEY, &[]);
        let hint_back = ctx.i18n.spans(HINT_BACK_KEY, &[]);
        let w_select = crate::face::spans_width(ctx, &hint_select, crate::PX_HINT);
        let w_back = crate::face::spans_width(ctx, &hint_back, crate::PX_HINT);
        let hint_x = bx + (BOX_W - (w_select + crate::PX_HINT + w_back)) / 2.0;
        let hint_y = Self::hint_y(ctx);
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

/// What a row shows as its name: the pack's own name, or the code when the pack gave none. A name
/// of nothing but spaces is no name at all — it would draw as a gap where a language should be —
/// and the stored name is left as the caller wrote it either way.
fn display_name(option: &LanguageOption) -> &str {
    if option.name().trim().is_empty() {
        option.code()
    } else {
        option.name()
    }
}
