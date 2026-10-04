//! The time zone screen: the display offset of D-25, in minutes east of UTC.
//!
//! Reached from a shelf menu (a later task wires that). What is on screen is a number and never
//! a place: there is no `tzdata` on the image and no network to ask, so the frontend offers the
//! one thing it can honestly offer — a fixed offset the player moves in quarter-hours and whole
//! hours. No city list, no DST rules, no zones named by anything but the offset itself.
//!
//! The model owns no clock and touches no card. `original` is the value the screen opened on and
//! `selected` is where the player has moved it: a caller previews `selected` on the runtime
//! clock, rolls back to `original` when the player cancels or a save fails, and does the saving
//! itself. Nothing here reads the clock, the file system, the environment or the backend.
//!
//! Draw order, mirroring the other menus (same panel, palette and text sizes): full-panel dim
//! (BLACK at alpha 0.6), box centred in the safe area (BACKDROP), title at PX_BODY in INK_DIM
//! centred with its top at box_y + [`TITLE_Y`], the current value at PX_TITLE in INK, D-25's
//! note at PX_BODY in INK_DIM, the adjust hint at PX_HINT, and the apply and cancel hints on the
//! bottom line at PX_HINT in INK_DIM.
//!
//! draw overlays the shelf underneath and does not clear.

use slot2_gfx::{Canvas, Color};
use slot2_i18n::{Arg, Span};

use crate::UiCtx;

/// The panel's title.
pub const TITLE_KEY: &str = "timezone-title";
/// The current value, with the offset passed in as an argument rather than glued on here.
pub const VALUE_KEY: &str = "timezone-value";
/// D-25 in one line: what this offset moves, and what it leaves alone.
pub const NOTE_KEY: &str = "timezone-note";
/// How to move the value.
pub const HINT_ADJUST_KEY: &str = "timezone-hint-adjust";
/// What A does.
pub const HINT_APPLY_KEY: &str = "timezone-hint-apply";
/// What B does.
pub const HINT_CANCEL_KEY: &str = "timezone-hint-cancel";

/// The minutes a card may hold, borrowed rather than written down twice: the store owns the file
/// format and the range that goes with it (D-25), and a second copy here would be a second thing
/// to keep in step. A value a card accepts and this screen then clamped would be a setting
/// nobody can explain.
pub const MIN_MINUTES: i32 = slot2_store::UTC_OFFSET_MINUTES_MIN;
pub const MAX_MINUTES: i32 = slot2_store::UTC_OFFSET_MINUTES_MAX;
/// UTC: what a card that has never been set holds.
pub const DEFAULT_MINUTES: i32 = slot2_store::DEFAULT_UTC_OFFSET_MINUTES;

/// One press of Left/Right. Fifteen minutes is the quarter-hour several real fixed offsets sit
/// on, and no offset anybody uses is between two of them.
pub const STEP_MINUTES: i32 = 15;
/// One press of Up/Down: a whole hour, so +09:00 is nine presses rather than thirty-six.
pub const STEP_HOURS: i32 = 60;

pub const BOX_W: f32 = 440.0;
// PAD + title + the value + the note + two hint lines, so everything fits 640x480 with room.
pub const BOX_H: f32 = 244.0;
pub const PAD: f32 = 16.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

/// Top of each line inside the box, in draw order. Spaced rather than stacked, because what is
/// between them is a gap and not another row.
pub const TITLE_Y: f32 = PAD;
pub const VALUE_Y: f32 = 56.0;
pub const NOTE_Y: f32 = 112.0;
pub const ADJUST_Y: f32 = 160.0;
/// The bottom line, measured up from the box's own bottom edge so a taller box moves it with it.
pub const HINTS_Y: f32 = BOX_H - PAD - crate::PX_HINT;

/// The time zone screen, open on the offset the card holds.
///
/// The two values are private so a caller cannot set `selected` without going through the
/// navigation, or hand `original` a number the screen never opened on. Read them back with
/// `original` and `selected`.
///
/// `Copy` because a screen that carries this will carry it in a `Screen`, which is itself `Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimezoneMenu {
    original: i32,
    selected: i32,
}

impl TimezoneMenu {
    /// The menu for the offset a card holds.
    ///
    /// A value outside [`MIN_MINUTES`]..=[`MAX_MINUTES`] is clamped to the nearer end rather
    /// than refused. A card cannot hold one — the store reads such a file as UTC — so an
    /// out-of-range value is a caller's mistake, and no error type or fallback exists to dress
    /// it up as a card's state. The nearest legal value is the honest picture of it.
    pub fn new(minutes: i32) -> Self {
        let minutes = clamp(minutes);
        TimezoneMenu {
            original: minutes,
            selected: minutes,
        }
    }

    /// What the card said when this screen opened. Navigation never moves it: it is where a
    /// cancelled screen and a failed save go back to.
    pub fn original(&self) -> i32 {
        self.original
    }

