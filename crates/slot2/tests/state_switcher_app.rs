//! The state switcher inside the running app: the Save State row opens it, A loads what it
//! has selected, and the game frame stays underneath. The entry, the load and the refresh
//! need a real session, so those tests use the established core fixture and skip loudly
//! without it; the screen, drawing and HUD tests run without one.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_platform::Gauge;
use slot2_store::{Card, Platform, StateKind};
use slot2_ui::hud::{GAUGE_W, HUD_H, HUD_MARGIN, WALL};
use slot2_ui::insert::EJECT_S;
use slot2_ui::state_switcher::DIM as SWITCHER_DIM;
use slot2_ui::{InGameMenu, UiCtx};

use slot2::app::{App, Screen};

/// A libretro core is global state and only one instance of a library may live at a time.
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

fn scratch(tag: &str) -> PathBuf {
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "slot2-switcherapp-{tag}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
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

/// SELECT held, then the shoulder: what `Gestures` turns into `Action::Chord`.
fn chord(a: &mut App, shoulder: Button, at: Instant) {
    a.feed(&Event::Button {
        button: Button::Select,
        pressed: true,
        at,
    });
    a.feed(&Event::Button {
        button: shoulder,
        pressed: true,
        at: at + ms(20),
    });
    a.feed(&Event::Button {
        button: shoulder,
        pressed: false,
        at: at + ms(60),
    });
    a.feed(&Event::Button {
        button: Button::Select,
        pressed: false,
        at: at + ms(80),
    });
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

/// A card with the test ROM on it, and an app whose cores directory is the real one.
fn app_with_rom(tag: &str, cores: PathBuf) -> (App, Card, PathBuf) {
    let root = scratch(tag);
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
    (app, Card::new(&root), root)
}

/// Insert the test cart and run until the game is on screen, with the insert's pending
/// requests drained the way the loop drains them every frame.
fn reach_the_game(a: &mut App, from: Instant) -> Instant {
    let mut now = from;
    tap(a, Button::A, now);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            return run(a, now, 0.2);
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

/// The Save State row, then A: the switcher as the player reaches it.
fn open_switcher(a: &mut App, at: Instant) -> Instant {
    let mut now = at;
    tap(a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    now += ms(200);
    tap(a, Button::Down, now);
    now += ms(200);
    tap(a, Button::A, now);
    now += ms(200);
    now
}

fn ui() -> UiCtx {
    UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None)
}

/// Panel 720x480, the panel every one of these tests is written against.
fn frame(a: &mut App, now: Instant) -> RecordingCanvas {
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    ctx.safe = safe;
    a.draw(&mut canvas, &mut ctx, now);
    canvas
}

/// Whether the gauge capsule is on screen: a quad exactly as wide as the gauge and as thin
/// as its wall, inside the band the HUD is pinned to.
fn has_capsule(c: &RecordingCanvas) -> bool {
    c.frame().iter().any(|o| match o {
        Op::Rect { y, w, h, .. } => {
            (*w - GAUGE_W).abs() < 0.01
                && (*h - WALL).abs() < 0.01
                && *y >= HUD_MARGIN - 1.0
                && y + h <= HUD_MARGIN + HUD_H + 1.0
        }
        _ => false,
    })
}

/// Where the game's own frame is drawn: the quad across the whole panel, through the effect the
/// GBA gets by default when its game says nothing about shaders. The texture behind it is the
/// core's own size; the quad is the panel.
fn game_frame_at(ops: &[Op]) -> Option<usize> {
    ops.iter().position(|o| match o {
        Op::ImageEffect {
            x, y, w, h, effect, ..
        } => {
            *x == 0.0
                && *y == 0.0
                && (*w - 720.0).abs() < 0.5
                && (*h - 480.0).abs() < 0.5
                && *effect == slot2_gfx::ShaderEffect::Lcd3x
        }
        _ => false,
    })
}

// ------------------------------------------------------------------ with a core

/// The namespace a GBA cart's states live in on this machine: the platform's own core, mGBA.
/// A session's own `state_namespace()` is the product answer; this is the fixture's line when
/// there is no session yet.
fn states_ns() -> slot2_store::StateNamespace {
    slot2_store::StateNamespace::new("mgba_libretro").expect("a core's base name")
}

#[test]
fn the_save_state_row_opens_the_switcher_and_a_loads_the_greatest_slot() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("open", cores);
    let mut now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    // Slot 2 is the only readable state. Slot 1 is overwritten with rubbish and the resume
    // state with more of it, so both the newest mtime and the newest resume point elsewhere:
    // only the greatest numbered slot loads anything.
    chord(&mut a, Button::R1, now);
    now = run(&mut a, now, 0.3);
    chord(&mut a, Button::R1, now);
    let slot2 = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(2))
        .unwrap();
    card.scoped_write_state(
        &cart,
        &states_ns(),
        StateKind::Numbered(1),
        b"not a state",
        None,
    )
    .unwrap();
    card.scoped_write_state(&cart, &states_ns(), StateKind::Resume, b"not a state", None)
        .unwrap();

    let mut quiet = open_switcher(&mut a, now + ms(200));
    assert!(
        matches!(a.screen, Screen::Switcher(_)),
        "A on Save State did not open the switcher: {:?}",
        a.screen
    );
    assert!(a.session().is_some(), "the switcher dropped the session");
    assert!(
        a.audio_paused(),
        "the game kept its sound under the switcher"
    );
    assert!(
        a.take_sink_request().is_none(),
        "opening the switcher touched the audio"
    );

    // The core is stopped while it is open — the same guard the in-game menu uses.
    let frames = a.session().unwrap().frames_run();
    for _ in 0..30 {
        quiet += Duration::from_micros(16_667);
        a.tick(quiet);
        a.run_frame();
    }
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran on under the switcher"
    );

    // The game frame first, then the switcher's own dim, with nothing clearing between them.
    let mut c = RecordingCanvas::new(720, 480);
    let mut ctx = ui();
    a.draw(&mut c, &mut ctx, quiet);
    let ops = c.frame();
    let game_at =
        game_frame_at(ops).unwrap_or_else(|| panic!("the game's frame was not drawn: {ops:?}"));
    let first_mark = ops
        .iter()
        .position(|o| {
            matches!(
                o,
                Op::Rect { .. } | Op::Image { .. } | Op::ImageEffect { .. }
            )
        })
        .expect("nothing was drawn");
    assert_eq!(
        first_mark, game_at,
        "the first thing drawn was not the game frame: {:?}",
        ops[first_mark]
    );
    let dim_at = ops
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, w, h, color }
            if *x == 0.0 && *y == 0.0 && *w == 720.0 && *h == 480.0 && *color == SWITCHER_DIM)
        })
        .unwrap_or_else(|| panic!("the switcher's dim was not drawn: {ops:?}"));
    assert!(game_at < dim_at, "the switcher went down before the game");
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the switcher cleared the game frame"
    );
    let (bx, by) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "the in-game menu was drawn under the switcher"
    );

    // A loads the slot the switcher was on. Slot 2 is the only readable one, so a success
    // says the greatest numbered slot was the one selected.
    tap(&mut a, Button::A, quiet);
    assert_eq!(a.toast_key(), Some("state-loaded"));
    assert_eq!(
        a.screen,
        Screen::Playing,
        "a load did not hand the game back"
    );
    assert!(a.session().is_some(), "a load dropped the session");
    assert!(
        a.take_sink_request().is_none(),
        "a load touched the audio, which is the sink's business"
    );

    // And it was that state: saving now writes slot 2's bytes back, unchanged.
    chord(&mut a, Button::R1, quiet + ms(200));
    assert_eq!(a.toast_key(), Some("state-saved"));
    let back = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(3))
        .unwrap();
    assert_eq!(back, slot2, "the switcher loaded a different state");
}

