//! The card's global settings file, `System/slot2.ini`: one key this version owns, and
//! everything else in the file left exactly as whoever wrote it left it.
//!
//! The volume level is the only thing here, and it is stored as a decision rather than as a
//! dump of state: the default is the absence of the key, an unreadable level is the default
//! rather than a refusal to boot, and a file this version cannot read is never written over.

use std::fs;
use std::path::PathBuf;

use slot2_store::{
    Card, Cart, Error, GameSettings, GlobalSettings, Ini, Platform, ScaleMode,
    DEFAULT_VOLUME_LEVEL, MAX_VOLUME_LEVEL,
};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-store-global-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn cart(card: &Card, p: Platform, stem: &str) -> Cart {
    let rom = card.games_dir(p).join(format!("{stem}.gba"));
    fs::create_dir_all(rom.parent().unwrap()).unwrap();
    fs::write(&rom, b"rom").unwrap();
    Cart {
        platform: p,
        stem: stem.into(),
        title: stem.into(),
        rom,
    }
}

// ------------------------------------------------------------------ reading

#[test]
fn a_missing_file_is_the_default_and_is_not_created() {
    let d = tempdir("missing");
    let card = Card::new(&d);
    card.ensure_layout();
    assert_eq!(DEFAULT_VOLUME_LEVEL, 70, "the default is the audio default");
    assert_eq!(GlobalSettings::default().volume, 70);

    let system = card.system_dir();
    let before = fs::read_dir(&system).unwrap().count();
    assert_eq!(card.read_global_settings(), GlobalSettings::default());
    assert_eq!(card.read_global_settings().volume, DEFAULT_VOLUME_LEVEL);
    assert!(
        !card.settings_path().exists(),
        "reading created the settings file"
    );
    assert_eq!(
        fs::read_dir(&system).unwrap().count(),
        before,
        "reading changed the System folder"
    );
}

#[test]
fn levels_round_trip_through_the_settings_file() {
    let d = tempdir("round-trip");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();
    assert_eq!(path, d.join("System").join("slot2.ini"));
    assert_eq!(path, card.settings_path(), "the write path moved");

    for level in [0u8, 1, 42, 99, MAX_VOLUME_LEVEL] {
        card.write_global_settings(&GlobalSettings { volume: level })
            .unwrap();
        assert!(path.is_file(), "level {level} was not stored");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("volume = {level}\n"),
            "level {level} was not written as a decimal"
        );
        assert_eq!(card.read_global_settings().volume, level);
    }
    assert_eq!(MAX_VOLUME_LEVEL, 100);

    // A card that has never had a `System` folder gets one, and its first volume change is not
    // a failure: the write goes through the same atomic path as every other card write.
    let fresh = tempdir("round-trip-fresh");
    let fresh_card = Card::new(&fresh);
    assert!(!fresh_card.settings_path().parent().unwrap().exists());
    fresh_card
        .write_global_settings(&GlobalSettings { volume: 33 })
        .unwrap();
    assert_eq!(
        fs::read_to_string(fresh_card.settings_path()).unwrap(),
        "volume = 33\n"
    );
    assert_eq!(fresh_card.read_global_settings().volume, 33);

    // Back to the default, and the file that only held the level goes with it.
    card.write_global_settings(&GlobalSettings::default())
        .unwrap();
    assert!(!path.exists(), "the empty settings file was left behind");
}

#[test]
fn padded_values_are_read() {
    let d = tempdir("padding");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();
    for text in [
        "volume= 85\n",
        "volume =85\n",
        "volume=85\n",
        "  volume = 85  \n",
    ] {
        fs::write(&path, text).unwrap();
        assert_eq!(
            card.read_global_settings().volume,
            85,
            "{text:?} was not read as 85"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            text,
            "a read rewrote the file"
        );
    }
}

