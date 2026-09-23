//! The contract for saying no, and for saying why. Task 17 makes these pass without editing
//! this file.

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::{detect, Geometry};
use slot2_ui::layout::SafeArea;
use slot2_ui::refusal::{Refusal, REFUSAL_S};
use slot2_ui::shelf::MOUTH_H;
use slot2_ui::toast::{Toast, TOAST_FADE_S, TOAST_S};
use slot2_ui::UiCtx;

const GEOMETRIES: [Geometry; 3] = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];

fn ctx() -> UiCtx {
    UiCtx::new(detect().profile, "en", Vec::new(), None)
}

// ------------------------------------------------------------------ the flinch

#[test]
fn a_refusal_moves_on_the_frame_it_happened() {
    // A flinch is a knock; everything after it is settling. Starting from zero would make
    // the one frame the player pressed the button the one frame that did not move, which
    // reads as the press being ignored rather than refused.
    let r = Refusal::new();
    assert!(r.active());
    assert!(
        r.offset().abs() > 3.0,
        "the first frame is only {} off centre",
        r.offset()
    );
}

#[test]
fn a_refusal_is_over_quickly_and_ends_where_it_started() {
    // Short enough not to look like something the player has to dismiss, and back on centre
    // when it stops — a screen left a few pixels off is a screen that looks broken.
    let mut r = Refusal::new();
    for _ in 0..(REFUSAL_S * 60.0) as u32 + 2 {
        r.tick(1.0 / 60.0);
    }
    assert!(!r.active(), "the refusal is still going");
    assert_eq!(r.offset(), 0.0, "the screen was left off centre");
}

#[test]
fn a_refusal_shakes_and_decays() {
    // Both halves matter. Something that only decays is a slide off centre and back;
    // something that only shakes never stops.
    let mut r = Refusal::new();
    let mut signs = 0;
    let mut last = r.offset();
    let mut early = 0.0f32;
    let mut late = 0.0f32;
    for i in 0..(REFUSAL_S * 60.0) as u32 {
        r.tick(1.0 / 60.0);
        let o = r.offset();
        if (o < 0.0) != (last < 0.0) {
            signs += 1;
        }
        if i < 4 {
            early = early.max(o.abs());
        }
        if i >= (REFUSAL_S * 60.0) as u32 - 5 {
            late = late.max(o.abs());
        }
        last = o;
    }
    assert!(
        signs >= 4,
        "it crossed centre {signs} times — that is a slide"
    );
    assert!(late < early * 0.5, "it did not decay: {early} then {late}");
}

#[test]
fn a_refusal_that_is_never_ticked_is_still_finite() {
    // A caller that forgets to tick gets a stuck screen, not a NaN.
    let r = Refusal::default();
    assert!(r.offset().is_finite());
    let mut r = Refusal::new();
    r.tick(f32::INFINITY);
    assert!(
        r.offset().is_finite(),
        "offset {} after a bad dt",
        r.offset()
    );
    assert!(!r.active());
}

// ------------------------------------------------------------------ the words

fn toast(key: &str) -> Toast {
    Toast::new(key, Vec::new())
}

#[test]
fn a_toast_fades_in_holds_and_fades_out() {
    let mut t = toast("core-missing");
    assert!(!t.done());
    let first = t.alpha();
    assert!(first < 0.5, "it snapped on at {first}");

    // Through the fade in.
    for _ in 0..(TOAST_FADE_S * 60.0) as u32 + 1 {
        t.tick(1.0 / 60.0);
    }
    assert!(
        t.alpha() > 0.9,
        "it is only {} lit after the fade in",
        t.alpha()
    );

    // Most of its life at full.
    for _ in 0..((TOAST_S - 2.0 * TOAST_FADE_S) * 60.0) as u32 - 2 {
        t.tick(1.0 / 60.0);
        assert!(t.alpha() > 0.9, "it dimmed mid-life to {}", t.alpha());
    }

    // And out.
    for _ in 0..(TOAST_FADE_S * 60.0) as u32 + 4 {
        t.tick(1.0 / 60.0);
    }
    assert!(t.done(), "the toast never went away");
    assert_eq!(t.alpha(), 0.0);
}

#[test]
fn a_toast_that_is_done_draws_nothing() {
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut t = toast("core-missing");
    for _ in 0..(TOAST_S * 60.0) as u32 + 10 {
        t.tick(1.0 / 60.0);
    }
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    t.draw(&mut canvas, &mut c, &safe);
    assert!(canvas.frame().is_empty(), "a finished toast still drew");
}

#[test]
fn a_toast_says_the_whole_sentence() {
    // The key goes through i18n. A toast that drew the key itself, or a sentence glued
    // together in Rust, would be untranslatable — and it would show as `core-missing` on
    // screen, which is the tell.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut t = toast("core-missing");
    t.tick(TOAST_FADE_S);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    t.draw(&mut canvas, &mut c, &safe);

    // Text reaches the screen as alpha coverage, so what proves the sentence was rasterised
    // is an upload. The key is eight words shorter than the sentence; if the key were drawn
    // instead, far less would have been laid out.
    let masks = canvas
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. }))
        .count();
    assert!(masks > 0, "nothing text-shaped was rasterised");

    let laid_out = c.i18n.t("core-missing");
    assert_ne!(laid_out, "core-missing", "the message pack has no such key");
    assert!(
        laid_out.len() > "core-missing".len(),
        "the sentence is not longer than its key, so this test proves nothing"
    );
}

