//! The row of cartridges, and where each one stands this frame.
//!
//! Ported from the original frontend's carousel, whose constants were argued out against a
//! real device. The reasoning is carried across with them: a number without its reason is a
//! number the next person deletes.
//!
//! This module does no drawing. It answers "where does each cart go", and something else
//! turns that into canvas calls.

use crate::layout::SafeArea;

/// How far apart carts sit along the row, in artwork units.
///
/// Chosen so the outer two sit fully on screen with the margin at the edge equal to the gap
/// beside the centre cart. At 286 the side carts were clipped off each edge.
pub const PITCH: f32 = 240.0;

/// How much smaller a cart beside the selection is drawn.
pub const SIDE_SCALE: f32 = 0.78;
pub const SIDE_ALPHA: f32 = 0.55;

/// Slots considered either side of the selection. Two reach the edges of a 720 row; the
/// third covers the lag while the spring is still catching up with a flick.
pub const SLOTS: i32 = 3;

/// Critically damped, so a flick lands on a cart instead of bouncing past and returning.
const OMEGA: f32 = 16.0;

// The slot the carts go into. Its proportions live here rather than with the drawing code
// because where the row stands is measured off them.

/// How much wider than a cartridge the slot's mouth is.
pub const MOUTH_EXTRA: f32 = 14.0;
/// The band of chrome across the foot of the panel that the mouth is cut into.
pub const MOUTH_H: f32 = 58.0;
/// The opening itself.
pub const SLIT_H: f32 = 9.0;
/// A highlight along the top edge of the band, which is what makes it read as a surface
/// standing proud of the background rather than a painted rectangle.
pub const LIP_H: f32 = 2.0;

/// How far above the slot the row's centre line sits.
///
/// The row is placed against the slot, not against the middle of the panel. What a shelf
/// shows is cartridges standing above the thing they go into, and measuring from the panel
/// centre breaks that as soon as the panel is taller: on a 720x720 the slot dropped to the
/// foot and the row stayed where it was, leaving them unrelated with a field of background
/// between.
///
/// The number is not chosen, it is measured: on a 480-tall panel, centring the row put its
/// centre at 240 and the slot band at 422. Keeping the distance keeps every 480-tall panel
/// exactly as it was, and moves the taller one down to match.
pub const CENTRE_ABOVE_SLOT: f32 = 182.0;

/// Where one cartridge sits on the row this frame, in panel pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// Which cart in the caller's list this is.
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// 1.0 for the selection, down to `SIDE_ALPHA` for its neighbours.
    pub alpha: f32,
}

pub struct Shelf {
    len: usize,
    index: usize,
    /// Where the row is, in pitches. Fractional while it slides.
    scroll: f32,
    vel: f32,
    /// Where it is sliding to. Fixed when the selection moves rather than recomputed from
    /// `scroll`, or the spring would chase a target that shifts under it every frame.
    target: f32,
}
impl Default for Shelf {
    fn default() -> Self {
        Shelf::new(0)
    }
}

impl Shelf {
    pub fn new(len: usize) -> Shelf {
        Shelf {
            len,
            index: 0,
            scroll: 0.0,
            vel: 0.0,
            target: 0.0,
        }
    }

    /// After a rescan. Clamps the selection onto the new row.
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        if len == 0 {
            *self = Shelf::new(0);
            return;
        }
        self.index = self.index.min(len - 1);
        self.target = self.nearest_image();
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn selected(&self) -> usize {
        self.index
    }

    pub fn select(&mut self, i: usize) {
        self.index = if self.len == 0 {
            0
        } else {
            i.min(self.len - 1)
        };
        self.target = self.nearest_image();
    }

    pub fn left(&mut self) {
        self.step(-1);
    }

    pub fn right(&mut self) {
        self.step(1);
    }

    fn step(&mut self, by: i32) {
        if self.len == 0 {
            return;
        }
        let n = self.len as i32;
        self.index = (self.index as i32 + by).rem_euclid(n) as usize;
        self.target = self.nearest_image();
    }

    /// The scroll position nearest the one we are at that shows the selected cart.
    ///
    /// The row is a ring, so every cart has infinitely many positions a pitch apart; this
    /// picks the closest. Wrapping from the first cart to the last then slides one pitch
    /// rather than winding back through the whole library.
    fn nearest_image(&self) -> f32 {
        if self.len == 0 {
            return 0.0;
        }
        let from = self.scroll.round();
        let n = self.len as f32;
        from + (self.index as f32 - from + n / 2.0).rem_euclid(n) - n / 2.0
    }