#[test]
fn an_unreadable_level_falls_back_to_the_default_without_touching_the_file() {
    let d = tempdir("invalid");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();
    // Empty, not a number, negative, past the top, and past what a level could even be held
    // in. Each is its own line on the card, and each means the same thing: no setting here.
    for text in [
        "volume = \n",
        "volume = abc\n",
        "volume = -5\n",
        "volume = 101\n",
        "volume = 1000\n",
        "volume = 7.5\n",
        "volume = 1 0\n",
    ] {
        fs::write(&path, text).unwrap();
        assert_eq!(
            card.read_global_settings().volume,
            DEFAULT_VOLUME_LEVEL,
            "{text:?} was not the default"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            text,
            "a read rewrote the invalid file"
        );
    }
}

// ------------------------------------------------------------------ writing

#[test]
fn unknown_keys_survive_every_write() {
    let d = tempdir("unknown-keys");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();
    // A future version's timezone and language keys, and one hand-written.
    let strangers = "future_key = something\nutc_offset_minutes = 540\nlanguage = ko\n";

    // Adding the level to a file that never had one.
    fs::write(&path, strangers).unwrap();
    card.write_global_settings(&GlobalSettings { volume: 12 })
        .unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "future_key = something\nlanguage = ko\nutc_offset_minutes = 540\nvolume = 12\n",
        "a stranger's key or spelling was lost or reordered"
    );

    // Changing it.
    fs::write(&path, format!("{strangers}volume = 30\n")).unwrap();
    card.write_global_settings(&GlobalSettings { volume: 55 })
        .unwrap();
    let ini = Ini::load(&path).unwrap();
    assert_eq!(ini.get("volume"), Some("55"));
    assert_eq!(ini.get("future_key"), Some("something"));
    assert_eq!(ini.get("utc_offset_minutes"), Some("540"));
    assert_eq!(ini.get("language"), Some("ko"));

    // And going back to the default: only this version's key goes.
    card.write_global_settings(&GlobalSettings::default())
        .unwrap();
    let ini = Ini::load(&path).unwrap();
    assert_eq!(ini.get(GlobalSettings::KEY_VOLUME), None);
    assert_eq!(ini.get("future_key"), Some("something"));
    assert_eq!(ini.get("utc_offset_minutes"), Some("540"));
    assert_eq!(ini.get("language"), Some("ko"));
    assert!(
        path.is_file(),
        "the file with somebody else's keys was removed"
    );
}

#[test]
fn the_default_removes_only_its_own_key_and_then_the_file() {
    let d = tempdir("default");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();

    // A file that held only the level is the default again, so it should not exist.
    fs::write(&path, "volume = 30\n").unwrap();
    card.write_global_settings(&GlobalSettings::default())
        .unwrap();
    assert!(!path.exists(), "the empty settings file was left behind");

    // Somebody else's key keeps the file, and is not touched.
    fs::write(&path, "volume = 30\nlanguage = ko\n").unwrap();
    card.write_global_settings(&GlobalSettings::default())
        .unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "language = ko\n");

    // A missing file is already the default: the write is a success that does nothing.
    fs::remove_file(&path).unwrap();
    card.write_global_settings(&GlobalSettings::default())
        .unwrap();
    assert!(!path.exists(), "a default write created a file");
}

#[test]
fn an_out_of_range_level_is_refused_and_the_file_is_left_alone() {
    let d = tempdir("out-of-range");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();
    let existing = b"future_key = keep\nvolume = 30\n".to_vec();
    fs::write(&path, &existing).unwrap();

    for level in [MAX_VOLUME_LEVEL + 1, 200, u8::MAX] {
        match card.write_global_settings(&GlobalSettings { volume: level }) {
            Err(Error::Invalid(_)) => {}
            other => panic!("volume {level} was not refused as invalid: {other:?}"),
        }
        assert_eq!(fs::read(&path).unwrap(), existing, "the file was changed");
        assert_eq!(
            card.read_global_settings().volume,
            30,
            "a refused write moved the stored level"
        );
    }

    // Neither does a refused write create the file it was going to replace.
    fs::remove_file(&path).unwrap();
    assert!(card
        .write_global_settings(&GlobalSettings { volume: 101 })
        .is_err());
    assert!(!path.exists(), "a refused write created the file");
}