#[test]
fn an_empty_card_opens_the_switcher_and_says_there_is_nothing_to_load() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("empty", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();
    assert!(card.scoped_list_states(&cart, &states_ns()).is_empty());

    let mut quiet = open_switcher(&mut a, now + ms(200));
    assert!(
        matches!(a.screen, Screen::Switcher(_)),
        "an empty card did not open the switcher: {:?}",
        a.screen
    );
    assert!(a.audio_paused(), "the empty view kept the game audible");

    tap(&mut a, Button::A, quiet);
    assert_eq!(a.toast_key(), Some("states-empty"));
    assert!(
        matches!(a.screen, Screen::Switcher(_)),
        "the empty view closed itself: {:?}",
        a.screen
    );
    assert!(a.session().is_some(), "the empty view dropped the session");
    assert!(a.take_sink_request().is_none());
    assert!(
        card.scoped_list_states(&cart, &states_ns()).is_empty(),
        "the empty view wrote a state"
    );
    assert!(
        !card
            .scoped_state_path(&cart, &states_ns(), StateKind::Resume)
            .is_file(),
        "the empty view wrote the resume state"
    );

    // Still paused, and still playable once it is closed.
    let frames = a.session().unwrap().frames_run();
    quiet = run(&mut a, quiet, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran on under the empty view"
    );
    tap(&mut a, Button::B, quiet); // back to the in-game menu
    let mut back = quiet + ms(200);
    tap(&mut a, Button::Up, back);
    back += ms(200);
    tap(&mut a, Button::A, back); // Continue
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    let frames = a.session().unwrap().frames_run();
    run(&mut a, back, 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game never started again"
    );
}

