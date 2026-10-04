//! What language the frontend starts in: the card's stored choice, and the explicit override a
//! developer can put in front of it.
//!
//! The environment is read by the binary's `boot()`, once; what is tested here is the decision
//! itself, taken with the override handed in, so nothing in this file touches a process-global
//! variable. The other half of the startup — what a requested code turns into — is
//! `UiCtx::new`'s answer, which is why the context is built here exactly as both backends build
//! it.

use std::fs;
use std::path::PathBuf;

use slot2_i18n::{I18n, FALLBACK};
use slot2_platform::by_target;
use slot2_store::{Card, DEFAULT_LANGUAGE};
use slot2_ui::UiCtx;

use slot2::requested_language;

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-langstart-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card in a fresh folder, holding the settings file a hand-edited card would.
fn card_with(tag: &str, settings: Option<&str>) -> Card {
    let card = Card::new(scratch(tag));
    card.ensure_layout();
    if let Some(text) = settings {
        fs::write(card.settings_path(), text).unwrap();
    }
    card
}

/// A pack the player put on the card by hand, under `System/Lang`.
fn card_pack(card: &Card, code: &str, text: &str) -> PathBuf {
    let dir = card.root().join("System").join("Lang");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{code}.ftl"));
    fs::write(&path, text).unwrap();
    path
}

/// The startup the two backends share: the request, then the context that answers what actually
/// runs.
fn start(card: &Card, env_override: Option<&str>) -> (String, UiCtx) {
    let requested = requested_language(card, env_override);
    let dir = card.root().join("System").join("Lang");
    let ctx = UiCtx::new(
        by_target("rgsp").unwrap(),
        &requested,
        Vec::new(),
        Some(&dir),
    );
    (requested, ctx)
}

/// Every entry of a folder, sorted, as names.
fn names(dir: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn a_card_that_says_nothing_starts_in_english() {
    let card = card_with("silent", None);
    let system = card.system_dir();
    let before = names(&system);

    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "en");
    assert_eq!(ctx.i18n.code(), "en");
    assert!(
        !card.settings_path().exists(),
        "the startup read created the settings file"
    );
    assert_eq!(names(&system), before, "the startup read changed the card");

    // A file that is there and says nothing about the language is the same answer.
    let card = card_with("no-key", Some("volume = 30\n"));
    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, DEFAULT_LANGUAGE);
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(
        fs::read_to_string(card.settings_path()).unwrap(),
        "volume = 30\n"
    );
}

#[test]
fn the_cards_own_code_is_asked_for_exactly_as_spelled() {
    for (tag, code) in [
        ("built-in", "ko"),
        ("mixed-case", "PT-br"),
        ("card-only", "ja_custom"),
        ("unknown", "xx-NotInstalled"),
    ] {
        let card = card_with(tag, Some(&format!("language = {code}\nvolume = 30\n")));
        let (requested, ctx) = start(&card, None);
        assert_eq!(requested, code, "{code} was not asked for as written");
        // Whether that code can be spoken is the context's answer, and only the two built-in
        // packs are compiled in.
        let want = if code == "ko" { "ko" } else { "en" };
        assert_eq!(ctx.i18n.code(), want, "{code}");
    }
}

#[test]
fn an_explicit_override_outranks_the_card() {
    let card = card_with("override", Some("language = ko\n"));
    let (requested, ctx) = start(&card, Some("en"));
    assert_eq!(
        requested, "en",
        "the card was asked instead of the override"
    );
    assert_eq!(ctx.i18n.code(), "en");
    // And the card still says what it said.
    assert_eq!(card.read_language(), "ko");
}

#[test]
fn an_override_that_cannot_load_does_not_fall_back_to_the_card() {
    let card = card_with("bad-override", Some("language = ko\n"));

    // An empty override is an explicit request for nothing, not an absence.
    let (requested, ctx) = start(&card, Some(""));
    assert_eq!(requested, "");
    assert_eq!(
        ctx.i18n.code(),
        "en",
        "an empty override fell back to the card"
    );

    // And so is a code nobody has a pack for: the card's `ko` is not the answer to it.
    let (requested, ctx) = start(&card, Some("xx-unknown"));
    assert_eq!(requested, "xx-unknown");
    assert_eq!(
        ctx.i18n.code(),
        "en",
        "an unknown override fell back to the card"
    );

    // The card's own language is still there for the next boot without the variable.
    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "ko");
    assert_eq!(ctx.i18n.code(), "ko");
}