    /// What is on screen now, which is what a caller previews on the runtime clock.
    pub fn selected(&self) -> i32 {
        self.selected
    }

    /// Whether the player has moved the value away from what the card holds.
    pub fn changed(&self) -> bool {
        self.selected != self.original
    }

    /// A quarter-hour west, stopping at the end of the range.
    pub fn left(&mut self) {
        self.selected = clamp(self.selected - STEP_MINUTES);
    }

    /// A quarter-hour east.
    pub fn right(&mut self) {
        self.selected = clamp(self.selected + STEP_MINUTES);
    }

    /// A whole hour east: on a screen where the value is the number it becomes, up is the
    /// larger value.
    pub fn up(&mut self) {
        self.selected = clamp(self.selected + STEP_HOURS);
    }

    /// A whole hour west.
    pub fn down(&mut self) {
        self.selected = clamp(self.selected - STEP_HOURS);
    }

    /// Top-left of the box on the panel, centred in the safe area.
    ///
    /// The safe area and not the panel: on a 720x720 the layout region is centred but it is not
    /// the whole screen, and a box placed against the panel would drift out of the region the
    /// rest of the UI is laid out in.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        let title = ctx.i18n.spans(TITLE_KEY, &[]);
        line(
            canvas,
            ctx,
            &title,
            crate::PX_BODY,
            bx,
            by + TITLE_Y,
            crate::splash::INK_DIM,
        );

        // The number is the message's own argument: the word standing in front of it, and where
        // that word goes, belong to the language pack.
        let value = ctx.i18n.spans(
            VALUE_KEY,
            &[("offset", Arg::from(format_offset(self.selected)))],
        );
        line(
            canvas,
            ctx,
            &value,
            crate::PX_TITLE,
            bx,
            by + VALUE_Y,
            crate::splash::INK,
        );

        let note = ctx.i18n.spans(NOTE_KEY, &[]);
        line(
            canvas,
            ctx,
            &note,
            crate::PX_BODY,
            bx,
            by + NOTE_Y,
            crate::splash::INK_DIM,
        );

        let adjust = ctx.i18n.spans(HINT_ADJUST_KEY, &[]);
        line(
            canvas,
            ctx,
            &adjust,
            crate::PX_HINT,
            bx,
            by + ADJUST_Y,
            crate::splash::INK_DIM,
        );

        // The two choices on one line, the way the other menus put their hints together: apply
        // first, then the way out, each keeping the button order its own language wrote.
        let apply = ctx.i18n.spans(HINT_APPLY_KEY, &[]);
        let cancel = ctx.i18n.spans(HINT_CANCEL_KEY, &[]);
        let w_apply = crate::face::spans_width(ctx, &apply, crate::PX_HINT);
        let w_cancel = crate::face::spans_width(ctx, &cancel, crate::PX_HINT);
        let hints_x = bx + (BOX_W - (w_apply + crate::PX_HINT + w_cancel)) / 2.0;
        let hints_y = by + HINTS_Y;
        crate::draw_spans(
            canvas,
            ctx,
            &apply,
            crate::PX_HINT,
            hints_x,
            hints_y,
            crate::splash::INK_DIM,
        );
        crate::draw_spans(
            canvas,
            ctx,
            &cancel,
            crate::PX_HINT,
            hints_x + w_apply + crate::PX_HINT,
            hints_y,
            crate::splash::INK_DIM,
        );
    }
}

/// `+09:00`, `-08:00`, `+05:45`: the sign of the offset, then its hours and minutes two digits
/// each.
///
/// Zero keeps its sign, so a value the player has been moving is never mistaken for one nobody
/// has touched. The three letters in front of it are not here: the message owns its own words,
/// and this owns the number.
pub fn format_offset(minutes: i32) -> String {
    let minutes = clamp(minutes);
    let sign = if minutes < 0 { '-' } else { '+' };
    // `clamp` has already kept i32::MIN out of the way, so the magnitude cannot overflow.
    let abs = minutes.abs();
    format!("{sign}{:02}:{:02}", abs / 60, abs % 60)
}

/// One whole message, centred in the box.
fn line(
    canvas: &mut dyn Canvas,
    ctx: &mut UiCtx,
    spans: &[Span],
    px: f32,
    bx: f32,
    y: f32,
    color: Color,
) {
    let w = crate::face::spans_width(ctx, spans, px);
    crate::draw_spans(canvas, ctx, spans, px, bx + (BOX_W - w) / 2.0, y, color);
}

/// A value inside the range the card and the clock share. Both ends are inclusive, and nothing
/// wraps: +14:00 followed by -12:00 would read as a 26-hour jump rather than as a setting.
fn clamp(minutes: i32) -> i32 {
    minutes.clamp(MIN_MINUTES, MAX_MINUTES)
}