#[test]
fn invalid_state_bytes_stay_in_the_switcher_and_leave_the_game_usable() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("invalid", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();
    card.scoped_write_state(
        &cart,
        &states_ns(),
        StateKind::Numbered(1),
        b"not a state",
        None,
    )
    .unwrap();

    let quiet = open_switcher(&mut a, now + ms(200));
    assert!(matches!(a.screen, Screen::Switcher(_)), "{:?}", a.screen);
    tap(&mut a, Button::A, quiet);
    assert_eq!(a.toast_key(), Some("state-load-failed"));
    assert!(
        matches!(a.screen, Screen::Switcher(_)),
        "a failed load left the switcher: {:?}",
        a.screen
    );
    assert!(a.session().is_some(), "a failed load dropped the session");
    assert!(
        a.take_sink_request().is_none(),
        "a failed load touched the audio"
    );
    assert_eq!(a.exit(), None);

    // The session is still a session: the game runs on once the switcher is closed.
    let mut back = quiet + ms(200);
    tap(&mut a, Button::B, back); // the in-game menu
    back += ms(200);
    tap(&mut a, Button::Up, back);
    back += ms(200);
    tap(&mut a, Button::A, back); // Continue
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    let frames = a.session().unwrap().frames_run();
    run(&mut a, back, 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game stopped after a state it could not read"
    );
}

#[test]
fn a_later_save_is_what_the_next_visit_offers() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("refresh", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    // First visit: one slot on the card, and out again.
    chord(&mut a, Button::R1, now);
    let mut back = open_switcher(&mut a, now + ms(200));
    assert!(matches!(a.screen, Screen::Switcher(_)));
    tap(&mut a, Button::B, back);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    back += ms(200);
    tap(&mut a, Button::B, back);
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    back = run(&mut a, back, 0.3);

    // A later save, then the second visit. Slot 1 is rubbish now, so a load that works is a
    // load of slot 2 — the one saved after the first visit put the switcher together.
    chord(&mut a, Button::R1, back);
    assert_eq!(a.toast_key(), Some("state-saved"));
    card.scoped_write_state(
        &cart,
        &states_ns(),
        StateKind::Numbered(1),
        b"not a state",
        None,
    )
    .unwrap();

    let quiet = open_switcher(&mut a, back + ms(200));
    assert!(matches!(a.screen, Screen::Switcher(_)), "{:?}", a.screen);
    tap(&mut a, Button::A, quiet);
    assert_eq!(
        a.toast_key(),
        Some("state-loaded"),
        "the second visit did not offer the newer state"
    );
    assert_eq!(a.screen, Screen::Playing);
    assert!(a.session().is_some());
}

