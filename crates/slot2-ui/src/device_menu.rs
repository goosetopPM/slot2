//! The Device submenu: volume, brightness and blue light.
//!
//! Reached from the in-game menu's Device row (task 63 wires that). Mirrors the display and
//! core submenus deliberately: the same overlay language, panel, spacing, text sizes and
//! palette, and no menu framework of its own.
//!
//! The model owns no backend and touches no file system. Volume is always available — the
//! audio sink is ours — while brightness and blue light arrive as `Option<u8>`, where `None`
//! means "this machine has not been shown to have it", not zero and not an error. An
//! unavailable row is drawn and never selected: the screen is the same everywhere, without
//! offering a control that does nothing. Nothing here guesses a sysfs path or a PWM channel;
//! that decision is a platform task's to make.
//!
//! Draw order: full-panel dim (BLACK at alpha 0.6), box centred in the safe area (BACKDROP),
//! title at PX_BODY in INK_DIM centred with top at box_y + PAD, the three ROWS each ROW_H tall
//! starting at box_y + PAD + 28 (the selected available row backed with INK at alpha 0.15, its
//! label PX_TITLE on the left and its value text PX_BODY on the right, with the level bar
//! under both: a track and a fill whose width is the level), then the hint line at PX_HINT in
//! INK_DIM at the bottom of the box — left/right adjust, the mute hint only while Volume is
//! selected, and back.
//!
//! draw overlays the paused game's last frame and does not clear.

use slot2_gfx::{Canvas, Color};
use slot2_i18n::{Arg, Span};

use crate::UiCtx;

/// The panel's title: the in-game row the player pressed to get here.
pub const TITLE_KEY: &str = "ingame-device";
/// A percentage of a level, with the number passed in.
pub const PERCENT_KEY: &str = "device-percent";
/// What a muted Volume row says where its percentage would be.
pub const MUTED_KEY: &str = "device-muted";
/// What a row says when this machine has no such control.
pub const UNAVAILABLE_KEY: &str = "device-unavailable";
/// How a level is raised and lowered.
pub const HINT_ADJUST_KEY: &str = "hint-adjust";
/// What A does on the Volume row.
pub const HINT_MUTE_KEY: &str = "hint-mute";
/// Leaving the submenu — the in-game menus' own back hint.
pub const HINT_BACK_KEY: &str = "hint-back";

/// The highest level any setting can be, and the whole length of a bar.
pub const MAX_LEVEL: u8 = 100;

pub const BOX_W: f32 = 340.0;
// PAD + title area + three rows + the hint line, so everything fits 640x480 with room to spare.
pub const BOX_H: f32 = 264.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 56.0;
pub const BAR_H: f32 = 6.0;
/// Inset of a row's label, value text and bar from the row's own edges.
pub const INSET: f32 = 12.0;
/// The bar's distance from the bottom of its row.
pub const BAR_BOTTOM: f32 = 10.0;
/// Where the label and the value text are centred in their row.
pub const TEXT_MID: f32 = 22.0;
/// The one highlighted row's backing.
pub const HIGHLIGHT_ALPHA: f32 = 0.15;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

/// The three settings, in the order the rows are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceSetting {
    Volume,
    Brightness,
    BlueLight,
}

impl DeviceSetting {
    /// The message key for the row's label.
    pub const fn key(self) -> &'static str {
        match self {
            DeviceSetting::Volume => "device-volume",
            DeviceSetting::Brightness => "device-brightness",
            DeviceSetting::BlueLight => "device-blue-light",
        }
    }
}

pub const ROWS: [DeviceSetting; 3] = [
    DeviceSetting::Volume,
    DeviceSetting::Brightness,
    DeviceSetting::BlueLight,
];

/// The bar's track. An available row's is the louder one; an unavailable row's is quieter, so
/// a control that cannot be used never reads like one that can.
pub fn track_color(available: bool) -> Color {
    crate::splash::INK.with_alpha(if available { 0.15 } else { 0.07 })
}

/// What a bar's fill is drawn in. A muted Volume keeps its remembered length and is drawn
/// dimmed: the level is still there to be restored, and what is coming out of the speaker is
/// nothing.
pub fn fill_color(muted: bool) -> Color {
    crate::splash::INK.with_alpha(if muted { 0.35 } else { 1.0 })
}

