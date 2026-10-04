//! The card's global settings file, `System/slot2.ini`, as the clock's display offset (D-25).
//!
//! The offset is minutes to add to UTC one moment before a time is shown, and nothing else:
//! the system clock, every stored timestamp and every file's mtime stay UTC. It is its own key
//! with its own API rather than a field of `GlobalSettings`, so that a volume flush and a clock
//! change cannot overwrite each other's value, and neither write can lose a key it does not own.

use std::fs;
use std::path::PathBuf;

use slot2_store::{
    Card, Error, GlobalSettings, Ini, DEFAULT_UTC_OFFSET_MINUTES, MAX_VOLUME_LEVEL,
    UTC_OFFSET_MINUTES_KEY, UTC_OFFSET_MINUTES_MAX, UTC_OFFSET_MINUTES_MIN,
};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-store-zone-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// A card with its folders made, which is what every case but the fresh-folder one wants.
fn card(name: &str) -> (Card, PathBuf) {
    let d = tempdir(name);
    let card = Card::new(&d);
    card.ensure_layout();
    (card, d)
}

fn read_ini(card: &Card) -> Ini {
    Ini::load(&card.settings_path()).unwrap()
}

// ------------------------------------------------------------------ reading

#[test]
fn a_missing_file_or_key_is_utc_and_a_read_creates_nothing() {
    let (card, d) = card("missing");
    assert_eq!(DEFAULT_UTC_OFFSET_MINUTES, 0, "the default is not UTC");
    assert_eq!(
        (UTC_OFFSET_MINUTES_MIN, UTC_OFFSET_MINUTES_MAX),
        (-720, 840)
    );
    assert_eq!(card.settings_path(), d.join("System").join("slot2.ini"));

    let system = card.system_dir();
    let before = fs::read_dir(&system).unwrap().count();
    assert_eq!(card.read_utc_offset_minutes(), 0);
    assert!(!card.settings_path().exists(), "a read created the file");
    assert_eq!(
        fs::read_dir(&system).unwrap().count(),
        before,
        "a read changed the System folder"
    );

    // A file that is there and says nothing about the offset is the same answer.
    fs::write(card.settings_path(), "volume = 30\n").unwrap();
    assert_eq!(card.read_utc_offset_minutes(), 0);
    assert_eq!(
        fs::read_to_string(card.settings_path()).unwrap(),
        "volume = 30\n",
        "a read rewrote the file"
    );
}

#[test]
fn whole_minutes_with_a_sign_and_padding_are_read() {
    let (card, _d) = card("parse");
    let path = card.settings_path();
    for (text, want) in [
        ("utc_offset_minutes = 0\n", 0),
        ("utc_offset_minutes = 540\n", 540),
        ("utc_offset_minutes = -480\n", -480),
        // A leading `+` is a sign, not a different number: a person who writes one means 60.
        ("utc_offset_minutes = +60\n", 60),
        ("utc_offset_minutes = -720\n", UTC_OFFSET_MINUTES_MIN),
        ("utc_offset_minutes = 840\n", UTC_OFFSET_MINUTES_MAX),
        // The file format's own padding, like every other value in this file.
        ("utc_offset_minutes=540\n", 540),
        ("utc_offset_minutes =540\n", 540),
        ("  utc_offset_minutes  =  -480  \n", -480),
        ("utc_offset_minutes = \u{a0}540\u{a0}\n", 540),
    ] {
        fs::write(&path, text).unwrap();
        assert_eq!(
            card.read_utc_offset_minutes(),
            want,
            "{text:?} was not read as {want}"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            text,
            "a read rewrote {text:?}"
        );
    }
}

