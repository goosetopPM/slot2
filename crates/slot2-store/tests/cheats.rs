//! The RetroArch `.cht` reader: `System/cheats/<PLAT>/<stem>.cht` on the card.
//!
//! Synthetic files only, in temporary card roots: the card is what is under test, not the
//! database those files come from.

use std::fs;
use std::path::{Path, PathBuf};

use slot2_store::{Card, Cart, Cheat, Error, Platform};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-cheats-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn cart(root: &Path, platform: Platform, stem: &str) -> Cart {
    Cart {
        platform,
        stem: stem.into(),
        title: stem.into(),
        rom: root.join("Games").join(platform.folder()).join(stem),
    }
}

/// A card with its layout made and one NES cart on it.
fn card(name: &str) -> (Card, Cart) {
    let root = tempdir(name);
    let card = Card::new(&root);
    card.ensure_layout();
    let cart = cart(&root, Platform::Nes, "Mappy");
    (card, cart)
}

/// Put `text` where the cart's cheat file goes and read it back.
fn load(card: &Card, cart: &Cart, text: &str) -> Result<Vec<Cheat>, Error> {
    fs::write(card.cheat_path(cart), text).unwrap();
    card.read_cheats(cart)
}

fn cheat(description: &str, code: &str, enabled: bool) -> Cheat {
    Cheat {
        description: description.into(),
        code: code.into(),
        enabled,
    }
}

#[test]
fn the_layout_has_a_cheats_folder_for_every_platform() {
    let root = tempdir("layout");
    let card = Card::new(&root);
    assert!(card.ensure_layout() >= 20, "the layout shrank");
    for p in Platform::ALL {
        assert!(
            root.join("System/cheats").join(p.folder()).is_dir(),
            "{p:?} has no cheat folder"
        );
    }
    // Everything that was there before is still there, and a second call adds nothing.
    assert!(root.join("Games/GBA").is_dir());
    assert!(root.join("Wallpapers").is_dir());
    assert!(root.join("System/Lang").is_dir());
    assert_eq!(card.ensure_layout(), 0, "a second call created folders");
}

#[test]
fn cheat_paths_follow_the_platform_folder_and_the_stem() {
    let root = tempdir("path");
    let card = Card::new(&root);
    let nes = cart(&root, Platform::Nes, "Mappy");
    let snes = cart(&root, Platform::Snes, "Mappy");
    let spaced = cart(&root, Platform::Gba, "Super Mario Bros. 3 (Japan)");

    assert_eq!(
        card.cheat_path(&nes),
        root.join("System/cheats/NES/Mappy.cht")
    );
    assert_eq!(
        card.cheat_path(&snes),
        root.join("System/cheats/SNES/Mappy.cht")
    );
    assert_ne!(
        card.cheat_path(&nes),
        card.cheat_path(&snes),
        "one stem, one file: platforms must not collide"
    );
    assert_eq!(
        card.cheat_path(&spaced),
        root.join("System/cheats/GBA/Super Mario Bros. 3 (Japan).cht")
    );
}

#[test]
fn a_missing_file_and_a_zero_count_are_both_no_cheats() {
    let (card, cart) = card("empty");
    assert!(
        card.read_cheats(&cart).unwrap().is_empty(),
        "a game with no cheat file has cheats"
    );
    assert!(load(&card, &cart, "cheats = 0\n").unwrap().is_empty());
    assert!(load(&card, &cart, "# nothing here\n\ncheats = 0\n")
        .unwrap()
        .is_empty());
}

#[test]
fn an_official_style_file_comes_back_in_index_order() {
    let (card, cart) = card("official");
    // Written out of order on purpose: the file's own count decides the record order.
    let text = "\
# libretro-database style
cheats = 3

cheat2_desc = \"Max rupees\"
cheat2_code = \"7E0019FF+7E001AFF\"

cheat0_desc = \"Infinite lives\"
cheat0_code = \"7E007C9A\"
cheat0_enable = false

cheat1_desc = \"Max hearts\"
cheat1_code = \"7E13F2FF\"
cheat1_enable = true
";
    let cheats = load(&card, &cart, text).unwrap();
    assert_eq!(
        cheats,
        vec![
            cheat("Infinite lives", "7E007C9A", false),
            cheat("Max hearts", "7E13F2FF", true),
            cheat("Max rupees", "7E0019FF+7E001AFF", false),
        ],
        "one multi-part code is one string, and a missing enable is off"
    );
}

#[test]
fn the_enable_flag_takes_retroarchs_spellings() {
    let (card, cart) = card("enable");
    let text = "\
cheats = 4
cheat0_desc = \"a\"
cheat0_code = \"1\"
cheat0_enable = TRUE
cheat1_desc = \"b\"
cheat1_code = \"2\"
cheat1_enable = False
cheat2_desc = \"c\"
cheat2_code = \"3\"
cheat2_enable = 1
cheat3_desc = \"d\"
cheat3_code = \"4\"
cheat3_enable = 0
";
    let enabled: Vec<bool> = load(&card, &cart, text)
        .unwrap()
        .iter()
        .map(|c| c.enabled)
        .collect();
    assert_eq!(enabled, vec![true, false, true, false]);
}

