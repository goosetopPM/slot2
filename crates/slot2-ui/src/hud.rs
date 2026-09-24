//! The two things the shelf says about the machine rather than about the games: how much
//! charge is left, and what time it is.
//!
//! Pinned to the panel corners, not to the safe area (DESIGN 167). The safe area is where
//! the *layout* lives -- the row, the slot, the toast -- because those have to hold together
//! on three panel shapes. The HUD belongs to the screen instead: on the 720x720 it rides up
//! into the extended band with the wallpaper, where it is out of the way of the shelf, and
//! on the 640x480 it lands in the same corner it would have anyway.
//!
//! Both clusters sit on a plate. A wallpaper is a photograph the player chose, and white ink
//! on an unknown photograph is ink that is sometimes not there at all.

use slot2_gfx::{Canvas, Color};
use slot2_platform::{clock, Battery, Charge};

use crate::art::ArtCache;
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
        let _ = (canvas, ctx, safe, battery, local_secs);
        todo!()
    }
}

/// The ink a reading is drawn in: the warning colour only when it is low *and* going down.
///
/// A pack at 8% with a cable in it is a pack on its way up, and a red gauge there is a
/// warning about a problem that is already being fixed.
pub fn ink(battery: Battery) -> Color {
    let _ = battery;
    todo!()
}

/// What the number says. The percent sign is part of it: a bare `72` beside a capsule is a
/// number that could be minutes.
pub fn percent_text(battery: Battery) -> String {
    let _ = battery;
    todo!()
}

/// What the clock says, which is `clock::hhmm` and nothing else. Here so the HUD has one
/// place that turns seconds into the string the face cache is keyed on.
pub fn clock_text(local_secs: i64) -> String {
    let _ = local_secs;
    todo!()
}

/// How wide the battery cluster is, plate included, at this reading. The number is the only
/// part that changes width, and the cluster is right-anchored, so this is what decides where
/// its left edge falls.
pub fn gauge_width(ctx: &mut UiCtx, battery: Battery) -> f32 {
    let _ = (ctx, battery);
    todo!()
}