#[test]
fn anything_else_is_utc_and_the_line_is_left_where_it_was() {
    let (card, _d) = card("invalid");
    let path = card.settings_path();
    // Each of these is either not a number or not a number this file may hold, and every one
    // of them means the same thing: no setting here.
    for text in [
        "utc_offset_minutes = \n",
        "utc_offset_minutes\n",
        "utc_offset_minutes = abc\n",
        "utc_offset_minutes = 9.5\n",
        "utc_offset_minutes = 1e3\n",
        "utc_offset_minutes = 1 0\n",
        "utc_offset_minutes = 540 KST\n",
        "utc_offset_minutes = 540KST\n",
        "utc_offset_minutes = -721\n",
        "utc_offset_minutes = 841\n",
        "utc_offset_minutes = 86400\n",
        "utc_offset_minutes = 2147483648\n",
        "utc_offset_minutes = -2147483649\n",
        "utc_offset_minutes = 99999999999999999999\n",
        "utc_offset_minutes = --60\n",
        "utc_offset_minutes = 0x21c\n",
    ] {
        fs::write(&path, text).unwrap();
        assert_eq!(
            card.read_utc_offset_minutes(),
            DEFAULT_UTC_OFFSET_MINUTES,
            "{text:?} was not the default"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            text,
            "a read rewrote {text:?}"
        );
    }
}

// ------------------------------------------------------------------ writing

#[test]
fn offsets_round_trip_through_the_settings_file() {
    let (card, _d) = card("round-trip");
    let path = card.settings_path();

    for minutes in [-720, -480, -1, 0, 1, 60, 540, 839, 840] {
        card.write_utc_offset_minutes(minutes).unwrap();
        assert_eq!(card.read_utc_offset_minutes(), minutes, "{minutes}");
        if minutes == 0 {
            assert!(!path.exists(), "UTC left a file behind");
            continue;
        }
        assert!(path.is_file(), "{minutes} was not stored");
        // One spelling per value: a signed decimal, never a `+` and never padded.
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("{UTC_OFFSET_MINUTES_KEY} = {minutes}\n"),
            "{minutes} was not written canonically"
        );
    }
}

#[test]
fn the_write_path_is_the_global_settings_file_and_its_folder_is_made() {
    // A card that has never had a `System` folder: the first offset change makes it, the same
    // way every other write on this card goes through the atomic path.
    let d = tempdir("fresh");
    let card = Card::new(&d);
    let path = d.join("System").join("slot2.ini");
    assert_eq!(card.settings_path(), path);
    assert!(!path.parent().unwrap().exists());

    card.write_utc_offset_minutes(540).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "utc_offset_minutes = 540\n"
    );
    assert_eq!(card.read_utc_offset_minutes(), 540);
    assert!(
        !path.with_extension("ini.tmp").exists(),
        "a temp file stayed"
    );
}

#[test]
fn an_out_of_range_write_is_refused_before_the_file_is_touched() {
    let (card, _d) = card("out-of-range");
    let path = card.settings_path();
    let existing = b"volume = 30\nfuture_key = keep\n".to_vec();
    fs::write(&path, &existing).unwrap();

    for minutes in [
        UTC_OFFSET_MINUTES_MIN - 1,
        UTC_OFFSET_MINUTES_MAX + 1,
        1440,
        -1440,
        i32::MAX,
        i32::MIN,
    ] {
        match card.write_utc_offset_minutes(minutes) {
            Err(Error::Invalid(_)) => {}
            other => panic!("{minutes} was not refused as invalid: {other:?}"),
        }
        assert_eq!(
            fs::read(&path).unwrap(),
            existing,
            "{minutes} changed the file"
        );
    }
    assert_eq!(card.read_utc_offset_minutes(), DEFAULT_UTC_OFFSET_MINUTES);

    // Neither does a refused write create the file it was going to replace.
    fs::remove_file(&path).unwrap();
    assert!(card.write_utc_offset_minutes(841).is_err());
    assert!(!path.exists(), "a refused write created the file");
}

