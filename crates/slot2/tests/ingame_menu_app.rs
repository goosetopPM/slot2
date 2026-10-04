//! The in-game menu inside a running app, with a real core behind it. Needs
//! `vendor/mgba_libretro.*` (build/cores.ps1) and the MIT test ROM; without the core the
//! whole file skips loudly rather than passing quietly.
//!
//! The unit tests in `app.rs` cover the transitions with the screen set directly. What is
//! here is what only a real session can show: that the core stops advancing while the menu
//! is open, that the menu is drawn over the game's own last frame, and that Eject takes the
//! session and the sink down the path the MENU hold already used.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};
use slot2_ui::in_game_menu::{DIM, INGAME_ITEMS};
use slot2_ui::insert::EJECT_S;

use slot2::app::{App, Screen, SinkRequest};

/// A libretro core is global state and only one instance of a library may live at a time,
/// so these tests take turns.
static SERIAL: Mutex<()> = Mutex::new(());

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

fn tap(a: &mut App, b: Button, at: Instant) {
    a.feed(&Event::Button {
        button: b,
        pressed: true,
        at,
    });
    a.feed(&Event::Button {
        button: b,
        pressed: false,
        at: at + ms(40),
    });
    a.tick(at + ms(60));
}

/// Run the loop at 60 Hz from `from` for `secs`, returning where the clock got to.
fn run(a: &mut App, from: Instant, secs: f32) -> Instant {
    let mut now = from;
    for _ in 0..(secs * 60.0).ceil() as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        a.run_frame();
    }
    now
}

/// A card with the test ROM on it, and an app whose cores directory is the real one.
fn app_with_rom(tag: &str, cores: PathBuf) -> (App, PathBuf) {
    let root = std::env::temp_dir().join(format!("slot2-ingamemenu-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    fs::copy(
        repo().join("assets/test/arm.gba"),
        card.games_dir(Platform::Gba).join("arm.gba"),
    )
    .unwrap();
    let app = App::with_card(
        card,
        cores,
        48_000,
        slot2::tuning_for(&slot2_platform::detect().profile),
        false,
        Screen::List,
    );
    (app, root)
}

/// Insert the test cart and run until the game is on screen.
///
/// The pending requests are drained on the way out: the real loop takes them every frame,
/// and a test that never builds a sink would otherwise read the insert's `Open` as if it
/// were the next thing the app asked for.
fn reach_the_game(a: &mut App, from: Instant) -> Instant {
    let mut now = from;
    tap(a, Button::A, now);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            return now;
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

fn ui() -> slot2_ui::UiCtx {
    slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None)
}

/// Whether this op is the game's own frame: the whole panel, drawn through the effect the GBA
/// gets by default when its game says nothing about shaders. The menu's own images are plain
/// draws and are deliberately not matched here.
fn game_frame(o: &Op) -> bool {
    matches!(o, Op::ImageEffect { x, y, w, h, effect, .. }
        if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
            && *effect == slot2_gfx::ShaderEffect::Lcd3x)
}

#[test]
fn the_menu_opens_over_the_game_and_leaves_it_where_it_stopped() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, _root) = app_with_rom("open", cores);
    let now = reach_the_game(&mut a, Instant::now());
    assert!(a.session().is_some(), "the core was never loaded");

    // The frame guard is what pauses the game, and it is the only thing that does.
    let frames = a.session().unwrap().frames_run();
    let now = run(&mut a, now, 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game was not running before the menu opened"
    );
    let frames = a.session().unwrap().frames_run();

    tap(&mut a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    assert!(a.session().is_some(), "the menu dropped the session");
    assert!(a.audio_paused(), "the game kept its sound under the menu");

    let mut quiet = now;
    for _ in 0..30 {
        quiet += Duration::from_micros(16_667);
        a.tick(quiet);
        a.run_frame();
    }
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran on under the menu"
    );

    // Drawn over the game's own frame, in that order, without clearing it.
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut ctx = ui();
    a.draw(&mut canvas, &mut ctx, quiet);
    let ops = canvas.frame();
    let game = ops
        .iter()
        .position(game_frame)
        .expect("the game's frame was not drawn under the menu");
    let dim = ops
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, w, h, color }
            if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
                && *color == DIM)
        })
        .expect("the menu overlay was not drawn over the game");
    assert!(game < dim, "the menu went down before the game frame");
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the menu cleared the game frame"
    );

    // And B hands the game back where it was.
    let back = quiet + ms(100);
    tap(&mut a, Button::B, back);
    assert_eq!(a.screen, Screen::Playing);
    assert!(
        a.session().is_some(),
        "closing the menu dropped the session"
    );
    assert!(
        a.take_sink_request().is_none(),
        "closing the menu tore the audio down"
    );
    assert!(!a.audio_paused(), "the game came back muted");
    let frames = a.session().unwrap().frames_run();
    run(&mut a, back, 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game never started again after the menu closed"
    );
}

#[test]
fn eject_from_the_menu_takes_the_existing_shutdown_path() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, _root) = app_with_rom("eject", cores);
    let now = reach_the_game(&mut a, Instant::now());

    tap(&mut a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)));

    // Eject is the last row; walk to it rather than trusting the default.
    let mut at = now + ms(100);
    for _ in 0..INGAME_ITEMS.len() - 1 {
        tap(&mut a, Button::Down, at);
        at += ms(100);
    }
    match a.screen {
        Screen::InGame(m) => assert_eq!(m.choice(), slot2_ui::InGameChoice::Eject),
        _ => panic!("the menu closed while walking it: {:?}", a.screen),
    }

    // The tap is fed by hand rather than through the helper: the eject animation fires its
    // own clip on the next tick, and that clip opens a ring of its own. What this checks is
    // the request the eject makes *first*, which is the session's teardown.
    a.feed(&Event::Button {
        button: Button::A,
        pressed: true,
        at,
    });
    a.feed(&Event::Button {
        button: Button::A,
        pressed: false,
        at: at + ms(40),
    });
    assert_eq!(
        a.screen,
        Screen::Ejecting,
        "the eject did not leave the game"
    );
    assert!(a.session().is_none(), "the session outlived the game");
    assert_eq!(
        a.take_sink_request(),
        Some(SinkRequest::Close),
        "the sink was not told to close"
    );
    assert_eq!(
        a.insert_seat(),
        Some(1.0),
        "the cart did not start out from the slot"
    );

    // And the eject keeps the rest of the path the MENU hold uses: the slot's clip, played
    // through a ring of its own.
    a.tick(at + ms(60));
    assert_eq!(a.take_sink_request(), Some(SinkRequest::Open));
    assert!(a.take_consumer().is_some(), "the eject clip had no ring");

    run(&mut a, at, EJECT_S + 0.1);
    assert_eq!(a.screen, Screen::List, "the cart did not come back out");
    assert_eq!(a.insert_seat(), None);
}