/// The Device submenu, open on one of its three rows.
///
/// The levels are a snapshot: a backend applies a change first and hands the value back, so
/// what the screen shows is what the machine is actually doing. Every field is private so a
/// caller cannot make an unavailable setting look usable; read them back with `value`,
/// `is_available` and `volume_muted`.
///
/// `Copy` because the app keeps it in a `Screen`, which is itself `Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceMenu {
    selected: DeviceSetting,
    volume: u8,
    muted: bool,
    brightness: Option<u8>,
    blue_light: Option<u8>,
}

impl DeviceMenu {
    /// The menu for a machine that reports `volume` (always), `brightness` and `blue_light`
    /// (`None` when there is no such control to offer).
    ///
    /// Levels above [`MAX_LEVEL`] are clamped rather than refused: a backend handing over 120
    /// means "all of it", and a full bar is the honest picture of that.
    pub fn new(volume: u8, muted: bool, brightness: Option<u8>, blue_light: Option<u8>) -> Self {
        DeviceMenu {
            // Volume is the first row and always available, so the selection always starts
            // somewhere the player can act.
            selected: DeviceSetting::Volume,
            volume: volume.min(MAX_LEVEL),
            muted,
            brightness: brightness.map(|v| v.min(MAX_LEVEL)),
            blue_light: blue_light.map(|v| v.min(MAX_LEVEL)),
        }
    }

    /// The row the highlight is on.
    pub fn selected(&self) -> DeviceSetting {
        self.selected
    }

    /// Whether this machine has the setting at all. Volume always does: the sink is ours.
    pub fn is_available(&self, setting: DeviceSetting) -> bool {
        match setting {
            DeviceSetting::Volume => true,
            DeviceSetting::Brightness => self.brightness.is_some(),
            DeviceSetting::BlueLight => self.blue_light.is_some(),
        }
    }

    /// The level a row shows, or `None` when the setting is unavailable. `None` is never a
    /// zero: nothing here turns "cannot" into "off".
    pub fn value(&self, setting: DeviceSetting) -> Option<u8> {
        match setting {
            DeviceSetting::Volume => Some(self.volume),
            DeviceSetting::Brightness => self.brightness,
            DeviceSetting::BlueLight => self.blue_light,
        }
    }

    /// Whether the volume is muted. Muting belongs to Volume and to nothing else.
    pub fn volume_muted(&self) -> bool {
        self.muted
    }

    /// Show a level a backend has actually applied.
    ///
    /// A setting this machine does not have stays exactly as it was: a value arriving for it
    /// is a caller's mistake, and quietly turning the row into a usable control would be a lie
    /// about the hardware. Everything is clamped to 0..=100.
    pub fn set_value(&mut self, setting: DeviceSetting, level: u8) {
        let level = level.min(MAX_LEVEL);
        match setting {
            DeviceSetting::Volume => self.volume = level,
            DeviceSetting::Brightness => {
                if let Some(v) = self.brightness.as_mut() {
                    *v = level;
                }
            }
            DeviceSetting::BlueLight => {
                if let Some(v) = self.blue_light.as_mut() {
                    *v = level;
                }
            }
        }
    }

