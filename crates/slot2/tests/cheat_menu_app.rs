//! The cheats menu inside the running app: the game's own `.cht` list, flipped from the in-game
//! menu and put on the core at once.
//!
//! Needs `vendor/mgba_libretro.*` and the MIT test ROM; without the core the file skips loudly.
//! The two cases that need no session (a screen with no game behind it) run either way.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Cart, Platform};
use slot2_ui::{CheatMenu, InGameMenu};

use slot2::app::{App, Screen};

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

fn ev(b: Button, pressed: bool, at: Instant) -> Event {
    Event::Button {
        button: b,
        pressed,
        at,
    }
}

fn tap(a: &mut App, b: Button, at: Instant) {
    a.feed(&ev(b, true, at));
    a.feed(&ev(b, false, at + ms(40)));
    a.tick(at + ms(60));
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

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-cheats-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// `cheats = n`, every entry complete, every other one on.
fn cheats_text(n: usize) -> String {
    let mut text = format!("cheats = {n}\n");
    for i in 0..n {
        text.push_str(&format!("cheat{i}_desc = \"Cheat {i}\"\n"));
        text.push_str(&format!("cheat{i}_code = \"7E007C{i:02X}\"\n"));
        if i % 2 == 1 {
            text.push_str(&format!("cheat{i}_enable = true\n"));
        }
    }
    text
}

/// A card with the test ROM on it — and a `.cht` beside it, written before the launch because
/// that is when the session reads it — sitting at the game with a frame drawn.
fn playing(tag: &str, cheats: Option<&str>) -> Option<(App, Card, Cart)> {
    let cores = core_dir()?;
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
    if let Some(text) = cheats {
        let path = card.cheat_path(&cart);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
    }

    let mut a = App::with_card(
        card.clone(),
        cores,
        48_000,
        slot2::tuning_for(&slot2_platform::detect().profile),
        false,
        Screen::List,
    );
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let mut now = t + ms(60);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            run(&mut a, now, 0.2);
            let _ = a.take_sink_request();
            return Some((a, card, cart));
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

/// The in-game menu, walked to the Cheats row and opened. Returns the time after the tap.
fn open_cheats(a: &mut App, at: Instant) -> Instant {
    tap(a, Button::Menu, at);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    let mut now = at + ms(200);
    for _ in 0..2 {
        tap(a, Button::Down, now);
        now += ms(200);
    }
    match a.screen {
        Screen::InGame(menu) => assert_eq!(
            menu.choice(),
            slot2_ui::InGameChoice::Cheats,
            "the menu is not on the Cheats row"
        ),
        other => panic!("{other:?}"),
    }
    tap(a, Button::A, now);
    assert!(
        matches!(a.screen, Screen::Cheats(..)),
        "the Cheats row opened something else: {:?}",
        a.screen
    );
    now + ms(200)
}

/// The open cheats menu's own walking state.
fn menu(a: &App) -> CheatMenu {
    match a.screen {
        Screen::Cheats(_, menu) => menu,
        other => panic!("the cheats menu is not open: {other:?}"),
    }
}

/// What the session says is on, in file order.
fn enabled(a: &App) -> Vec<bool> {
    a.session()
        .expect("a session")
        .cheats()
        .iter()
        .map(|c| c.enabled)
        .collect()
}

const TWO: &str = "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"7E007C9A\"\ncheat1_desc = \"b\"\ncheat1_code = \"7E007C9B\"\ncheat1_enable = true\n";

#[test]
fn the_cheats_row_opens_the_session_list() {
    let _serial = serial();
    let text = "cheats = 3\ncheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E007C9A\"\ncheat1_desc = \"Max hearts\"\ncheat1_code = \"7E13F2FF\"\ncheat1_enable = true\ncheat2_desc = \"No damage\"\ncheat2_code = \"7E007C9B+7E007C9C\"\n";
    let Some((mut a, _card, _cart)) = playing("open", Some(text)) else {
        return;
    };
    open_cheats(&mut a, Instant::now());

    assert_eq!(menu(&a).selected_index(), Some(0), "not on the first cheat");
    assert_eq!(menu(&a).first_visible(), 0);
    let list: Vec<&str> = a
        .session()
        .unwrap()
        .cheats()
        .iter()
        .map(|c| c.description.as_str())
        .collect();
    assert_eq!(list, ["Infinite lives", "Max hearts", "No damage"]);
    assert_eq!(enabled(&a), [false, true, false]);
    assert!(a.audio_paused(), "the game kept its sound under the cheats");
    assert!(a.take_sink_request().is_none());
}

#[test]
fn an_empty_list_opens_and_a_does_nothing() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("empty", None) else {
        return;
    };
    let now = open_cheats(&mut a, Instant::now());

    assert_eq!(
        menu(&a).selected_index(),
        None,
        "an empty list has a selection"
    );
    assert!(a.session().unwrap().cheats().is_empty());
    let frames = a.session().unwrap().frames_run();
    tap(&mut a, Button::A, now);
    assert!(
        matches!(a.screen, Screen::Cheats(..)),
        "A closed the empty menu"
    );
    assert_eq!(a.toast_key(), None, "the empty menu said something");
    assert_eq!(a.session().unwrap().frames_run(), frames);
    assert!(a.take_sink_request().is_none());
}

#[test]
fn navigation_wraps_and_scrolls_inside_the_app() {
    let _serial = serial();
    let len = slot2_ui::cheat_menu::MAX_ROWS + 3;
    let text = cheats_text(len);
    let Some((mut a, _card, _cart)) = playing("scroll", Some(&text)) else {
        return;
    };
    let mut now = open_cheats(&mut a, Instant::now());
    assert_eq!(menu(&a).selected_index(), Some(0));
    let before = enabled(&a);

    for step in 1..len {
        tap(&mut a, Button::Down, now);
        now += ms(200);
        let m = menu(&a);
        assert_eq!(m.selected_index(), Some(step));
        let first = m.first_visible();
        assert!(
            first <= step && step < first + slot2_ui::cheat_menu::MAX_ROWS,
            "row {step} is off screen (first {first})"
        );
    }
    tap(&mut a, Button::Down, now);
    assert_eq!(
        menu(&a).selected_index(),
        Some(0),
        "down from the last cheat did not wrap"
    );
    assert_eq!(
        menu(&a).first_visible(),
        0,
        "the window did not come back up"
    );
    now += ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(
        menu(&a).selected_index(),
        Some(len - 1),
        "up from the first cheat did not wrap"
    );
    assert_eq!(
        menu(&a).first_visible(),
        len - slot2_ui::cheat_menu::MAX_ROWS
    );

    // Looking around is not flipping: the session is where it was.
    assert_eq!(enabled(&a), before, "navigation changed a cheat");
}

#[test]
fn toggling_flips_the_session_and_stays_open() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("toggle", Some(TWO)) else {
        return;
    };
    let mut now = open_cheats(&mut a, Instant::now());
    assert_eq!(enabled(&a), [false, true]);

    tap(&mut a, Button::A, now);
    assert_eq!(enabled(&a), [true, true], "A did not turn the cheat on");
    assert!(
        matches!(a.screen, Screen::Cheats(..)),
        "the menu closed on a toggle"
    );
    assert_eq!(menu(&a).selected_index(), Some(0), "the row moved");
    assert_eq!(a.toast_key(), None, "a toggle that worked said something");

    // The second entry, off: the first keeps the state it was given.
    now += ms(200);
    tap(&mut a, Button::Down, now);
    now += ms(200);
    tap(&mut a, Button::A, now);
    assert_eq!(enabled(&a), [true, false]);
    assert_eq!(menu(&a).selected_index(), Some(1));

    // Back into the game the toggles were applied to, and it still runs.
    now += ms(200);
    tap(&mut a, Button::B, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    now += ms(200);
    for _ in 0..2 {
        tap(&mut a, Button::Up, now);
        now += ms(200);
    }
    tap(&mut a, Button::A, now);
    assert_eq!(
        a.screen,
        Screen::Playing,
        "Continue did not go back to the game"
    );
    let frames = a.session().unwrap().frames_run();
    run(&mut a, now + ms(200), 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game did not run after a toggle"
    );
    assert!(a.session().unwrap().last_frame().is_some());
}

#[test]
fn toggling_and_reopening_leave_the_card_alone() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("file", Some(TWO)) else {
        return;
    };
    let path = card.cheat_path(&cart);
    let bytes = fs::read(&path).unwrap();

    let mut now = open_cheats(&mut a, Instant::now());
    tap(&mut a, Button::A, now);
    now += ms(200);
    tap(&mut a, Button::Down, now);
    now += ms(200);
    tap(&mut a, Button::A, now);
    now += ms(200);
    tap(&mut a, Button::B, now);
    now += ms(200);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);

    // Straight back in through the row it handed back: a fresh menu, starting at the first
    // entry, showing what the session says rather than what the file said.
    tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Cheats(..)), "{:?}", a.screen);
    assert_eq!(
        menu(&a).selected_index(),
        Some(0),
        "reopening did not start at the first cheat"
    );
    assert_eq!(
        enabled(&a),
        [true, false],
        "the session's own state was lost"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        bytes,
        "the cheat file was rewritten"
    );
}

