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

/// Which way a cartridge is moving through the slot.
///
/// The two directions have their own profiles — a cartridge is pushed in against the
/// connector and falls back out of it — and a caller that knows the screen should not have to
/// remember which field of the skin that is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Insert,
    Eject,
}

/// Where a cartridge is on its way through the slot: how far in, and which way.
///
/// The two belong together. A seat means nothing without its direction — the same 0.5 is a
/// cartridge on its way down on one profile and one on its way up on the other — so a caller
/// hands over one value rather than two arguments it could pair wrongly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Insertion {
    pub seat: f32,
    pub motion: Motion,
}

/// One platform's travel profile: where in the animation the cartridge's foot meets the lip,
/// where it lets go, and how far it creeps while it is held.
///
/// All three are fractions of the *animation*, not of the journey, and the animation's length
/// is the same everywhere ([`INSERT_S`]): what a platform owns is its rhythm, not its
/// duration. A pak and a GBA cart do not fall the same distance — the row centres each
/// cartridge rather than standing them all on one floor — and each platform gives that distance
/// its own contact and release beat. The shared duration keeps the state-machine clock stable;
/// these values change only how speed is distributed inside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    contact: f32,
    release: f32,
    creep: f32,
}

impl Curve {
    /// The profile with these beats, as fractions of the whole animation.
    ///
    /// Bounds are checked where the table is built, so a mistyped platform fails the build
    /// rather than drawing an insert that stalls or runs backwards.
    pub const fn new(contact: f32, release: f32, creep: f32) -> Curve {
        assert!(contact > 0.0, "contact must be after the start");
        assert!(contact < release, "contact must come before release");
        assert!(release < 1.0, "release must be before the end");
        assert!(creep > 0.0, "a dead stop reads as a dropped frame");
        assert!(creep < 0.08, "the creep is a hesitation, not the push");
        Curve {
            contact,
            release,
            creep,
        }
    }

    /// Where along the animation the foot meets the lip.
    pub const fn contact(self) -> f32 {
        self.contact
    }

    /// Where along the animation the cartridge is pushed through.
    pub const fn release(self) -> f32 {
        self.release
    }

    /// How far the cartridge travels while it is held against the connector.
    pub const fn creep(self) -> f32 {
        self.creep
    }

    /// The fraction of the *journey* covered at `seat`, in three parts: the cart falls to the
    /// lip, rests on it, then is pushed through and settles.
    ///
    /// `collision` is the fraction of the journey at which the foot really meets the lip
    /// ([`collision_at`]) — derived from the panel and the cartridge, never tuned. A single
    /// ease covers the same ground but arrives seated without ever having met anything, which
    /// is what makes it read as a card going down a chute rather than a cartridge going into a
    /// machine.
    pub fn journey(self, collision: f32, seat: f32) -> f32 {
        let seat = seat.clamp(0.0, 1.0);
        let collision = collision.clamp(0.0, 1.0);
        // The held phase ends where the push begins; clamped so that a cartridge that had
        // already met the lip at the end of the journey cannot be dragged backwards.
        let held = (collision + self.creep).min(1.0);

        if seat < self.contact {
            collision * ease(seat / self.contact)
        } else if seat < self.release {
            let through = (seat - self.contact) / (self.release - self.contact);
            collision + (held - collision) * through
        } else {
            let settle = (seat - self.release) / (1.0 - self.release);
            held + (1.0 - held) * ease(settle)
        }
    }
}

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
    let u = u.clamp(0.0, 1.0);
    u * u * u * (u * (6.0 * u - 15.0) + 10.0)
}

/// How far the cart is into the slot `t` seconds into an insert: 0.0 standing on the row,
/// 1.0 swallowed.
pub fn seat_in(t: f32) -> f32 {
    (t / SEATED_AT).clamp(0.0, 1.0)
}

/// The same, `t` seconds into an eject: 1.0 seated, falling to 0.0 back on the row.
pub fn seat_out(t: f32) -> f32 {
    1.0 - (t / EJECT_S).clamp(0.0, 1.0)
}

/// Where the row's shared centre line is on this panel. The same line `shelf::placements`
/// stands its carts on, and asked for here rather than duplicated, so a cart cannot jump on
/// the frame the button is pressed.
pub fn row_centre(safe: &SafeArea) -> f32 {
    safe.panel_h as f32 - MOUTH_H - CENTRE_ABOVE_SLOT
}

/// Where a cartridge of height `h` stands on the row: its top edge.
pub fn rest_y(safe: &SafeArea, h: f32) -> f32 {
    row_centre(safe) - h / 2.0
}

/// Where a cartridge of height `h` has its top edge once seated.
pub fn seated_y(safe: &SafeArea) -> f32 {
    safe.panel_h as f32 - MOUTH_H + LIP_H + SEATED_BELOW_LIP
}

/// How far into the travel the cart's foot meets the lip, as a fraction of the journey.
///
/// Derived rather than tuned, because the catch is a collision: it happens where the foot
/// meets the lip and nowhere else, whatever that works out to in animation time. The
/// numerator is the drop from where the cart stands to where its foot lands; the denominator
/// is the whole journey. Every platform's [`Curve`] is then sampled against this same number,
/// which is why two shelves with different rhythms still seat the cartridge in the slot.
pub fn collision_at(safe: &SafeArea, h: f32) -> f32 {
    let lip_y = safe.panel_h as f32 - MOUTH_H;
    let rest_y = rest_y(safe, h);
    let seated_y = seated_y(safe);
    (lip_y - (rest_y + h)) / (seated_y - rest_y)
}

/// Where the cart is at `seat`, on the platform's own profile.
///
/// `rest_x` is where the row would draw it, asked of the row rather than assumed to be the
/// middle of the panel: pressing A while the row is still sliding must not teleport the cart
/// to the centre on that frame. `cart` is the cartridge's natural size — a cart is an object
/// and goes into the slot at the size it is.
pub fn travel(safe: &SafeArea, curve: Curve, cart: (f32, f32), rest_x: f32, seat: f32) -> Travel {
    let (w, h) = cart;
    let j = curve.journey(collision_at(safe, h), seat);
    let centred_x = (safe.panel_w as f32 - w) / 2.0;
    let seated_y = seated_y(safe);
    let rest_y = rest_y(safe, h);

    Travel {
        x: rest_x + (centred_x - rest_x) * j,
        y: rest_y + (seated_y - rest_y) * j,
        w,
        h,
    }
}