    /// Show that the volume was muted or unmuted. The level is kept either way: unmuting goes
    /// back to where the player was.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn toggle_muted(&mut self) {
        self.muted = !self.muted;
    }

    /// Move the highlight up one available row, wrapping. Unavailable rows are stepped over:
    /// they are on screen to be understood, not to be chosen.
    pub fn up(&mut self) {
        self.selected = self.walk(-1);
    }

    /// Move the highlight down one available row, wrapping.
    pub fn down(&mut self) {
        self.selected = self.walk(1);
    }

    /// The next available row in `dir`. Volume is always available, so the walk always has an
    /// answer and a menu with nothing else to offer stays where it is.
    fn walk(&self, dir: isize) -> DeviceSetting {
        let n = ROWS.len() as isize;
        let start = ROWS
            .iter()
            .position(|row| *row == self.selected)
            .unwrap_or(0) as isize;
        for step in 1..=n {
            let row = ROWS[(start + dir * step).rem_euclid(n) as usize];
            if self.is_available(row) {
                return row;
            }
        }
        // Unreachable while Volume is available, and the honest answer if that ever changes.
        self.selected
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

    /// The bar's track for row `i`, as `(x, y, w, h)`.
    pub fn bar_rect(ctx: &UiCtx, i: usize) -> (f32, f32, f32, f32) {
        let (bx, _) = Self::box_origin(ctx);
        (
            bx + PAD + INSET,
            Self::row_y(ctx, i) + ROW_H - BAR_BOTTOM - BAR_H,
            BOX_W - 2.0 * (PAD + INSET),
            BAR_H,
        )
    }

    /// How wide the filled part of a bar is at `level`: nothing at 0, the whole track at
    /// [`MAX_LEVEL`], and never past either end.
    pub fn fill_width(ctx: &UiCtx, level: u8) -> f32 {
        let (_, _, track, _) = Self::bar_rect(ctx, 0);
        track * f32::from(level.min(MAX_LEVEL)) / f32::from(MAX_LEVEL)
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

        for (i, setting) in ROWS.iter().enumerate() {
            let row_y = Self::row_y(ctx, i);
            let available = self.is_available(*setting);
            if *setting == self.selected {
                // Only an available row can be selected, so this backing never lands on a row
                // the player cannot act on.
                canvas.rect(
                    bx + PAD,
                    row_y,
                    BOX_W - 2.0 * PAD,
                    ROW_H,
                    crate::splash::INK.with_alpha(HIGHLIGHT_ALPHA),
                );
            }

            let label_spans = ctx.i18n.spans(setting.key(), &[]);
            let line_h = ctx.fonts.measure("", crate::PX_TITLE).line_height as f32;
            crate::draw_spans(
                canvas,
                ctx,
                &label_spans,
                crate::PX_TITLE,
                bx + PAD + INSET,
                row_y + TEXT_MID - line_h / 2.0,
                if available {
                    crate::splash::INK
                } else {
                    crate::splash::INK_DIM
                },
            );

            // The number, the word for muted, or the word for "not here". Never all three, and
            // never a percentage standing in for a control that does not exist.
            let muted = *setting == DeviceSetting::Volume && self.muted;
            let (value_spans, value_color) = if !available {
                (ctx.i18n.spans(UNAVAILABLE_KEY, &[]), crate::splash::INK_DIM)
            } else if muted {
                (ctx.i18n.spans(MUTED_KEY, &[]), crate::splash::INK_DIM)
            } else {
                let level = self.value(*setting).unwrap_or(0);
                (
                    ctx.i18n
                        .spans(PERCENT_KEY, &[("value", Arg::from(i64::from(level)))]),
                    crate::splash::INK,
                )
            };
            let value_w = crate::face::spans_width(ctx, &value_spans, crate::PX_BODY);
            let value_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
            crate::draw_spans(
                canvas,
                ctx,
                &value_spans,
                crate::PX_BODY,
                bx + BOX_W - PAD - INSET - value_w,
                row_y + TEXT_MID - value_h / 2.0,
                value_color,
            );

            // The bar: a track that is always the whole width, and a fill that is the level.
            let (bar_x, bar_y, bar_w, _) = Self::bar_rect(ctx, i);
            canvas.rect(bar_x, bar_y, bar_w, BAR_H, track_color(available));
            if available {
                let fill = Self::fill_width(ctx, self.value(*setting).unwrap_or(0));
                // Nothing at zero: a zero-width quad is a draw call that says nothing.
                if fill > 0.0 {
                    canvas.rect(bar_x, bar_y, fill, BAR_H, fill_color(muted));
                }
            }
        }

        // The hints: how to change a level, what A does here, and the way out. The mute hint is
        // only true on the Volume row, and an unavailable row is never selected, so no hint is
        // ever offered for a control that is not there.
        let mut keys: Vec<&str> = vec![HINT_ADJUST_KEY];
        if self.selected == DeviceSetting::Volume {
            keys.push(HINT_MUTE_KEY);
        }
        keys.push(HINT_BACK_KEY);
        let lines: Vec<Vec<Span>> = keys.iter().map(|key| ctx.i18n.spans(key, &[])).collect();
        let widths: Vec<f32> = lines
            .iter()
            .map(|spans| crate::face::spans_width(ctx, spans, crate::PX_HINT))
            .collect();
        let total: f32 =
            widths.iter().sum::<f32>() + crate::PX_HINT * (lines.len().saturating_sub(1)) as f32;
        let mut pen = bx + (BOX_W - total) / 2.0;
        let hint_y = by + BOX_H - PAD - crate::PX_HINT;
        for (spans, w) in lines.iter().zip(widths.iter()) {
            crate::draw_spans(
                canvas,
                ctx,
                spans,
                crate::PX_HINT,
                pen,
                hint_y,
                crate::splash::INK_DIM,
            );
            pen += w + crate::PX_HINT;
        }
    }
}
