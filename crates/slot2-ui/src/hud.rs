//! What the frontend says about the machine rather than about the games: how much charge is
//! left and what time it is, in the panel's top corners, and — while the player is winding
//! time backwards or forwards — one badge at the top centre.
//!
//! All of it is pinned to the panel, not to the safe area (DESIGN 167). The safe area is where
//! the *layout* lives -- the row, the slot, the toast -- because those have to hold together
//! on three panel shapes. The HUD belongs to the screen instead: on the 720x720 it rides up
//! into the extended band with the wallpaper, where it is out of the way of the shelf, and
//! on the 640x480 it lands in the same corner it would have anyway. The badge follows the same
//! rule: centred on the panel, in that band, not on the layout below it.
//!
//! Everything sits on a plate. A wallpaper is a photograph the player chose, and white ink
//! on an unknown photograph is ink that is sometimes not there at all.

use slot2_gfx::{Canvas, Color};
use slot2_i18n::Arg;
use slot2_platform::{clock, Battery, Charge};

use crate::art::ArtCache;
use crate::face;
use crate::layout::SafeArea;
use crate::UiCtx;

/// The charging mark. Its own drawing rather than a glyph, because the font that has a bolt
/// is not the font that is on the card.
pub const BOLT_SVG: &str = include_str!("../../../assets/art/bolt.svg");

/// How far the plates are held off the panel edges.
pub const HUD_MARGIN: f32 = 12.0;

/// The plate, and what both clusters are centred in. One height for both corners so the band
/// reads as one thing rather than as two widgets that happen to be at the same altitude.
///
/// A flat number rather than `PX_HINT` plus padding, because the thing that has to fit is
/// not the em size but the line box: `PX_HINT` text is 20 px of ascent and descent on the UI
/// font, and a plate sized off the 14 would have the digits sitting exactly on its edge.
/// Everything in the band is centred on this on its own measured height -- the capsule is
/// `GAUGE_H`, the text is whatever the font chain rasterised -- so neither has to know the
/// other is there.
pub const HUD_H: f32 = 28.0;
/// Padding either side of the contents, inside the plate.
const PAD_X: f32 = 10.0;

/// The capsule, in the proportions of the thing it is a picture of: wider than tall, with a
/// nub on the positive end.
pub const GAUGE_W: f32 = 22.0;
pub const GAUGE_H: f32 = 11.0;
// The doc comment above claims wider than tall; this is what makes that a fact the compiler
// enforces rather than a sentence someone could quietly falsify by editing one constant.
const _: () = assert!(GAUGE_H < GAUGE_W);

/// The wall of the capsule, drawn as four rects rather than an outline: the canvas has only
/// filled quads. Public so a test can bound the fill against the capsule inner edge instead
/// of trusting a hand-copied number that could drift from this one.
pub const WALL: f32 = 1.5;
pub const NUB_W: f32 = 2.5;
pub const NUB_H: f32 = 4.0;

/// The bolt slot, left of the capsule, reserved whether or not a cable is in.
///
/// Squeezed inside the capsule the bolt had an 8 px box to live in, came out as a smudge,
/// and punched a hole in whatever fill was under it. Out here it sits at a size that reads.
/// Reserved unconditionally because a slot that appears when the cable goes in is a capsule
/// that jumps sideways at the moment the player is looking at it.
pub const BOLT_W: f32 = 14.0;
pub const BOLT_H: f32 = 14.0;
/// Between the bolt slot and the capsule. A zero or negative gap would let the bolt quad
/// touch the capsule, which is the defect this separation exists to avoid.
pub const BOLT_GAP: f32 = 5.0;
const _: () = assert!(BOLT_GAP > 0.0);

/// Between the capsule and the number.
const GAP: f32 = 7.0;
/// Text size of the percent and the clock. The hint size, so the corner furniture reads as
/// quieter than anything the layout says.
const PX_TEXT: f32 = crate::PX_HINT;

/// Below this, and going down, the gauge changes colour.
///
/// Fifteen is about twenty minutes of play on this pack, which is time to find a cable and
/// not so much warning that the colour becomes part of the furniture.
pub const LOW_PERCENT: u8 = 15;