// ------------------------------------------------------------------ deleting and undoing

/// Save one numbered state from the running game and hand back the moment it landed.
fn quick_save(a: &mut App, at: Instant) -> Instant {
    chord(a, Button::R1, at);
    assert_eq!(a.toast_key(), Some("state-saved"));
    at + ms(200)
}

#[test]
fn x_deletes_the_selected_state_and_y_puts_the_last_one_back() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("delete", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    let now = quick_save(&mut a, now);
    let now = run(&mut a, now, 0.3);
    let now = quick_save(&mut a, now);
    let one = card.scoped_state_path(&cart, &states_ns(), StateKind::Numbered(1));
    let one_state = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(1))
        .unwrap();
    let one_thumb = std::fs::read(one.with_extension("png")).expect("a thumbnail");
    let two = card.scoped_state_path(&cart, &states_ns(), StateKind::Numbered(2));
    assert!(two.exists() && two.with_extension("png").exists());

    let at = open_switcher(&mut a, now);
    // The switcher opens on the greatest, so the first X takes slot 2.
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-deleted"));
    assert!(matches!(a.screen, Screen::Switcher(_)), "{:?}", a.screen);
    assert!(a.session().is_some(), "a delete dropped the session");
    assert!(a.audio_paused(), "the game came back audible");
    assert!(
        a.take_sink_request().is_none(),
        "a delete touched the audio"
    );
    assert_eq!(a.exit(), None);
    assert!(!two.exists(), "the state stayed on the card");
    assert!(
        !two.with_extension("png").exists(),
        "the picture stayed too"
    );

    // The selection moved to slot 1, so a second X takes that as well and leaves nothing.
    let at = at + ms(200);
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-deleted"));
    assert!(
        !one.exists(),
        "the selection did not move to the state below"
    );
    let at = at + ms(200);
    tap(&mut a, Button::A, at);
    assert_eq!(
        a.toast_key(),
        Some("states-empty"),
        "the empty view was not empty"
    );
    assert!(matches!(a.screen, Screen::Switcher(_)), "{:?}", a.screen);

    // Y puts back the last deletion and only that one: the first is permanent now.
    let at = at + ms(200);
    tap(&mut a, Button::Y, at);
    assert_eq!(a.toast_key(), Some("state-restored"));
    assert_eq!(
        card.scoped_read_state(&cart, &states_ns(), StateKind::Numbered(1))
            .unwrap(),
        one_state
    );
    assert_eq!(
        std::fs::read(one.with_extension("png")).unwrap(),
        one_thumb,
        "the raw thumbnail bytes did not come back"
    );
    assert!(!two.exists(), "the older deletion came back with it");
    assert_eq!(card.scoped_list_states(&cart, &states_ns()).len(), 1);
    assert!(matches!(a.screen, Screen::Switcher(_)), "{:?}", a.screen);
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());

    // A second Y has nothing left to do.
    let at = at + ms(200);
    tap(&mut a, Button::Y, at);
    assert_eq!(a.toast_key(), Some("undo-empty"));
    assert_eq!(
        card.scoped_read_state(&cart, &states_ns(), StateKind::Numbered(1))
            .unwrap(),
        one_state
    );
}

