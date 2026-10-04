//! The contract for when the slot makes its noise. Task 19 makes these pass without editing
//! this file.
//!
//! The whole of keeping the sound with the picture is *when the clip starts*. Each clip has a
//! lead — how far into it the contacts are — and the caller starts it that long before the
//! cart reaches them. Since Task 99 the lead is the shelf's own: the clip is played at that
//! platform's speed, so its contacts sit at a different point in the samples, and the cue has
//! to be computed at the same speed. So the two halves of this file are the two halves of
//! that: what is in the ring (the shelf's render, whole) and when the ring was handed it.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_audio::{Consumer, Sfx};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};
use slot2_ui::insert::{EJECT_S, SEATED_AT};
use slot2_ui::skin::{self, SoundProfile};

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

/// An app on `platform`'s shelf with `n` carts of that platform's own kind.
///
/// The app boots on the Game Boy Advance shelf, so the others are reached with R1 — the key a
/// player uses, rather than any private index.
fn app_on(platform: Platform, n: usize) -> (App, PathBuf) {
    let i = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-sfx-{}-{i}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    let ext = platform.extensions().first().copied().unwrap_or("bin");
    for g in 0..n {
        std::fs::write(
            card.games_dir(platform).join(format!("Game {g:02}.{ext}")),
            b"rom",
        )
        .unwrap();
    }
    let mut a = App::with_card(
        card,
        std::env::temp_dir().join("slot2-no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    let mut now = Instant::now();
    for _ in 0..Platform::ALL.len() {
        if a.platform() == platform {
            break;
        }
        tap(&mut a, Button::R1, now);
        now += Duration::from_millis(80);
    }
    assert_eq!(
        a.platform(),
        platform,
        "never reached the {platform:?} shelf"
    );
    // Let the clock settle, so the frame the insert starts on is the same size whatever the
    // shelf took to walk to.
    a.tick(now + Duration::from_millis(100));
    (a, root)
}

/// Take everything the sink has not played yet.
fn drain(c: &mut Consumer) -> Vec<i16> {
    let mut out = vec![0i16; c.available()];
    let n = c.read(&mut out);
    out.truncate(n);
    out
}

/// Tick from `from` at 60 Hz until the app hands the loop something to play, and give back the
/// animation time it had reached and the samples it wrote. The one sample: the clip is written
/// in one go, so what comes out is the whole of it.
fn until_clip(a: &mut App, from: Instant, limit: f32) -> Option<(f32, Vec<i16>)> {
    let mut now = from;
    let mut elapsed = 0.0;
    for _ in 0..(limit * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        elapsed += 1.0 / 60.0;
        a.tick(now);
        if a.take_sink_request() == Some(SinkRequest::Open) {
            let mut c = a.take_consumer().expect("an Open with no consumer");
            return Some((elapsed, drain(&mut c)));
        }
    }
    None
}

/// Both noises of one cart: the way in, then the way back out with no core to keep it seated.
/// Panics unless the app made exactly those two, so "one noise per direction" is asserted here
/// and read out of the pair below.
fn two_clips(a: &mut App, from: Instant) -> (Vec<i16>, Vec<i16>) {
    let mut now = from;
    let mut clips: Vec<Vec<i16>> = Vec::new();
    for _ in 0..((SEATED_AT + EJECT_S + 0.3) * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.take_sink_request() == Some(SinkRequest::Open) {
            let mut c = a.take_consumer().expect("an Open with no consumer");
            clips.push(drain(&mut c));
        }
    }
    assert_eq!(
        clips.len(),
        2,
        "expected an insert and an eject, got {} noises",
        clips.len()
    );
    let eject = clips.remove(1);
    (clips.remove(0), eject)
}

/// The clip a shelf's profile asks the audio crate for, at the rate the app's sink opens at.
fn expected(clip: Sfx, profile: slot2_ui::skin::SoundProfile) -> Vec<i16> {
    clip.render_styled(48_000, profile.speed(), profile.gain())
}

/// Compare without printing two whole clips when they differ.
fn assert_clip(what: &str, got: &[i16], want: &[i16]) {
    assert_eq!(
        got.len(),
        want.len(),
        "{what}: {} samples against {}",
        got.len(),
        want.len()
    );
    assert!(
        got == want,
        "{what}: the samples are not this shelf's render"
    );
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
    // This shelf's own lead, not the recording's: the clip is read at the shelf's speed, so
    // its contacts are not where the recording put them. The cue and the samples are measured
    // at the same effective speed, which is what keeps the two together.
    let lead = Sfx::Insert.lead_at_speed(skin::skin(a.platform()).sfx_in.speed());
    let want = SEATED_AT - lead;
    assert!(
        (at - want).abs() < 0.05,
        "the clip started at {at}s, and the contacts are at {} - {lead} = {want}",
        SEATED_AT
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
    // This shelf's own speed, not the recording's length: the clip handed over is the one the
    // profile renders. The ring keeps one slot free, so a frame either way is the ring and not
    // a clip that stopped early.
    let profile = skin::skin(a.platform()).sfx_in;
    let want = expected(Sfx::Insert, profile).len() / 2;
    assert!(
        frames.abs_diff(want) <= 1,
        "only {frames} of {want} frames were waiting when the sink was opened"
    );
}

#[test]
fn an_insert_makes_one_noise_not_one_a_frame() {
    let (mut a, _root) = app(1);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    // Counted over the insert and no further. This cart has no core, so it is on its way back
    // out the frame after it seats, and the eject's own noise belongs to the test below. A
    // clip fired once a frame still shows up here: the threshold is a lead ahead of the seat,
    // which is six frames of them.
    let mut opens = 0;
    let mut now = t + Duration::from_millis(60);
    for _ in 0..((SEATED_AT + 0.2) * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.take_sink_request() == Some(SinkRequest::Open) {
            opens += 1;
            let _ = a.take_consumer();
        }
        if a.screen != Screen::Inserting {
            break;
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
    let eject = expected(Sfx::Eject, skin::skin(a.platform()).sfx_out).len() / 2;
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
        assert!(a.take_sink_request().is_none(), "{b:?} asked for a sink");
    }
}

// --- one shelf's own noise --------------------------------------------------------------

/// The two ends of the table: the slowest cart of the seven going in and out (the NES) and the
/// fastest connector (the Mega Drive). Everything the profiles do has to hold at both.
const EXTREMES: [Platform; 2] = [Platform::Nes, Platform::Md];

#[test]
fn a_shelf_plays_the_clip_its_own_profile_renders() {
    // The wiring, not the table: the samples in the ring are what the audio crate renders from
    // this shelf's profile, so the profile really reaches the sink instead of being read and
    // dropped.
    for platform in EXTREMES {
        let (mut a, _root) = app_on(platform, 4);
        let profile = skin::skin(platform).sfx_in;
        let t = Instant::now();
        tap(&mut a, Button::A, t);

        let (_, samples) = until_clip(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.2)
            .unwrap_or_else(|| panic!("{platform:?} made no insert noise"));
        assert_clip(
            &format!("{platform:?} insert"),
            &samples,
            &expected(Sfx::Insert, profile),
        );
        // And the profile is not the neutral one, or this would pass on a shelf that ignored
        // it. Both of these platforms play the clip at a speed the recording does not have.
        assert_ne!(profile.speed(), 1.0, "{platform:?} has nothing to prove");
        assert_ne!(samples, expected(Sfx::Insert, SoundProfile::new(1.0, 1.0)));
    }
}

#[test]
fn the_contacts_are_cued_off_the_shelfs_own_lead() {
    // The clip is started its own lead before the cart seats, and "its own" is the lead the
    // shelf's speed implies. Falling back to the recording's lead puts the contacts on a
    // different frame — which is what the last assertion says: the frame the clip actually
    // started on is the shelf's own, and not the recording's shared one.
    for platform in EXTREMES {
        let (mut a, _root) = app_on(platform, 4);
        let profile = skin::skin(platform).sfx_in;
        let t = Instant::now();
        tap(&mut a, Button::A, t);

        let (at, _) = until_clip(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.2)
            .unwrap_or_else(|| panic!("{platform:?} made no insert noise"));
        // The frame the contacts are heard on, from where the clip actually started and the
        // lead this shelf reads its clip at.
        let heard = at + Sfx::Insert.lead_at_speed(profile.speed());
        assert!(
            (heard - SEATED_AT).abs() < 0.05,
            "{platform:?}: the contacts land at {heard}s, not at the seat {SEATED_AT}s"
        );

        let own = SEATED_AT - Sfx::Insert.lead_at_speed(profile.speed());
        let shared = SEATED_AT - Sfx::Insert.lead();
        assert!(
            (at - own).abs() < (at - shared).abs(),
            "{platform:?}: the clip started at {at}s, which is where the recording's shared \
             lead {shared} would have started it, not where this shelf's {own} did"
        );
    }
}

#[test]
fn a_refused_cart_makes_one_noise_each_way_and_all_of_each() {
    // A cart with no core goes in and comes back out. Each direction asks for the sink once,
    // and each hands over the whole of its own shelf's clip — not the recording, and not
    // dripped in a frame at a time.
    let (mut a, _root) = app_on(Platform::Nes, 4);
    let s = skin::skin(Platform::Nes);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    let (insert, eject) = two_clips(&mut a, t + Duration::from_millis(60));
    assert_clip("nes insert", &insert, &expected(Sfx::Insert, s.sfx_in));
    assert_clip("nes eject", &eject, &expected(Sfx::Eject, s.sfx_out));
    assert_eq!(
        a.screen,
        Screen::List,
        "the refusal did not end on the shelf"
    );
    assert_eq!(a.insert_seat(), None);
}

#[test]
fn the_seven_shelves_do_not_sound_alike() {
    // Frame length or actual samples, not the profile fields: two shelves could carry fourteen
    // different numbers and still be wired to one render. Each clip is held first against what
    // its own profile says it should be, and then against every shelf before it.
    let mut inserts: Vec<(Platform, Vec<i16>)> = Vec::new();
    let mut ejects: Vec<(Platform, Vec<i16>)> = Vec::new();
    for platform in Platform::ALL {
        let (mut a, _root) = app_on(platform, 3);
        let s = skin::skin(platform);
        let t = Instant::now();
        tap(&mut a, Button::A, t);

        let (insert, eject) = two_clips(&mut a, t + Duration::from_millis(60));
        assert_clip(
            &format!("{platform:?} insert"),
            &insert,
            &expected(Sfx::Insert, s.sfx_in),
        );
        assert_clip(
            &format!("{platform:?} eject"),
            &eject,
            &expected(Sfx::Eject, s.sfx_out),
        );
        for (q, other) in &inserts {
            assert!(
                other != &insert,
                "{platform:?} and {q:?} hand the sink the same insert"
            );
        }
        for (q, other) in &ejects {
            assert!(
                other != &eject,
                "{platform:?} and {q:?} hand the sink the same eject"
            );
        }
        inserts.push((platform, insert));
        ejects.push((platform, eject));
    }
}
