//! The contract for the shelf row and the artwork cache. Task 11 makes these pass without
//! editing this file.

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::Geometry;
use slot2_store::Platform;
use slot2_ui::art::ArtCache;
use slot2_ui::layout::SafeArea;
use slot2_ui::shelf::{Placement, Shelf, SIDE_ALPHA, SIDE_SCALE};
use slot2_ui::skin;

/// A GBA cart, which is what most of these are laid out with.
const CART: (f32, f32) = (240.0, 135.0);

fn safe(g: Geometry) -> SafeArea {
    SafeArea::for_geometry(g)
}

const GEOMETRIES: [Geometry; 3] = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];

/// The placement of the selected cart, which is the one at full size and full alpha.
fn selected(p: &[Placement], index: usize) -> &Placement {
    p.iter()
        .find(|q| q.index == index && q.alpha >= 0.999)
        .expect("the selection was not placed")
}

// --- the row -------------------------------------------------------------------------

#[test]
fn an_empty_shelf_places_nothing() {
    let s = Shelf::new(0);
    assert_eq!(s.selected(), 0);
    assert!(s.placements(&safe(Geometry::W720H480), CART).is_empty());
}

#[test]
fn one_cart_stands_alone_in_the_middle() {
    // Ported decision: a single cart is not repeated into the side slots. The selection
    // never changes, so nothing would ever move, and three identical faces standing still
    // read as a drawing fault rather than as a carousel.
    let s = Shelf::new(1);
    for g in GEOMETRIES {
        let p = s.placements(&safe(g), CART);
        assert_eq!(p.len(), 1, "{g:?} drew a lone cart {} times", p.len());
        assert_eq!(p[0].index, 0);
    }
}

#[test]
fn two_carts_repeat_to_fill_the_row() {
    // Also ported, and deliberately odd: with two carts one of them is drawn twice at once.
    // The alternatives were a hole in the row and a centred pair, and neither scrolls —
    // there is nothing to put in the slot the row moves into. A row that shows a cart twice
    // is what a carousel of two is.
    let s = Shelf::new(2);
    let p = s.placements(&safe(Geometry::W720H480), CART);
    assert!(
        p.len() >= 3,
        "a row of two should fill its slots, got {}",
        p.len()
    );
    assert!(p.iter().any(|q| q.index == 0) && p.iter().any(|q| q.index == 1));
}

#[test]
fn the_selection_is_centred_on_the_panel_at_rest() {
    // The panel, not the safe area: DESIGN §5 says the extension either side of the safe
    // area carries the background and the shelf, so a 720-wide panel shows more of the
    // neighbours rather than shifting the selection off centre.
    for g in GEOMETRIES {
        let sa = safe(g);
        let s = Shelf::new(5);
        let p = s.placements(&sa, CART);
        let sel = selected(&p, 0);
        let middle = sel.x + sel.w / 2.0;
        assert!(
            (middle - sa.panel_w as f32 / 2.0).abs() < 1.0,
            "{g:?}: selection centred at {middle}, panel is {} wide",
            sa.panel_w
        );
        assert_eq!((sel.w, sel.h), CART, "the selection is drawn full size");
    }
}

#[test]
fn carts_share_a_centre_line_not_a_floor() {
    // A Game Boy pak is 253 tall and a GBA cart 135. Measured from a shared floor the pak
    // sat 59px higher and crowded the top of the screen; the shelves share a centre so a
    // row reads the same whichever platform it holds.
    let sa = safe(Geometry::W720H480);
    let tall = Shelf::new(1).placements(&sa, (240.0, 253.0));
    let short = Shelf::new(1).placements(&sa, CART);
    let mid = |p: &[Placement]| p[0].y + p[0].h / 2.0;
    assert!(
        (mid(&tall) - mid(&short)).abs() < 1.0,
        "a tall cart centred at {} and a short one at {}",
        mid(&tall),
        mid(&short)
    );
}

#[test]
fn a_neighbour_keeps_its_foot_on_the_selections_foot_line() {
    // A side cart is smaller, and it shrinks upward from the line the selection stands on
    // rather than about its own middle — otherwise the row reads as carts floating.
    let sa = safe(Geometry::W720H480);
    let s = Shelf::new(5);
    let p = s.placements(&sa, CART);
    let sel = selected(&p, 0);
    let floor = sel.y + sel.h;
    for q in p.iter().filter(|q| q.alpha < 0.999) {
        assert!(
            ((q.y + q.h) - floor).abs() < 1.0,
            "a neighbour's foot is at {} and the selection's at {floor}",
            q.y + q.h
        );
    }
}

#[test]
fn neighbours_are_smaller_and_dimmer() {
    let s = Shelf::new(5);
    let p = s.placements(&safe(Geometry::W720H480), CART);
    let sides: Vec<&Placement> = p.iter().filter(|q| q.alpha < 0.999).collect();
    assert!(!sides.is_empty(), "a row of five should show neighbours");
    for q in sides {
        assert!((q.w - CART.0 * SIDE_SCALE).abs() < 1.0, "width {}", q.w);
        assert!((q.alpha - SIDE_ALPHA).abs() < 0.01, "alpha {}", q.alpha);
    }
}

#[test]
fn nothing_is_placed_entirely_off_the_panel() {
    let sa = safe(Geometry::W640H480);
    let s = Shelf::new(30);
    for q in s.placements(&sa, CART) {
        assert!(
            q.x + q.w > 0.0 && q.x < sa.panel_w as f32,
            "a cart at x={} w={} is off a {}-wide panel",
            q.x,
            q.w,
            sa.panel_w
        );
    }
}

