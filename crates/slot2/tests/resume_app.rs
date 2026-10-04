//! Launching a shelf cart: a tap picks up where the player left off, a hold starts over.
//!
//! The resume path needs a real core, so those tests use the established fixture and skip
//! loudly without it; the failed-startup path runs with no core at all.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_input::{Button, Event};
use slot2_store::{Card, Cart, Platform, StateKind};
use slot2_ui::insert::EJECT_S;

use slot2::app::{App, Screen, SinkRequest};

static SERIAL: Mutex<()> = Mutex::new(());
static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn core_dir() -> Option<PathBuf> {
    let d = repo().join("vendor");
    let name = if cfg!(windows) {
        "mgba_libretro.dll"
    } else if cfg!(target_os = "macos") {
        "mgba_libretro.dylib"
    } else {
        "mgba_libretro.so"
    };
    if d.join(name).is_file() {
        Some(d)
    } else {
        eprintln!(
            "no core in {} — skipping (run build/cores.ps1)",
            d.display()
        );
        None
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn ev(b: Button, pressed: bool, at: Instant) -> Event {
    Event::Button {
        button: b,
        pressed,
        at,
    }
}

fn scratch(tag: &str) -> PathBuf {
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-resume-{tag}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card with the test ROM on it, and the cart that names it.
fn card_with_rom(tag: &str) -> (Card, Cart) {
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    fs::copy(
        repo().join("assets/test/arm.gba"),
        card.games_dir(Platform::Gba).join("arm.gba"),
    )
    .unwrap();
    let cart = Cart {
        platform: Platform::Gba,
        stem: "arm".into(),
        title: "arm".into(),
        rom: card.games_dir(Platform::Gba).join("arm.gba"),
    };
    (card, cart)
}

fn app_for(card: Card, cores: PathBuf) -> App {
    App::with_card(
        card,
        cores,
        48_000,
        slot2::tuning_for(&slot2_platform::detect().profile),
        false,
        Screen::List,
    )
}

fn tap(a: &mut App, b: Button, at: Instant) {
    a.feed(&ev(b, true, at));
    a.feed(&ev(b, false, at + ms(40)));
    a.tick(at + ms(60));
}

/// SELECT held, then the shoulder: what `Gestures` turns into `Action::Chord`.
fn chord(a: &mut App, shoulder: Button, at: Instant) {
    a.feed(&ev(Button::Select, true, at));
    a.feed(&ev(shoulder, true, at + ms(20)));
    a.feed(&ev(shoulder, false, at + ms(60)));
    a.feed(&ev(Button::Select, false, at + ms(80)));
    a.tick(at + ms(100));
}

fn run(a: &mut App, from: Instant, secs: f32) -> Instant {
    let mut now = from;
    for _ in 0..(secs * 60.0).ceil() as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        a.run_frame();
    }
    now
}

/// Tick the insert through to the game **without** running a core frame: the state the launch
/// landed on is the state the first Playing frame starts from, and a test that ran frames on
/// the way would be looking at a game that had already moved on.
fn seat_to_playing(a: &mut App, from: Instant) -> Instant {
    let mut now = from;
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            return now;
        }
    }
    panic!("the cart never reached the game: {:?}", a.screen);
}

/// Play for a moment at the seat, then leave with the MENU hold: stopping writes the resume
/// state, which is what the rest of these tests launch from.
fn play_then_leave(a: &mut App, from: Instant) -> Instant {
    let mut now = run(a, from, 0.3);
    a.feed(&ev(Button::Menu, true, now));
    for _ in 0..180 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen != Screen::Playing {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "the hold did not eject");
    let now = run(a, now, EJECT_S + 0.1);
    assert_eq!(a.screen, Screen::List, "the cart never came back out");
    now
}

// ------------------------------------------------------------------ launching

/// The namespace a GBA cart's states live in on this machine: the platform's own core, mGBA.
/// A session's own `state_namespace()` is the product answer; this is the fixture's line when
/// there is no session yet.
fn states_ns() -> slot2_store::StateNamespace {
    slot2_store::StateNamespace::new("mgba_libretro").expect("a core's base name")
}

