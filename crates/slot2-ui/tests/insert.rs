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
    self, collision_at, seat_in, seat_out, travel, Curve, Insertion, Motion, Travel, EJECT_S,
    INSERT_HOLD_S, INSERT_S, SEATED_AT,
};
use slot2_ui::layout::SafeArea;
use slot2_ui::shelf::{Shelf, MOUTH_H};
use slot2_ui::shelf_view::ShelfView;
use slot2_ui::{skin, svg, UiCtx};

const GEOMETRIES: [Geometry; 3] = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];

const PLATFORMS: [Platform; 7] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Nes,
    Platform::Snes,
    Platform::Md,
    Platform::Sms,
];

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

/// Where a frame draws the port trim: its op index and its rect. The trim is the image as
/// tall as the mouth and as wide as the table says, which nothing else on the shelf is.
fn port_at(ops: &[Op], p: Platform) -> Option<(usize, f32, f32, f32, f32)> {
    let want = skin::skin(p).port_size;
    ops.iter().enumerate().find_map(|(i, o)| match o {
        Op::Image { x, y, w, h, .. } if (w - want.0).abs() < 0.5 && (h - MOUTH_H).abs() < 0.5 => {
            Some((i, *x, *y, *w, *h))
        }
        _ => None,
    })
}

/// Where the travel puts a cart at `seat`, on a row that is at rest.
fn at(safe: &SafeArea, p: Platform, seat: f32) -> Travel {
    at_motion(safe, p, Motion::Insert, seat)
}

/// Where the platform's own profile puts the cart at `seat`, going `motion`'s way.
fn at_motion(safe: &SafeArea, p: Platform, motion: Motion, seat: f32) -> Travel {
    let cart = cart_of(p);
    let rest_x = (safe.panel_w as f32 - cart.0) / 2.0;
    travel(safe, curve_of(p, motion), cart, rest_x, seat)
}

/// The profile one platform uses for one direction.
fn curve_of(p: Platform, motion: Motion) -> Curve {
    let s = skin::skin(p);
    match motion {
        Motion::Insert => s.insert,
        Motion::Eject => s.eject,
    }
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
    let start = travel(
        &safe,
        curve_of(Platform::Gba, Motion::Insert),
        cart,
        rest_x,
        0.0,
    );
    assert!(
        (start.x - rest_x).abs() < 0.5,
        "it jumped on the first frame"
    );

    let end = travel(
        &safe,
        curve_of(Platform::Gba, Motion::Insert),
        cart,
        rest_x,
        1.0,
    );
    assert!(
        (end.x + end.w / 2.0 - safe.panel_w as f32 / 2.0).abs() < 1.0,
        "it arrived at {} instead of the mouth",
        end.x
    );

    let mut last = rest_x;
    for i in 0..=STEPS {
        let x = travel(
            &safe,
            curve_of(Platform::Gba, Motion::Insert),
            cart,
            rest_x,
            i as f32 / STEPS as f32,
        )
        .x;
        assert!(x >= last - 0.01, "it slid back to {x} from {last}");
        last = x;
    }
}

// ---------------------------------------------------------------- the profiles

#[test]
fn every_platform_profile_walks_the_whole_journey() {
    // All fourteen profiles, on the cart each belongs to, end to end: the row, the slot, and a
    // monotonic walk between them with nothing outside the range.
    for p in PLATFORMS {
        let cart = cart_of(p);
        for motion in [Motion::Insert, Motion::Eject] {
            let curve = curve_of(p, motion);
            for g in GEOMETRIES {
                let safe = SafeArea::for_geometry(g);
                let collision = collision_at(&safe, cart.1);
                assert!(
                    collision > 0.0 && collision < 1.0,
                    "{g:?} {p:?}: the lip is {collision} of the way down"
                );

                let mut last = f32::NEG_INFINITY;
                for i in 0..=100 {
                    let seat = i as f32 / 100.0;
                    let j = curve.journey(collision, seat);
                    assert!(
                        j.is_finite() && (0.0..=1.0).contains(&j),
                        "{g:?} {p:?} {motion:?} at {seat}: {j}"
                    );
                    assert!(
                        j >= last - 1e-5,
                        "{g:?} {p:?} {motion:?}: the journey went backwards at {seat}"
                    );
                    last = j;
                }
                assert!(
                    curve.journey(collision, 0.0).abs() < 1e-6,
                    "{p:?} {motion:?}: seat 0 is not the row"
                );
                assert!(
                    (curve.journey(collision, 1.0) - 1.0).abs() < 1e-6,
                    "{p:?} {motion:?}: seat 1 is not the slot"
                );
            }

            // Out of range clamps, the way `seat_in` and `seat_out` do.
            let safe = SafeArea::for_geometry(Geometry::W720H480);
            let collision = collision_at(&safe, cart.1);
            assert!(
                curve.journey(collision, -3.0).abs() < 1e-6,
                "{p:?} {motion:?}"
            );
            assert!(
                (curve.journey(collision, 7.0) - 1.0).abs() < 1e-6,
                "{p:?} {motion:?}"
            );
        }
    }
}

