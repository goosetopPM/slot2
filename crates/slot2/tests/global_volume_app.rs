//! The frontend's own volume level across runs: the level a machine starts on comes from the
//! card's global settings, and the level a press leaves behind goes back there once the presses
//! stop. No core, no sink and no audio: what is measured is the card and `App::volume`.
//!
//! Every test builds its own temporary card, so nothing here depends on the working directory
//! happening to hold a settings file.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_input::{Button, Event};
use slot2_store::{Card, Cart, GameSettings, Platform, ScaleMode};
use slot2_ui::{DeviceMenu, DeviceSetting, InGameMenu, PowerMenu};

use slot2::app::{App, Exit, Screen, VOLUME_SAVE_DELAY_MS};

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

/// Press and release `b`, and say when the release — the moment the app acts on — happened.
///
/// The volume keys have no hold and no double tap, so a `Tap` arrives on release: the delay is
/// counted from here rather than from the press.
fn press(a: &mut App, b: Button, at: Instant) -> Instant {
    a.feed(&ev(b, true, at));
    let released = at + ms(40);
    a.feed(&ev(b, false, released));
    a.tick(released + ms(20));
    released
}

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-globalvol-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card in a fresh temporary folder, holding what a hand-edited settings file would.
///
/// The file is written the way a person would write it — `volume=30`, keys in whatever order —
/// because the file format's own spelling sorts the keys and trims the spaces. Any write at all
/// is then visible in the bytes, including one that stores the value the file already had.
fn make_card(tag: &str, file: Option<&str>) -> Card {
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    if let Some(text) = file {
        fs::write(card.settings_path(), text).unwrap();
    }
    card
}

/// The card as a hand-written level, which is what a saved file looks like before a rewrite.
fn card_at(tag: &str, level: u8) -> Card {
    make_card(tag, Some(&format!("language = ko\nvolume={level}\n")))
}

