//! Quick save and quick load inside a running app: SELECT+R1 writes the next numbered
//! state, SELECT+L1 puts the greatest numbered one back. Needs `vendor/mgba_libretro.*`
//! (build/cores.ps1) and the MIT test ROM; without the core the whole file skips loudly.
//!
//! A state is core bytes, so there is nothing to prove here without a core: the fixtures
//! below load the real one and drive the app the way the loop does.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform, StateKind};

use slot2::app::{App, Screen};

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

/// A card with the test ROM on it, and an app whose cores directory is the real one. The
/// card is handed back too: the app keeps its own copy, and a test that wants to look at
/// the state files needs one that points at the same root.
fn app_with_rom(tag: &str, cores: PathBuf) -> (App, Card, PathBuf) {
    let root = std::env::temp_dir().join(format!("slot2-quickstate-{tag}-{}", std::process::id()));
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
    (app, Card::new(&root), root)
}

/// Insert the test cart and run until the game is on screen, then let it play for a fifth of
/// a second: a state is core bytes and a thumbnail, and neither exists before the core has
/// produced a frame. The insert's pending requests are drained on the way out, because the
/// real loop takes them every frame.
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

fn numbered(card: &Card, cart: &slot2_store::Cart) -> Vec<u32> {
    // The product writes its states into the session's own core namespace, so a test that
    // counted the flat directory would be reading a directory the app never touches.
    let mut ns: Vec<u32> = card
        .scoped_list_states(cart, &states_ns())
        .iter()
        .filter_map(|s| match s.kind {
            StateKind::Numbered(n) => Some(n),
            StateKind::Resume => None,
        })
        .collect();
    ns.sort_unstable();
    ns
}

/// The namespace a GBA cart's states live in on this machine: the platform's own core, mGBA.
/// A session's own `state_namespace()` is the product answer; this is the fixture's line when
/// there is no session yet.
fn states_ns() -> slot2_store::StateNamespace {
    slot2_store::StateNamespace::new("mgba_libretro").expect("a core's base name")
}

#[test]
fn select_r1_writes_numbered_states_with_thumbnails_and_a_toast() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("save", cores);
    let mut now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"), "the save said nothing");
    assert_eq!(a.screen, Screen::Playing, "a quick save changed the screen");
    assert!(a.session().is_some(), "a quick save dropped the session");
    assert!(
        a.take_sink_request().is_none(),
        "a quick save touched the audio"
    );
    assert_eq!(numbered(&card, &cart), vec![1]);
    let slot = card.scoped_state_path(&cart, &states_ns(), StateKind::Numbered(1));
    assert!(slot.is_file(), "no state file at {}", slot.display());
    assert!(
        slot.with_extension("png").is_file(),
        "no thumbnail beside the state"
    );
    assert!(card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(1))
        .is_some());

    // The game never stopped: it is a tap away from where it was.
    let frames = a.session().unwrap().frames_run();
    now = run(&mut a, now, 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game stopped running after a save"
    );

    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert_eq!(numbered(&card, &cart), vec![1, 2], "the second save");
    assert!(card
        .scoped_state_path(&cart, &states_ns(), StateKind::Numbered(2))
        .with_extension("png")
        .is_file());
    assert_eq!(a.screen, Screen::Playing);
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());

    // And the message is on screen, over the game picture: the game frame is the wide thing
    // on the panel, drawn through the GBA's default effect, and the only narrow *plain* image
    // on this screen is the line the toast drew.
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    a.draw(&mut canvas, &mut ctx, now);
    let ops = canvas.frame();
    let game = ops
        .iter()
        .position(|o| {
            matches!(o, Op::ImageEffect { w, effect, .. }
                if (*w - 720.0).abs() < 0.5 && *effect == slot2_gfx::ShaderEffect::Lcd3x)
        })
        .expect("the game frame was not drawn");
    assert!(
        ops.iter()
            .skip(game + 1)
            .any(|o| matches!(o, Op::Image { w, .. } if *w < 200.0)),
        "the save said nothing on screen"
    );
}

#[test]
fn select_l1_takes_the_greatest_numbered_slot_and_never_resume() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("pick", cores);
    let mut now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    // Slot 2 is the only real state. Slot 1 and the resume state are decoys written after
    // it, so the greatest slot, the newest mtime and the resume entry all point different
    // ways: only "the greatest numbered slot" loads anything.
    chord(&mut a, Button::R1, now);
    now = run(&mut a, now, 0.3);
    chord(&mut a, Button::R1, now);
    assert_eq!(numbered(&card, &cart), vec![1, 2]);

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

    now = run(&mut a, now, 0.2);
    chord(&mut a, Button::L1, now);
    assert_eq!(
        a.toast_key(),
        Some("state-loaded"),
        "slot 2 was not the one loaded"
    );
    assert_eq!(a.screen, Screen::Playing, "a quick load changed the screen");
    assert!(a.session().is_some(), "a quick load dropped the session");
    assert!(
        a.take_sink_request().is_none(),
        "a quick load touched the audio"
    );
}

