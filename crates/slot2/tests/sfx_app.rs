//! The contract for when the slot makes its noise. Task 19 makes these pass without editing
//! this file.
//!
//! The whole of keeping the sound with the picture is *when the clip starts*. Each clip has a
//! lead — how far into it the contacts are — and the caller starts it that long before the
//! cart reaches them.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_audio::Sfx;
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};
use slot2_ui::insert::SEATED_AT;

use slot2::app::{App, Screen, SinkRequest};

static NEXT_CARD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn app(n: usize) -> (App, PathBuf) {
    let i = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-sfx-{}-{i}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for g in 0..n {
        std::fs::write(
            card.games_dir(Platform::Gba).join(format!("Game {g:02}.gba")),
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

/// Run to 60 Hz, stopping as soon as the app hands the loop something to play. Returns how
/// many seconds of animation had passed, and the frames waiting in the ring.
fn until_sound(a: &mut App, from: Instant, limit: f32) -> Option<(f32, usize)> {
    let mut now = from;
    let mut elapsed = 0.0;
    for _ in 0..(limit * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        elapsed += 1.0 / 60.0;
        a.tick(now);
        if a.take_sink_request() == Some(SinkRequest::Open) {
            let c = a.take_consumer().expect("an Open with no consumer");
            return Some((elapsed, c.frames()));
        }
    }
    None
}

#[test]
fn the_contacts_are_heard_when_they_are_seen() {
    // The clip starts its own lead before the cart reaches the contacts, so the loudest part
    // of the sound lands on the frame the cart seats. Starting it when the cart arrives puts
    // the whole sound late by the length of its lead, which is a tenth of a second — small
    // enough to sound like a cheap recording rather than like a mistake, which is worse.
    let (mut a, _root) = app(1);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    let (at, _) = until_sound(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.2)
        .expect("an insert made no sound");
    let want = SEATED_AT - Sfx::Insert.lead();
    assert!(
        (at - want).abs() < 0.05,
        "the clip started at {at}s, and the contacts are at {} - {} = {want}",
        SEATED_AT,
        Sfx::Insert.lead()
    );
}

#[test]
fn the_whole_clip_is_handed_over_at_once() {
    // The frame after this one may be the one that loads a core, which blocks for about a
    // second on the device. The sink has its own thread and plays what is in the ring; a clip
    // dripped in a frame at a time would stop dead at the seat, which is the exact moment it
    // is supposed to be making its point.
    let (mut a, _root) = app(1);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    let (_, frames) = until_sound(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.2)
        .expect("an insert made no sound");
    let want = (Sfx::Insert.seconds() * 48_000.0) as usize;
    assert!(
        frames + 1 >= want,
        "only {frames} of {want} frames were waiting when the sink was opened"
    );
}

#[test]
fn an_insert_makes_one_noise_not_one_a_frame() {
    let (mut a, _root) = app(1);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    let mut opens = 0;
    let mut now = t + Duration::from_millis(60);
    for _ in 0..((SEATED_AT + 0.2) * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.take_sink_request() == Some(SinkRequest::Open) {
            opens += 1;
            let _ = a.take_consumer();
        }
    }
    assert_eq!(opens, 1, "one insert asked for {opens} sinks");
}

#[test]
fn coming_back_out_has_its_own_noise() {
    // A cart with no core goes all the way in and comes back out. The way out is a different
    // recording, and it is not the way in played backwards.
    let (mut a, _root) = app(1);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    // Past the insert's clip and into the eject.
    let mut now = t + Duration::from_millis(60);
    let mut opens = Vec::new();
    for _ in 0..((SEATED_AT + 0.4) * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.take_sink_request() == Some(SinkRequest::Open) {
            let c = a.take_consumer().expect("an Open with no consumer");
            opens.push(c.frames());
        }
    }
    assert_eq!(
        opens.len(),
        2,
        "expected an insert and an eject, got {} noises",
        opens.len()
    );
    let eject = (Sfx::Eject.seconds() * 48_000.0) as usize;
    assert!(
        opens[1] + 1 >= eject,
        "the second noise was {} frames, not an eject's {eject}",
        opens[1]
    );
    assert_ne!(
        opens[0], opens[1],
        "both directions handed over the same clip"
    );
}

#[test]
fn a_shelf_that_refuses_a_press_stays_quiet() {
    // Nothing went into the slot, so the slot did not make a noise. The screen flinching is
    // the whole of the answer; a sound as well would claim something happened.
    let (mut a, _root) = app(0);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    assert!(
        until_sound(&mut a, t + Duration::from_millis(60), 1.0).is_none(),
        "an empty shelf made a noise"
    );
}

#[test]
fn a_still_shelf_asks_for_no_sink() {
    // Nothing is happening. Holding a device open costs power on a handheld.
    let (mut a, _root) = app(4);
    let t = Instant::now();
    assert!(
        until_sound(&mut a, t, 2.0).is_none(),
        "a shelf doing nothing opened a sink"
    );
}

#[test]
fn walking_the_row_is_not_a_noise() {
    // The slot's noises belong to the slot. Scrolling is not an insert.
    let (mut a, _root) = app(6);
    let t = Instant::now();
    let mut now = t;
    for b in [Button::Right, Button::Left, Button::R1, Button::L1] {
        tap(&mut a, b, now);
        now += Duration::from_millis(120);
        assert!(
            a.take_sink_request().is_none(),
            "{b:?} asked for a sink"
        );
    }
}