/// An app over `card`, with no core directory and no game: the volume is the frontend's, so
/// nothing here needs either.
fn app(card: &Card) -> App {
    App::with_card(
        card.clone(),
        PathBuf::from(".").join("no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    )
}

/// The Device menu, over a game that is not there, with the level the app is running at.
fn on_the_device_menu(a: &mut App) {
    let menu = DeviceMenu::new(a.volume.level(), a.volume.is_muted(), None, None);
    a.screen = Screen::Device(InGameMenu::default(), menu);
}

fn shown_level(a: &App) -> Option<u8> {
    match a.screen {
        Screen::Device(_, menu) => menu.value(DeviceSetting::Volume),
        other => panic!("the Device menu is not open: {other:?}"),
    }
}

fn text(card: &Card) -> String {
    fs::read_to_string(card.settings_path()).unwrap()
}

// ------------------------------------------------------------------ startup

#[test]
fn a_stored_level_is_what_the_machine_starts_on() {
    for level in [0u8, 42, 100] {
        let card = card_at(&format!("load-{level}"), level);
        let before = fs::read(card.settings_path()).unwrap();
        let mut a = app(&card);
        assert_eq!(a.volume.level(), level, "the stored level was not taken");
        assert!(!a.volume.is_muted(), "a machine started muted");
        assert_eq!(a.screen, Screen::List);
        // Booting writes nothing: what is on the card is what is running.
        let mut now = Instant::now();
        for _ in 0..5 {
            now += ms(200);
            a.tick(now);
        }
        assert_eq!(
            fs::read(card.settings_path()).unwrap(),
            before,
            "level {level}: booting rewrote the settings file"
        );
    }
}

#[test]
fn a_missing_or_broken_file_starts_at_the_default_and_is_left_alone() {
    // No file at all.
    let card = make_card("missing", None);
    let mut a = app(&card);
    assert_eq!(a.volume.level(), 70, "a missing file is not the default");
    assert!(!a.volume.is_muted());
    let mut now = Instant::now();
    for _ in 0..5 {
        now += ms(300);
        a.tick(now);
    }
    assert!(
        !card.settings_path().exists(),
        "booting created a settings file"
    );

    // Values the store already refuses to believe: the app must not have a second opinion, and
    // must not quietly repair the file either.
    for (tag, file) in [
        ("letters", "volume = abc\n"),
        ("over", "volume = 101\n"),
        ("negative", "volume = -5\n"),
        ("empty", "volume = \n"),
    ] {
        let card = make_card(tag, Some(file));
        let a = app(&card);
        assert_eq!(a.volume.level(), 70, "{tag}: not the default");
        assert!(!a.volume.is_muted(), "{tag}: started muted");
        assert_eq!(text(&card), file, "{tag}: booting rewrote the file");
    }

    // A file that is not text at all. Reading it is the store's job, and it is not a boot
    // failure; writing it is not something the app should try behind the player's back.
    let card = make_card("utf8", None);
    let damaged = b"volume = 30\n\xff\xfe not utf-8\n".to_vec();
    fs::write(card.settings_path(), &damaged).unwrap();
    let a = app(&card);
    assert_eq!(a.volume.level(), 70, "a damaged file is not the default");
    assert_eq!(
        fs::read(card.settings_path()).unwrap(),
        damaged,
        "booting rewrote a file it could not read"
    );
}

// ------------------------------------------------------------------ the delay

#[test]
fn a_physical_press_is_saved_once_the_presses_stop() {
    let card = card_at("physical", 30);
    let mut a = app(&card);
    let t = Instant::now();
    let released = press(&mut a, Button::VolUp, t);
    assert_eq!(a.volume.level(), 35, "the press did not move the level");
    assert_eq!(
        text(&card),
        "language = ko\nvolume=30\n",
        "the press wrote to the card immediately"
    );

    a.tick(released + ms(VOLUME_SAVE_DELAY_MS - 1));
    assert_eq!(
        text(&card),
        "language = ko\nvolume=30\n",
        "the write came before the delay was up"
    );
    a.tick(released + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "language = ko\nvolume = 35\n",
        "the level was not saved"
    );
    assert_eq!(card.read_global_settings().volume, 35);

    // Once it is on the card it is not written again.
    let after = fs::read(card.settings_path()).unwrap();
    a.tick(released + ms(5_000));
    assert_eq!(
        fs::read(card.settings_path()).unwrap(),
        after,
        "it was written twice"
    );
}

#[test]
fn the_device_menu_moves_and_saves_the_same_volume() {
    let card = card_at("device", 30);
    let mut a = app(&card);
    on_the_device_menu(&mut a);
    let t = Instant::now();
    let up = press(&mut a, Button::Right, t);
    assert_eq!(a.volume.level(), 35, "Right did not step the volume up");
    assert_eq!(
        shown_level(&a),
        Some(35),
        "the menu did not follow the level"
    );
    assert_eq!(
        text(&card),
        "language = ko\nvolume=30\n",
        "Right wrote immediately"
    );

    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(text(&card), "language = ko\nvolume = 35\n");

    // And back down through the menu: the level it lands on is saved the same way, after the
    // same delay rather than at once.
    let down = press(&mut a, Button::Left, up + ms(1_000));
    assert_eq!(a.volume.level(), 30);
    assert_eq!(shown_level(&a), Some(30));
    assert_eq!(
        text(&card),
        "language = ko\nvolume = 35\n",
        "Left wrote immediately"
    );
    a.tick(down + ms(VOLUME_SAVE_DELAY_MS - 1));
    assert_eq!(
        text(&card),
        "language = ko\nvolume = 35\n",
        "the write came early"
    );
    a.tick(down + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "language = ko\nvolume = 30\n",
        "the level Left landed on was not saved"
    );
}

#[test]
fn a_second_change_pushes_the_write_back() {
    let card = card_at("reschedule", 30);
    let mut a = app(&card);
    let t = Instant::now();
    let first = press(&mut a, Button::VolUp, t);
    let second = press(&mut a, Button::VolUp, first + ms(400));
    assert_eq!(a.volume.level(), 40);

    // The first delay runs out with nothing written: the change moved the deadline.
    a.tick(first + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "language = ko\nvolume=30\n",
        "the first level was written"
    );
    a.tick(second + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "language = ko\nvolume = 40\n",
        "the level the presses stopped on was not saved"
    );
}

#[test]
fn going_back_to_the_saved_level_cancels_the_write() {
    let card = card_at("cancel", 30);
    let mut a = app(&card);
    let t = Instant::now();
    let up = press(&mut a, Button::VolUp, t);
    let back = press(&mut a, Button::VolDown, up + ms(100));
    assert_eq!(a.volume.level(), 30);

    a.tick(back + ms(VOLUME_SAVE_DELAY_MS * 2));
    assert_eq!(
        text(&card),
        "language = ko\nvolume=30\n",
        "a level that came back to the card was written anyway"
    );
}

// ------------------------------------------------------------------ what is not a change

#[test]
fn a_clamped_press_is_not_storage_activity() {
    // At the top, and at the bottom: the clamp swallows the press and nothing becomes due.
    for (tag, file, button) in [
        ("clamp-top", "volume=100\n", Button::VolUp),
        ("clamp-bottom", "volume=0\n", Button::VolDown),
    ] {
        let card = make_card(tag, Some(file));
        let mut a = app(&card);
        let t = Instant::now();
        press(&mut a, button, t);
        a.tick(t + ms(VOLUME_SAVE_DELAY_MS * 3));
        assert_eq!(
            text(&card),
            file,
            "{tag}: a clamped press wrote to the card"
        );
    }

    // Nor does one push a write that is already pending further out: the level did not move, so
    // the deadline it was given is still the deadline.
    let card = make_card("clamp-pending", Some("volume=70\n"));
    let mut a = app(&card);
    let mut now = Instant::now();
    let mut last = now;
    for _ in 0..6 {
        now += ms(100);
        last = press(&mut a, Button::VolUp, now);
    }
    assert_eq!(
        a.volume.level(),
        100,
        "six steps from 70 did not reach the top"
    );
    let due = last + ms(VOLUME_SAVE_DELAY_MS);
    press(&mut a, Button::VolUp, due - ms(100));
    assert_eq!(
        a.volume.level(),
        100,
        "the clamp let the level past the top"
    );
    a.tick(due);
    assert_eq!(
        text(&card),
        "volume = 100\n",
        "a clamped press pushed the pending write out"
    );
}

#[test]
fn a_mute_toggle_neither_writes_nor_delays_a_pending_one() {
    // Muting on its own is not a level change: nothing to write.
    let card = make_card("mute", Some("volume=30\n"));
    let mut a = app(&card);
    on_the_device_menu(&mut a);
    let t = Instant::now();
    let muted = press(&mut a, Button::A, t);
    assert!(a.volume.is_muted());
    assert_eq!(shown_level(&a), Some(30), "the menu lost the level");
    a.tick(muted + ms(VOLUME_SAVE_DELAY_MS * 3));
    assert_eq!(text(&card), "volume=30\n", "muting wrote to the card");

    // In the middle of a pending write it does not push the write out either.
    let card = make_card("mute-pending", Some("volume=30\n"));
    let mut a = app(&card);
    on_the_device_menu(&mut a);
    let t = Instant::now();
    let up = press(&mut a, Button::Right, t);
    press(&mut a, Button::A, up + ms(400));
    assert!(a.volume.is_muted());
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "volume = 35\n",
        "the mute pushed the pending write out"
    );

    // Muted at the top, stepping up only lifts the mute: the level does not move, so there is
    // still nothing to write.
    let card = make_card("mute-at-the-top", Some("volume=100\n"));
    let mut a = app(&card);
    on_the_device_menu(&mut a);
    let t = Instant::now();
    press(&mut a, Button::A, t);
    assert!(a.volume.is_muted());
    let up = press(&mut a, Button::Right, t + ms(200));
    assert!(!a.volume.is_muted(), "stepping up did not lift the mute");
    assert_eq!(a.volume.level(), 100);
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS * 3));
    assert_eq!(
        text(&card),
        "volume=100\n",
        "lifting the mute wrote a level"
    );
}