#[test]
fn load_puts_the_core_back_where_the_state_was() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("restore", cores);
    let mut now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    // Slot 1 holds the state at B; slot 2 holds the state a third of a second later, at C.
    chord(&mut a, Button::R1, now);
    let at_b = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(1))
        .unwrap();
    now = run(&mut a, now, 0.3);
    chord(&mut a, Button::R1, now);
    let at_c = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(2))
        .unwrap();
    assert_ne!(at_b, at_c, "the game did not move on, so nothing is proved");

    // Back to B. Slot 2 is dropped first so the load has one candidate, and the save after
    // it lands in slot 2 again.
    card.scoped_delete_state(&cart, &states_ns(), StateKind::Numbered(2))
        .unwrap();
    now = run(&mut a, now, 0.2);
    chord(&mut a, Button::L1, now);
    assert_eq!(a.toast_key(), Some("state-loaded"));
    chord(&mut a, Button::R1, now + ms(200));

    let back = card
        .scoped_read_state(&cart, &states_ns(), StateKind::Numbered(2))
        .unwrap();
    assert_ne!(back, at_c, "the load did nothing at all");
    assert_eq!(
        back, at_b,
        "the core was not put back exactly where the state was taken"
    );
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert!(a.session().is_some());
    assert_eq!(a.screen, Screen::Playing);
}

#[test]
fn no_numbered_state_yet_is_a_message_not_a_load() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("empty", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();
    assert!(numbered(&card, &cart).is_empty());

    chord(&mut a, Button::L1, now);
    assert_eq!(a.toast_key(), Some("states-empty"));
    assert_eq!(a.screen, Screen::Playing, "a failed load left the game");
    assert!(a.session().is_some(), "a failed load dropped the session");
    assert!(a.take_sink_request().is_none());
    assert!(
        numbered(&card, &cart).is_empty(),
        "a quick load wrote a state"
    );
    assert!(
        !card
            .scoped_state_path(&cart, &states_ns(), StateKind::Resume)
            .is_file(),
        "a quick load wrote the resume state"
    );

    let frames = a.session().unwrap().frames_run();
    run(&mut a, now, 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game stopped after a load with nothing to load"
    );
}

#[test]
fn failures_are_messages_and_leave_the_game_running() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("fail", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    // A file where the state folder belongs: serializing still works, writing cannot. The
    // platform folder above it has to exist first, or there is nowhere to put the file.
    let states_dir = card.states_dir(&cart);
    fs::create_dir_all(states_dir.parent().unwrap()).unwrap();
    fs::write(&states_dir, b"not a directory").unwrap();
    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-save-failed"));
    assert_eq!(a.screen, Screen::Playing, "a failed save left the game");
    assert!(a.session().is_some(), "a failed save dropped the session");
    assert!(a.take_sink_request().is_none());
    assert_eq!(a.exit(), None);

    // And it is only the save that failed: take the blockage away and the next one works.
    fs::remove_file(&states_dir).unwrap();
    chord(&mut a, Button::R1, now + ms(200));
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert_eq!(numbered(&card, &cart), vec![1]);

    // A state that is not a state: the bytes are there, the core refuses them.
    card.scoped_write_state(
        &cart,
        &states_ns(),
        StateKind::Numbered(1),
        b"not a state",
        None,
    )
    .unwrap();
    chord(&mut a, Button::L1, now + ms(400));
    assert_eq!(a.toast_key(), Some("state-load-failed"));
    assert_eq!(a.screen, Screen::Playing, "a failed load left the game");
    assert!(a.session().is_some(), "a failed load dropped the session");
    assert!(a.take_sink_request().is_none());
    assert_eq!(a.exit(), None);

    // The core is still a core: the game carries on after refusing the state.
    let frames = a.session().unwrap().frames_run();
    run(&mut a, now + ms(400), 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game stopped after a failed load"
    );
}

#[test]
fn the_chords_do_nothing_under_a_menu_over_the_game() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (mut a, card, _root) = app_with_rom("menu", cores);
    let now = reach_the_game(&mut a, Instant::now());
    let cart = a.session().unwrap().cart().clone();

    // The menu is the one screen with a live session that is not `Playing`, so it is the
    // one place where a chord could quietly save or load behind the menu.
    tap(&mut a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    let chosen = match a.screen {
        Screen::InGame(m) => m.choice(),
        _ => unreachable!(),
    };

    chord(&mut a, Button::R1, now + ms(200));
    assert_eq!(a.toast_key(), None, "SELECT+R1 saved under the menu");
    chord(&mut a, Button::L1, now + ms(400));
    assert_eq!(a.toast_key(), None, "SELECT+L1 loaded under the menu");
    assert!(numbered(&card, &cart).is_empty(), "a state was written");
    match a.screen {
        Screen::InGame(m) => assert_eq!(m.choice(), chosen, "the menu moved"),
        other => panic!("the menu closed: {other:?}"),
    }
    assert!(a.session().is_some());
    assert_eq!(a.exit(), None);
}
