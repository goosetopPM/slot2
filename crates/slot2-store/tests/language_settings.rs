//! The card's global settings file, `System/slot2.ini`, as the player's language choice.
//!
//! The value is the stem of a language pack under `System/Lang` — `ko`, `pt-BR`, `zh-Hant` — and
//! nothing else. It is its own key with its own API rather than a field of `GlobalSettings`, for
//! the same reason the clock's offset is: the volume is written on a delay of its own, and a flush
//! that read the language as a field would write the default back over a card that holds a real
//! code. Which languages exist is not this crate's question, so a code is judged only as a file
//! name, and an installed-looking one is not required to be installed.

use std::fs;
use std::path::PathBuf;

use slot2_store::{
    Card, Error, GlobalSettings, Ini, DEFAULT_LANGUAGE, LANGUAGE_KEY, UTC_OFFSET_MINUTES_KEY,
};

/// The settings source itself. What the store must never reach for is a claim about this file, so
/// it is read here rather than trusted.
const SETTINGS_SRC: &str = include_str!("../src/settings.rs");

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-store-lang-{}-{name}", std::process::id()));
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

/// Every entry of a folder, sorted, as `(name, is_dir)`.
fn entries(dir: &std::path::Path) -> Vec<(String, bool)> {
    let mut v: Vec<(String, bool)> = fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (
                e.file_name().to_string_lossy().into_owned(),
                e.file_type().unwrap().is_dir(),
            )
        })
        .collect();
    v.sort();
    v
}

// ------------------------------------------------------------------ reading

#[test]
fn a_missing_file_or_key_is_english_and_a_read_creates_nothing() {
    let (card, d) = card("missing");
    assert_eq!(DEFAULT_LANGUAGE, "en");
    assert_eq!(LANGUAGE_KEY, "language");
    assert_eq!(card.settings_path(), d.join("System").join("slot2.ini"));

    let system = card.system_dir();
    let before = entries(&system);
    assert_eq!(card.read_language(), "en");
    assert!(!card.settings_path().exists(), "a read created the file");
    assert_eq!(entries(&system), before, "a read changed the System folder");

    // A file that is there and says nothing about the language is the same answer.
    fs::write(card.settings_path(), "volume = 30\n").unwrap();
    assert_eq!(card.read_language(), "en");
    assert_eq!(
        fs::read_to_string(card.settings_path()).unwrap(),
        "volume = 30\n",
        "a read rewrote the file"
    );
}

#[test]
fn a_usable_code_round_trips_exactly() {
    let (card, _d) = card("round-trip");
    let path = card.settings_path();
    for code in [
        // The codes this build ships, and codes it does not: whether a pack exists is the
        // frontend's question, and a card-only pack is placed under `System/Lang` by hand.
        "ko",
        "pt-BR",
        "zh-Hant",
        "ja_custom",
        "xx-NotInstalled",
        "a.b",
        "1.2.3",
        // A code nobody would write but that is still one file's worth of name.
        "한국어",
        "pt-BR-x-private",
    ] {
        card.write_language(code).unwrap();
        assert_eq!(card.read_language(), code, "{code} did not round-trip");
        assert_eq!(
            read_ini(&card).get(LANGUAGE_KEY),
            Some(code),
            "{code} was not stored under its own key"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("language = {code}\n"),
            "{code} is not the only thing in the file"
        );
        // Back to English and off again, so each case starts from a card that says nothing.
        card.write_language(DEFAULT_LANGUAGE).unwrap();
        assert_eq!(card.read_language(), DEFAULT_LANGUAGE);
    }

    // The longest code a file stem may be, and one byte past it.
    let sixty_four = "b".repeat(64);
    card.write_language(&sixty_four).unwrap();
    assert_eq!(card.read_language(), sixty_four);
    card.write_language(DEFAULT_LANGUAGE).unwrap();
    fs::write(&path, format!("language = {}\n", "b".repeat(65))).unwrap();
    assert_eq!(card.read_language(), DEFAULT_LANGUAGE);
}

#[test]
fn spelling_is_never_normalized() {
    let (card, _d) = card("spelling");
    for code in ["PT-br", "KO", "zh-hant", "ja_custom", "ZH-HANT", "Pt"] {
        card.write_language(code).unwrap();
        assert_eq!(card.read_language(), code, "{code} came back changed");
        assert_eq!(
            read_ini(&card).get(LANGUAGE_KEY),
            Some(code),
            "{code} was lowercased or expanded on the way to the file"
        );
    }
}

