//! The Device submenu inside the running app: the volume that is playing, changed from the
//! in-game menu.
//!
//! The volume belongs to the frontend rather than to the game, so most of what is here needs
//! no core at all. The half that needs a real game under the overlay — that the core stops
//! while the menu is open, and that the menu is drawn over the game's own last frame — uses
//! `vendor/mgba_libretro.*` (build/cores.ps1) and the MIT test ROM, and skips loudly without
//! them rather than passing quietly.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_audio::volume::STEP;
use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, GameSettings, Platform};
use slot2_ui::device_menu::DIM;
use slot2_ui::hud::{HUD_H, HUD_MARGIN};
use slot2_ui::in_game_menu::{BOX_H as MENU_BOX_H, BOX_W as MENU_BOX_W};
use slot2_ui::splash::BACKDROP;
use slot2_ui::{DeviceMenu, DeviceSetting, InGameChoice, InGameMenu, UiCtx};

use slot2::app::{App, Screen, VOLUME_SAVE_DELAY_MS};

/// A libretro core is global state and only one instance of a library may live at a time, so
/// the tests that load one take turns.
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

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-device-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card with the test ROM on it, and the app sitting at the game with a frame drawn.
fn playing(tag: &str) -> Option<(App, Card)> {
    let cores = core_dir()?;
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    fs::copy(
        repo().join("assets/test/arm.gba"),
        card.games_dir(Platform::Gba).join("arm.gba"),
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
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            run(&mut a, now, 0.2); // a frame for the session to draw
            let _ = a.take_sink_request();
            return Some((a, card));
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

/// An app with a card and no core on it, so nothing behind the Device menu needs a library.
fn without_game(tag: &str) -> App {
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    App::with_card(
        card,
        PathBuf::from(".").join("no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    )
}

fn device_of(a: &App) -> DeviceMenu {
    match a.screen {
        Screen::Device(_, menu) => menu,
        other => panic!("the Device menu is not open: {other:?}"),
    }
}

/// The in-game menu the open Device menu was reached from.
fn parent_of(a: &App) -> InGameMenu {
    match a.screen {
        Screen::Device(parent, _) => parent,
        other => panic!("the Device menu is not open: {other:?}"),
    }
}

/// The level the open Device menu is showing for the volume.
fn level_of(a: &App) -> Option<u8> {
    device_of(a).value(DeviceSetting::Volume)
}

/// Walk the in-game menu to the Device row with real taps, and say where the clock got to.
///
/// The row is found in `INGAME_ITEMS` rather than counted by hand: a menu that gained or lost
/// a row would otherwise park these tests on whichever row happened to be fifth.
fn walk_to_device_row(a: &mut App, from: Instant) -> Instant {
    let index = slot2_ui::in_game_menu::INGAME_ITEMS
        .iter()
        .position(|c| *c == InGameChoice::Device)
        .expect("the in-game menu has no Device row");
    let mut now = from;
    for _ in 0..index {
        tap(a, Button::Down, now);
        now += ms(200);
    }
    match a.screen {
        Screen::InGame(menu) => {
            assert_eq!(
                menu.selected_index(),
                index,
                "the Device row is not selected"
            );
            assert_eq!(menu.choice(), InGameChoice::Device);
        }
        other => panic!("the in-game menu is not open: {other:?}"),
    }
    now
}

/// The Device menu open over a game that is not there. `Screen::InGame` is set directly, as
/// the in-game menu can only be reached from a running game and no core is needed for this.
fn device_without_game(tag: &str) -> (App, Instant) {
    let mut a = without_game(tag);
    a.screen = Screen::InGame(InGameMenu::default());
    let now = walk_to_device_row(&mut a, Instant::now());
    let now = now + ms(200);
    tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Device(..)), "{:?}", a.screen);
    (a, now + ms(200))
}

// ------------------------------------------------------------------ opening

#[test]
fn the_device_row_opens_on_the_running_volume() {
    let _serial = serial();
    let Some((mut a, _card)) = playing("open") else {
        return;
    };
    let t = Instant::now();
    tap(&mut a, Button::Menu, t);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    let now = walk_to_device_row(&mut a, t + ms(200));
    let parent = match a.screen {
        Screen::InGame(menu) => menu,
        other => panic!("{other:?}"),
    };
    tap(&mut a, Button::A, now);

    let Screen::Device(got_parent, menu) = a.screen else {
        panic!("the Device row opened something else: {:?}", a.screen);
    };
    assert_eq!(
        got_parent, parent,
        "the Device row handed back another menu"
    );
    assert_eq!(got_parent.choice(), InGameChoice::Device);
    assert_eq!(
        got_parent.selected_index(),
        parent.selected_index(),
        "the row the menu was opened from was not preserved"
    );

    // The volume is the frontend's own default, minted by the app and not by the card.
    assert_eq!(
        menu.selected(),
        DeviceSetting::Volume,
        "did not open on the volume"
    );
    assert_eq!(menu.value(DeviceSetting::Volume), Some(70));
    assert!(!menu.volume_muted());
    assert!(menu.is_available(DeviceSetting::Volume));
    // Nothing here reports brightness or blue light, so both rows say so rather than showing a
    // zero the machine never had.
    assert_eq!(menu.value(DeviceSetting::Brightness), None);
    assert_eq!(menu.value(DeviceSetting::BlueLight), None);
    assert!(!menu.is_available(DeviceSetting::Brightness));
    assert!(!menu.is_available(DeviceSetting::BlueLight));
    assert_eq!(a.volume.level(), 70, "the menu and the volume disagree");
    assert!(a.session().is_some(), "the Device row dropped the session");
    assert!(
        a.take_sink_request().is_none(),
        "opening the row touched the audio"
    );
}

#[test]
fn the_device_menu_opens_with_no_game_behind_it() {
    // The volume is not the game's, so the row opens on a shelf-side app too — and draws over
    // nothing rather than refusing or panicking.
    let (mut a, now) = device_without_game("no-game");
    assert!(a.session().is_none());
    assert_eq!(level_of(&a), Some(70));
    assert_eq!(device_of(&a).selected(), DeviceSetting::Volume);
    assert_eq!(parent_of(&a).choice(), InGameChoice::Device);

    // Nothing behind it: the dim is the first mark of the frame, not something over a game.
    let mut ctx = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, now);
    let ops = c.frame();
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the menu cleared a frame it does not own"
    );
    match ops
        .iter()
        .find(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
    {
        Some(Op::Rect { x, y, w, h, color }) => {
            assert_eq!((*x, *y, *w, *h), (0.0, 0.0, 720.0, 480.0));
            assert_eq!(*color, DIM, "the overlay did not dim the panel first");
        }
        other => panic!("the overlay went down over something: {other:?}"),
    }
    assert!(
        ops.iter().any(|o| matches!(o, Op::Image { .. })),
        "the overlay drew no text at all"
    );
}

// ------------------------------------------------------------------ navigating and changing

#[test]
fn up_and_down_stay_on_the_volume() {
    // Brightness and blue light are unavailable on this machine, so there is only one row the
    // walk can land on — in either direction, however many times it is asked.
    let (mut a, mut now) = device_without_game("rows");
    for _ in 0..3 {
        tap(&mut a, Button::Down, now);
        now += ms(200);
        assert_eq!(device_of(&a).selected(), DeviceSetting::Volume);
        tap(&mut a, Button::Up, now);
        now += ms(200);
        assert_eq!(device_of(&a).selected(), DeviceSetting::Volume);
    }
    assert_eq!(level_of(&a), Some(70), "walking the rows changed the level");
}

#[test]
fn left_and_right_move_one_step_and_clamp_at_both_ends() {
    let (mut a, mut now) = device_without_game("steps");
    assert_eq!(a.volume.level(), 70);

    tap(&mut a, Button::Right, now);
    now += ms(200);
    assert_eq!(a.volume.level(), 70 + STEP, "right was not one step up");
    assert_eq!(
        level_of(&a),
        Some(70 + STEP),
        "the menu is not showing the volume"
    );
    tap(&mut a, Button::Left, now);
    now += ms(200);
    assert_eq!(a.volume.level(), 70, "left was not one step down");
    assert_eq!(level_of(&a), Some(70));

    // Up to the top and past it: the level clamps, and so does the picture of it.
    for _ in 0..8 {
        tap(&mut a, Button::Right, now);
        now += ms(200);
    }
    assert_eq!(a.volume.level(), 100);
    assert_eq!(level_of(&a), Some(100));
    // Down to the bottom and past it.
    for _ in 0..21 {
        tap(&mut a, Button::Left, now);
        now += ms(200);
    }
    assert_eq!(a.volume.level(), 0);
    assert_eq!(level_of(&a), Some(0));
    assert!(
        a.take_sink_request().is_none(),
        "changing the volume touched the sink"
    );
}

#[test]
fn a_mutes_and_unmutes_without_touching_the_level() {
    let (mut a, mut now) = device_without_game("mute");
    tap(&mut a, Button::A, now);
    now += ms(200);
    assert!(a.volume.is_muted(), "A did not mute");
    assert_eq!(a.volume.level(), 70, "muting moved the level");
    assert!(
        device_of(&a).volume_muted(),
        "the menu does not say it is muted"
    );
    assert_eq!(
        level_of(&a),
        Some(70),
        "the remembered level left the screen"
    );

    tap(&mut a, Button::A, now);
    now += ms(200);
    assert!(!a.volume.is_muted(), "A did not unmute");
    assert_eq!(a.volume.level(), 70);
    assert!(!device_of(&a).volume_muted());
}

#[test]
fn a_muted_volume_steps_down_in_place_and_steps_up_audible() {
    let (mut a, mut now) = device_without_game("muted-steps");
    tap(&mut a, Button::A, now);
    now += ms(200);
    assert!(a.volume.is_muted());

    // Down keeps the mute and moves the level the player will come back to.
    tap(&mut a, Button::Left, now);
    now += ms(200);
    assert!(a.volume.is_muted(), "stepping down unmuted");
    assert_eq!(a.volume.level(), 65);
    assert!(device_of(&a).volume_muted());
    assert_eq!(
        level_of(&a),
        Some(65),
        "the remembered level is not on screen"
    );

    // Up is audible again, and starts from where the mute left the level.
    tap(&mut a, Button::Right, now);
    now += ms(200);
    assert!(!a.volume.is_muted(), "stepping up did not unmute");
    assert_eq!(a.volume.level(), 70);
    assert!(!device_of(&a).volume_muted());
}

// ------------------------------------------------------------------ the physical keys

#[test]
fn the_physical_volume_keys_move_the_menu_with_the_volume() {
    let (mut a, mut now) = device_without_game("physical");
    tap(&mut a, Button::VolUp, now);
    now += ms(200);
    assert_eq!(a.volume.level(), 75);
    assert_eq!(
        level_of(&a),
        Some(75),
        "the menu did not follow the physical key"
    );
    tap(&mut a, Button::VolDown, now);
    now += ms(200);
    assert_eq!(a.volume.level(), 70);
    assert_eq!(level_of(&a), Some(70));

    // A physical key obeys the same mute rules the menu's own steps do.
    tap(&mut a, Button::A, now);
    now += ms(200);
    assert!(a.volume.is_muted());
    tap(&mut a, Button::VolDown, now);
    now += ms(200);
    assert!(a.volume.is_muted(), "the physical key unmuted");
    assert_eq!(a.volume.level(), 65);
    assert!(device_of(&a).volume_muted());
    assert_eq!(level_of(&a), Some(65));
    tap(&mut a, Button::VolUp, now);
    now += ms(200);
    assert!(!a.volume.is_muted(), "the physical key did not unmute");
    assert_eq!(a.volume.level(), 70);
    assert!(!device_of(&a).volume_muted());
}

#[test]
fn the_physical_volume_keys_still_work_away_from_the_device_screen() {
    let mut a = without_game("global-volume");
    a.screen = Screen::List;
    let t = Instant::now();
    tap(&mut a, Button::VolUp, t);
    assert_eq!(a.volume.level(), 75);
    assert_eq!(a.screen, Screen::List, "a volume key moved the screen");
    let now = t + ms(200);
    tap(&mut a, Button::VolDown, now);
    assert_eq!(a.volume.level(), 70);
    // And the global rules are the volume's own, mute included.
    a.volume.set_muted(true);
    let now = now + ms(200);
    tap(&mut a, Button::VolDown, now);
    assert!(a.volume.is_muted(), "the shelf unmuted");
    assert_eq!(a.volume.level(), 65);
}

// ------------------------------------------------------------------ closing and coming back

#[test]
fn closing_returns_to_the_same_device_row() {
    for (i, closer) in [Button::B, Button::Menu].into_iter().enumerate() {
        let (mut a, now) = device_without_game(&format!("close-{i}"));
        let parent = parent_of(&a);
        tap(&mut a, closer, now);
        let Screen::InGame(back) = a.screen else {
            panic!("{closer:?} left the Device menu on {:?}", a.screen);
        };
        assert_eq!(back, parent, "{closer:?} lost the row it was opened from");
        assert_eq!(back.choice(), InGameChoice::Device);
        assert!(a.session().is_none(), "{closer:?} changed the session");
        assert!(
            a.take_sink_request().is_none(),
            "{closer:?} touched the audio"
        );

        // A goes straight back in, on the volume the app still has.
        let now = now + ms(200);
        tap(&mut a, Button::A, now);
        assert!(
            matches!(a.screen, Screen::Device(..)),
            "{closer:?} reopened something else: {:?}",
            a.screen
        );
        assert_eq!(level_of(&a), Some(a.volume.level()));
    }
}

#[test]
fn the_level_and_the_mute_outlive_the_menu() {
    let _serial = serial();
    let Some((mut a, _card)) = playing("survive") else {
        return;
    };
    let t = Instant::now();
    tap(&mut a, Button::Menu, t);
    let mut now = walk_to_device_row(&mut a, t + ms(200));
    tap(&mut a, Button::A, now);
    now += ms(200);
    // Two steps down and a mute: the state a player leaves the menu in.
    tap(&mut a, Button::Left, now);
    now += ms(200);
    tap(&mut a, Button::Left, now);
    now += ms(200);
    tap(&mut a, Button::A, now);
    now += ms(200);
    assert_eq!(a.volume.level(), 60);
    assert!(a.volume.is_muted());

    // Out to the in-game menu, and then out to the game.
    tap(&mut a, Button::B, now);
    now += ms(200);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    tap(&mut a, Button::B, now);
    now += ms(200);
    assert_eq!(a.screen, Screen::Playing);
    assert_eq!(a.volume.level(), 60, "the level did not survive the menu");
    assert!(a.volume.is_muted(), "the mute did not survive the menu");

    // The game is playing again rather than still paused, and it is playing at that volume:
    // what the core is fed comes from the same object the menu changed.
    let frames = a.session().unwrap().frames_run();
    let now = run(&mut a, now, 0.2);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "the game did not come back"
    );

    // Reopened, the menu shows what the app kept rather than where it started.
    let now = now + ms(200);
    tap(&mut a, Button::Menu, now);
    let now = walk_to_device_row(&mut a, now + ms(200));
    tap(&mut a, Button::A, now);
    assert_eq!(level_of(&a), Some(60), "the reopened menu forgot the level");
    assert!(
        device_of(&a).volume_muted(),
        "the reopened menu forgot the mute"
    );
    assert_eq!(device_of(&a).selected(), DeviceSetting::Volume);
}

