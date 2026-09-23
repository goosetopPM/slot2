//! The contract for a cartridge going into the slot. Task 14 makes these pass without
//! editing this file.
//!
//! The last three tasks each passed their contract and still had a real fault, every one of
//! them in a state the tests only saw at rest. An insert is nothing *but* intermediate
//! states, so almost everything here is checked while the cart is moving.

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::{detect, Geometry};
use slot2_store::Platform;
use slot2_ui::insert::{
    self, seat_in, seat_out, travel, Travel, EJECT_S, INSERT_HOLD_S, INSERT_S, SEATED_AT,
};
use slot2_ui::layout::SafeArea;
use slot2_ui::shelf::{Shelf, MOUTH_H};
use slot2_ui::shelf_view::ShelfView;
use slot2_ui::{skin, UiCtx};

const GEOMETRIES: [Geometry; 3] = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];

/// The frames of an insert, at the rate the loop actually runs.
const STEPS: usize = 44; // 0.73 s at 60 Hz

fn ctx() -> UiCtx {
    UiCtx::new(detect().profile, "en", Vec::new(), None)
}

fn images(canvas: &RecordingCanvas) -> Vec<(f32, f32, f32, f32, f32)> {
    canvas
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image {
                x, y, w, h, tint, ..
            } => Some((*x, *y, *w, *h, tint.a)),
            _ => None,
        })
        .collect()
}

fn cart_of(p: Platform) -> (f32, f32) {
    skin::skin(p).cart_size
}

/// Where the travel puts a cart at `seat`, on a row that is at rest.
fn at(safe: &SafeArea, p: Platform, seat: f32) -> Travel {
    let cart = cart_of(p);
    let rest_x = (safe.panel_w as f32 - cart.0) / 2.0;
    travel(safe, cart, rest_x, seat)
}

// ---------------------------------------------------------------- the clock

// The cart is seated well before the animation ends, and the rest of it is the dwell that
// covers the core load. If SEATED_AT were the whole thing there would be nothing on screen
// while the core comes up.
//
// Checked at compile time rather than in a `#[test]`, because that is what these are: a
// relationship between three constants, which either holds when the crate is built or does
// not. Asserting it at run time is the thing clippy calls a constant assertion, and it is
// right — there is no run time at which it could be any different.
const _: () = assert!(SEATED_AT < INSERT_S, "the cart seats at the very end");
const _: () = assert!(
    SEATED_AT + INSERT_HOLD_S - INSERT_S < 1e-6 && SEATED_AT + INSERT_HOLD_S - INSERT_S > -1e-6,
    "the seat, the hold and the whole do not add up"
);
const _: () = assert!(EJECT_S > 0.0);

#[test]
fn the_seat_runs_from_the_row_to_the_slot_and_stops_there() {
    assert_eq!(seat_in(0.0), 0.0);
    assert!(
        (seat_in(SEATED_AT) - 1.0).abs() < 1e-5,
        "not seated on time"
    );
    // The hold is the cart sitting still, not the cart carrying on through the floor.
    assert_eq!(seat_in(INSERT_S), 1.0);
    assert_eq!(seat_in(INSERT_S * 10.0), 1.0);
    assert_eq!(seat_in(-1.0), 0.0);

    // Out is in, backwards.
    assert_eq!(seat_out(0.0), 1.0);
    assert!((seat_out(EJECT_S)).abs() < 1e-5, "never reaches the row");
    assert_eq!(seat_out(EJECT_S * 10.0), 0.0);
}

#[test]
fn the_ease_has_no_kick_at_either_end() {
    // Smootherstep, or the catch shows as a step in speed.
    assert_eq!(insert::ease(0.0), 0.0);
    assert!((insert::ease(1.0) - 1.0).abs() < 1e-5);

    let d = 1e-3;
    let start = insert::ease(d) - insert::ease(0.0);
    let middle = insert::ease(0.5 + d) - insert::ease(0.5);
    let end = insert::ease(1.0) - insert::ease(1.0 - d);
    assert!(
        start < middle * 0.1 && end < middle * 0.1,
        "the ease starts or ends moving: {start} {middle} {end}"
    );

    let mut last = -1.0;
    for i in 0..=100 {
        let v = insert::ease(i as f32 / 100.0);
        assert!(v >= last, "the ease goes backwards at {i}");
        last = v;
    }
}