#[test]
fn a_tap_picks_up_where_the_player_left_off() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart) = card_with_rom("resume");
    let mut a = app_for(card.clone(), cores);
    let t = Instant::now();

    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    let now = play_then_leave(&mut a, now);
    let resume = card.scoped_state_path(&cart, &states_ns(), StateKind::Resume);
    assert!(resume.is_file(), "leaving a game wrote no resume state");
    let saved = fs::read(&resume).unwrap();

    // The next scan sees it, and a tap goes back in with it.
    a.rescan();
    tap(&mut a, Button::A, now);
    let now = seat_to_playing(&mut a, now + ms(60));
    assert!(
        a.session().is_some(),
        "the resume launch started no session"
    );
    assert_eq!(
        a.take_sink_request(),
        Some(SinkRequest::Open),
        "a resume launch opened no sink, or more than one"
    );
    assert_eq!(a.take_sink_request(), None, "a second sink was opened");

    // Nothing has run since the load, so a save now writes exactly what was resumed.
    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"));
    let back = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(1))
        .unwrap();
    assert_eq!(back, saved, "the resume state was not the one loaded");
    assert!(
        resume.is_file(),
        "the resume state was consumed by reading it"
    );
}

#[test]
fn a_tap_with_no_resume_starts_fresh_without_a_word() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart) = card_with_rom("fresh");
    let mut a = app_for(card.clone(), cores);
    let t = Instant::now();

    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    assert!(a.session().is_some());
    assert_eq!(a.toast_key(), None, "a fresh start said something");
    assert!(!card
        .scoped_state_path(&cart, &states_ns(), StateKind::Resume)
        .exists());

    // And the game runs.
    let frames = a.session().unwrap().frames_run();
    run(&mut a, now, 0.2);
    assert!(a.session().unwrap().frames_run() > frames);
}

#[test]
fn a_resume_that_vanishes_before_the_seat_falls_back_fresh() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart) = card_with_rom("vanished");
    card.scoped_write_state(
        &cart,
        &states_ns(),
        StateKind::Resume,
        b"not really a state",
        None,
    )
    .unwrap();
    let mut a = app_for(card.clone(), cores);
    a.rescan();
    let t = Instant::now();

    // The shelf was scanned while it was there; it is gone by the time the cart seats.
    fs::remove_file(card.scoped_state_path(&cart, &states_ns(), StateKind::Resume)).unwrap();
    tap(&mut a, Button::A, t);
    let _ = seat_to_playing(&mut a, t + ms(60));
    assert!(a.session().is_some(), "a missing resume stopped the launch");
    assert_eq!(a.toast_key(), None, "a missing resume is not a failure");
}

#[test]
fn a_resume_that_will_not_load_starts_fresh_and_keeps_the_file() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart) = card_with_rom("corrupt");
    let junk = b"not a state at all, not even close";
    card.scoped_write_state(&cart, &states_ns(), StateKind::Resume, junk, None)
        .unwrap();
    let mut a = app_for(card.clone(), cores);
    a.rescan();
    let t = Instant::now();

    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    assert_eq!(a.toast_key(), Some("resume-load-failed"));
    assert!(a.session().is_some(), "a broken resume cost the game");
    assert_eq!(
        fs::read(card.scoped_state_path(&cart, &states_ns(), StateKind::Resume)).unwrap(),
        junk,
        "the resume this run could not use was removed"
    );
    assert_eq!(a.take_sink_request(), Some(SinkRequest::Open));
    assert_eq!(a.take_sink_request(), None, "a second sink was opened");

    // The fresh session is a session: the game runs.
    let frames = a.session().unwrap().frames_run();
    run(&mut a, now, 0.2);
    assert!(a.session().unwrap().frames_run() > frames);
}