// ------------------------------------------------------------------ pause, draw and the card

#[test]
fn the_device_screen_pauses_the_game_and_leaves_the_sink_alone() {
    let _serial = serial();
    let Some((mut a, _card)) = playing("pause") else {
        return;
    };
    let t = Instant::now();
    tap(&mut a, Button::Menu, t);
    let mut now = walk_to_device_row(&mut a, t + ms(200));
    tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Device(..)), "{:?}", a.screen);
    assert!(
        a.audio_paused(),
        "the game kept its sound under the Device menu"
    );
    assert!(
        a.take_sink_request().is_none(),
        "opening the menu touched the audio"
    );
    assert!(a.session().is_some(), "the menu dropped the session");

    // A few presses first, so what is measured is the screen rather than the frame it opened on.
    let frames = a.session().unwrap().frames_run();
    assert!(frames > 0, "the session had not run a frame to pause");
    for b in [
        Button::Down,
        Button::Up,
        Button::Right,
        Button::A,
        Button::Left,
    ] {
        now += ms(200);
        tap(&mut a, b, now);
    }
    now = run(&mut a, now, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran under the Device menu"
    );
    assert!(
        a.take_sink_request().is_none(),
        "the Device menu opened or closed the sink"
    );
    let _ = now;
}

/// Whether this op is the game's own frame: the whole panel, drawn through the effect the GBA
/// gets by default when its game says nothing about shaders.
fn game_frame(o: &Op) -> bool {
    matches!(o, Op::ImageEffect { x, y, w, h, effect, .. }
        if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
            && *effect == slot2_gfx::ShaderEffect::Lcd3x)
}