#[test]
fn a_muted_volume_saves_the_level_it_remembers_and_no_mute_key() {
    let card = make_card("muted-save", Some("volume=30\nlanguage = ko\n"));
    let mut a = app(&card);
    on_the_device_menu(&mut a);
    let t = Instant::now();
    press(&mut a, Button::A, t);
    let down = press(&mut a, Button::Left, t + ms(400));
    assert!(a.volume.is_muted(), "stepping down unmuted");
    assert_eq!(a.volume.level(), 25);

    a.tick(down + ms(VOLUME_SAVE_DELAY_MS));
    let saved = text(&card);
    assert_eq!(saved, "language = ko\nvolume = 25\n");
    assert!(a.volume.is_muted(), "saving changed the mute");
    assert!(!saved.contains("mute"), "a mute key was written: {saved}");

    // Stepping up lifts the mute and saves the level it lands on; the file still has no mute.
    let up = press(&mut a, Button::Right, down + ms(1_500));
    assert!(!a.volume.is_muted());
    assert_eq!(a.volume.level(), 30);
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));
    let saved = text(&card);
    assert_eq!(saved, "language = ko\nvolume = 30\n");
    for absent in ["mute", "bright", "blue", "utc"] {
        assert!(!saved.contains(absent), "{absent} was written: {saved}");
    }
}

