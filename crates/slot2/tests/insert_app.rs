//! The contract for the insert inside the app. Task 15 makes these pass without editing
//! this file.
//!
//! Everything here drives `App` the way the loop does — feed an event, tick a clock, draw —
//! so what it checks is the frontend a player touches rather than a state machine on paper.
//!
//! Most of it runs with no core on disk, on purpose. That is not a compromise: a card with a
//! game whose core is missing is a real card, it is what a half-built SD looks like, and the
//! frontend has to send the cart back out instead of sitting on the seated frame for ever.
//! The one test that needs a real core says so and skips loudly.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};
use slot2_ui::insert::{EJECT_S, INSERT_S, SEATED_AT};

use slot2::app::{App, Exit, Screen};

/// A libretro core is global state and only one may live at a time, so the test that loads
/// one takes this.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Tests run in parallel, so each one needs a card of its own.
static NEXT_CARD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn card_with(counts: &[(Platform, usize)]) -> (Card, PathBuf) {
    let n = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-insertapp-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for (p, n) in counts {
        for i in 0..*n {
            let ext = p.extensions().first().copied().unwrap_or("bin");
            let rom = card.games_dir(*p).join(format!("Game {i:02}.{ext}"));
            std::fs::write(&rom, b"rom").unwrap();
        }
    }
    (card, root)
}