/// One ink for the wall, the fill and the number, so each cluster reads as one control.
pub const INK: Color = Color::from_rgb8(0xF5, 0xF2, 0xEF);
/// What a low battery that is still going down is drawn in.
pub const LOW_INK: Color = Color::from_rgb8(0xE5, 0x6B, 0x4A);
/// The plate both clusters sit on. The toast plate, at the alpha something peripheral wants.
const PLATE: Color = Color::from_rgb8(0x1E, 0x21, 0x26);
const PLATE_ALPHA: f32 = 0.55;

/// What the time controls are doing, for the badge at the top of the panel.
///
/// Two things and no more: winding backwards, or running forwards faster than real time. The
/// speed travels with the fast forward so the badge can say `4×` without knowing what the App
/// decided it is, and so a second speed needs no new variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeControl {
    Rewind,
    FastForward { speed: u32 },
}

/// The corner furniture, and the bolt it keeps rasterised.
///
/// Holds a cache rather than borrowing the shelf cache because the HUD outlives the shelf:
/// it is drawn over the power menu and over an empty card too.
#[derive(Default)]
pub struct Hud {
    art: ArtCache,
}

impl Hud {
    /// Draw both corners. `battery` is the last reading, `None` where there is no gauge;
    /// `local_secs` is the local time in epoch seconds, `None` where the clock was never
    /// set. Either being `None` draws that corner not at all -- an empty capsule says the
    /// battery is flat, and a confident `09:00` off a dead RTC is worse than no clock.
    ///
    /// Draws in panel coordinates off `safe.panel_w`/`panel_h`, never off `safe.x`/`y`.
    pub fn draw(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        safe: &SafeArea,
        battery: Option<Battery>,
        local_secs: Option<i64>,
    ) {
        // Panel corners, never the safe-area offset: the HUD belongs to the screen, and on
        // the square panel the safe area sits 120 px down (DESIGN 167).
        let top = HUD_MARGIN;
        let band_mid = top + HUD_H / 2.0;
        let panel_w = safe.panel_w as f32;

        if let Some(secs) = local_secs {
            let text = clock_text(secs);
            let text_w = face::measure(ctx, &text, PX_TEXT);
            let x = HUD_MARGIN;
            canvas.rect(
                x,
                top,
                text_w + 2.0 * PAD_X,
                HUD_H,
                PLATE.with_alpha(PLATE_ALPHA),
            );
            // Centred on the band, on the face's own height, so the digits sit mid-plate
            // whatever the font chain rasterised.
            let f = face::face(canvas, ctx, &text, PX_TEXT);
            let y = band_mid - f.h as f32 / 2.0;
            face::draw_text(canvas, ctx, &text, PX_TEXT, x + PAD_X, y, INK);
        }

        if let Some(b) = battery {
            let w = gauge_width(ctx, b);
            let x = panel_w - HUD_MARGIN - w;
            canvas.rect(x, top, w, HUD_H, PLATE.with_alpha(PLATE_ALPHA));
            draw_gauge(canvas, ctx, self, x + PAD_X, band_mid, b);
        }
    }

    /// Draw the time-control badge, when there is one, at the top centre of the panel.
    ///
    /// `None` draws nothing at all, and this draws nothing else: no clock, no battery, no
    /// clear. The caller owns the frame and decides whether the screen on show permits a
    /// badge at all.
    ///
    /// Panel coordinates, like the corners, and the same band and margin: on the square panel
    /// the badge belongs in the extended band above the layout, not 120 px down with it. The
    /// message is measured first — a language decides how wide `되감기` is — and the plate is
    /// the text plus the padding the corners use.
    pub fn draw_time_control(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        safe: &SafeArea,
        control: Option<TimeControl>,
    ) {
        let Some(control) = control else {
            return;
        };
        let text = match control {
            TimeControl::Rewind => ctx.i18n.t("time-rewind"),
            TimeControl::FastForward { speed } => ctx
                .i18n
                .t_args("time-fast-forward", &[("speed", Arg::from(speed))]),
        };

        let text_w = face::measure(ctx, &text, PX_TEXT);
        let w = text_w + 2.0 * PAD_X;
        let x = (safe.panel_w as f32 - w) / 2.0;
        let top = HUD_MARGIN;
        canvas.rect(x, top, w, HUD_H, PLATE.with_alpha(PLATE_ALPHA));
        // Centred on the band on the face's own height, exactly as the corners are.
        let f = face::face(canvas, ctx, &text, PX_TEXT);
        face::draw_text(
            canvas,
            ctx,
            &text,
            PX_TEXT,
            x + PAD_X,
            top + HUD_H / 2.0 - f.h as f32 / 2.0,
            INK,
        );
    }
}