#[test]
fn quoted_values_decode_and_the_code_is_kept_exactly() {
    let (card, cart) = card("quoting");
    let text = r#"cheats = 4
cheat0_desc = "체력 회복 #1 = 회복"
cheat0_code = "7E007C9A"
cheat1_desc = "say \"hi\""
cheat1_code = "AA\\BB"
cheat2_desc = "wild #=?*+card"
cheat2_code = "AAAA-BBBB?*C,D"
cheat3_desc = "bare text is taken as written"
cheat3_code = F0-01+E1
"#;
    let cheats = load(&card, &cart, text).unwrap();
    assert_eq!(cheats[0].description, "체력 회복 #1 = 회복");
    assert_eq!(cheats[1].description, "say \"hi\"", "escaped quotes");
    assert_eq!(cheats[1].code, "AA\\BB", "escaped backslash");
    assert_eq!(cheats[2].description, "wild #=?*+card");
    assert_eq!(
        cheats[2].code, "AAAA-BBBB?*C,D",
        "a core's punctuation is not this layer's to rewrite"
    );
    assert_eq!(
        cheats[3],
        cheat("bare text is taken as written", "F0-01+E1", false)
    );
}

#[test]
fn retroarch_metadata_this_version_does_not_read_is_ignored() {
    let (card, cart) = card("metadata");
    let text = r#"cheats = 1
cheat_database = "whatever"
cheat0_desc = "Infinite lives"
cheat0_code = "7E007C9A"
cheat0_enable = 1
cheat0_handler = "1"
cheat0_mem_search = "0"
cheat0_rumble_type = "0"
this line has no equals sign
"#;
    assert_eq!(
        load(&card, &cart, text).unwrap(),
        vec![cheat("Infinite lives", "7E007C9A", true)]
    );
}

#[test]
fn unknown_indexed_metadata_is_not_an_entry() {
    let (card, cart) = card("metadata-index");

    // Far-out metadata with no owned field is not an index the file claimed, so there is
    // nothing to check against the count and nothing to refuse.
    let text = "cheats = 0\ncheat4294967295_handler = \"1\"\n";
    assert!(load(&card, &cart, text).unwrap().is_empty());

    // And beside a real entry it is not a gap, before or after it.
    let text = "cheats = 1\ncheat8750_handler = \"1\"\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat999_mem_search = \"0\"\n";
    assert_eq!(
        load(&card, &cart, text).unwrap(),
        vec![cheat("a", "b", false)]
    );

    // It does not stand in for an index either: metadata is not a description or a code.
    let text = "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat1_handler = \"1\"\n";
    match load(&card, &cart, text) {
        Err(Error::Invalid(message)) => assert!(
            message.contains("cheat1_desc"),
            "{message:?} does not name the index that is short"
        ),
        other => panic!("metadata satisfied a declared index: {other:?}"),
    }

    // An owned field out of range is still an error, whatever metadata sits beside it.
    let text = "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat1_desc = \"c\"\ncheat7_handler = \"1\"\n";
    match load(&card, &cart, text) {
        Err(Error::Invalid(message)) => assert!(message.contains("cheat1"), "{message:?}"),
        other => panic!("an out-of-range owned field was accepted: {other:?}"),
    }
}

#[test]
fn a_huge_declared_count_with_no_entries_is_refused() {
    // The declared count is a claim. Nothing is allocated or looped over on the strength of
    // it alone, so a two-line file is refused as fast as it is read.
    let (card, cart) = card("huge-count");
    match load(&card, &cart, "cheats = 4294967295\n") {
        Err(Error::Invalid(message)) => assert!(
            message.contains("4294967295"),
            "the refusal does not say what the file claimed: {message:?}"
        ),
        other => panic!("a count nothing backs was accepted: {other:?}"),
    }
}

#[test]
fn a_huge_declared_count_with_one_entry_names_the_missing_index() {
    let (card, cart) = card("huge-count-one");
    let text = "cheats = 4294967295\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\n";
    match load(&card, &cart, text) {
        Err(Error::Invalid(message)) => assert!(
            message.contains("cheat1_desc") && message.contains("4294967295"),
            "the mismatch is not diagnosable: {message:?}"
        ),
        other => panic!("a count no file could satisfy was accepted: {other:?}"),
    }
}

#[test]
fn a_count_beyond_u32_is_not_a_count() {
    let (card, cart) = card("count-overflow");
    match load(&card, &cart, "cheats = 4294967296\n") {
        Err(Error::Invalid(message)) => assert!(message.contains("cheats"), "{message:?}"),
        other => panic!("a count that is not a count was accepted: {other:?}"),
    }
}