#[test]
fn unknown_keys_survive_the_debounced_write() {
    let card = make_card(
        "unknown",
        Some("future_key = something\nlanguage = ko\nvolume=30\n"),
    );
    let mut a = app(&card);
    let t = Instant::now();
    let up = press(&mut a, Button::VolUp, t);
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "future_key = something\nlanguage = ko\nvolume = 35\n",
        "a key this version does not own was lost"
    );
}

// ------------------------------------------------------------------ failures

#[test]
fn a_failed_write_leaves_the_running_volume_alone_and_is_not_retried() {
    let card = make_card("failed", None);
    let damaged = b"volume = 30\n\xff\xfe not utf-8\n".to_vec();
    fs::write(card.settings_path(), &damaged).unwrap();
    let mut a = app(&card);
    let t = Instant::now();
    let up = press(&mut a, Button::VolUp, t);
    assert_eq!(a.volume.level(), 75, "the press did not move the level");

    let due = up + ms(VOLUME_SAVE_DELAY_MS);
    a.tick(due);
    assert_eq!(a.volume.level(), 75, "a failed save rolled the level back");
    assert!(!a.volume.is_muted(), "a failed save changed the mute");
    assert_eq!(a.screen, Screen::List, "a failed save moved the screen");
    assert_eq!(
        fs::read(card.settings_path()).unwrap(),
        damaged,
        "the file it could not read was rewritten"
    );
    assert!(
        a.take_sink_request().is_none(),
        "the volume write touched the sink"
    );

    // The attempt is spent. Ticking on must not try again every frame.
    for i in 1..10 {
        a.tick(due + ms(100 * i));
    }
    assert_eq!(
        fs::read(card.settings_path()).unwrap(),
        damaged,
        "the failed write was retried on its own"
    );

    // Even with the damage gone, a spent write does not come back by itself.
    let healthy = b"language = ko\nvolume = 30\n".to_vec();
    fs::write(card.settings_path(), &healthy).unwrap();
    for i in 10..20 {
        a.tick(due + ms(100 * i));
    }
    assert_eq!(
        fs::read(card.settings_path()).unwrap(),
        healthy,
        "a spent write came back on a later tick"
    );

    // A new change is a new attempt, and this one lands.
    let again = press(&mut a, Button::VolUp, due + ms(3_000));
    assert_eq!(a.volume.level(), 80);
    a.tick(again + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(text(&card), "language = ko\nvolume = 80\n");
}

#[test]
fn a_directory_in_the_settings_file_s_place_does_not_stop_the_volume() {
    let card = make_card("directory", None);
    let path = card.settings_path();
    fs::create_dir_all(&path).unwrap();
    let mut a = app(&card);
    let t = Instant::now();
    let up = press(&mut a, Button::VolUp, t);
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(a.volume.level(), 75, "a failed save changed the level");
    assert!(!a.volume.is_muted());
    assert!(path.is_dir(), "the directory was removed");

    for i in 1..10 {
        a.tick(up + ms(VOLUME_SAVE_DELAY_MS + 100 * i));
    }
    assert!(path.is_dir(), "the directory came back");
    assert_eq!(
        fs::read_dir(&path).unwrap().count(),
        0,
        "something was written into the directory"
    );
}

// ------------------------------------------------------------------ leaving

#[test]
fn a_power_exit_saves_the_level_at_once() {
    let card = make_card("exit-power", Some("volume=30\n"));
    let mut a = app(&card);
    let t = Instant::now();
    press(&mut a, Button::VolUp, t);
    assert_eq!(text(&card), "volume=30\n", "the press wrote immediately");
    press(&mut a, Button::Power, t + ms(100));
    assert_eq!(a.exit(), Some(Exit::PowerOff), "the power key did not exit");
    assert_eq!(
        text(&card),
        "volume = 35\n",
        "leaving did not save the level the player set"
    );

    // The power menu's two exits, both before the delay is up.
    for (downs, want) in [(1usize, Exit::Reboot), (2, Exit::PowerOff)] {
        let card = make_card(&format!("exit-menu-{downs}"), Some("volume=30\n"));
        let mut a = app(&card);
        a.screen = Screen::Power(PowerMenu::default());
        let t = Instant::now();
        press(&mut a, Button::VolUp, t);
        let mut now = t + ms(100);
        for _ in 0..downs {
            now += ms(100);
            press(&mut a, Button::Down, now);
        }
        now += ms(100);
        press(&mut a, Button::A, now);
        assert_eq!(a.exit(), Some(want), "{downs} downs did not exit");
        assert_eq!(
            text(&card),
            "volume = 35\n",
            "{downs} downs did not save the level"
        );
    }
}

#[test]
fn a_failing_flush_does_not_hold_up_an_exit() {
    let damaged = b"volume = 30\n\xff\xfe not utf-8\n".to_vec();
    let card = make_card("exit-failed", None);
    fs::write(card.settings_path(), &damaged).unwrap();
    let mut a = app(&card);
    let t = Instant::now();
    press(&mut a, Button::VolUp, t);
    press(&mut a, Button::Power, t + ms(100));
    assert_eq!(
        a.exit(),
        Some(Exit::PowerOff),
        "a card that will not answer held the exit up"
    );
    assert_eq!(a.volume.level(), 75, "a failed flush changed the level");
    assert_eq!(fs::read(card.settings_path()).unwrap(), damaged);

    for (downs, want) in [(1usize, Exit::Reboot), (2, Exit::PowerOff)] {
        let card = make_card(&format!("exit-failed-{downs}"), None);
        fs::write(card.settings_path(), &damaged).unwrap();
        let mut a = app(&card);
        a.screen = Screen::Power(PowerMenu::default());
        let t = Instant::now();
        press(&mut a, Button::VolUp, t);
        let mut now = t + ms(100);
        for _ in 0..downs {
            now += ms(100);
            press(&mut a, Button::Down, now);
        }
        press(&mut a, Button::A, now + ms(100));
        assert_eq!(a.exit(), Some(want), "{downs} downs did not exit");
        assert_eq!(fs::read(card.settings_path()).unwrap(), damaged);
    }
}

#[test]
fn closing_a_menu_and_ejecting_are_not_exits() {
    // Out of the in-game menu: a screen change, and the pending write waits for its delay.
    let card = make_card("close", Some("volume=30\n"));
    let mut a = app(&card);
    a.screen = Screen::Playing;
    let t = Instant::now();
    press(&mut a, Button::Menu, t);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    let up = press(&mut a, Button::VolUp, t + ms(200));
    let closed = press(&mut a, Button::B, up + ms(200));
    assert_eq!(a.screen, Screen::Playing, "B did not close the menu");
    assert_eq!(
        text(&card),
        "volume=30\n",
        "closing the menu flushed the level"
    );
    a.tick(closed + ms(200));
    assert_eq!(text(&card), "volume=30\n", "the write came early");
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "volume = 35\n",
        "the pending write did not survive the screen change"
    );

    // The eject is a hold, not an exit: the cart leaves and the level still lands afterwards.
    let card = make_card("eject", Some("volume=30\n"));
    let mut a = app(&card);
    a.screen = Screen::Playing;
    let t = Instant::now();
    let up = press(&mut a, Button::VolUp, t);
    a.feed(&ev(Button::Menu, true, up + ms(100)));
    a.tick(up + ms(700));
    assert!(matches!(a.screen, Screen::Ejecting), "{:?}", a.screen);
    assert_eq!(text(&card), "volume=30\n", "the eject flushed the level");
    a.feed(&ev(Button::Menu, false, up + ms(800)));
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));
    assert_eq!(
        text(&card),
        "volume = 35\n",
        "the eject threw the level away"
    );
}