#[test]
fn every_platform_profile_meets_the_lip_and_hesitates_on_it() {
    // The catch is a collision, not a beat sheet: at each platform's contact the foot is on the
    // lip itself, whatever the profile's timing is. And past that the cart moves — a dead stop
    // reads as a dropped frame — but far less than it does either side of it.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let lip = safe.panel_h as f32 - MOUTH_H;
    for p in PLATFORMS {
        let cart = cart_of(p);
        for motion in [Motion::Insert, Motion::Eject] {
            let curve = curve_of(p, motion);
            let contact = curve.contact();
            let release = curve.release();
            let at = travel(&safe, curve, cart, 0.0, contact);
            assert!(
                (at.y + at.h - lip).abs() < 0.5,
                "{p:?} {motion:?}: the foot meets {} where the lip is {lip}",
                at.y + at.h
            );

            let step = |seat: f32| {
                let a = travel(&safe, curve, cart, 0.0, seat).y;
                let b = travel(&safe, curve, cart, 0.0, seat + 0.01).y;
                (b - a).abs()
            };
            let caught = step((contact + release) / 2.0);
            assert!(
                caught > 0.0,
                "{p:?} {motion:?}: the cart is dead still on the lip"
            );
            // Inside the fall and inside the push, so what is compared with is the fastest
            // hundredth of a second in each, not the ends of the easing.
            let fall = (1..=9)
                .map(|i| step(contact * i as f32 / 10.0))
                .fold(0.0, f32::max);
            let push = (1..=9)
                .map(|i| step(release + (1.0 - release) * i as f32 / 10.0))
                .fold(0.0, f32::max);
            assert!(
                caught * 3.0 < fall && caught * 3.0 < push,
                "{p:?} {motion:?}: caught {caught} against a fall of {fall} and a push of {push}"
            );
        }
    }
}

#[test]
fn the_profiles_do_not_travel_alike() {
    // Field values differing is not enough: two shelves whose curves put the cart in the same
    // place at the same moments are one curve with a comment on it. Compared at one shared
    // collision fraction, so this measures rhythm and not cartridge weight.
    let collision = 0.4;
    for motion in [Motion::Insert, Motion::Eject] {
        let samples: Vec<(Platform, [f32; 3])> = PLATFORMS
            .iter()
            .map(|p| {
                let curve = curve_of(*p, motion);
                (
                    *p,
                    [0.25f32, 0.5, 0.75].map(|seat| curve.journey(collision, seat)),
                )
            })
            .collect();
        for (i, (p, a)) in samples.iter().enumerate() {
            for (q, b) in &samples[i + 1..] {
                assert!(
                    (0..3).any(|k| (a[k] - b[k]).abs() > 0.005),
                    "{p:?} and {q:?} travel alike on {motion:?}: {a:?} against {b:?}"
                );
            }
        }
    }
}