#[test]
fn every_other_key_survives_adding_changing_and_clearing_the_offset() {
    let (card, _d) = card("other-keys");
    let path = card.settings_path();
    // A real volume, a future version's keys, a hand-written one — and an unreadable volume,
    // which is the player's value and not this write's business either.
    let strangers = "future_key = something\nlanguage = ko\n";

    fs::write(&path, format!("{strangers}volume = 30\n")).unwrap();
    card.write_utc_offset_minutes(60).unwrap();
    assert_eq!(card.read_utc_offset_minutes(), 60);
    let ini = read_ini(&card);
    assert_eq!(ini.get("volume"), Some("30"), "the volume was lost");
    assert_eq!(ini.get("future_key"), Some("something"));
    assert_eq!(ini.get("language"), Some("ko"));
    assert_eq!(card.read_global_settings().volume, 30);

    // Changing it leaves the same keys in place.
    card.write_utc_offset_minutes(-480).unwrap();
    let ini = read_ini(&card);
    assert_eq!(ini.get(UTC_OFFSET_MINUTES_KEY), Some("-480"));
    assert_eq!(ini.get("volume"), Some("30"));
    assert_eq!(ini.get("future_key"), Some("something"));
    assert_eq!(ini.get("language"), Some("ko"));

    // Back to UTC: its own key goes, the others stay, and the file stays with them.
    card.write_utc_offset_minutes(0).unwrap();
    let ini = read_ini(&card);
    assert_eq!(ini.get(UTC_OFFSET_MINUTES_KEY), None);
    assert_eq!(ini.get("volume"), Some("30"));
    assert_eq!(ini.get("future_key"), Some("something"));
    assert_eq!(ini.get("language"), Some("ko"));
    assert!(
        path.is_file(),
        "the file with somebody else's keys was removed"
    );

    // An unreadable volume is left exactly as written, too.
    fs::write(&path, "volume = nope\nfuture_key = keep\n").unwrap();
    card.write_utc_offset_minutes(540).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("volume = nope"), "{text}");
    assert!(text.contains("future_key = keep"), "{text}");
    assert_eq!(card.read_utc_offset_minutes(), 540);
    assert_eq!(
        card.read_global_settings().volume,
        70,
        "the frontend reads an unreadable volume as its default, and still writes nothing over it"
    );
}

#[test]
fn a_volume_write_leaves_the_offset_line_alone() {
    let (card, _d) = card("volume-write");
    let path = card.settings_path();

    for spelling in ["540", "-480", "nope", "", "841"] {
        fs::write(
            &path,
            format!("{UTC_OFFSET_MINUTES_KEY} = {spelling}\nfuture_key = keep\n"),
        )
        .unwrap();
        card.write_global_settings(&GlobalSettings { volume: 42 })
            .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains(&format!("{UTC_OFFSET_MINUTES_KEY} = {spelling}\n")),
            "{spelling:?} was rewritten by a volume save: {text}"
        );
        assert!(text.contains("future_key = keep"), "{text}");
        assert_eq!(card.read_global_settings().volume, 42);

        // And a volume save back to the default keeps the offset line as well.
        card.write_global_settings(&GlobalSettings::default())
            .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains(&format!("{UTC_OFFSET_MINUTES_KEY} = {spelling}\n")),
            "{spelling:?} was lost going back to the default volume: {text}"
        );
    }
    assert_eq!(MAX_VOLUME_LEVEL, 100, "the volume contract moved");
}

#[test]
fn the_default_removes_only_its_own_key_and_then_the_file() {
    let (card, _d) = card("default-removal");
    let path = card.settings_path();

    // A file that held only the offset is UTC again, so it should not exist.
    fs::write(&path, "utc_offset_minutes = 540\n").unwrap();
    card.write_utc_offset_minutes(0).unwrap();
    assert!(!path.exists(), "the empty settings file was left behind");

    // Somebody else's key keeps the file, and is not touched.
    fs::write(&path, "utc_offset_minutes = 540\nlanguage = ko\n").unwrap();
    card.write_utc_offset_minutes(0).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "language = ko\n");

    // And a file that only ever held an unreadable offset goes with the default write too:
    // there was nothing else in it to keep.
    fs::write(&path, "utc_offset_minutes = nope\n").unwrap();
    card.write_utc_offset_minutes(0).unwrap();
    assert!(!path.exists(), "the file kept nothing and stayed");
}

#[test]
fn a_default_write_to_a_card_with_no_file_creates_nothing() {
    let (card, _d) = card("default-noop");
    let path = card.settings_path();
    card.write_utc_offset_minutes(0).unwrap();
    assert!(!path.exists(), "a default write created a file");
    assert_eq!(card.read_utc_offset_minutes(), 0);

    // Through the whole set of default writes: still nothing to show for them.
    for _ in 0..3 {
        card.write_utc_offset_minutes(0).unwrap();
        card.write_global_settings(&GlobalSettings::default())
            .unwrap();
    }
    assert!(!path.exists());
}