#[test]
fn b_and_menu_return_to_the_cheats_row() {
    let _serial = serial();
    for closer in [Button::B, Button::Menu] {
        let Some((mut a, _card, _cart)) = playing("close", Some(TWO)) else {
            return;
        };
        let now = open_cheats(&mut a, Instant::now());
        tap(&mut a, closer, now);
        match a.screen {
            Screen::InGame(menu) => assert_eq!(
                menu.choice(),
                slot2_ui::InGameChoice::Cheats,
                "{closer:?} lost the row"
            ),
            other => panic!("{closer:?} left on {other:?}"),
        }
        assert!(
            a.take_sink_request().is_none(),
            "{closer:?} touched the audio"
        );
        assert_eq!(a.exit(), None);
    }
}

#[test]
fn the_cheats_screen_pauses_the_core() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("pause", Some(TWO)) else {
        return;
    };
    let t = open_cheats(&mut a, Instant::now());
    assert!(a.session().is_some(), "the cheats menu dropped the session");
    assert!(a.audio_paused(), "the game kept its sound");
    assert!(a.take_sink_request().is_none());

    let frames = a.session().unwrap().frames_run();
    let (produced, ..) = a.session().unwrap().audio_health();
    run(&mut a, t, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran under the cheats menu"
    );
    let (after, ..) = a.session().unwrap().audio_health();
    assert_eq!(after, produced, "audio advanced under the cheats menu");
    assert!(a.take_sink_request().is_none());
}