#[test]
fn drawing_shows_the_game_frame_under_the_device_overlay() {
    let _serial = serial();
    let Some((mut a, _card)) = playing("draw") else {
        return;
    };
    let t = Instant::now();
    tap(&mut a, Button::Menu, t);
    let now = walk_to_device_row(&mut a, t + ms(200));
    tap(&mut a, Button::A, now);

    let mut ctx = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, now + ms(100));
    let ops = c.frame();

    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the Device menu cleared the game frame"
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
                && *color == DIM)
        })
        .expect("the Device overlay was not drawn");
    assert!(game < dim, "the overlay went down before the game");
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

    // The panel the overlay is made of, over the frame.
    let (bx, by) = DeviceMenu::box_origin(&ctx);
    assert!(
        ops.iter()
            .any(|o| matches!(o, Op::Rect { x, y, w, h, color }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5
                && *color == BACKDROP)),
        "the Device panel was not drawn"
    );
    // Not the game's own menu underneath, and not the shelf's furniture over it.
    let (mx, my) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, w, h, .. }
            if (*x - mx).abs() < 0.5 && (*y - my).abs() < 0.5
                && (*w - MENU_BOX_W).abs() < 0.5 && (*h - MENU_BOX_H).abs() < 0.5)),
        "the parent in-game menu was drawn under the Device menu"
    );
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { y, h, .. }
            if (*y - HUD_MARGIN).abs() < 0.5 && (*h - HUD_H).abs() < 0.01)),
        "the HUD band was drawn over the game"
    );
}