// ---------------------------------------------------------------- the travel

#[test]
fn the_cart_starts_exactly_where_the_row_left_it() {
    // The frame A is pressed. If the travel measures from anywhere but the row's own line,
    // the cart jumps on that frame — and the row is measured from the slot, not from the
    // middle of the panel, so a panel that is taller than 480 is where this goes wrong.
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        for p in [Platform::Gba, Platform::Gb] {
            let cart = cart_of(p);
            let mut shelf = Shelf::new(5);
            shelf.set_len(5);
            let row = shelf.placements(&safe, cart);
            let sel = row
                .iter()
                .find(|pl| pl.index == shelf.selected())
                .expect("the row did not draw its selection");

            let t = at(&safe, p, 0.0);
            assert!(
                (t.x - sel.x).abs() < 0.5 && (t.y - sel.y).abs() < 0.5,
                "{g:?} {p:?}: the cart jumps from {},{} to {},{} on the first frame",
                sel.x,
                sel.y,
                t.x,
                t.y
            );
            assert!((t.w - cart.0).abs() < 0.5 && (t.h - cart.1).abs() < 0.5);
        }
    }
}

#[test]
fn a_seated_cart_is_in_the_slot_on_every_panel() {
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        let band = safe.panel_h as f32 - MOUTH_H;
        for p in [Platform::Gba, Platform::Gb, Platform::Snes] {
            let t = at(&safe, p, 1.0);
            // Its top edge is just inside the mouth: past the band, not still above it and
            // not so far through that the label is gone.
            assert!(
                t.y > band && t.y < band + MOUTH_H,
                "{g:?} {p:?}: seated top edge {} against a band at {band}",
                t.y
            );
            // Centred on the mouth, which cannot move.
            let mid = t.x + t.w / 2.0;
            assert!(
                (mid - safe.panel_w as f32 / 2.0).abs() < 1.0,
                "{g:?} {p:?}: seated at {mid} on a {}-wide panel",
                safe.panel_w
            );
        }
    }
}

#[test]
fn every_platform_seats_its_top_edge_in_the_same_place() {
    // How much cartridge is left sticking out of the machine is the slot's business, not the
    // cartridge's. Deriving the seat from the cart's own height swallows a tall pak deeper
    // than a GBA cart, which reads as two different machines.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let gba = at(&safe, Platform::Gba, 1.0).y;
    for p in [Platform::Gb, Platform::Gbc, Platform::Snes, Platform::Md] {
        let y = at(&safe, p, 1.0).y;
        assert!(
            (y - gba).abs() < 0.5,
            "{p:?} seats at {y} where a GBA cart seats at {gba}"
        );
    }
}

#[test]
fn the_cart_only_ever_goes_down() {
    // A travel that overshoots and comes back reads as a bounce, and there is nothing for a
    // cartridge to bounce off on the way in.
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        for p in [Platform::Gba, Platform::Gb] {
            let mut last = f32::NEG_INFINITY;
            for i in 0..=STEPS {
                let seat = i as f32 / STEPS as f32;
                let y = at(&safe, p, seat).y;
                assert!(y.is_finite(), "{g:?} {p:?}: {y} at seat {seat}");
                assert!(
                    y >= last - 0.01,
                    "{g:?} {p:?}: rose from {last} to {y} at seat {seat}"
                );
                last = y;
            }
        }
    }
}

#[test]
fn the_cart_is_caught_on_the_lip_on_the_way_in() {
    // The whole reason the travel is not one ease: the cart falls, meets the lip, hesitates,
    // and is pushed through. Without the catch it reads as a card going down a chute.
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        for p in [Platform::Gba, Platform::Gb] {
            let step = |seat: f32| at(&safe, p, seat + 0.01).y - at(&safe, p, seat).y;

            // Somewhere in the middle the cart is barely moving, and either side of that it
            // is moving properly.
            let caught = step(0.50);
            let falling = (1..=9).map(|i| step(i as f32 * 0.04)).fold(0.0, f32::max);
            let pushed = (70..=95).map(|i| step(i as f32 * 0.01)).fold(0.0, f32::max);

            assert!(
                caught * 3.0 < falling,
                "{g:?} {p:?}: no hesitation — {caught} against a fall of {falling}"
            );
            assert!(
                caught * 3.0 < pushed,
                "{g:?} {p:?}: no push — {caught} against {pushed}"
            );
            assert!(
                caught > 0.0,
                "{g:?} {p:?}: a dead stop reads as a dropped frame"
            );
        }
    }
}