#[test]
fn a_deleted_middle_state_leaves_the_selection_on_the_number_above_it() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("middle", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    let mut now = quick_save(&mut a, now);
    for _ in 0..2 {
        now = run(&mut a, now, 0.3);
        now = quick_save(&mut a, now);
    }
    let at = open_switcher(&mut a, now);
    let at = at + ms(200);
    tap(&mut a, Button::Left, at); // 3 → 2
    let at = at + ms(200);
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-deleted"));
    assert!(!card
        .scoped_state_path(&cart, &states_ns(), StateKind::Numbered(2))
        .exists());

    // The selection is the number that was above it: slot 1 is rubbish now, so a load that
    // works can only have been slot 3.
    card.scoped_write_state(
        &cart,
        &states_ns(),
        StateKind::Numbered(1),
        b"not a state",
        None,
    )
    .unwrap();
    let at = at + ms(200);
    tap(&mut a, Button::A, at);
    assert_eq!(
        a.toast_key(),
        Some("state-loaded"),
        "the selection did not move up"
    );
    assert_eq!(a.screen, Screen::Playing);

    // The load took the game back, and the deletion is still waiting to be undone. Y puts it
    // back and leaves the selection on it: with 1 and 3 unreadable, a load that works can
    // only be the state that was restored.
    let at = open_switcher(&mut a, at + ms(200));
    tap(&mut a, Button::Y, at);
    assert_eq!(
        a.toast_key(),
        Some("state-restored"),
        "a load cleared the undo"
    );
    assert!(card
        .scoped_state_path(&cart, &states_ns(), StateKind::Numbered(2))
        .exists());
    card.scoped_write_state(
        &cart,
        &states_ns(),
        StateKind::Numbered(3),
        b"not a state",
        None,
    )
    .unwrap();
    let at = at + ms(200);
    tap(&mut a, Button::A, at);
    assert_eq!(
        a.toast_key(),
        Some("state-loaded"),
        "the selection did not land on the restored slot"
    );
    assert_eq!(a.screen, Screen::Playing);
}

#[test]
fn the_undo_outlives_the_menu_but_not_the_session() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("survives", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    let now = quick_save(&mut a, now);
    let at = open_switcher(&mut a, now);
    let at = at + ms(200);
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-deleted"));
    let state = card.scoped_state_path(&cart, &states_ns(), StateKind::Numbered(1));
    assert!(!state.exists());

    // Out through the in-game menu to the game and back in: the session never stopped, so the
    // undo is still there.
    let at = at + ms(200);
    tap(&mut a, Button::B, at);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    let at = at + ms(200);
    tap(&mut a, Button::B, at);
    assert_eq!(a.screen, Screen::Playing);
    let at = open_switcher(&mut a, at + ms(200));
    tap(&mut a, Button::Y, at);
    assert_eq!(
        a.toast_key(),
        Some("state-restored"),
        "the undo did not survive the trip"
    );
    assert!(state.exists());

    // A fresh delete, and then the cart comes out. The undo belongs to the session.
    let at = at + ms(200);
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-deleted"));
    let at = at + ms(200);
    tap(&mut a, Button::B, at);
    let at = at + ms(200);
    tap(&mut a, Button::B, at);
    assert_eq!(a.screen, Screen::Playing);
    a.feed(&Event::Button {
        button: Button::Menu,
        pressed: true,
        at,
    });
    let mut ejecting = at;
    for _ in 0..180 {
        ejecting += Duration::from_micros(16_667);
        a.tick(ejecting);
        if a.screen != Screen::Playing {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "the hold did not eject");
    let list = run(&mut a, ejecting, EJECT_S + 0.1);
    assert_eq!(a.screen, Screen::List);

    // The cart goes back in, and the new session has nothing to undo.
    let now = reach_the_game(&mut a, list);
    let at = open_switcher(&mut a, now);
    tap(&mut a, Button::Y, at);
    assert_eq!(
        a.toast_key(),
        Some("undo-empty"),
        "the undo outlived the session"
    );
    assert!(
        !state.exists(),
        "a stopped session's undo wrote a state back"
    );
}

#[test]
fn a_delete_that_fails_keeps_the_undo_that_was_already_pending() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("fails", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    let now = quick_save(&mut a, now);
    let now = run(&mut a, now, 0.3);
    let now = quick_save(&mut a, now);
    let two_state = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(2))
        .unwrap();

    let at = open_switcher(&mut a, now);
    let at = at + ms(200);
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-deleted"));

    // Slot 1 cannot be removed: a directory where its state file belongs. The delete fails,
    // and the undo for slot 2 has to survive that.
    let one = card.scoped_state_path(&cart, &states_ns(), StateKind::Numbered(1));
    std::fs::remove_file(&one).unwrap();
    std::fs::create_dir(&one).unwrap();
    let at = at + ms(200);
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-delete-failed"));
    assert!(matches!(a.screen, Screen::Switcher(_)), "{:?}", a.screen);
    assert!(a.session().is_some(), "a failed delete dropped the session");
    assert!(a.take_sink_request().is_none());
    assert_eq!(a.exit(), None);

    let at = at + ms(200);
    tap(&mut a, Button::Y, at);
    assert_eq!(
        a.toast_key(),
        Some("state-restored"),
        "the failed delete lost the undo it did not take"
    );
    assert_eq!(
        card.scoped_read_state(&cart, &states_ns(), StateKind::Numbered(2))
            .unwrap(),
        two_state
    );
}