#[test]
fn an_unreadable_file_is_never_overwritten() {
    let d = tempdir("invalid-utf8");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();
    let damaged = b"volume = 30\n\xff\xfe not utf-8\n".to_vec();
    fs::write(&path, &damaged).unwrap();

    for want in [GlobalSettings { volume: 55 }, GlobalSettings::default()] {
        match card.write_global_settings(&want) {
            Err(Error::Io(p, _)) => assert_eq!(p, path, "{want:?} failed elsewhere"),
            other => panic!("{want:?} was not refused: {other:?}"),
        }
        assert_eq!(
            fs::read(&path).unwrap(),
            damaged,
            "{want:?} rewrote a file it could not read"
        );
    }

    // Reading it is still the default rather than an error: a damaged file is not a broken
    // frontend.
    assert_eq!(card.read_global_settings(), GlobalSettings::default());
}

#[test]
fn a_directory_in_the_settings_file_s_place_is_never_removed() {
    let d = tempdir("directory");
    let card = Card::new(&d);
    card.ensure_layout();
    let path = card.settings_path();
    fs::create_dir_all(&path).unwrap();

    for want in [GlobalSettings { volume: 55 }, GlobalSettings::default()] {
        match card.write_global_settings(&want) {
            Err(Error::Io(p, _)) => assert_eq!(p, path, "{want:?} failed elsewhere"),
            other => panic!("{want:?} was not refused: {other:?}"),
        }
        assert!(path.is_dir(), "{want:?} removed the directory");
    }
    assert_eq!(card.read_global_settings(), GlobalSettings::default());
}

// ------------------------------------------------------------------ what is not owned

#[test]
fn the_file_holds_only_the_key_this_version_owns() {
    let d = tempdir("ownership");
    let card = Card::new(&d);
    card.ensure_layout();
    card.write_global_settings(&GlobalSettings { volume: 40 })
        .unwrap();
    let text = fs::read_to_string(card.settings_path()).unwrap();
    assert_eq!(text, "volume = 40\n");
    // Not the mute the runtime keeps, not the two controls with no backend, not the timezone
    // D-25 leaves to the settings menu.
    for absent in ["mute", "bright", "blue", "utc", "offset", "lang", "time"] {
        assert!(!text.contains(absent), "{absent} was written: {text}");
    }

    // Keys of those shapes somebody else already wrote are read past, not claimed.
    let mut ini = Ini::parse(
        "mute = yes\nmuted = true\nbrightness = 40\nutc_offset_minutes = 540\nlanguage = ko\n",
    );
    assert_eq!(GlobalSettings::from_ini(&ini).volume, DEFAULT_VOLUME_LEVEL);
    GlobalSettings::from_ini(&ini).apply_to(&mut ini);
    assert_eq!(ini.get("volume"), None, "the default claimed a key");
    assert_eq!(ini.get("mute"), Some("yes"));
    assert_eq!(ini.get("muted"), Some("true"));
    assert_eq!(ini.get("brightness"), Some("40"));
    assert_eq!(ini.get("utc_offset_minutes"), Some("540"));
    assert_eq!(ini.get("language"), Some("ko"));
}

#[test]
fn writing_the_global_file_leaves_a_game_s_file_alone() {
    let d = tempdir("two-files");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Integer),
            ..Default::default()
        },
    )
    .unwrap();
    let game_path = card.game_settings_path(&cart);
    let before = fs::read(&game_path).unwrap();

    card.write_global_settings(&GlobalSettings { volume: 25 })
        .unwrap();
    assert_ne!(
        card.settings_path(),
        game_path,
        "the two files are one file"
    );
    assert_eq!(
        fs::read(&game_path).unwrap(),
        before,
        "the game's file was rewritten"
    );
    assert_eq!(card.read_settings(&cart).scale, Some(ScaleMode::Integer));
    assert_eq!(card.read_global_settings().volume, 25);
}