#[test]
fn a_stored_code_with_no_pack_boots_english_and_is_left_alone() {
    let card = card_with("unknown-stored", Some("language = xx-NotInstalled\n"));
    let bytes = fs::read(card.settings_path()).unwrap();

    let (requested, ctx) = start(&card, None);
    assert_eq!(
        requested, "xx-NotInstalled",
        "the stored choice was not asked for"
    );
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(fs::read(card.settings_path()).unwrap(), bytes);

    // Asking for it again is the same answer: the startup does not remember a fallback and write
    // it back over the player's choice.
    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "xx-NotInstalled");
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(fs::read(card.settings_path()).unwrap(), bytes);
}

#[test]
fn a_card_only_pack_runs_when_it_loads() {
    let card = card_with("card-pack", Some("language = xx-card\n"));
    let pack = card_pack(
        &card,
        "xx-card",
        "lang-name = Card Only\npower-off = Card Power\n",
    );
    let pack_bytes = fs::read(&pack).unwrap();

    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "xx-card");
    assert_eq!(
        ctx.i18n.code(),
        "xx-card",
        "the card's own pack was not loaded"
    );
    assert_eq!(ctx.i18n.t("lang-name"), "Card Only");
    assert_eq!(ctx.i18n.t("power-off"), "Card Power");
    // A key the card's pack does not define comes from the built-in English underneath.
    assert_eq!(ctx.i18n.t("resume"), "Resume");
    assert_eq!(fs::read(&pack).unwrap(), pack_bytes);
}

#[test]
fn a_broken_card_pack_boots_english_and_is_left_where_it_is() {
    let card = card_with("broken-pack", Some("language = xx-broken\n"));
    let pack = card_pack(&card, "xx-broken", "this is not = = valid\n{{{\n");
    let pack_bytes = fs::read(&pack).unwrap();
    let settings_bytes = fs::read(card.settings_path()).unwrap();

    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "xx-broken");
    assert_eq!(
        ctx.i18n.code(),
        "en",
        "a pack that will not parse was run anyway"
    );
    assert_eq!(
        fs::read(&pack).unwrap(),
        pack_bytes,
        "the pack was rewritten"
    );
    assert_eq!(
        fs::read(card.settings_path()).unwrap(),
        settings_bytes,
        "the choice was rewritten because its pack was broken"
    );
}

#[test]
fn a_built_in_pack_runs_and_a_card_pack_of_the_same_code_wins() {
    // The built-in Korean pack, with nothing on the card.
    let card = card_with("built-in-ko", Some("language = ko\n"));
    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "ko");
    assert_eq!(ctx.i18n.code(), "ko");
    assert_eq!(ctx.i18n.t("power-off"), "전원 끄기");

    // The same code, with a card pack: the card's own words are used, and the pack underneath is
    // still the built-in Korean.
    let pack = card_pack(&card, "ko", "power-off = 카드 전원\n");
    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "ko");
    assert_eq!(ctx.i18n.code(), "ko");
    assert_eq!(
        ctx.i18n.t("power-off"),
        "카드 전원",
        "the card's pack was skipped"
    );
    assert_eq!(
        ctx.i18n.t("resume"),
        "계속하기",
        "the built-in pack was lost"
    );

    // A card pack for that same built-in code that will not parse: the frontend still boots, in
    // the built-in English rather than half in a broken Korean.
    fs::write(&pack, "{{{\n").unwrap();
    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, "ko");
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(ctx.i18n.t("power-off"), "Power off");
}

#[test]
fn an_unusable_settings_file_is_english_and_is_left_alone() {
    let card = card_with("damaged", None);
    fs::write(
        card.settings_path(),
        [0xffu8, 0xfe, b'l', b'=', 0x80, b'\n'],
    )
    .unwrap();
    let bytes = fs::read(card.settings_path()).unwrap();

    let (requested, ctx) = start(&card, None);
    assert_eq!(requested, DEFAULT_LANGUAGE);
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(fs::read(card.settings_path()).unwrap(), bytes);
}

#[test]
fn the_two_defaults_are_the_same_word() {
    // The store's default and the pack every language falls back to have to name the same
    // language, or a card that says nothing would start in one the packs cannot speak. The store
    // does not depend on the i18n crate, so the check lives here, at the boundary that uses both.
    assert_eq!(DEFAULT_LANGUAGE, "en");
    assert_eq!(FALLBACK, "en");
    assert_eq!(DEFAULT_LANGUAGE, FALLBACK);
    let embedded = I18n::embedded(FALLBACK).expect("the fallback pack is compiled in");
    assert_eq!(embedded.code(), DEFAULT_LANGUAGE);
}