/// One corner of the HUD: the bolt slot, the capsule and the percent, left to right from
/// `(x, band_mid)` -- the mid-line of the band, on which everything in it is centred on its
/// own measured height.
fn draw_gauge(
    canvas: &mut dyn Canvas,
    ctx: &mut UiCtx,
    hud: &mut Hud,
    x: f32,
    band_mid: f32,
    battery: Battery,
) {
    let ink = ink(battery);

    // The bolt slot, reserved whether or not anything is charging. Drawing it empty keeps
    // the capsule still when a cable moves (the contract behind `BOLT_GAP`).
    let bolt_y = band_mid - BOLT_H / 2.0;
    if battery.charge == Charge::Charging {
        if let Some(tex) = hud.art.mask(canvas, BOLT_SVG, BOLT_W as u32, BOLT_H as u32) {
            canvas.image(tex, x, bolt_y, BOLT_W, BOLT_H, ink);
        }
    }

    // The capsule, right of the slot: four walls, then the fill inside them, then the nub
    // on the positive end. The walls sit on whole pixels so a 1.5 px wall does not straddle
    // two of them and read at half strength.
    let cap_x = x + BOLT_W + BOLT_GAP;
    let cap_y = (band_mid - GAUGE_H / 2.0).round();
    let cap_w = GAUGE_W;
    let cap_h = GAUGE_H;
    let wall = WALL;
    canvas.rect(cap_x, cap_y, cap_w, wall, ink);
    canvas.rect(cap_x, cap_y + cap_h - wall, cap_w, wall, ink);
    canvas.rect(cap_x, cap_y + wall, wall, cap_h - 2.0 * wall, ink);
    canvas.rect(
        cap_x + cap_w - wall,
        cap_y + wall,
        wall,
        cap_h - 2.0 * wall,
        ink,
    );

    let fill_h = cap_h - 4.0 * wall;
    let inner_w = cap_w - 4.0 * wall;
    let fill_w = inner_w * f32::from(battery.percent) / 100.0;
    if fill_w > 0.0 {
        canvas.rect(cap_x + 2.0 * wall, cap_y + 2.0 * wall, fill_w, fill_h, ink);
    }

    canvas.rect(
        cap_x + cap_w,
        cap_y + (cap_h - NUB_H) / 2.0,
        NUB_W,
        NUB_H,
        ink,
    );

    // The number, right of the nub, in the same ink as the capsule so the cluster reads as
    // one control. The percent sign is part of the text, not decoration the caller adds.
    let text = percent_text(battery);
    let f = face::face(canvas, ctx, &text, PX_TEXT);
    let text_x = cap_x + cap_w + NUB_W + GAP;
    face::draw_text(
        canvas,
        ctx,
        &text,
        PX_TEXT,
        text_x,
        band_mid - f.h as f32 / 2.0,
        ink,
    );
}

/// The ink a reading is drawn in: the warning colour only when it is low *and* going down.
///
/// A pack at 8% with a cable in it is a pack on its way up, and a red gauge there is a
/// warning about a problem that is already being fixed.
pub fn ink(battery: Battery) -> Color {
    if battery.charge == Charge::Discharging && battery.percent < LOW_PERCENT {
        LOW_INK
    } else {
        INK
    }
}

/// What the number says. The percent sign is part of it: a bare `72` beside a capsule is a
/// number that could be minutes.
pub fn percent_text(battery: Battery) -> String {
    format!("{}%", battery.percent)
}

/// What the clock says, which is `clock::hhmm` and nothing else. Here so the HUD has one
/// place that turns seconds into the string the face cache is keyed on.
pub fn clock_text(local_secs: i64) -> String {
    clock::hhmm(local_secs)
}

/// How wide the battery cluster is, plate included, at this reading. The number is the only
/// part that changes width, and the cluster is right-anchored, so this is what decides where
/// its left edge falls.
pub fn gauge_width(ctx: &mut UiCtx, battery: Battery) -> f32 {
    let text = percent_text(battery);
    PAD_X + BOLT_W + BOLT_GAP + GAUGE_W + NUB_W + GAP + face::measure(ctx, &text, PX_TEXT) + PAD_X
}