#[test]
fn the_catch_lands_on_the_same_frame_whatever_the_panel() {
    // Everything is measured off the slot, so the beat of the insert belongs to the slot and
    // does not drift when the panel gets taller. A catch that moved would make a 720x720
    // insert a visibly different mechanism from a 640x480 one.
    let mut when: Vec<f32> = Vec::new();
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        // The frame the foot first reaches the band.
        let band = safe.panel_h as f32 - MOUTH_H;
        let mut hit = 1.0;
        for i in 0..=200 {
            let seat = i as f32 / 200.0;
            let t = at(&safe, Platform::Gba, seat);
            if t.y + t.h >= band {
                hit = seat;
                break;
            }
        }
        when.push(hit);
    }
    let spread =
        when.iter().cloned().fold(0.0, f32::max) - when.iter().cloned().fold(1.0, f32::min);
    assert!(
        spread < 0.02,
        "the catch lands at {when:?} on the three panels"
    );
}

#[test]
fn a_cart_that_starts_off_centre_arrives_centred() {
    // A is pressed while the row is still sliding. The cart has to come to the slot, because
    // the slot cannot come to the cart.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let cart = cart_of(Platform::Gba);
    let rest_x = 40.0;
    let start = travel(&safe, cart, rest_x, 0.0);
    assert!(
        (start.x - rest_x).abs() < 0.5,
        "it jumped on the first frame"
    );

    let end = travel(&safe, cart, rest_x, 1.0);
    assert!(
        (end.x + end.w / 2.0 - safe.panel_w as f32 / 2.0).abs() < 1.0,
        "it arrived at {} instead of the mouth",
        end.x
    );

    let mut last = rest_x;
    for i in 0..=STEPS {
        let x = travel(&safe, cart, rest_x, i as f32 / STEPS as f32).x;
        assert!(x >= last - 0.01, "it slid back to {x} from {last}");
        last = x;
    }
}

// ---------------------------------------------------------------- the drawing

#[test]
fn the_resting_insert_is_the_resting_shelf() {
    // seat 0.0 is the frame the button was pressed on and nothing has moved yet. If this
    // frame differs from the shelf's own, the screen flickers as the insert starts.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let t = ["A", "B", "C", "D"];

    let mut shelf_view = ShelfView::default();
    shelf_view.shelf.set_len(4);
    let mut a = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    shelf_view.draw(&mut a, &mut c, &safe, Platform::Gba, &t);

    let mut insert_view = ShelfView::default();
    insert_view.shelf.set_len(4);
    let mut b = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    insert_view.draw_insert(&mut b, &mut c, &safe, Platform::Gba, &t, 0.0);

    let mut want = images(&a);
    let mut got = images(&b);
    let key = |v: &mut Vec<(f32, f32, f32, f32, f32)>| {
        v.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
    };
    key(&mut want);
    key(&mut got);
    assert_eq!(
        want.len(),
        got.len(),
        "the shelf drew {} things and the first insert frame drew {}",
        want.len(),
        got.len()
    );
    for (w, g) in want.iter().zip(got.iter()) {
        for (a, b) in [(w.0, g.0), (w.1, g.1), (w.2, g.2), (w.3, g.3), (w.4, g.4)] {
            assert!((a - b).abs() < 0.5, "{want:?}\n differs from \n{got:?}");
        }
    }
}