/// Every file under `root`, with its bytes. Comparing two of these settles "nothing was
/// written" without a timestamp or a mock: the bytes either changed or they did not.
fn tree(root: &std::path::Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                out.insert(path.clone(), fs::read(&path).unwrap());
            }
        }
    }
    out
}

#[test]
fn the_device_menu_writes_only_the_volume_after_the_delay() {
    let _serial = serial();
    let Some((mut a, card)) = playing("volume-write") else {
        return;
    };
    let before = tree(card.root());
    assert!(
        !before.is_empty(),
        "the card was empty; the comparison would prove nothing"
    );
    let t = Instant::now();
    tap(&mut a, Button::Menu, t);
    let mut now = walk_to_device_row(&mut a, t + ms(200));
    tap(&mut a, Button::A, now);

    // Everything the menu can do, including the physical keys, and the way out. The volume keys
    // are the fifth and sixth press and the gap is 150 ms, so the last level change is 300 ms
    // before the last press: every check here is inside the delay, which is what makes "nothing
    // yet" the answer that has to hold.
    for b in [
        Button::Right,
        Button::Right,
        Button::Left,
        Button::A,
        Button::VolUp,
        Button::VolDown,
        Button::Up,
        Button::Down,
    ] {
        now += ms(150);
        tap(&mut a, b, now);
        assert_eq!(tree(card.root()), before, "{b:?} wrote to the card");
    }
    now += ms(150);
    tap(&mut a, Button::B, now);
    assert_eq!(
        tree(card.root()),
        before,
        "closing the menu wrote to the card"
    );
    let level = a.volume.level();
    assert_eq!(
        level, 75,
        "the presses did not leave the level this test is about"
    );

    // Once the presses stop, the level is what is written — one key, in the file the frontend
    // owns, and nothing else on the card moves.
    a.tick(now + ms(VOLUME_SAVE_DELAY_MS));
    let mut after = before;
    after.insert(
        card.settings_path(),
        format!("volume = {level}\n").into_bytes(),
    );
    assert_eq!(
        tree(card.root()),
        after,
        "the delay wrote something other than the volume"
    );
    assert_eq!(card.read_global_settings().volume, level);
    assert_eq!(
        card.read_settings(a.session().unwrap().cart()),
        GameSettings::default(),
        "the Device menu wrote a game setting"
    );
}