/// Whether this op is the game's own frame: the whole panel, drawn through the effect the GBA
/// gets by default when its game says nothing about shaders.
fn game_frame(o: &Op) -> bool {
    matches!(o, Op::ImageEffect { x, y, w, h, effect, .. }
        if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
            && *effect == slot2_gfx::ShaderEffect::Lcd3x)
}

#[test]
fn drawing_shows_the_game_frame_under_the_cheats_menu() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("draw", Some(TWO)) else {
        return;
    };
    let t = open_cheats(&mut a, Instant::now());

    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, t + ms(100));
    let ops = c.frame();

    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the cheats menu cleared the game frame"
    );
    let game = ops
        .iter()
        .position(game_frame)
        .expect("the game frame was not drawn");
    let dim = ops
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, w, h, color }
            if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
                && *color == slot2_ui::cheat_menu::DIM)
        })
        .expect("the cheats menu was not drawn");
    assert!(game < dim, "the cheats menu went down before the game");
    assert_eq!(
        ops.iter()
            .find(|o| matches!(
                o,
                Op::Rect { .. } | Op::Image { .. } | Op::ImageEffect { .. }
            ))
            .map(game_frame),
        Some(true),
        "the first mark was not the game frame"
    );

    // Not the parent menu, not the switcher, not the corner HUD, not the time badge.
    let (bx, by) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "the parent menu was drawn under the cheats menu"
    );
    let (cx, cy, ..) = slot2_ui::StateSwitcher::card_rect(&ctx, 0);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - cx).abs() < 0.5 && (*y - cy).abs() < 0.5)),
        "a switcher card was drawn under the cheats menu"
    );
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { y, h, .. }
            if (*y - slot2_ui::hud::HUD_MARGIN).abs() < 0.5
                && (*h - slot2_ui::hud::HUD_H).abs() < 0.01)),
        "the top band was drawn over the game"
    );

    // The cheats panel itself is the menu's box, where its own layout says it goes.
    let (bx, by) = slot2_ui::cheat_menu::CheatMenu::box_origin(&ctx);
    assert!(
        ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "no cheats panel at {bx},{by}"
    );
}