#[test]
fn one_cart_is_on_screen_once() {
    // The insert draws the chosen cart, so the row must stop drawing it. Two copies of one
    // cartridge makes the travel read as a duplicate sliding away from the original.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let t = ["A", "B", "C", "D", "E"];
    let cart = cart_of(Platform::Gba);

    for i in 1..=STEPS {
        let seat = i as f32 / STEPS as f32;
        let mut v = ShelfView::default();
        v.shelf.set_len(5);
        let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
        let mut c = ctx();
        v.draw_insert(&mut canvas, &mut c, &safe, Platform::Gba, &t, seat);

        // The cart at its full natural size is the one going in; the row's neighbours are
        // drawn smaller. Exactly one full-size cartridge may be on screen.
        let full = images(&canvas)
            .iter()
            .filter(|(_, _, w, h, _)| (w - cart.0).abs() < 0.5 && (h - cart.1).abs() < 0.5)
            .count();
        assert!(
            full <= 2,
            "seat {seat}: {full} full-size quads — the row is still drawing the chosen cart"
        );
    }
}

#[test]
fn the_row_parts_for_the_cart_going_in() {
    // The neighbours make way outwards and fade. A row that stayed put while one of its
    // carts left would read as the cart falling out of the picture.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let cart = cart_of(Platform::Gba);
    let shelf = {
        let mut s = Shelf::new(5);
        s.set_len(5);
        s
    };

    // The cart immediately right of the selection. Following one neighbour rather than the
    // widest spread, because past halfway the outer ones have parted right off the panel and
    // stopped being drawn at all — which is correct, and would read as the row closing back
    // up if the test measured it.
    let neighbour = (shelf.selected() + 1) % 5;
    let x_of = |recede: f32| -> Option<f32> {
        shelf
            .parted(&safe, cart, recede)
            .iter()
            .find(|p| p.index == neighbour)
            .map(|p| p.x)
    };

    let rest = x_of(0.0).expect("the row at rest has no neighbour");
    let mut last = rest;
    for i in 1..=5 {
        let recede = i as f32 / 10.0;
        let s = x_of(recede).unwrap_or_else(|| panic!("the neighbour vanished at {recede}"));
        assert!(
            s > last,
            "the row closed back up at {recede}: {s} <= {last}"
        );
        last = s;
    }
    assert!(last > rest + 50.0, "the row barely moved: {rest} to {last}");

    // And it fades as it goes, so a row that has fully made way is gone rather than piled up
    // against the edges.
    assert!(
        shelf
            .parted(&safe, cart, 1.0)
            .iter()
            .all(|p| p.alpha <= 0.001),
        "a fully receded row is still visible"
    );

    // And the selection is gone from the row, because the insert is drawing it.
    assert!(
        !shelf
            .parted(&safe, cart, 0.5)
            .iter()
            .any(|p| p.index == shelf.selected()),
        "the row is still drawing the cart that is going into the slot"
    );
    // At rest it is not, because nothing is going in.
    assert!(
        shelf
            .parted(&safe, cart, 0.0)
            .iter()
            .any(|p| p.index == shelf.selected()),
        "the row at rest dropped its selection"
    );
}

#[test]
fn the_cart_goes_behind_the_slot_not_over_it() {
    // The band is the front of the machine. Drawn before the cart, a seated cartridge lies
    // on top of the device face instead of being in it.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let t = ["A", "B", "C"];
    let mut v = ShelfView::default();
    v.shelf.set_len(3);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw_insert(&mut canvas, &mut c, &safe, Platform::Gba, &t, 1.0);

    let band = safe.panel_h as f32 - MOUTH_H;
    let last_cart = canvas
        .frame()
        .iter()
        .rposition(|o| matches!(o, Op::Image { y, .. } if *y > band));
    // The band, not the cart's own label plate: a full-width rect at the foot of the panel.
    let last_chrome = canvas.frame().iter().rposition(
        |o| matches!(o, Op::Rect { y, w, .. } if *y >= band - 0.5 && *w > safe.panel_w as f32 / 2.0),
    );
    let (Some(cart), Some(chrome)) = (last_cart, last_chrome) else {
        panic!("a seated insert drew no cart ({last_cart:?}) or no slot ({last_chrome:?})");
    };
    assert!(
        chrome > cart,
        "the cart is drawn at {cart} and the slot at {chrome}: the cart is on top of the machine"
    );
}