#[test]
fn an_unsafe_code_reads_as_english_and_is_refused_by_write() {
    let (card, _d) = card("unsafe");
    let path = card.settings_path();
    fs::write(&path, "volume = 30\n").unwrap();
    let kept = fs::read(&path).unwrap();

    // Nothing that could not be one file's name is written, and a refusal leaves the card alone.
    for code in [
        "",
        ".",
        "..",
        "a/b",
        "a\\b",
        "ko KR",
        "ko\tKR",
        "\u{0}",
        "ko\u{0}",
        " ko",
        "ko ",
        "\tko\t",
        &"b".repeat(65),
    ] {
        assert!(
            matches!(card.write_language(code), Err(Error::Invalid(_))),
            "{code:?} was written"
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            kept,
            "{code:?} touched the file it refused"
        );
    }

    // Read back, the same values are English — and the padding the INI grammar allows outside a
    // value is the parser's business, so a code wrapped in spaces arrives trimmed and is read.
    for (text, want) in [
        ("language = \n", DEFAULT_LANGUAGE),
        ("language = .\n", DEFAULT_LANGUAGE),
        ("language = ..\n", DEFAULT_LANGUAGE),
        ("language = a/b\n", DEFAULT_LANGUAGE),
        ("language = a\\b\n", DEFAULT_LANGUAGE),
        ("language = ko KR\n", DEFAULT_LANGUAGE),
        ("language = ko\tKR\n", DEFAULT_LANGUAGE),
        ("language = \u{0}\n", DEFAULT_LANGUAGE),
        ("language = ko\u{0}\n", DEFAULT_LANGUAGE),
        ("language =   ko   \n", "ko"),
        ("  language  =  pt-BR\n", "pt-BR"),
    ] {
        fs::write(&path, text).unwrap();
        assert_eq!(card.read_language(), want, "{text:?}");
    }
    let too_long = format!("language = {}\n", "b".repeat(65));
    fs::write(&path, &too_long).unwrap();
    assert_eq!(card.read_language(), DEFAULT_LANGUAGE);
}

#[test]
fn an_unreadable_file_fails_without_damage() {
    // Bytes that are not UTF-8, which is a file the store cannot read and must not rewrite.
    let (card, _d) = card("damaged");
    let path = card.settings_path();
    fs::write(&path, [0xffu8, 0xfe, b'l', b'=', 0x80, b'\n']).unwrap();
    let damaged = fs::read(&path).unwrap();
    assert_eq!(card.read_language(), DEFAULT_LANGUAGE);
    assert!(matches!(card.write_language("ko"), Err(Error::Io(..))));
    assert_eq!(fs::read(&path).unwrap(), damaged, "the file was rewritten");
    assert!(matches!(
        card.write_language(DEFAULT_LANGUAGE),
        Err(Error::Io(..))
    ));
    assert_eq!(fs::read(&path).unwrap(), damaged);
}

#[test]
fn a_folder_in_the_settings_file_s_place_fails_without_damage() {
    // A directory where the settings file belongs: nothing to read, and nothing to replace.
    let (card, _d) = card("folder-in-place");
    let system = card.system_dir();
    fs::create_dir(card.settings_path()).unwrap();
    let before = entries(&system);
    assert_eq!(card.read_language(), DEFAULT_LANGUAGE);
    assert!(matches!(card.write_language("ko"), Err(Error::Io(..))));
    assert_eq!(entries(&system), before, "the settings folder moved");
    assert!(card.settings_path().is_dir());
}

#[test]
fn writing_english_leaves_no_file_behind() {
    let (card, _d) = card("english");
    let path = card.settings_path();

    // From a card with no file at all, repeated: the write has nothing to do and does nothing.
    for _ in 0..3 {
        card.write_language(DEFAULT_LANGUAGE).unwrap();
        assert!(!path.exists(), "an English write created the file");
    }

    // From a file that held only the language, the key goes and the empty file with it.
    card.write_language("ko").unwrap();
    assert!(path.is_file());
    card.write_language(DEFAULT_LANGUAGE).unwrap();
    assert!(!path.exists(), "the emptied file was left behind");

    // And from a file that holds anything else, only its own key goes.
    fs::write(&path, "language = ko\nvolume = 30\n").unwrap();
    card.write_language(DEFAULT_LANGUAGE).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "volume = 30\n",
        "English took another key with it"
    );
}

// ------------------------------------------------------------------ other keys

