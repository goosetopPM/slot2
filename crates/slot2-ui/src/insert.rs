//! A cartridge going into the slot, and coming back out.
//!
//! Ported from the original frontend's `slot_chrome`, whose travel was argued out against a
//! real device. This module does no drawing: it answers "where is the cart this frame", and
//! `shelf_view` turns that into canvas calls.
//!
//! One thing did not port unchanged. The original measured the row from the middle of the
//! panel; SLOT2 measures it from the slot (`shelf::CENTRE_ABOVE_SLOT`), because a 720-tall
//! panel drops the slot to the foot and a row centred on the panel would be left hanging
//! unrelated to it. Everything here is therefore measured off the slot too, and the catch
//! works out the same fraction on every panel instead of drifting with the panel's height.

use crate::layout::SafeArea;
use crate::shelf::{CENTRE_ABOVE_SLOT, LIP_H, MOUTH_H};

/// The whole insert, in seconds.
pub const INSERT_S: f32 = 0.73;

/// How long the cart sits seated before the game takes the screen. This is the dwell that
/// hides the core load: the picture on screen while a core is coming up is a cartridge fully
/// in the slot, which is what loading looks like.
pub const INSERT_HOLD_S: f32 = 0.28;

/// When the cart reaches the slot. Everything after this is the hold.
pub const SEATED_AT: f32 = INSERT_S - INSERT_HOLD_S;

/// The way back out. The same length as the way in, without the hold — there is nothing to
/// wait for on the way out.
pub const EJECT_S: f32 = SEATED_AT;

/// How far the cart next to the selection is pushed aside as the chosen one goes in. Enough
/// to clear the frame from where it stands.
pub const PART: f32 = 130.0;

/// How far into the travel the cart is caught on the lip, as a fraction of the animation.
///
/// Fractions of the *animation*, not of the journey, and so the same two numbers for every
/// cartridge. A pak and a GBA cart do not fall the same distance — the row centres each
/// cartridge rather than standing them all on one floor — but the beat of the thing belongs
/// to the slot and not to the cartridge: the catch has to land on the same frame and the
/// hesitation has to last as long, or one shelf's insert reads as a different mechanism from
/// another's. What differs between them is speed, which is what ought to differ when one
/// object has further to go than another in the same time.
const CATCH_IN: f32 = 0.42;
const CATCH_OUT: f32 = 0.62;

/// How far the cart creeps while it is caught. Not zero: a dead stop reads as a dropped
/// frame, a crawl reads as resistance.
const CREEP: f32 = 0.03;

/// How far below the lip the cart's *top* edge comes to rest.
///
/// It is the top edge that stops here, so how much cartridge is left sticking out of the
/// machine is set by the slot and is the same for a pak as for a GBA cart. A seat derived
/// from the cartridge instead would swallow a tall one deeper.
const SEATED_BELOW_LIP: f32 = 4.0;

/// Where one cartridge is this frame, in panel pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Travel {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Smootherstep. Zero velocity at both ends, so the parts of the travel meet the catch
/// without a step in speed.
pub fn ease(u: f32) -> f32 {
    todo!()
}

/// How far the cart is into the slot `t` seconds into an insert: 0.0 standing on the row,
/// 1.0 swallowed.
pub fn seat_in(t: f32) -> f32 {
    todo!()
}

/// The same, `t` seconds into an eject: 1.0 seated, falling to 0.0 back on the row.
pub fn seat_out(t: f32) -> f32 {
    todo!()
}

/// Where the row's shared centre line is on this panel. The same line `shelf::placements`
/// stands its carts on, and asked for here rather than duplicated, so a cart cannot jump on
/// the frame the button is pressed.
pub fn row_centre(safe: &SafeArea) -> f32 {
    todo!()
}

/// Where a cartridge of height `h` stands on the row: its top edge.
pub fn rest_y(safe: &SafeArea, h: f32) -> f32 {
    todo!()
}

/// Where a cartridge of height `h` has its top edge once seated.
pub fn seated_y(safe: &SafeArea) -> f32 {
    todo!()
}

/// How far into the travel the cart's foot meets the lip.
///
/// Derived rather than tuned, because the catch is a collision: it happens where the foot
/// meets the lip and nowhere else, whatever that works out to in animation time. The
/// numerator is the drop from where the cart stands to where its foot lands; the denominator
/// is the whole journey.
fn catch_at(safe: &SafeArea, h: f32) -> f32 {
    todo!()
}

/// The fraction of the journey covered at `seat`, in three parts: the cart falls to the lip,
/// rests on it, then is pushed through and settles. A single ease covers the same ground but
/// arrives seated without ever having met anything, which is what makes it read as a card
/// going down a chute rather than a cartridge going into a machine.
fn journey(safe: &SafeArea, h: f32, seat: f32) -> f32 {
    todo!()
}

/// Where the cart is at `seat`.
///
/// `rest_x` is where the row would draw it, asked of the row rather than assumed to be the
/// middle of the panel: pressing A while the row is still sliding must not teleport the cart
/// to the centre on that frame. `cart` is the cartridge's natural size — a cart is an object
/// and goes into the slot at the size it is.
pub fn travel(safe: &SafeArea, cart: (f32, f32), rest_x: f32, seat: f32) -> Travel {
    todo!()
}
