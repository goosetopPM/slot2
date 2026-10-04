//! The card's display offset reaches the clock when the app starts.
//!
//! One key of `System/slot2.ini` (`utc_offset_minutes`) is read once in `App::with_card` and
//! applied to the process-global clock the HUD reads. That global is why this file holds
//! exactly one `#[test]`: two tests in one binary would race over it, while every scenario
//! here is cheap enough to run in sequence and hand the offset back as it found it.
//!
//! Nothing here depends on the wall clock, on sleeping, on the environment or on a reset hook,
//! and no texture, core or GL window is involved: what is measured is the card and the clock.

use std::fs;
use std::path::PathBuf;

use slot2_platform::clock::{self, OFFSET_MAX, OFFSET_MIN, SET_AFTER};
use slot2_store::{
    Card, DEFAULT_UTC_OFFSET_MINUTES, DEFAULT_VOLUME_LEVEL, UTC_OFFSET_MINUTES_MAX,
    UTC_OFFSET_MINUTES_MIN,
};

use slot2::app::{App, Screen};

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-tzstartup-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card in a fresh temporary folder, holding the bytes a hand-edited settings file would.
/// `None` writes no file at all, which is a card nobody has ever set.
fn card_with(tag: &str, file: Option<&[u8]>) -> Card {
    let card = Card::new(scratch(tag));
    card.ensure_layout();
    if let Some(bytes) = file {
        fs::write(card.settings_path(), bytes).unwrap();
    }
    card
}

/// An app over `card`, with no core directory and no game: the offset and the volume are the
/// frontend's, so nothing here needs either.
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

#[test]
fn the_card_offset_is_the_clock_the_app_starts_on() {
    let original = clock::utc_offset_min();

    // The production compile-time seal in `app.rs`, said out loud where a person reads the test
    // list: both crates bound the same range around the same default, and today those numbers
    // are these. A difference is a contract error, not a number to nudge.
    assert_eq!(UTC_OFFSET_MINUTES_MIN, -720);
    assert_eq!(UTC_OFFSET_MINUTES_MAX, 840);
    assert_eq!(DEFAULT_UTC_OFFSET_MINUTES, 0);
    assert_eq!(OFFSET_MIN, UTC_OFFSET_MINUTES_MIN);
    assert_eq!(OFFSET_MAX, UTC_OFFSET_MINUTES_MAX);

    // The HUD decision is pure: the same UTC sample shifted by an offset moves by exactly that
    // many minutes, and visibility is judged on the sample before any shift, so no offset can
    // pull a 1970 reading into the corner.
    let sample = SET_AFTER + 3 * 3_600 + 1_234;
    let utc_view = clock::hud_local(sample, 0).expect("a 2020 sample is a set clock");
    assert_eq!(clock::hud_local(sample, 540).unwrap() - utc_view, 540 * 60);
    assert_eq!(
        clock::hud_local(sample, -480).unwrap() - utc_view,
        -480 * 60
    );
    for offset in [OFFSET_MIN, -480, 0, 540, OFFSET_MAX] {
        assert_eq!(
            clock::hud_local(SET_AFTER - 1, offset),
            None,
            "an offset of {offset} moved an unset sample into view"
        );
    }

    // A stored offset is what the app runs on from the moment it is built, and the card
    // outranks whatever the process was holding before — which is what `stale` is for. The
    // volume is the store's existing contract, and the file is read, never rewritten.
    for (tag, saved) in [
        ("east", 540),
        ("west", -480),
        ("low", OFFSET_MIN),
        ("high", OFFSET_MAX),
        ("utc", 0),
    ] {
        clock::set_utc_offset_min(if saved == 0 { 60 } else { 0 }).unwrap();
        let card = card_with(
            tag,
            Some(format!("volume = 30\nutc_offset_minutes = {saved}\n").as_bytes()),
        );
        let before = fs::read(card.settings_path()).unwrap();
        let a = app(&card);
        assert_eq!(
            clock::utc_offset_min(),
            saved,
            "{tag}: the clock does not hold the card's offset"
        );
        assert_eq!(
            a.volume.level(),
            30,
            "{tag}: the card's volume was not loaded"
        );
        assert_eq!(
            fs::read(card.settings_path()).unwrap(),
            before,
            "{tag}: startup rewrote the settings file"
        );
    }

    // The two keys are read independently: an offset on a card that never set a volume leaves
    // the volume at the default rather than at whatever a previous card held.
    let card = card_with("offset-only", Some(b"utc_offset_minutes = -480\n"));
    let a = app(&card);
    assert_eq!(clock::utc_offset_min(), -480);
    assert_eq!(a.volume.level(), DEFAULT_VOLUME_LEVEL);

    // A card that says nothing about time is UTC. That is an answer rather than a missing one:
    // it overwrites the 540 that ran before, so no earlier app's offset outlives it.
    clock::set_utc_offset_min(540).unwrap();
    let card = card_with("no-key", Some(b"volume = 30\n"));
    let before = fs::read(card.settings_path()).unwrap();
    let a = app(&card);
    assert_eq!(clock::utc_offset_min(), 0);
    assert_eq!(a.volume.level(), 30);
    assert_eq!(fs::read(card.settings_path()).unwrap(), before);

    // And one with no settings file at all: the startup read is a read. It creates no file and
    // leaves the settings folder exactly as it found it.
    clock::set_utc_offset_min(-480).unwrap();
    let card = card_with("no-file", None);
    let system = card.settings_path().parent().unwrap().to_path_buf();
    let entries = fs::read_dir(&system).unwrap().count();
    let a = app(&card);
    assert_eq!(clock::utc_offset_min(), 0);
    assert_eq!(a.volume.level(), DEFAULT_VOLUME_LEVEL);
    assert!(
        !card.settings_path().exists(),
        "the startup read created the settings file"
    );
    assert_eq!(fs::read_dir(&system).unwrap().count(), entries);

    // A value the store cannot accept reads as UTC, and the bytes are left exactly as they
    // were: the app is not the place that repairs somebody's card.
    for (tag, bytes) in [
        ("text", b"utc_offset_minutes = KST\n".to_vec()),
        ("fraction", b"utc_offset_minutes = 9.5\n".to_vec()),
        ("past", b"utc_offset_minutes = 900\n".to_vec()),
        ("not-utf8", b"utc_offset_minutes = \xff\xfe\n".to_vec()),
    ] {
        clock::set_utc_offset_min(540).unwrap();
        let card = card_with(tag, Some(&bytes));
        let before = fs::read(card.settings_path()).unwrap();
        let _ = app(&card);
        assert_eq!(clock::utc_offset_min(), 0, "{tag}: not UTC");
        assert_eq!(
            fs::read(card.settings_path()).unwrap(),
            before,
            "{tag}: startup rewrote the settings file"
        );
    }

    // Back where the process started. An assertion above that panics skips this, which is
    // accepted: one test in one binary is the isolation, and a reset hook for the global would
    // be a production affordance existing only for this test.
    let _ = clock::set_utc_offset_min(original);
}