#[test]
fn every_profile_starts_on_the_row_and_ends_in_the_slot() {
    for p in PLATFORMS {
        let cart = cart_of(p);
        let mut row = Shelf::new(2);
        row.set_len(2);
        for motion in [Motion::Insert, Motion::Eject] {
            let curve = curve_of(p, motion);
            for g in GEOMETRIES {
                let safe = SafeArea::for_geometry(g);
                let rest_x = (safe.panel_w as f32 - cart.0) / 2.0;
                let start = travel(&safe, curve, cart, rest_x, 0.0);
                let end = travel(&safe, curve, cart, rest_x, 1.0);

                // The row's own line, asked of the row rather than restated here.
                let placed = row
                    .placements(&safe, cart)
                    .into_iter()
                    .find(|pl| pl.index == row.selected())
                    .expect("the row did not draw its selection");
                assert!(
                    (start.x - placed.x).abs() < 0.5 && (start.y - placed.y).abs() < 0.5,
                    "{g:?} {p:?} {motion:?}: seat 0 is at {},{} where the row is at {},{}",
                    start.x,
                    start.y,
                    placed.x,
                    placed.y
                );
                assert_eq!((start.w, start.h), cart);
                assert!(
                    (end.y - insert::seated_y(&safe)).abs() < 0.5,
                    "{g:?} {p:?} {motion:?}: seat 1 is at {} not {}",
                    end.y,
                    insert::seated_y(&safe)
                );
                assert!(
                    (end.x + end.w / 2.0 - safe.panel_w as f32 / 2.0).abs() < 1.0,
                    "{g:?} {p:?} {motion:?}: seat 1 is not centred"
                );

                // And the frames between them: down on the way in, up on the way out, never
                // the other way on either.
                let seats: Vec<f32> = match motion {
                    Motion::Insert => (0..=STEPS).map(|i| i as f32 / STEPS as f32).collect(),
                    Motion::Eject => (0..=STEPS).rev().map(|i| i as f32 / STEPS as f32).collect(),
                };
                let mut last = travel(&safe, curve, cart, rest_x, seats[0]).y;
                for (i, seat) in seats.iter().enumerate().skip(1) {
                    let y = travel(&safe, curve, cart, rest_x, *seat).y;
                    assert!(y.is_finite(), "{g:?} {p:?} {motion:?} frame {i}: {y}");
                    assert!(
                        (y - last).abs() < 0.01 || (motion == Motion::Insert) == (y > last),
                        "{g:?} {p:?} {motion:?}: frame {i} moved the wrong way, {last} to {y}"
                    );
                    last = y;
                }
            }
        }
    }
}