    /// Where the row is sliding to, in pitches.
    pub fn scroll_target(&self) -> f32 {
        self.target
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    /// Advance the spring. `dt` in seconds.
    pub fn update(&mut self, dt: f32) {
        let accel = -2.0 * OMEGA * self.vel - OMEGA * OMEGA * (self.scroll - self.target);
        self.vel += accel * dt;
        self.scroll += self.vel * dt;
    }

    /// True while the row is still moving.
    pub fn settling(&self) -> bool {
        (self.scroll - self.target).abs() > 0.001 || self.vel.abs() > 0.001
    }

    /// The cart `off` slots right of the selection, or `None` when that slot stays empty.
    ///
    /// A row of **two** fills every slot, which means one of the two carts is drawn twice at
    /// once. That is deliberate, and it was arrived at on the device after trying the
    /// alternatives: a row with a hole where the repeat would be, and a centred pair.
    /// Neither scrolls — there is nothing to put in the slot the row moves into. A carousel
    /// of two is a thing that shows one of them twice.
    ///
    /// A row of **one** does not repeat. The selection never changes, so nothing would ever
    /// move, and three identical faces standing still read as a drawing fault rather than as
    /// a ring.
    ///
    /// Filling every slot rather than only the three on screen is what makes the scroll
    /// continuous: each offset holds the same cart before and after a press, so the row
    /// slides by a pitch instead of a cart blinking out at one edge and in at the other.
    /// `placements` throws away the ones that land off the panel.
    pub fn cart_at_offset(&self, off: i32) -> Option<usize> {
        let n = self.len as i32;
        if n == 0 {
            return None;
        }
        let at = |off: i32| (self.index as i32 + off).rem_euclid(n) as usize;
        if n == 2 {
            return Some(at(off));
        }
        // Past halfway round, a slot is better reached from the other side; only the
        // nearest image of a cart gets drawn, so a short row leaves its far slots empty.
        let r = off.rem_euclid(n);
        let nearest = if r * 2 > n { r - n } else { r };
        (nearest == off).then(|| at(off))
    }

    /// Every cart to draw this frame, nearest the selection last so it lands on top.
    /// Anything entirely off the panel is dropped.
    ///
    /// Laid out against the panel, not the 640×480 safe area: DESIGN §5 gives the extension
    /// either side of the safe area to the background and the shelf, so a wider panel shows
    /// more of the neighbours rather than moving the selection off centre.
    pub fn placements(&self, safe: &SafeArea, cart: (f32, f32)) -> Vec<Placement> {
        if self.len == 0 {
            return Vec::new();
        }
        let panel_w = safe.panel_w as f32;
        let panel_h = safe.panel_h as f32;
        let (natural_w, natural_h) = cart;

        // Every platform's carts share a *centre* line, not a floor. A Game Boy pak is 253
        // units tall against a GBA cart's 135, and measured from a shared floor the pak sat
        // 59px higher and crowded the top of the screen. A shrunken neighbour then keeps its
        // foot on the line the selection stands on rather than shrinking about its own
        // middle, so the row reads as objects standing on a shelf instead of carts floating.
        //
        // That shared line is placed against the slot rather than the panel's middle; see
        // `CENTRE_ABOVE_SLOT`.
        let centre = panel_h - MOUTH_H - CENTRE_ABOVE_SLOT;
        let floor = centre + natural_h / 2.0;

        let mut out: Vec<Placement> = (-SLOTS..=SLOTS)
            .filter_map(|slot| {
                let index = self.cart_at_offset(slot)?;
                // How far this slot is from the middle of the panel, in pitches. Fractional
                // while the row slides, which is what makes a cart shrink and fade *into*
                // the side rather than snapping there when the selection changes.
                let offset = self.target + slot as f32 - self.scroll;
                let away = offset.abs().min(1.0);
                let scale = 1.0 + (SIDE_SCALE - 1.0) * away;

                let w = natural_w * scale;
                let h = natural_h * scale;
                let x = panel_w / 2.0 + offset * PITCH - w / 2.0;
                (x + w > 0.0 && x < panel_w).then_some(Placement {
                    index,
                    x,
                    y: floor - h,
                    w,
                    h,
                    alpha: 1.0 + (SIDE_ALPHA - 1.0) * away,
                })
            })
            .collect();

        // Nearest the middle draws last. Sorting on distance rather than on "is it the
        // selection" keeps it right mid-scroll, when no cart is exactly the selection and
        // two of them are equally close to being it.
        out.sort_by(|a, b| {
            let d = |p: &Placement| ((p.x + p.w / 2.0) - panel_w / 2.0).abs();
            d(b).total_cmp(&d(a))
        });
        out
    }
}