#[test]
fn a_hold_starts_over_and_drops_the_resume_only_once_the_session_is_up() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart) = card_with_rom("hold");
    let mut a = app_for(card.clone(), cores);
    let t = Instant::now();

    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    let now = play_then_leave(&mut a, now);
    let resume = card.scoped_state_path(&cart, &states_ns(), StateKind::Resume);
    let resume_thumb = resume.with_extension("png");
    assert!(
        resume.is_file() && resume_thumb.is_file(),
        "leaving a game wrote no resume state and thumbnail to hold"
    );
    let saved = fs::read(&resume).unwrap();

    // The hold starts the insert, and both files are still there: the session is not up yet.
    a.feed(&ev(Button::A, true, now));
    a.tick(now + ms(700));
    assert_eq!(
        a.screen,
        Screen::Inserting,
        "the hold did not start an insert"
    );
    assert!(
        resume.is_file() && resume_thumb.is_file(),
        "the resume went before the session was up"
    );

    let now = seat_to_playing(&mut a, now + ms(800));
    assert!(
        a.session().is_some(),
        "the hold launch never started a core"
    );
    assert!(
        !resume.exists() && !resume_thumb.exists(),
        "a fresh launch left the old resume state or its thumbnail behind"
    );

    // It really is a new game: the state it starts from is not the one that was saved.
    chord(&mut a, Button::R1, now);
    let started = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(1))
        .unwrap();
    assert_ne!(
        started, saved,
        "a fresh launch began at the state it was meant to abandon"
    );

    // And letting go afterwards does not launch anything else.
    let now = now + ms(200);
    a.feed(&ev(Button::A, false, now));
    a.tick(now + ms(100));
    assert_eq!(
        a.screen,
        Screen::Playing,
        "the release started another insert"
    );
}

#[test]
fn a_resume_path_that_cannot_be_read_says_so() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart) = card_with_rom("unreadable");
    // A folder where the state belongs: the path is there, and no state can be read out of it.
    // The disappeared-file race is silent, but this is not a race — the card holds something
    // the launch was asked to use and cannot.
    let resume = card.scoped_state_path(&cart, &states_ns(), StateKind::Resume);
    fs::create_dir_all(&resume).unwrap();
    let mut a = app_for(card.clone(), cores);
    a.rescan();
    let t = Instant::now();

    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    assert_eq!(a.toast_key(), Some("resume-load-failed"));
    assert!(a.session().is_some(), "an unreadable resume cost the game");
    assert!(resume.is_dir(), "the resume it could not use was removed");

    let frames = a.session().unwrap().frames_run();
    run(&mut a, now, 0.2);
    assert!(a.session().unwrap().frames_run() > frames);
}

#[test]
fn a_launch_that_cannot_start_keeps_the_resume() {
    // No core on this machine's cores path: the insert fails the way it always has.
    let (_card, cart) = card_with_rom("nocore-from");
    let (card, _root) = {
        let root = scratch("nocore");
        let card = Card::new(&root);
        card.ensure_layout();
        let cart = Cart {
            platform: Platform::Gba,
            stem: "arm".into(),
            title: "arm".into(),
            rom: card.games_dir(Platform::Gba).join("arm.gba"),
        };
        fs::write(&cart.rom, b"rom").unwrap();
        (card, root)
    };
    let junk = b"the state from the last time";
    card.scoped_write_state(&cart, &states_ns(), StateKind::Resume, junk, None)
        .unwrap();
    let mut a = app_for(card.clone(), PathBuf::from(".").join("no-cores"));

    let t = Instant::now();
    a.feed(&ev(Button::A, true, t));
    a.tick(t + ms(700));
    assert_eq!(a.screen, Screen::Inserting);
    let mut now = t + ms(800);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Ejecting {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "a failed launch did not eject");
    assert_eq!(
        a.toast_key(),
        Some("core-missing"),
        "the failure said nothing"
    );
    assert_eq!(
        fs::read(card.scoped_state_path(&cart, &states_ns(), StateKind::Resume)).unwrap(),
        junk,
        "a launch that never started took the resume with it"
    );
}