#[test]
fn a_state_that_vanished_leaves_the_list_matching_the_card() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("vanished", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    let now = quick_save(&mut a, now);
    let now = run(&mut a, now, 0.3);
    let now = quick_save(&mut a, now);
    let two = card.scoped_state_path(&cart, &states_ns(), StateKind::Numbered(2));

    let at = open_switcher(&mut a, now);
    // The file goes away behind the app's back: the listing was offering something the card
    // does not have any more.
    std::fs::remove_file(&two).unwrap();
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-delete-failed"));
    assert!(matches!(a.screen, Screen::Switcher(_)), "{:?}", a.screen);
    assert!(a.session().is_some());
    assert_eq!(
        card.scoped_list_states(&cart, &states_ns()).len(),
        1,
        "a failed delete wrote something"
    );

    // And the switcher is in step with the card again: the selection is the slot that is
    // actually there, so A loads it. A stale list would have tried the one that is gone.
    let at = at + ms(200);
    tap(&mut a, Button::A, at);
    assert_eq!(
        a.toast_key(),
        Some("state-loaded"),
        "the list was not brought back in step with the card"
    );
    assert_eq!(a.screen, Screen::Playing);
}

// ------------------------------------------------------------------ without a core

#[test]
fn the_switcher_does_not_wear_the_hud() {
    let root = scratch("hud");
    let card = Card::new(&root);
    card.ensure_layout();
    let mut a = App::with_card(
        card,
        std::env::temp_dir().join("slot2-no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    // A sysfs root with a battery in it, so the gauge has something to draw if it is asked.
    let sysfs = scratch("sysfs");
    let dir = sysfs.join("class/power_supply/bat0");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("type"), "Battery").unwrap();
    fs::write(dir.join("capacity"), "64").unwrap();
    fs::write(dir.join("status"), "Discharging").unwrap();
    a.set_gauge(Gauge::probe(&sysfs));

    let t = Instant::now();
    a.tick(t);
    assert!(
        has_capsule(&frame(&mut a, t)),
        "the shelf lost its gauge, so this test proves nothing about the switcher"
    );

    a.screen = Screen::Switcher(InGameMenu::default());
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    ctx.safe = safe;
    a.draw(&mut canvas, &mut ctx, t);
    let ops = canvas.frame();
    assert!(!has_capsule(&canvas), "the gauge drew over the switcher");
    assert_eq!(
        ops.iter()
            .filter(|o| matches!(o, Op::Rect { x, y, w, h, .. }
                if *x == 0.0 && *y == 0.0 && *w == 720.0 && *h == 480.0))
            .count(),
        1,
        "the switcher's dim was not drawn exactly once"
    );
    let (bx, by) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "the in-game menu was drawn under the switcher"
    );
}