#[test]
fn a_menu_past_the_end_of_the_list_says_so_and_changes_nothing() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("mismatch", Some(TWO)) else {
        return;
    };
    let path = card.cheat_path(&cart);
    let bytes = fs::read(&path).unwrap();
    open_cheats(&mut a, Instant::now());

    // The screen and the session are the only two things that know the list; a menu built for a
    // longer one is what them disagreeing looks like, and it is constructible from the outside
    // because `Screen` and `CheatMenu::new` are both public.
    let mut longer = CheatMenu::new(7);
    for _ in 0..5 {
        longer.down();
    }
    assert_eq!(longer.selected_index(), Some(5));
    let Screen::Cheats(parent, _) = a.screen else {
        panic!("the cheats menu is not open: {:?}", a.screen);
    };
    a.screen = Screen::Cheats(parent, longer);

    let before = enabled(&a);
    tap(&mut a, Button::A, Instant::now());
    assert_eq!(
        a.toast_key(),
        Some("cheat-toggle-failed"),
        "a refused toggle said nothing"
    );
    assert_eq!(enabled(&a), before, "the session followed a missing index");
    match a.screen {
        Screen::Cheats(_, menu) => assert_eq!(menu.selected_index(), Some(5), "the menu moved"),
        other => panic!("a refused toggle left the cheats menu: {other:?}"),
    }
    assert_eq!(fs::read(&path).unwrap(), bytes, "the file was rewritten");
    assert!(a.session().is_some());

    // And the message is on top of the menu, not under it.
    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, Instant::now());
    let ops = c.frame();
    let menu_dim = ops
        .iter()
        .position(|o| matches!(o, Op::Rect { color, .. } if *color == slot2_ui::cheat_menu::DIM))
        .expect("the cheats menu was not drawn");
    // The toast's own plate, which is the mark that says it is over everything else. Its alpha
    // fades with the message, so the check is on the plate's colour rather than on one value
    // of it.
    let toast = ops
        .iter()
        .position(|o| match o {
            Op::Rect { color, .. } => color.to_u8()[..3] == [0x1E, 0x21, 0x26],
            _ => false,
        })
        .expect("the toast was not drawn");
    assert!(
        toast > menu_dim,
        "the toast went down under the cheats menu"
    );
}

#[test]
fn a_cheats_screen_without_a_session_does_not_panic() {
    // No core, no session: the row stays a row, and a screen that somehow exists without one
    // still draws and still takes input.
    let root = scratch("nosession");
    let card = Card::new(&root);
    card.ensure_layout();
    fs::write(card.games_dir(Platform::Gba).join("arm.gba"), b"rom").unwrap();
    let mut a = App::with_card(
        card.clone(),
        PathBuf::from(".").join("no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    let mut row = InGameMenu::default();
    for _ in 0..2 {
        row.down();
    }
    assert_eq!(row.choice(), slot2_ui::InGameChoice::Cheats);
    a.screen = Screen::InGame(row);
    tap(&mut a, Button::A, Instant::now());
    assert!(
        matches!(a.screen, Screen::InGame(_)),
        "the Cheats row opened something with no game: {:?}",
        a.screen
    );

    a.screen = Screen::Cheats(InGameMenu::default(), CheatMenu::new(3));
    tap(&mut a, Button::A, Instant::now());
    assert!(
        matches!(a.screen, Screen::Cheats(..)),
        "A left a screen with no session: {:?}",
        a.screen
    );
    assert_eq!(
        a.toast_key(),
        None,
        "a screen with no session said something"
    );

    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, Instant::now());
    assert!(
        c.frame().iter().any(|o| matches!(o, Op::Rect { color, .. }
            if *color == slot2_ui::cheat_menu::DIM)),
        "a session-less cheats screen drew nothing"
    );
}

#[test]
fn a_broken_cheat_file_says_the_cheats_would_not_load() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let root = scratch("broken");
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
    // Declares two cheats and describes one: the file is refused whole.
    let path = card.cheat_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"7E007C9A\"\n",
    )
    .unwrap();

    let mut a = App::with_card(
        card.clone(),
        cores,
        48_000,
        slot2::tuning_for(&slot2_platform::detect().profile),
        false,
        Screen::List,
    );
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let mut now = t + ms(60);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.toast_key().is_some() {
            break;
        }
    }
    assert_eq!(
        a.toast_key(),
        Some("cheat-load-failed"),
        "a broken cheat file was not classified as one"
    );
    assert!(
        a.session().is_none(),
        "a game started with a cheat file that would not load"
    );
    assert!(
        matches!(a.screen, Screen::Ejecting | Screen::List),
        "the refused launch left the app on {:?}",
        a.screen
    );
}