// ------------------------------------------------------------------ nothing else

#[test]
fn the_global_write_touches_nothing_else() {
    let root = scratch("nothing-else");
    let card = Card::new(&root);
    card.ensure_layout();
    let rom = card.games_dir(Platform::Gba).join("arm.gba");
    fs::write(&rom, b"rom").unwrap();
    let cart = Cart {
        platform: Platform::Gba,
        stem: "arm".into(),
        title: "arm".into(),
        rom,
    };
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Integer),
            ..Default::default()
        },
    )
    .unwrap();
    fs::write(card.settings_path(), "volume=30\n").unwrap();

    let mut a = app(&card);
    let game_path = card.game_settings_path(&cart);
    let game_before = fs::read(&game_path).unwrap();
    let t = Instant::now();
    let up = press(&mut a, Button::VolUp, t);
    a.tick(up + ms(VOLUME_SAVE_DELAY_MS));

    assert_eq!(text(&card), "volume = 35\n");
    assert_eq!(
        fs::read(&game_path).unwrap(),
        game_before,
        "the game's own file was rewritten"
    );
    assert_eq!(card.read_settings(&cart).scale, Some(ScaleMode::Integer));
    assert!(a.session().is_none(), "a volume write made a session");
    assert!(
        a.take_sink_request().is_none(),
        "a volume write touched the sink"
    );
    assert_eq!(a.screen, Screen::List, "a volume write moved the screen");
}
