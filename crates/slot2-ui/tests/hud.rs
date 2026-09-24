//! The contract for the corner furniture. Task 20 makes these pass without editing this
//! file.
//!
//! Two clusters, on the panel corners rather than on the layout: the clock top left, the
//! gauge top right. What is checked here is where they land on three panel shapes, that the
//! capsule holds still when a cable moves, and that the two things that would be worse than
//! an empty corner -- an empty capsule, a clock that was never set -- are not drawn.

use slot2_gfx::{Color, Op, RecordingCanvas};
use slot2_platform::clock::SET_AFTER;
use slot2_platform::{Battery, Charge, Geometry};
use slot2_ui::hud::{
    ink, Hud, BOLT_GAP, BOLT_W, GAUGE_H, GAUGE_W, HUD_H, HUD_MARGIN, INK, LOW_INK, LOW_PERCENT,
    NUB_W, WALL,
};
use slot2_ui::layout::SafeArea;
use slot2_ui::UiCtx;

const GEOMETRIES: [Geometry; 3] = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];

/// Mid-morning, well after the epoch, so the clock counts as set.
const NOON: i64 = SET_AFTER + 12 * 3600 + 34 * 60;

fn ctx(g: Geometry) -> UiCtx {
    let mut c = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    c.safe = SafeArea::for_geometry(g);
    c
}

fn bat(percent: u8, charge: Charge) -> Option<Battery> {
    Some(Battery { percent, charge })
}

fn frame(g: Geometry, battery: Option<Battery>, local: Option<i64>) -> (RecordingCanvas, SafeArea) {
    let safe = SafeArea::for_geometry(g);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx(g);
    let mut hud = Hud::default();
    hud.draw(&mut canvas, &mut c, &safe, battery, local);
    (canvas, safe)
}

/// Every drawn op as `(x, y, w, h, tint)`. Uploads carry no position and are skipped.
fn drawn(c: &RecordingCanvas) -> Vec<(f32, f32, f32, f32, Color)> {
    c.frame()
        .iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
            Op::Image {
                x, y, w, h, tint, ..
            } => Some((*x, *y, *w, *h, *tint)),
            _ => None,
        })
        .collect()
}

fn rects(c: &RecordingCanvas) -> Vec<(f32, f32, f32, f32, Color)> {
    c.frame()
        .iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
            _ => None,
        })
        .collect()
}

fn images(c: &RecordingCanvas) -> Vec<(f32, f32, f32, f32, Color)> {
    c.frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image {
                x, y, w, h, tint, ..
            } => Some((*x, *y, *w, *h, *tint)),
            _ => None,
        })
        .collect()
}

/// The union of everything drawn.
fn bounds(ops: &[(f32, f32, f32, f32, Color)]) -> (f32, f32, f32, f32) {
    let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (x, y, w, h, _) in ops {
        b.0 = b.0.min(*x);
        b.1 = b.1.min(*y);
        b.2 = b.2.max(x + w);
        b.3 = b.3.max(y + h);
    }
    b
}