#[test]
fn an_insert_and_an_eject_meet_in_the_slot() {
    // The refusal frame: `Inserting` ends at the seat and `Ejecting` begins there on the other
    // profile. Both profiles have to put the cart in the same place or it jumps that frame.
    for p in PLATFORMS {
        let cart = cart_of(p);
        for g in GEOMETRIES {
            let safe = SafeArea::for_geometry(g);
            let rest_x = (safe.panel_w as f32 - cart.0) / 2.0;
            let in_ = travel(&safe, curve_of(p, Motion::Insert), cart, rest_x, 1.0);
            let out = travel(&safe, curve_of(p, Motion::Eject), cart, rest_x, 1.0);
            assert!(
                (in_.x - out.x).abs() < 0.01
                    && (in_.y - out.y).abs() < 0.01
                    && (in_.w - out.w).abs() < 0.01
                    && (in_.h - out.h).abs() < 0.01,
                "{g:?} {p:?}: the cart jumps when it is refused: {in_:?} to {out:?}"
            );
        }
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
    insert_view.draw_insert(
        &mut b,
        &mut c,
        &safe,
        Platform::Gba,
        &t,
        Insertion {
            seat: 0.0,
            motion: Motion::Insert,
        },
    );

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
        v.draw_insert(
            &mut canvas,
            &mut c,
            &safe,
            Platform::Gba,
            &t,
            Insertion {
                seat,
                motion: Motion::Insert,
            },
        );

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
    v.draw_insert(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &t,
        Insertion {
            seat: 1.0,
            motion: Motion::Insert,
        },
    );

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
    v.draw_insert(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &t,
        Insertion {
            seat: 1.0,
            motion: Motion::Insert,
        },
    );

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
            v.draw_insert(
                &mut canvas,
                &mut c,
                &safe,
                Platform::Gba,
                &t,
                Insertion {
                    seat,
                    motion: Motion::Insert,
                },
            );
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

    v.draw_insert(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &t,
        Insertion {
            seat: 0.0,
            motion: Motion::Insert,
        },
    );
    let warm = canvas.ops.len();

    for i in 1..=STEPS {
        v.draw_insert(
            &mut canvas,
            &mut c,
            &safe,
            Platform::Gba,
            &t,
            Insertion {
                seat: i as f32 / STEPS as f32,
                motion: Motion::Insert,
            },
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
            Insertion {
                seat: i as f32 / STEPS as f32,
                motion: Motion::Insert,
            },
        );
        assert!(
            !images(&canvas).is_empty(),
            "frame {i} of a one-cart insert drew no cart"
        );
    }
}

#[test]
fn the_port_trim_is_drawn_after_the_cart_going_in() {
    // The trim is on the front of the machine, so it goes on after the cartridge does: a
    // cartridge drawn over the machine's face would be lying on it rather than in it.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let t = ["A", "B", "C"];
    let mut v = ShelfView::default();
    v.shelf.set_len(3);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw_insert(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &t,
        Insertion {
            seat: 1.0,
            motion: Motion::Insert,
        },
    );

    let band = safe.panel_h as f32 - MOUTH_H;
    let cart = canvas
        .frame()
        .iter()
        .rposition(|o| matches!(o, Op::Image { y, .. } if *y > band))
        .expect("a seated insert drew no cart");
    let (port, ..) = port_at(canvas.frame(), Platform::Gba).expect("a seated insert drew no trim");
    assert!(
        port > cart,
        "the trim is drawn at {port} and the cart at {cart}: the cart is over the machine's face"
    );
}

#[test]
fn a_seated_cart_shows_through_the_port_trim() {
    // Two contracts meet here. The frame says the trim is drawn after the cartridge; the
    // drawing says its middle is empty. Neither alone keeps a seated cartridge visible: trim
    // drawn first is covered by nothing, and a trim painted solid hides the game in the slot.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let t = ["A", "B", "C"];
    let mut v = ShelfView::default();
    v.shelf.set_len(3);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw_insert(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &t,
        Insertion {
            seat: 1.0,
            motion: Motion::Insert,
        },
    );

    let band = safe.panel_h as f32 - MOUTH_H;
    let cart = canvas
        .frame()
        .iter()
        .rposition(|o| matches!(o, Op::Image { y, .. } if *y > band))
        .expect("a seated insert drew no cart");
    let (port, x, y, w, h) =
        port_at(canvas.frame(), Platform::Gba).expect("a seated insert drew no trim");
    assert!(port > cart, "the trim is not over the cart at all");

    // Down the middle of the mouth, where the cartridge sits: the trim must paint nothing
    // there. The mask is the same drawing the frame drew, so its middle is the middle.
    let s = skin::skin(Platform::Gba);
    let mask = svg::rasterize(s.port, s.port_size.0 as u32, s.port_size.1 as u32)
        .expect("the trim rasterises");
    let (px, py) = (safe.panel_w as f32 / 2.0, band + MOUTH_H * 0.75);
    assert!(
        px >= x && px <= x + w && py >= y && py <= y + h,
        "the trim does not cover the mouth"
    );
    let mx = (((px - x) * mask.w as f32 / w) as u32).min(mask.w - 1);
    let my = (((py - y) * mask.h as f32 / h) as u32).min(mask.h - 1);
    assert_eq!(
        mask.a[(my * mask.w + mx) as usize],
        0,
        "the trim paints over the seated cart at {px},{py}"
    );
}

#[test]
fn the_port_trim_stays_on_the_panel_on_every_geometry() {
    // Seven platforms, three panels, at rest and seated. A trim that measured itself off the
    // cart instead of the mouth would run off the side of the widest cart's shelf.
    let t = ["A", "B"];
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        for p in PLATFORMS {
            for seat in [0.0f32, 1.0] {
                let mut v = ShelfView::default();
                v.shelf.set_len(2);
                let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
                let mut c = ctx();
                v.draw_insert(
                    &mut canvas,
                    &mut c,
                    &safe,
                    p,
                    &t,
                    Insertion {
                        seat,
                        motion: Motion::Insert,
                    },
                );

                let (_, x, y, w, h) = port_at(canvas.frame(), p)
                    .unwrap_or_else(|| panic!("{g:?} {p:?} seat {seat}: no trim"));
                for val in [x, y, w, h] {
                    assert!(val.is_finite(), "{g:?} {p:?} seat {seat}: {x},{y} {w}x{h}");
                }
                assert!(w > 0.0 && h > 0.0, "{g:?} {p:?} seat {seat}: {w}x{h}");
                assert_eq!(h, MOUTH_H, "{g:?} {p:?} seat {seat}: trim height {h}");
                assert!(
                    x >= 0.0 && x + w <= safe.panel_w as f32,
                    "{g:?} {p:?} seat {seat}: the trim at {x}..{} leaves a {}-wide panel",
                    x + w,
                    safe.panel_w
                );
            }
        }
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
    v.draw_insert(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &[],
        Insertion {
            seat: 0.5,
            motion: Motion::Insert,
        },
    );
    assert!(!canvas.frame().is_empty(), "an empty insert drew nothing");
}