#[test]
fn a_seated_cart_is_still_visible_in_the_mouth() {
    // The gap the first version of this contract left, and the fault it let through: the
    // machine's face was painted across the mouth as well as either side of it, so a
    // cartridge arrived at the slot and vanished behind it but for the nine-pixel opening.
    // Every ordering test still passed, because the chrome really was drawn after the cart.
    //
    // A cart is *in* the mouth, not behind it. What occludes is the plastic either side.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let t = ["A", "B", "C"];
    let mut v = ShelfView::default();
    v.shelf.set_len(3);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw_insert(&mut canvas, &mut c, &safe, Platform::Gba, &t, 1.0);

    let band = safe.panel_h as f32 - MOUTH_H;
    let cart = canvas
        .frame()
        .iter()
        .rposition(|o| matches!(o, Op::Image { y, .. } if *y > band))
        .expect("a seated insert drew no cart");

    // Down the middle of the mouth, a little below the opening: after the cart is drawn,
    // nothing may be painted over this point.
    let (px, py) = (safe.panel_w as f32 / 2.0, band + MOUTH_H * 0.75);
    for (i, op) in canvas.frame().iter().enumerate().skip(cart + 1) {
        if let Op::Rect { x, y, w, h, .. } = op {
            assert!(
                !(px >= *x && px <= x + w && py >= *y && py <= y + h),
                "op {i} paints {x},{y} {w}x{h} over the seated cart at {px},{py}"
            );
        }
    }
}

#[test]
fn nothing_leaves_the_panel_sideways() {
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        let t = ["A", "B", "C", "D", "E", "F"];
        for i in 0..=STEPS {
            let seat = i as f32 / STEPS as f32;
            let mut v = ShelfView::default();
            v.shelf.set_len(6);
            let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
            let mut c = ctx();
            v.draw_insert(&mut canvas, &mut c, &safe, Platform::Gba, &t, seat);
            for (x, y, w, h, a) in images(&canvas) {
                for val in [x, y, w, h, a] {
                    assert!(val.is_finite(), "{g:?} seat {seat}: {x},{y} {w}x{h} a{a}");
                }
                assert!(w > 0.0 && h > 0.0, "{g:?} seat {seat}: {w}x{h}");
                assert!((0.0..=1.0).contains(&a), "{g:?} seat {seat}: alpha {a}");
                // A cart on its way into the slot runs off the *bottom* — that is the slot
                // swallowing it. Off the side is a bug.
                assert!(
                    x + w > -1.0 && x < safe.panel_w as f32 + 1.0,
                    "{g:?} seat {seat}: drawn at x={x} w={w} on a {}-wide panel",
                    safe.panel_w
                );
            }
        }
    }
}

#[test]
fn the_whole_animation_uploads_nothing() {
    // Forty-four frames. Rasterising the artwork per frame because its size changes is the
    // fault task 12 had, and an insert changes size and position on every single frame.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let t = ["A", "B", "C", "D", "E"];
    let mut v = ShelfView::default();
    v.shelf.set_len(5);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();

    v.draw_insert(&mut canvas, &mut c, &safe, Platform::Gba, &t, 0.0);
    let warm = canvas.ops.len();

    for i in 1..=STEPS {
        v.draw_insert(
            &mut canvas,
            &mut c,
            &safe,
            Platform::Gba,
            &t,
            i as f32 / STEPS as f32,
        );
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(warm)
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count();
    assert_eq!(uploads, 0, "an insert uploaded {uploads} textures");
}

#[test]
fn a_row_of_one_still_inserts() {
    // Nothing to part, and the selection is the only cart there is.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut v = ShelfView::default();
    v.shelf.set_len(1);
    for i in 0..=STEPS {
        let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
        let mut c = ctx();
        v.draw_insert(
            &mut canvas,
            &mut c,
            &safe,
            Platform::Gb,
            &["Only"],
            i as f32 / STEPS as f32,
        );
        assert!(
            !images(&canvas).is_empty(),
            "frame {i} of a one-cart insert drew no cart"
        );
    }
}

#[test]
fn an_empty_row_is_a_screen_not_a_crash() {
    // Nothing should ever get here — there is no cart to insert — but a panic in the draw
    // loop is a black screen and a dead handset with no message anywhere.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut v = ShelfView::default();
    v.shelf.set_len(0);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw_insert(&mut canvas, &mut c, &safe, Platform::Gba, &[], 0.5);
    assert!(!canvas.frame().is_empty(), "an empty insert drew nothing");
}