/// The four walls of the capsule: the only quads as thin as `WALL` in either direction.
fn walls(c: &RecordingCanvas) -> Vec<(f32, f32, f32, f32)> {
    let mut v: Vec<(f32, f32, f32, f32)> = rects(c)
        .into_iter()
        .filter(|(_, _, w, h, _)| (*w - WALL).abs() < 0.01 || (*h - WALL).abs() < 0.01)
        .map(|(x, y, w, h, _)| (x, y, w, h))
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

/// The fill colour of a frame, for the warning tests.
fn fill_ink(c: &RecordingCanvas) -> Color {
    fill(c).expect("no fill").4
}

/// The fill: the one quad as tall as the capsule interior.
fn fill(c: &RecordingCanvas) -> Option<(f32, f32, f32, f32, Color)> {
    let inner_h = GAUGE_H - 4.0 * WALL;
    let mut found = rects(c)
        .into_iter()
        .filter(|(_, _, _, h, _)| (*h - inner_h).abs() < 0.01);
    let one = found.next();
    assert!(
        found.next().is_none(),
        "more than one quad the height of the capsule interior"
    );
    one
}

// ------------------------------------------------------------------ what is not drawn

#[test]
fn nothing_to_say_draws_nothing() {
    for g in GEOMETRIES {
        let (c, _) = frame(g, None, None);
        assert!(
            drawn(&c).is_empty(),
            "{g:?} drew {} ops with no gauge and no clock",
            drawn(&c).len()
        );
    }
}

#[test]
fn no_gauge_leaves_the_right_corner_empty() {
    // An empty capsule says the battery is flat. A machine with no gauge has nothing to say
    // about its charge, which is a different statement and needs a different picture: none.
    let (c, safe) = frame(Geometry::W720H480, None, Some(NOON));
    let ops = drawn(&c);
    assert!(!ops.is_empty(), "the clock went missing too");
    let (_, _, right, _) = bounds(&ops);
    assert!(
        right < safe.panel_w as f32 / 2.0,
        "something was drawn on the right at {right} with no gauge"
    );
}

#[test]
fn a_clock_that_was_never_set_leaves_the_left_corner_empty() {
    let (c, safe) = frame(Geometry::W720H480, bat(64, Charge::Discharging), None);
    let ops = drawn(&c);
    assert!(!ops.is_empty(), "the gauge went missing too");
    let (left, _, _, _) = bounds(&ops);
    assert!(
        left > safe.panel_w as f32 / 2.0,
        "something was drawn on the left at {left} with no clock"
    );
}

// ------------------------------------------------------------------ where it lands

#[test]
fn the_hud_hangs_off_the_panel_not_off_the_layout() {
    // DESIGN 167. The safe area is where the layout lives; the HUD belongs to the screen, so
    // on the square panel it rides up into the extended band instead of following the box
    // down. A HUD placed with `safe.px`/`safe.py` passes on the 640 and fails here.
    for g in GEOMETRIES {
        let (c, safe) = frame(g, bat(64, Charge::Discharging), Some(NOON));
        let ops = drawn(&c);
        let (left, top, right, bottom) = bounds(&ops);

        assert!(
            (left - HUD_MARGIN).abs() < 0.51,
            "{g:?} left edge at {left}, wanted {HUD_MARGIN}"
        );
        assert!(
            (right - (safe.panel_w as f32 - HUD_MARGIN)).abs() < 0.51,
            "{g:?} right edge at {right}, wanted {}",
            safe.panel_w as f32 - HUD_MARGIN
        );
        assert!(
            (top - HUD_MARGIN).abs() < 0.51,
            "{g:?} top edge at {top}, wanted {HUD_MARGIN}"
        );
        assert!(
            bottom <= HUD_MARGIN + HUD_H + 0.51,
            "{g:?} ran {bottom} down the panel; the band is {HUD_H} tall"
        );
    }
}

#[test]
fn the_square_panel_puts_it_above_the_safe_area() {
    // The one geometry where the two placements differ by enough to see. 720x720 offsets the
    // safe area by (40, 120), so a HUD that followed it would start 120 px down.
    let g = Geometry::W720H720;
    let (c, safe) = frame(g, bat(64, Charge::Discharging), Some(NOON));
    assert_eq!(
        (safe.x, safe.y),
        (40, 120),
        "the geometry moved under this test"
    );
    let (left, top, _, bottom) = bounds(&drawn(&c));
    assert!(
        top < safe.y as f32,
        "the HUD sat at {top}, inside the safe area"
    );
    assert!(
        bottom < safe.y as f32,
        "the HUD reached {bottom}, into the safe area"
    );
    assert!(
        left < safe.x as f32,
        "the HUD sat at {left}, inside the safe area"
    );
}

#[test]
fn everything_stays_on_the_panel() {
    for g in GEOMETRIES {
        for b in [
            bat(0, Charge::Discharging),
            bat(100, Charge::Charging),
            bat(7, Charge::Full),
        ] {
            let (c, safe) = frame(g, b, Some(NOON));
            for (x, y, w, h, _) in drawn(&c) {
                assert!(
                    x >= 0.0
                        && y >= 0.0
                        && x + w <= safe.panel_w as f32
                        && y + h <= safe.panel_h as f32,
                    "{g:?} {b:?} drew ({x}, {y}, {w}, {h}) off a {}x{} panel",
                    safe.panel_w,
                    safe.panel_h
                );
            }
        }
    }
}

#[test]
fn each_cluster_sits_on_a_plate() {
    // Over a wallpaper the player chose, white ink on its own is ink that is sometimes not
    // there. Each corner gets exactly one plate, and it covers what is on it.
    let (c, safe) = frame(Geometry::W720H480, bat(64, Charge::Discharging), Some(NOON));
    let mid = safe.panel_w as f32 / 2.0;
    let plates: Vec<_> = rects(&c)
        .into_iter()
        .filter(|(_, _, _, h, _)| (*h - HUD_H).abs() < 0.01)
        .collect();
    assert_eq!(
        plates.len(),
        2,
        "wanted one plate per corner, got {plates:?}"
    );

    for (px, py, pw, ph, color) in plates {
        assert!(color.a < 1.0, "the plate is opaque at alpha {}", color.a);
        let side: Vec<_> = drawn(&c)
            .into_iter()
            .filter(|(x, _, w, _, _)| (x + w / 2.0 < mid) == (px + pw / 2.0 < mid))
            .collect();
        for (x, y, w, h, _) in side {
            assert!(
                x >= px - 0.01
                    && y >= py - 0.01
                    && x + w <= px + pw + 0.01
                    && y + h <= py + ph + 0.01,
                "({x}, {y}, {w}, {h}) hangs off its plate ({px}, {py}, {pw}, {ph})"
            );
        }
    }
}

// ------------------------------------------------------------------ the capsule

#[test]
fn the_fill_is_the_charge() {
    let inner = GAUGE_W - 4.0 * WALL;
    let (c, _) = frame(Geometry::W720H480, bat(0, Charge::Discharging), None);
    assert!(
        fill(&c).is_none_or(|(_, _, w, _, _)| w <= 0.0),
        "a flat battery drew a fill"
    );

    let mut last = 0.0;
    for p in [10u8, 25, 50, 75, 100] {
        let (c, _) = frame(Geometry::W720H480, bat(p, Charge::Discharging), None);
        let (_, _, w, _, _) = fill(&c).unwrap_or_else(|| panic!("{p}% drew no fill"));
        let want = inner * f32::from(p) / 100.0;
        assert!(
            (w - want).abs() < 0.51,
            "{p}% filled {w} of {inner}, wanted {want}"
        );
        assert!(w > last, "{p}% is no wider than the step below it");
        last = w;
    }
    assert!(
        (last - inner).abs() < 0.51,
        "a full battery filled {last} of {inner}"
    );
}

#[test]
fn the_capsule_does_not_move_when_a_cable_goes_in() {
    // The bolt has a slot of its own, reserved whether or not anything is charging. A bolt
    // that only takes room when it is drawn is a capsule that jumps sideways at the moment
    // the player is looking at it.
    let a = frame(Geometry::W720H480, bat(50, Charge::Discharging), None).0;
    let b = frame(Geometry::W720H480, bat(50, Charge::Charging), None).0;
    assert_eq!(
        walls(&a),
        walls(&b),
        "the capsule moved when the cable went in"
    );
    assert_eq!(
        walls(&a).len(),
        4,
        "a capsule is four walls, got {:?}",
        walls(&a)
    );

    let fa = fill(&a).unwrap();
    let fb = fill(&b).unwrap();
    assert_eq!(
        (fa.0, fa.1, fa.2, fa.3),
        (fb.0, fb.1, fb.2, fb.3),
        "the fill moved or resized when the cable went in"
    );
}

#[test]
fn charging_shows_a_mark_that_discharging_does_not() {
    let a = frame(Geometry::W720H480, bat(50, Charge::Discharging), None).0;
    let b = frame(Geometry::W720H480, bat(50, Charge::Charging), None).0;
    assert_eq!(
        images(&b).len(),
        images(&a).len() + 1,
        "charging added {} images, wanted one bolt",
        images(&b).len() as i64 - images(&a).len() as i64
    );
}

#[test]
fn the_bolt_never_reaches_the_capsule() {
    let (c, _) = frame(Geometry::W720H480, bat(50, Charge::Charging), None);
    let capsule_left = walls(&c).first().map(|w| w.0).expect("no capsule");
    let bolt = images(&c)
        .into_iter()
        .find(|(_, _, w, _, _)| (*w - BOLT_W).abs() < 0.01)
        .expect("no bolt");
    assert!(
        bolt.0 + bolt.2 <= capsule_left - BOLT_GAP + 0.01,
        "the bolt ends at {} and the capsule starts at {capsule_left}",
        bolt.0 + bolt.2
    );
}

#[test]
fn the_nub_is_on_the_positive_end() {
    let (c, _) = frame(Geometry::W720H480, bat(50, Charge::Discharging), None);
    let capsule_right = walls(&c).iter().map(|w| w.0 + w.2).fold(f32::MIN, f32::max);
    let nub = rects(&c)
        .into_iter()
        .find(|(_, _, w, _, _)| (*w - NUB_W).abs() < 0.01)
        .expect("no nub");
    assert!(
        nub.0 >= capsule_right - 0.01,
        "the nub at {} is inside a capsule that ends at {capsule_right}",
        nub.0
    );
}

// ------------------------------------------------------------------ the warning

#[test]
fn a_low_battery_reads_differently_only_while_it_is_going_down() {
    let low = |charge| fill_ink(&frame(Geometry::W720H480, bat(LOW_PERCENT - 1, charge), None).0);
    assert_eq!(
        low(Charge::Discharging),
        LOW_INK,
        "a low pack drew as normal"
    );
    // A pack at 14% with a cable in it is a pack on its way up. A warning there is a warning
    // about a problem that is already being fixed.
    assert_eq!(
        low(Charge::Charging),
        INK,
        "a low pack on charge drew as a warning"
    );
    assert_eq!(low(Charge::Full), INK, "a full pack drew as a warning");

    let ok = fill_ink(
        &frame(
            Geometry::W720H480,
            bat(LOW_PERCENT, Charge::Discharging),
            None,
        )
        .0,
    );
    assert_eq!(ok, INK, "the threshold itself is not low yet");
}

#[test]
fn the_ink_rule_stands_on_its_own() {
    // The same rule the drawing uses, checkable without a canvas.
    assert_eq!(
        ink(Battery {
            percent: LOW_PERCENT - 1,
            charge: Charge::Discharging
        }),
        LOW_INK
    );
    for c in [Charge::Charging, Charge::Full, Charge::Unknown] {
        assert_eq!(
            ink(Battery {
                percent: LOW_PERCENT - 1,
                charge: c
            }),
            INK,
            "{c:?}"
        );
    }
    assert_eq!(
        ink(Battery {
            percent: 100,
            charge: Charge::Discharging
        }),
        INK
    );
}

// ------------------------------------------------------------------ the cost per frame

#[test]
fn a_hud_that_does_not_change_costs_nothing_to_redraw() {
    // The clock changes once a minute and the percent less often than that. A face uploaded
    // per frame is a texture allocation sixty times a second for text that did not move.
    let g = Geometry::W720H480;
    let safe = SafeArea::for_geometry(g);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx(g);
    let mut hud = Hud::default();

    hud.draw(
        &mut canvas,
        &mut c,
        &safe,
        bat(64, Charge::Charging),
        Some(NOON),
    );
    let uploads = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
            .count()
    };
    assert!(
        uploads(&canvas.ops) > 0,
        "the first frame uploaded nothing at all"
    );

    let before = canvas.ops.len();
    for _ in 0..8 {
        hud.draw(
            &mut canvas,
            &mut c,
            &safe,
            bat(64, Charge::Charging),
            Some(NOON),
        );
    }
    assert_eq!(
        uploads(&canvas.ops[before..]),
        0,
        "eight unchanged frames uploaded something"
    );
}