#[test]
fn a_toast_sits_above_the_slot_on_every_panel() {
    // In the gap between the feet of the row and the band. Over the slot it is unreadable;
    // up among the carts it covers the thing it is talking about.
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        let mut t = toast("core-missing");
        t.tick(TOAST_FADE_S);
        let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
        let mut c = ctx();
        t.draw(&mut canvas, &mut c, &safe);

        let band = safe.panel_h as f32 - MOUTH_H;
        let (mut top, mut bottom) = (f32::INFINITY, f32::NEG_INFINITY);
        for op in canvas.frame() {
            let (y, h) = match op {
                Op::Rect { y, h, .. } | Op::Image { y, h, .. } => (*y, *h),
                _ => continue,
            };
            top = top.min(y);
            bottom = bottom.max(y + h);
        }
        assert!(top.is_finite(), "{g:?}: the toast drew nothing");
        assert!(
            bottom <= band + 1.0,
            "{g:?}: the toast runs to {bottom}, over a slot that starts at {band}"
        );
        assert!(
            top > band - 120.0,
            "{g:?}: the toast is at {top}, up among the carts"
        );
    }
}

#[test]
fn a_toast_is_centred_and_stays_on_the_panel() {
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        let mut t = Toast::new(
            "cart-broken",
            vec![(
                "title".to_string(),
                slot2_i18n::Arg::Str("A Very Long Game Name Indeed, The Sequel".into()),
            )],
        );
        t.tick(TOAST_FADE_S);
        let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
        let mut c = ctx();
        t.draw(&mut canvas, &mut c, &safe);

        let (mut left, mut right) = (f32::INFINITY, f32::NEG_INFINITY);
        for op in canvas.frame() {
            let (x, w) = match op {
                Op::Rect { x, w, .. } | Op::Image { x, w, .. } => (*x, *w),
                _ => continue,
            };
            left = left.min(x);
            right = right.max(x + w);
        }
        assert!(left.is_finite(), "{g:?}: nothing drawn");
        assert!(
            left >= -0.5 && right <= safe.panel_w as f32 + 0.5,
            "{g:?}: the toast runs from {left} to {right} on a {}-wide panel",
            safe.panel_w
        );
        let mid = (left + right) / 2.0;
        assert!(
            (mid - safe.panel_w as f32 / 2.0).abs() < 2.0,
            "{g:?}: the toast is centred on {mid}, not {}",
            safe.panel_w as f32 / 2.0
        );
    }
}

#[test]
fn a_name_nobody_agreed_the_length_of_still_fits() {
    // The gap the first version of this contract left. Its "long" title was forty
    // characters, which fits a 640 panel with room to spare, so the test passed on code that
    // centred a plate of whatever width the text came out at — and a real filename is longer
    // than that. Off both edges, the part of the message that gets cut is the game's name.
    let long = "Shin Megami Tensei Devil Survivor Overclocked Special Edition \
                Director's Cut (USA, Europe) (Rev 2)";
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        let mut t = Toast::new(
            "cart-broken",
            vec![("title".to_string(), slot2_i18n::Arg::Str(long.into()))],
        );
        t.tick(TOAST_FADE_S);
        let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
        let mut c = ctx();
        t.draw(&mut canvas, &mut c, &safe);

        let (mut left, mut right) = (f32::INFINITY, f32::NEG_INFINITY);
        for op in canvas.frame() {
            let (x, w) = match op {
                Op::Rect { x, w, .. } | Op::Image { x, w, .. } => (*x, *w),
                _ => continue,
            };
            left = left.min(x);
            right = right.max(x + w);
        }
        assert!(left.is_finite(), "{g:?}: nothing drawn");
        assert!(
            left >= -0.5 && right <= safe.panel_w as f32 + 0.5,
            "{g:?}: a long name ran from {left} to {right} on a {}-wide panel",
            safe.panel_w
        );
    }
}

#[test]
fn a_toast_has_something_behind_it() {
    // It is drawn over a wallpaper and over the carts. Text alone on top of artwork is text
    // nobody can read.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut t = toast("core-missing");
    t.tick(TOAST_FADE_S);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    t.draw(&mut canvas, &mut c, &safe);

    let first_plate = canvas
        .frame()
        .iter()
        .position(|o| matches!(o, Op::Rect { .. }));
    let first_text = canvas
        .frame()
        .iter()
        .position(|o| matches!(o, Op::Image { .. }));
    let (Some(plate), Some(text)) = (first_plate, first_text) else {
        panic!("the toast drew a plate ({first_plate:?}) or text ({first_text:?}), not both");
    };
    assert!(plate < text, "the plate was drawn over the words");
}

#[test]
fn a_toast_does_not_rasterise_every_frame() {
    // Three seconds at sixty. It says the same thing the whole time.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut t = toast("core-missing");
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();

    t.draw(&mut canvas, &mut c, &safe);
    let warm = canvas.ops.len();
    for _ in 0..(TOAST_S * 60.0) as u32 - 2 {
        t.tick(1.0 / 60.0);
        t.draw(&mut canvas, &mut c, &safe);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(warm)
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count();
    assert!(
        uploads <= 2,
        "three seconds of toast uploaded {uploads} times"
    );
}