#[test]
fn a_language_write_keeps_every_other_key() {
    let (card, _d) = card("keep-others");
    let path = card.settings_path();
    fs::write(
        &path,
        "future_key = something\nutc_offset_minutes = 540\nvolume = 30\n",
    )
    .unwrap();

    card.write_language("pt-BR").unwrap();
    let ini = read_ini(&card);
    assert_eq!(ini.get(LANGUAGE_KEY), Some("pt-BR"));
    assert_eq!(ini.get("volume"), Some("30"), "the level was lost");
    assert_eq!(
        ini.get(UTC_OFFSET_MINUTES_KEY),
        Some("540"),
        "the offset was lost"
    );
    assert_eq!(
        ini.get("future_key"),
        Some("something"),
        "a stranger's key was lost"
    );

    // Back to English: the language key goes, everything else stays.
    card.write_language(DEFAULT_LANGUAGE).unwrap();
    let ini = read_ini(&card);
    assert_eq!(ini.get(LANGUAGE_KEY), None);
    assert_eq!(ini.get("volume"), Some("30"));
    assert_eq!(ini.get(UTC_OFFSET_MINUTES_KEY), Some("540"));
    assert_eq!(ini.get("future_key"), Some("something"));
}

#[test]
fn a_volume_write_and_a_clock_write_keep_the_language() {
    let (card, _d) = card("keep-language");
    let path = card.settings_path();
    fs::write(&path, "language = ko\n").unwrap();

    card.write_global_settings(&GlobalSettings { volume: 30 })
        .unwrap();
    assert_eq!(
        card.read_language(),
        "ko",
        "the volume flush ate the language"
    );
    assert_eq!(read_ini(&card).get("volume"), Some("30"));

    card.write_utc_offset_minutes(540).unwrap();
    assert_eq!(
        card.read_language(),
        "ko",
        "the clock write ate the language"
    );
    assert_eq!(card.read_utc_offset_minutes(), 540);

    // And the language write is the one that can still change it, without disturbing the rest.
    card.write_language("ja_custom").unwrap();
    assert_eq!(card.read_utc_offset_minutes(), 540);
    assert_eq!(card.read_global_settings().volume, 30);
    assert_eq!(card.read_language(), "ja_custom");
}

#[test]
fn a_write_leaves_no_temp_file_and_touches_no_other_file() {
    let (card, _d) = card("no-temp");
    let path = card.settings_path();
    let system = card.system_dir();

    // A pack the player put on the card by hand, and a game's own settings file: neither is this
    // write's business.
    let lang = system.join("Lang");
    fs::create_dir_all(&lang).unwrap();
    let pack = lang.join("ko.ftl");
    fs::write(&pack, "power-off = 끄기\n").unwrap();
    let games = system.join("games").join("GBA");
    fs::create_dir_all(&games).unwrap();
    let game = games.join("arm.ini");
    fs::write(&game, "scale = integer\n").unwrap();

    fs::write(&path, "volume = 30\n").unwrap();
    let before = entries(&system);
    card.write_language("ko").unwrap();
    assert_eq!(
        entries(&system),
        before,
        "a language write changed the settings folder"
    );
    card.write_language(DEFAULT_LANGUAGE).unwrap();
    assert_eq!(
        entries(&system),
        before,
        "an English write changed the settings folder"
    );
    assert!(
        !entries(&system)
            .iter()
            .any(|(name, _)| name.ends_with(".tmp")),
        "a temporary file was left beside the settings file"
    );

    assert_eq!(fs::read_to_string(&pack).unwrap(), "power-off = 끄기\n");
    assert_eq!(fs::read_to_string(&game).unwrap(), "scale = integer\n");
    assert_eq!(card.read_language(), DEFAULT_LANGUAGE);
    assert_eq!(
        read_ini(&card).get("volume"),
        Some("30"),
        "the level did not survive the language writes"
    );
}

// ------------------------------------------------------------------ boundaries

#[test]
fn the_store_reads_no_environment_and_no_language_pack() {
    // A card that names a language says so whatever the process around it thinks: the value comes
    // from the file and from nowhere else. (Reading the environment to prove it is not consulted
    // would mean setting a process-global variable, which this test does not do; the source check
    // below is the binding part.)
    let (card, _d) = card("no-environment");
    fs::write(card.settings_path(), "language = ko\n").unwrap();
    assert_eq!(card.read_language(), "ko");

    for forbidden in [
        "std::env",   // the environment is not this file's to read
        "env::var",   //
        "slot2_i18n", // the store does not depend on the packs
        "I18n",       //
        "\"Lang\"",   // and never opens the folder the packs live in
        "available(", // no pack listing, no parse, no font choice
        "locale",     // nor the operating system's idea of where the machine is
    ] {
        assert!(
            !SETTINGS_SRC.contains(forbidden),
            "settings.rs reaches for {forbidden}"
        );
    }
    // The keys and the two APIs this task is about are the whole of what it touched.
    for expected in [
        "LANGUAGE_KEY",
        "DEFAULT_LANGUAGE",
        "read_language",
        "write_language",
    ] {
        assert!(
            SETTINGS_SRC.contains(expected),
            "settings.rs has no {expected}"
        );
    }
}