// ------------------------------------------------------------------ what cannot be read

#[test]
fn an_unreadable_file_or_a_directory_is_never_written() {
    // A portable pair of failures: bytes that are not UTF-8, and a directory where the file
    // belongs. Neither depends on a permission bit, which does not mean the same thing twice.
    let (card, _d) = card("unreadable");
    let path = card.settings_path();
    let damaged = b"volume = 30\n\xff\xfe not utf-8\n".to_vec();
    fs::write(&path, &damaged).unwrap();

    for minutes in [540, 0] {
        match card.write_utc_offset_minutes(minutes) {
            Err(Error::Io(p, _)) => assert_eq!(p, path, "{minutes} failed elsewhere"),
            other => panic!("{minutes} was not refused: {other:?}"),
        }
        assert_eq!(
            fs::read(&path).unwrap(),
            damaged,
            "{minutes} rewrote a file it could not read"
        );
    }
    assert_eq!(
        card.read_utc_offset_minutes(),
        0,
        "a damaged file is a clock on UTC, not a broken frontend"
    );

    // The volume API has the same answer for the same file, so neither can quietly fix it.
    match card.write_global_settings(&GlobalSettings { volume: 55 }) {
        Err(Error::Io(p, _)) => assert_eq!(p, path),
        other => panic!("the volume write was not refused: {other:?}"),
    }
    assert_eq!(fs::read(&path).unwrap(), damaged);

    fs::remove_file(&path).unwrap();
    fs::create_dir_all(&path).unwrap();
    for minutes in [540, 0] {
        match card.write_utc_offset_minutes(minutes) {
            Err(Error::Io(p, _)) => assert_eq!(p, path, "{minutes} failed elsewhere"),
            other => panic!("{minutes} was not refused: {other:?}"),
        }
        assert!(path.is_dir(), "{minutes} removed the directory");
    }
    assert_eq!(card.read_utc_offset_minutes(), 0);
    assert!(
        fs::read_dir(&path).unwrap().next().is_none(),
        "something was written into the directory"
    );
}

// ------------------------------------------------------------------ what is not consulted

#[test]
fn the_stored_value_is_only_ever_what_the_file_says() {
    // The host-only `SLOT2_UTC_OFFSET_MIN` is the *platform* clock's development default, and
    // nothing here looks at it: the value the card API answers with is the file's, whether the
    // file holds an offset or nothing at all. The variable is deliberately not set here — tests
    // share one process, and a test that moves it would be moving another test's answer.
    let (card, _d) = card("no-environment");
    let path = card.settings_path();
    let env_before = std::env::var_os("SLOT2_UTC_OFFSET_MIN");

    // The empty answer, then a value the file holds, then a value it stops holding: an API that
    // consulted the environment or remembered the last call would answer the same twice.
    assert_eq!(card.read_utc_offset_minutes(), 0);
    card.write_utc_offset_minutes(540).unwrap();
    assert_eq!(card.read_utc_offset_minutes(), 540);
    fs::write(&path, "volume = 30\n").unwrap();
    assert_eq!(card.read_utc_offset_minutes(), 0);
    // Hand-written padding the API never produced, read and left exactly as it is.
    let hand_written = "utc_offset_minutes =  -480 \n";
    fs::write(&path, hand_written).unwrap();
    assert_eq!(card.read_utc_offset_minutes(), -480);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        hand_written,
        "a read normalised the file"
    );

    // Nothing was set, cleared or otherwise moved in the environment, and the only *file* this
    // API makes in `System` is the settings file: no clock, no mtime, no local-time copy of
    // anything. The folders beside it are the card layout's own.
    assert_eq!(std::env::var_os("SLOT2_UTC_OFFSET_MIN"), env_before);
    for entry in fs::read_dir(card.system_dir()).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        assert!(
            entry.file_type().unwrap().is_dir() || name == "slot2.ini",
            "the offset write left {name:?} behind"
        );
    }
}