#[test]
fn the_selection_is_drawn_last_so_it_lands_on_top() {
    let s = Shelf::new(5);
    let p = s.placements(&safe(Geometry::W720H480), CART);
    assert!(p.last().expect("placements").alpha >= 0.999, "{p:?}");
}

#[test]
fn moving_wraps_both_ways() {
    let mut s = Shelf::new(4);
    s.left();
    assert_eq!(
        s.selected(),
        3,
        "left from the first should wrap to the last"
    );
    s.right();
    assert_eq!(s.selected(), 0);
    for _ in 0..4 {
        s.right();
    }
    assert_eq!(s.selected(), 0, "four rights on a row of four returns");
}

#[test]
fn the_row_takes_the_short_way_round() {
    // Wrapping from the first cart to the last must slide one pitch, not thirty.
    let mut s = Shelf::new(30);
    let before = s.scroll_target();
    s.left();
    let after = s.scroll_target();
    assert!(
        (after - before + 1.0).abs() < 0.001,
        "wrapping moved the row by {}, not one pitch",
        after - before
    );
}

#[test]
fn the_row_settles_without_bouncing_past() {
    // Critically damped: a flick lands on a cart instead of overshooting and coming back.
    let mut s = Shelf::new(10);
    s.right();
    let target = s.scroll_target();
    let mut overshoot: f32 = 0.0;
    for _ in 0..600 {
        s.update(1.0 / 60.0);
        overshoot = overshoot.max(s.scroll() - target);
    }
    assert!(overshoot < 0.02, "the row overshot by {overshoot}");
    assert!((s.scroll() - target).abs() < 0.01, "the row never settled");
    assert!(!s.settling(), "a settled row still reports movement");
}

#[test]
fn a_rescan_that_shortens_the_row_keeps_the_selection_on_it() {
    let mut s = Shelf::new(10);
    s.select(9);
    s.set_len(3);
    assert!(
        s.selected() < 3,
        "selection {} is off a row of 3",
        s.selected()
    );
    s.set_len(0);
    assert_eq!(s.selected(), 0);
    assert!(s.placements(&safe(Geometry::W720H480), CART).is_empty());
}

#[test]
fn placements_are_finite_on_every_panel() {
    for g in GEOMETRIES {
        for len in [0usize, 1, 2, 3, 7, 40] {
            let mut s = Shelf::new(len);
            s.right();
            s.update(1.0 / 60.0);
            for q in s.placements(&safe(g), CART) {
                for v in [q.x, q.y, q.w, q.h, q.alpha] {
                    assert!(v.is_finite(), "{g:?} len {len}: {q:?}");
                }
                assert!(q.w > 0.0 && q.h > 0.0, "{g:?} len {len}: {q:?}");
                assert!(q.index < len, "{g:?}: index {} is off the row", q.index);
            }
        }
    }
}

// --- the artwork cache ---------------------------------------------------------------

#[test]
fn artwork_is_rasterised_once_per_size() {
    // A cart is redrawn sixty times a second. Rasterising an SVG each time is not an
    // option, and neither is a texture upload.
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut cache = ArtCache::default();
    let art = skin::skin(Platform::Gba).cart;

    let a = cache.mask(&mut canvas, art, 240, 135).expect("rasterises");
    let b = cache.mask(&mut canvas, art, 240, 135).expect("rasterises");
    assert_eq!(a, b, "the same artwork at the same size is one texture");

    let uploads = canvas
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. }))
        .count();
    assert_eq!(uploads, 1, "{uploads} uploads for one drawing");

    // A different size is a different drawing: the art is vector and redrawn, not scaled.
    let c = cache.mask(&mut canvas, art, 120, 68).expect("rasterises");
    assert_ne!(a, c);
    assert_eq!(cache.len(), 2);
}

#[test]
fn two_platforms_artwork_does_not_collide() {
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut cache = ArtCache::default();
    let gba = cache.mask(&mut canvas, skin::skin(Platform::Gba).cart, 200, 200);
    let gb = cache.mask(&mut canvas, skin::skin(Platform::Gb).cart, 200, 200);
    assert!(gba.is_some() && gb.is_some());
    assert_ne!(gba, gb, "two drawings at one size share a texture");
}

#[test]
fn a_drawing_that_will_not_parse_is_given_up_on_once() {
    // Artwork is overridable from the card later, so a broken file has to cost a missing
    // cartridge rather than the frontend — and it must not be re-parsed every frame.
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut cache = ArtCache::default();
    assert!(cache.mask(&mut canvas, "not an svg", 64, 64).is_none());
    assert!(cache.mask(&mut canvas, "not an svg", 64, 64).is_none());
    assert_eq!(
        canvas
            .ops
            .iter()
            .filter(|o| matches!(o, Op::UploadAlpha8 { .. }))
            .count(),
        0
    );
    assert!(cache.remembers_failure("not an svg", 64, 64));
}

#[test]
fn clearing_the_cache_frees_what_it_uploaded() {
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut cache = ArtCache::default();
    let art = skin::skin(Platform::Gba).cart;
    cache.mask(&mut canvas, art, 64, 36);
    cache.mask(&mut canvas, art, 128, 72);
    assert_eq!(cache.len(), 2);

    cache.clear(&mut canvas);
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
    let frees = canvas
        .ops
        .iter()
        .filter(|o| matches!(o, Op::Free(_)))
        .count();
    assert_eq!(frees, 2, "a cleared cache left textures behind");
}