/// An app whose cores directory is empty, so every load fails.
fn app(counts: &[(Platform, usize)]) -> (App, PathBuf) {
    let (card, root) = card_with(counts);
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

/// Run the loop for `secs` at 60 Hz from `from`, returning where the clock got to.
fn run(a: &mut App, from: Instant, secs: f32) -> Instant {
    let frames = (secs * 60.0).ceil() as u32;
    let mut now = from;
    for _ in 0..frames {
        now += Duration::from_micros(16_667);
        a.tick(now);
    }
    now
}

// ------------------------------------------------------------------ pressing A

#[test]
fn pressing_a_starts_the_insert_not_the_game() {
    // The cart has to go in before the game comes up. Jumping straight to a black screen
    // while the core loads is the cut this whole animation exists to remove.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    assert_eq!(a.screen, Screen::Inserting);
    assert!(a.session().is_none(), "the core was loaded before the cart moved");
    let seat = a.insert_seat().expect("nothing is going into the slot");
    assert!(seat < 0.2, "the cart is already {seat} of the way in");
}

#[test]
fn an_empty_shelf_has_nothing_to_insert() {
    let (mut a, _root) = app(&[(Platform::Gba, 0)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    assert_eq!(a.screen, Screen::List, "an empty shelf started an insert");
    assert_eq!(a.insert_seat(), None);
}

// ------------------------------------------------------------------ the travel

#[test]
fn the_cart_goes_in_on_the_clock() {
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    // Frame by frame up to the seat, which is where the core load happens and the state
    // machine can legitimately leave `Inserting`.
    let mut now = t + Duration::from_millis(60);
    let mut last = a.insert_seat().unwrap_or(0.0);
    let frames = (SEATED_AT * 60.0) as u32 - 2;
    for i in 0..frames {
        now += Duration::from_micros(16_667);
        a.tick(now);
        let seat = a
            .insert_seat()
            .unwrap_or_else(|| panic!("frame {i}: the insert ended early"));
        assert!(seat >= last - 1e-4, "frame {i}: the cart went back up");
        assert!((0.0..=1.0).contains(&seat), "frame {i}: seat {seat}");
        last = seat;
    }
    assert!(
        last > 0.85,
        "the cart is only {last} of the way in when it should be nearly seated"
    );
    assert!(
        a.session().is_none(),
        "the core was loaded before the cart reached the slot"
    );
}

#[test]
fn the_core_is_not_asked_for_until_the_cart_seats() {
    // The load is what stalls a frame, and the dwell after the cart is seated is what it is
    // hidden behind. Loading at the press instead freezes the first frame of the animation,
    // which is the jump cut back again with extra steps.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    let before = run(&mut a, t + Duration::from_millis(60), SEATED_AT - 0.05);
    assert_eq!(
        a.screen,
        Screen::Inserting,
        "the insert ended before the cart reached the slot"
    );
    assert!(a.session().is_none());
    assert!(a.take_sink_request().is_none(), "audio opened early");

    // And by a frame or two past the seat it has been asked for and refused.
    run(&mut a, before, 0.1);
    assert_ne!(
        a.screen,
        Screen::Inserting,
        "the cart is seated and nothing has tried to load the core"
    );
}

#[test]
fn the_animation_runs_at_the_panels_rate() {
    // A session's frame time is its core's, which is not 60 — and there is no core running
    // while a cart is on its way in, whatever the session field says.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let ft = a.frame_time().as_secs_f32();
    assert!(
        (ft - 1.0 / 60.0).abs() < 1e-4,
        "an insert is paced at {ft}s a frame"
    );
}

#[test]
fn a_stalled_frame_does_not_throw_the_cart_through_the_floor() {
    // The same clamp the row needs. A frame that costs a second must cost the animation some
    // lag, not skip the seat — the load is triggered by crossing it.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    a.tick(t + Duration::from_secs(5));
    assert_eq!(
        a.screen,
        Screen::Inserting,
        "one long frame swallowed the whole insert"
    );
    let seat = a.insert_seat().expect("the insert vanished");
    assert!(seat < 0.2, "one long frame took the cart {seat} of the way in");
}

// ------------------------------------------------------------------ refusal

#[test]
fn a_core_that_will_not_load_sends_the_cart_back_out() {
    // A card with a game whose core is missing. Sitting on the seated frame for ever is a
    // dead handset; the cart has to come back out and leave the player where they were.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    let now = run(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.05);
    assert_eq!(a.screen, Screen::Ejecting, "the cart did not come back out");
    assert!(a.session().is_none());

    // And it comes out the way it went in, not by blinking back onto the row.
    let mut now = now;
    let mut last = a.insert_seat().expect("nothing is coming out");
    assert!(last > 0.8, "it started back out from {last}, not from the slot");
    for i in 0..((EJECT_S * 60.0) as u32 - 2) {
        now += Duration::from_micros(16_667);
        a.tick(now);
        let seat = a
            .insert_seat()
            .unwrap_or_else(|| panic!("frame {i}: the eject ended early"));
        assert!(seat <= last + 1e-4, "frame {i}: the cart went back down");
        last = seat;
    }

    let now = run(&mut a, now, 0.1);
    assert_eq!(a.screen, Screen::List, "the eject never reached the shelf");
    assert_eq!(a.insert_seat(), None);
    let _ = now;
}

#[test]
fn a_refused_cart_never_reaches_the_game() {
    // Every frame of it, not only the end: a single frame of `Playing` with no session is a
    // black screen the player sees.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let mut now = t + Duration::from_millis(60);
    for _ in 0..((INSERT_S + EJECT_S + 0.5) * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        assert_ne!(a.screen, Screen::Playing, "reached the game with no core");
    }
    assert_eq!(a.screen, Screen::List);
    assert_eq!(a.selected(), 0, "the shelf lost its place");
}

// ------------------------------------------------------------------ input

#[test]
fn the_row_is_not_touched_while_a_cart_is_going_in() {
    // Scrolling the row out from under a cart that is already on its way into the slot
    // leaves the wrong game loading.
    let (mut a, _root) = app(&[(Platform::Gba, 5), (Platform::Gb, 3)]);
    let t = Instant::now();
    tap(&mut a, Button::Right, t);
    let now = run(&mut a, t, 0.5);
    let chosen = a.selected();
    let platform = a.platform();

    tap(&mut a, Button::A, now);
    let mut now = now + Duration::from_millis(60);
    for b in [Button::Right, Button::Left, Button::R1, Button::L1] {
        tap(&mut a, b, now);
        now += Duration::from_millis(80);
        assert_eq!(a.selected(), chosen, "{b:?} moved the row mid-insert");
        assert_eq!(a.platform(), platform, "{b:?} changed shelf mid-insert");
    }
}

#[test]
fn power_still_works_mid_insert() {
    // Whatever else is going on, the power button turns it off.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = run(&mut a, t + Duration::from_millis(60), 0.2);
    assert_eq!(a.screen, Screen::Inserting);
    tap(&mut a, Button::Power, now);
    assert_eq!(a.exit(), Some(Exit::PowerOff));
}

// ------------------------------------------------------------------ drawing

#[test]
fn the_screen_shows_the_cart_going_in() {
    // The state machine could be perfect and draw the old shelf the whole way.
    let (mut a, _root) = app(&[(Platform::Gba, 5)]);
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    let t = Instant::now();
    tap(&mut a, Button::A, t);

    let lowest = |c: &RecordingCanvas| -> f32 {
        c.frame()
            .iter()
            .filter_map(|o| match o {
                Op::Image { y, h, .. } => Some(y + h),
                _ => None,
            })
            .fold(f32::NEG_INFINITY, f32::max)
    };

    a.draw(&mut canvas, &mut ctx, t);
    let start = lowest(&canvas);
    assert!(start.is_finite(), "the first frame of an insert drew nothing");

    let now = run(&mut a, t + Duration::from_millis(60), SEATED_AT - 0.05);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    a.draw(&mut canvas, &mut ctx, now);
    let end = lowest(&canvas);
    assert!(
        end > start + 20.0,
        "the cart got from {start} to {end}: it is not going anywhere"
    );
}

#[test]
fn drawing_an_insert_does_not_upload_every_frame() {
    let (mut a, _root) = app(&[(Platform::Gba, 6)]);
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    let t = Instant::now();

    a.draw(&mut canvas, &mut ctx, t);
    tap(&mut a, Button::A, t);
    let warm = canvas.ops.len();

    let mut now = t + Duration::from_millis(60);
    for _ in 0..((SEATED_AT - 0.05) * 60.0) as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        a.draw(&mut canvas, &mut ctx, now);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(warm)
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count();
    assert!(uploads <= 2, "an insert uploaded {uploads} textures");
}

// ------------------------------------------------------------------ with a core

/// The one path that needs a real core: a cart that does load, all the way to the game and
/// back out again. Skips loudly rather than silently — a test that quietly passes on a
/// checkout with no `vendor/` is how the device job shipped a card with no cores on it.
#[test]
fn a_cart_that_loads_reaches_the_game_and_ejects_back_to_the_shelf() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());

    let name = if cfg!(windows) {
        "mgba_libretro.dll"
    } else if cfg!(target_os = "macos") {
        "mgba_libretro.dylib"
    } else {
        "mgba_libretro.so"
    };
    let cores = repo().join("vendor");
    if !cores.join(name).is_file() {
        eprintln!(
            "SKIPPED a_cart_that_loads_reaches_the_game_and_ejects_back_to_the_shelf: \
             no core in {} (run build/cores.ps1)",
            cores.display()
        );
        return;
    }

    let (card, _root) = card_with(&[]);
    let rom = card.games_dir(Platform::Gba).join("arm.gba");
    std::fs::copy(repo().join("assets/test/arm.gba"), &rom).unwrap();

    let mut a = App::with_card(
        card,
        cores,
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    let t = Instant::now();
    assert_eq!(a.shelf_len(), 1, "the test ROM is not on the shelf");
    tap(&mut a, Button::A, t);

    // The core is loaded at the seat and the animation still plays its dwell out: the game
    // does not take the screen the instant the core is ready.
    let now = run(&mut a, t + Duration::from_millis(60), SEATED_AT + 0.02);
    assert_eq!(
        a.screen,
        Screen::Inserting,
        "the game took the screen before the insert finished"
    );
    assert!(a.session().is_some(), "the core was never loaded");

    let now = run(&mut a, now, INSERT_S);
    assert_eq!(a.screen, Screen::Playing, "the insert never ended");

    // Out again. The hold fires from a tick, so stop the moment the game lets go rather
    // than running on — a couple more seconds would take the eject with them.
    a.feed(&Event::Button {
        button: Button::Menu,
        pressed: true,
        at: now,
    });
    let mut now = now;
    for _ in 0..180 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen != Screen::Playing {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "leaving a game was a jump cut");
    assert!(a.session().is_none(), "the session outlived the game");

    let now = run(&mut a, now, EJECT_S + 0.1);
    assert_eq!(a.screen, Screen::List);
    assert_eq!(a.insert_seat(), None);
    let _ = now;
}
