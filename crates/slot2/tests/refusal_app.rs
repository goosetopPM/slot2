//! The contract for what the frontend does when it will not do what it was asked. Task 17
//! makes these pass without editing this file.
//!
//! The hole this closes: a cart whose core is missing goes all the way into the slot and
//! comes back out with no explanation. That is the most common thing to be wrong with a
//! half-built card, and the player can fix it — if they are told what it is.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};
use slot2_ui::insert::SEATED_AT;
use slot2_ui::toast::TOAST_S;

use slot2::app::{App, Screen};

static NEXT_CARD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn app(n: usize) -> (App, PathBuf) {
    let i = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-refusal-{}-{i}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for g in 0..n {
        std::fs::write(
            card.games_dir(Platform::Gba)
                .join(format!("Game {g:02}.gba")),
            b"rom",
        )
        .unwrap();
    }
    let a = App::with_card(
        card,
        std::env::temp_dir().join("slot2-no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    (a, root)
}

fn ui() -> slot2_ui::UiCtx {
    slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None)
}

fn safe() -> slot2_ui::layout::SafeArea {
    slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480)
}

fn tap(a: &mut App, b: Button, at: Instant) {
    a.feed(&Event::Button {
        button: b,
        pressed: true,
        at,
    });
    a.feed(&Event::Button {
        button: b,
        pressed: false,
        at: at + Duration::from_millis(40),
    });
    a.tick(at + Duration::from_millis(60));
}

fn run(a: &mut App, from: Instant, secs: f32) -> Instant {
    let mut now = from;
    for _ in 0..(secs * 60.0).ceil() as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
    }
    now
}

/// Every rect and image in a frame, as (x, y, w, h).
fn boxes(canvas: &RecordingCanvas) -> Vec<(f32, f32, f32, f32)> {
    canvas
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .collect()
}

// ------------------------------------------------------------------ saying why

#[test]
fn a_missing_core_says_which_thing_is_missing() {
    // Not "something went wrong". The player has to be able to go and fix it, and what they
    // would have to fix is a file that is not on the card.
    let (mut a, _root) = app(4);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    assert_eq!(a.toast_key(), None, "it complained before it tried");

    run(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.1);
    assert_eq!(
        a.toast_key(),
        Some("core-missing"),
        "a cart with no core came back out saying nothing"
    );
    assert_eq!(a.screen, Screen::Ejecting);
}

#[test]
fn the_message_goes_away_on_its_own() {
    // It is a toast, not a dialog. Nothing to dismiss and nothing left on screen.
    let (mut a, _root) = app(4);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = run(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.1);
    assert!(a.toast_key().is_some());

    run(&mut a, now, TOAST_S + 0.2);
    assert_eq!(a.toast_key(), None, "the message stayed up");
}

#[test]
fn the_message_outlives_the_animation_that_caused_it() {
    // The eject is 0.45s and the message is three. A message that went with the cart would
    // be gone before a player who was watching the cartridge looked up.
    let (mut a, _root) = app(4);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = run(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.1);

    let now = run(&mut a, now, 0.6);
    assert_eq!(a.screen, Screen::List, "the eject should be over by now");
    assert_eq!(
        a.toast_key(),
        Some("core-missing"),
        "the message left with the cart"
    );
    let _ = now;
}

#[test]
fn the_words_reach_the_screen() {
    // The state machine could be perfect and draw nothing.
    let (mut a, _root) = app(4);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = run(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.3);

    let s = safe();
    let mut plain = RecordingCanvas::new(s.panel_w, s.panel_h);
    let mut ctx = ui();
    a.draw(&mut plain, &mut ctx, now);
    let with_toast = plain
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. }))
        .count();
    assert!(with_toast > 0, "nothing text-shaped was rasterised");
}

// ------------------------------------------------------------------ saying no

#[test]
fn pressing_a_on_an_empty_shelf_is_refused_not_ignored() {
    // Today nothing happens at all, which is indistinguishable from a dead button.
    let (mut a, _root) = app(0);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    assert_eq!(a.screen, Screen::List, "an empty shelf started an insert");
    assert!(
        a.refusal_offset().abs() > 1.0,
        "the screen did not flinch: offset {}",
        a.refusal_offset()
    );
}

#[test]
fn the_flinch_is_over_and_leaves_the_screen_straight() {
    let (mut a, _root) = app(0);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    run(&mut a, t + Duration::from_millis(60), 0.5);
    assert_eq!(
        a.refusal_offset(),
        0.0,
        "the screen was left off centre for good"
    );
}

#[test]
fn a_refused_cart_does_not_make_the_screen_flinch_as_well() {
    // The cart is on screen and carries the refusal by coming back out. A screen that
    // flinched too would read as two separate failures.
    let (mut a, _root) = app(4);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let mut now = t + Duration::from_millis(60);
    for _ in 0..((SEATED_AT + 0.4) * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        assert_eq!(
            a.refusal_offset(),
            0.0,
            "the screen flinched while the cart was already coming back out"
        );
    }
}

#[test]
fn a_flinch_moves_the_whole_screen_together() {
    // Not some of it. A shelf where the carts shook and the slot did not would read as the
    // carts falling over.
    let (mut a, _root) = app(0);
    let s = safe();
    let mut ctx = ui();
    let t = Instant::now();

    let mut still = RecordingCanvas::new(s.panel_w, s.panel_h);
    a.draw(&mut still, &mut ctx, t);
    let before = boxes(&still);

    tap(&mut a, Button::A, t);
    let off = a.refusal_offset();
    assert!(off.abs() > 1.0, "nothing to compare: offset {off}");

    let mut shaken = RecordingCanvas::new(s.panel_w, s.panel_h);
    a.draw(&mut shaken, &mut ctx, t + Duration::from_millis(60));
    let after = boxes(&shaken);

    assert_eq!(
        before.len(),
        after.len(),
        "a flinch changed what is on screen, not where it is"
    );
    for (i, (b, a2)) in before.iter().zip(after.iter()).enumerate() {
        assert!(
            (a2.0 - b.0 - off).abs() < 0.5,
            "op {i} moved by {} and the screen by {off}",
            a2.0 - b.0
        );
        assert!((a2.1 - b.1).abs() < 0.5, "op {i} moved vertically");
        assert!(
            (a2.2 - b.2).abs() < 0.5 && (a2.3 - b.3).abs() < 0.5,
            "op {i} changed size during a flinch"
        );
    }
}

#[test]
fn a_flinch_does_not_stop_the_player_doing_something_else() {
    // Three tenths of a second is short, but it is not a modal.
    let (mut a, _root) = app(0);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    assert!(a.refusal_offset().abs() > 1.0);

    tap(&mut a, Button::R1, t + Duration::from_millis(80));
    assert_ne!(
        a.platform(),
        Platform::Gba,
        "the shoulder did nothing during a flinch"
    );
}