#[test]
fn a_malformed_file_fails_whole() {
    // Each case says which key the error has to name: a file that is refused for the wrong
    // reason is a parser that is wrong in two places.
    let cases: &[(&str, &str, &str)] = &[
        (
            "no count",
            "cheat0_desc = \"a\"\ncheat0_code = \"b\"\n",
            "cheats",
        ),
        ("count is a word", "cheats = two\n", "cheats"),
        ("count is negative", "cheats = -1\n", "cheats"),
        ("count is empty", "cheats =\n", "cheats"),
        ("count twice", "cheats = 0\ncheats = 0\n", "cheats"),
        (
            "description twice",
            "cheats = 1\ncheat0_desc = \"a\"\ncheat0_desc = \"b\"\ncheat0_code = \"c\"\n",
            "cheat0_desc",
        ),
        (
            "code twice",
            "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"c\"\ncheat0_code = \"d\"\n",
            "cheat0_code",
        ),
        (
            "enable twice",
            "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"c\"\ncheat0_enable = 0\ncheat0_enable = 1\n",
            "cheat0_enable",
        ),
        (
            "a gap",
            "cheats = 3\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat2_desc = \"c\"\ncheat2_code = \"d\"\n",
            "cheat1_desc",
        ),
        (
            "an index beyond the count",
            "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat1_desc = \"c\"\ncheat1_code = \"d\"\n",
            "cheat1",
        ),
        ("no description", "cheats = 1\ncheat0_code = \"b\"\n", "cheat0_desc"),
        ("no code", "cheats = 1\ncheat0_desc = \"a\"\n", "cheat0_code"),
        (
            "empty description",
            "cheats = 1\ncheat0_desc = \"\"\ncheat0_code = \"b\"\n",
            "cheat0_desc",
        ),
        (
            "blank description",
            "cheats = 1\ncheat0_desc = \"   \"\ncheat0_code = \"b\"\n",
            "cheat0_desc",
        ),
        (
            "empty code",
            "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"\"\n",
            "cheat0_code",
        ),
        (
            "enable is not a flag",
            "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat0_enable = maybe\n",
            "cheat0_enable",
        ),
        (
            "quote never closed",
            "cheats = 1\ncheat0_desc = \"a\ncheat0_code = \"b\"\n",
            "cheat0_desc",
        ),
        (
            "text after the quote",
            "cheats = 1\ncheat0_desc = \"a\" b\ncheat0_code = \"b\"\n",
            "cheat0_desc",
        ),
        (
            "unknown escape",
            "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"b\\qc\"\n",
            "cheat0_code",
        ),
    ];
    for (name, text, want) in cases {
        let (c, cart) = card(&name.replace(' ', "-"));
        match load(&c, &cart, text) {
            Err(Error::Invalid(message)) => assert!(
                message.contains(want),
                "{name}: {message:?} does not name {want}"
            ),
            other => panic!("{name}: not a malformed-file error: {other:?}"),
        }
    }
}

#[test]
fn a_refused_file_leaves_nothing_half_loaded() {
    let (card, cart) = card("refused");
    let broken = "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat1_desc = \"c\"\n";
    let e = load(&card, &cart, broken).unwrap_err();
    assert!(
        matches!(e, Error::Invalid(ref m) if m.contains("cheat1_code")),
        "the error does not say which index is broken: {e}"
    );

    // The same card, corrected, comes back whole.
    let fixed = "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\ncheat1_desc = \"c\"\ncheat1_code = \"d\"\n";
    assert_eq!(
        load(&card, &cart, fixed).unwrap(),
        vec![cheat("a", "b", false), cheat("c", "d", false)]
    );
}

#[test]
fn a_file_that_cannot_be_read_names_the_path() {
    let (damaged_card, damaged_cart) = card("unreadable");
    let path = damaged_card.cheat_path(&damaged_cart);
    let bytes = vec![0xff, 0xfe, 0x00, 0x01];
    fs::write(&path, &bytes).unwrap();

    match damaged_card.read_cheats(&damaged_cart) {
        Err(Error::Io(p, _)) => assert_eq!(p, path),
        other => panic!("invalid UTF-8 was not an I/O error: {other:?}"),
    }
    assert_eq!(fs::read(&path).unwrap(), bytes, "the file was rewritten");

    // A directory where the file belongs is the portable unreadable case on a card.
    let (blocked_card, blocked_cart) = card("directory");
    let path = blocked_card.cheat_path(&blocked_cart);
    fs::create_dir_all(&path).unwrap();
    match blocked_card.read_cheats(&blocked_cart) {
        Err(Error::Io(p, _)) => assert_eq!(p, path),
        other => panic!("a directory was not an I/O error: {other:?}"),
    }
    assert!(path.is_dir(), "the directory was removed");
}

#[test]
fn reading_never_writes() {
    let (card, cart) = card("readonly");
    let path = card.cheat_path(&cart);
    let text = "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"b\"\n";
    fs::write(&path, text).unwrap();
    let _ = card.read_cheats(&cart).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        text,
        "a read rewrote the file"
    );
    let leftovers: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers, vec!["Mappy.cht"], "{leftovers:?}");
}
