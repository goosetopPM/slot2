//! The font a language pack asks for, and what happens when it cannot have it.
//!
//! `UiCtx::new` builds the chain from the language that really loaded: the pack's `lang-font`
//! first (by path, lazily), the embedded UI font, then the CJK fallback. No font file is read
//! at context creation — the first glyph that needs a slot is what parses it — and a preference
//! that is missing, unsafe or corrupt costs the default fonts and nothing else. The card's own
//! `System/Fonts` is searched before the build's assets, in the caller's order.

use std::fs;
use std::path::{Path, PathBuf};

use slot2_platform::by_target;
use slot2_text::FontId;
use slot2_ui::UiCtx;

/// The UI font, by the name tests copy it under. Latin only: no Hangul.
const OPEN_SANS: &str = "OpenSans-Regular.ttf";
/// The CJK font, by the name tests copy it under. Latin and Hangul.
const NOTO: &str = "NotoSansKR-Regular.otf";

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts")
}

/// One of the repository's own fonts. Nothing here ships a second binary font: the differences
/// the tests lean on are the coverage the two real files already have.
fn asset(name: &str) -> PathBuf {
    let p = assets().join(name);
    assert!(p.is_file(), "the shipped font {name} is missing");
    p
}

fn copy(from: &Path, to: &Path) {
    fs::copy(from, to).unwrap();
}

fn ctx_of(lang: &str, dirs: Vec<PathBuf>, card_lang_dir: Option<&Path>) -> UiCtx {
    UiCtx::new(by_target("rgsp").unwrap(), lang, dirs, card_lang_dir)
}

/// A card in a scratch folder, removed when the test that made it ends.
struct Card {
    root: PathBuf,
}

impl Card {
    fn new(tag: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("slot2-langfont-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("System").join("Lang")).unwrap();
        fs::create_dir_all(root.join("System").join("Fonts")).unwrap();
        Card { root }
    }

    fn lang_dir(&self) -> PathBuf {
        self.root.join("System").join("Lang")
    }

    fn fonts_dir(&self) -> PathBuf {
        self.root.join("System").join("Fonts")
    }

    /// A pack that names its own language and, when it has one, its preferred font.
    fn pack(&self, code: &str, font: Option<&str>) {
        let text = match font {
            Some(f) => format!("lang-name = {code}\nlang-font = {f}\n"),
            None => format!("lang-name = {code}\n"),
        };
        fs::write(self.lang_dir().join(format!("{code}.ftl")), text).unwrap();
    }

    /// A real font, copied onto the card under `name`.
    fn bring_font(&self, name: &str, from: &str) -> PathBuf {
        let to = self.fonts_dir().join(name);
        copy(&asset(from), &to);
        to
    }

    /// The context both backends would build for this card: its own `System/Fonts` first, the
    /// build's assets after, and its `System/Lang` as the pack directory.
    fn ctx(&self, lang: &str) -> UiCtx {
        ctx_of(
            lang,
            vec![self.fonts_dir(), assets()],
            Some(&self.lang_dir()),
        )
    }

    fn ctx_with(&self, lang: &str, dirs: Vec<PathBuf>) -> UiCtx {
        ctx_of(lang, dirs, Some(&self.lang_dir()))
    }
}

impl Drop for Card {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// ---------------------------------------------------------------- the built-in chain

#[test]
fn english_has_no_preference_and_keeps_the_default_chain() {
    let mut ctx = ctx_of("en", vec![assets()], None);
    assert_eq!(ctx.i18n.font(), None, "English names no font");
    assert_eq!(ctx.fonts.len(), 2, "the UI font and the CJK fallback");
    assert!(ctx.fonts.is_loaded(FontId(0)), "the embedded font is eager");
    assert!(
        !ctx.fonts.is_loaded(FontId(1)),
        "the CJK font waits until a glyph needs it"
    );

    // Latin comes from the UI font, and asking for it does not wake the CJK font: a machine
    // that never shows a Korean or Japanese title never pays for those megabytes.
    assert_eq!(ctx.fonts.resolve('A'), Some(FontId(0)));
    assert!(!ctx.fonts.is_loaded(FontId(1)), "Latin woke the CJK font");
    assert_eq!(ctx.fonts.resolve('가'), Some(FontId(1)));
    assert!(ctx.fonts.is_loaded(FontId(1)));
    assert_eq!(
        ctx.fonts.resolve('漢'),
        Some(FontId(1)),
        "hanja is CJK work"
    );
    assert_eq!(ctx.fonts.resolve('あ'), Some(FontId(1)), "kana too");
}

#[test]
fn korean_prefers_noto_and_registers_it_once() {
    assert!(
        asset(NOTO).is_file(),
        "the ko pack names a font this build does not ship"
    );
    let mut ctx = ctx_of("ko", vec![assets()], None);

    assert_eq!(
        ctx.i18n.font().as_deref(),
        Some(NOTO),
        "the Korean pack's preference is the file that is really on the card"
    );
    assert_eq!(
        ctx.fonts.len(),
        2,
        "the preferred font is the CJK fallback as well, not a second copy of it"
    );
    assert!(
        !ctx.fonts.is_loaded(FontId(0)),
        "a preferred font is registered by path, not parsed at context creation"
    );
    assert!(
        ctx.fonts.is_loaded(FontId(1)),
        "the embedded UI font is the eager fallback behind it"
    );

    // The pack asked for this font, so it draws the body — Latin included, which is why it
    // sits in front of the embedded one.
    assert_eq!(ctx.fonts.resolve('A'), Some(FontId(0)));
    assert_eq!(ctx.fonts.resolve('가'), Some(FontId(0)));
    assert_eq!(ctx.fonts.resolve('漢'), Some(FontId(0)));
    assert!(ctx.fonts.is_loaded(FontId(0)));
}

// ---------------------------------------------------------------- a card's own font

#[test]
fn a_card_language_brings_its_own_preferred_font() {
    let card = Card::new("card-font");
    card.pack("xx-font", Some("CardPreferred.ttf"));
    card.bring_font("CardPreferred.ttf", NOTO);

    let mut ctx = card.ctx("xx-font");
    assert_eq!(ctx.i18n.code(), "xx-font");
    assert_eq!(ctx.i18n.font().as_deref(), Some("CardPreferred.ttf"));
    assert_eq!(
        ctx.fonts.len(),
        3,
        "the card's font, the embedded UI font, and the CJK fallback"
    );
    assert!(
        !ctx.fonts.is_loaded(FontId(0)),
        "context creation read the card's font"
    );

    // The card's font is the first slot, and it is the one that draws: the Hangul a default
    // chain would have sent to the CJK fallback comes from it instead.
    assert_eq!(ctx.fonts.resolve('A'), Some(FontId(0)));
    assert_eq!(ctx.fonts.resolve('가'), Some(FontId(0)));
    assert_eq!(ctx.fonts.resolve('漢'), Some(FontId(0)));
}

#[test]
fn the_first_font_directory_with_the_name_wins() {
    let card = Card::new("dirs");
    card.pack("yy-font", Some("Same.ttf"));
    // One name, two directories, two different fonts behind it. What the chain does with
    // Hangul says which copy was taken: OpenSans has no Hangul, Noto has it.
    let first = card.root.join("A");
    let second = card.root.join("B");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    copy(&asset(OPEN_SANS), &first.join("Same.ttf"));
    copy(&asset(NOTO), &second.join("Same.ttf"));

    let mut a_first = card.ctx_with("yy-font", vec![first.clone(), second.clone(), assets()]);
    assert_eq!(a_first.fonts.len(), 3);
    assert_eq!(a_first.fonts.resolve('A'), Some(FontId(0)));
    assert_eq!(
        a_first.fonts.resolve('가'),
        Some(FontId(2)),
        "the first directory's copy is the Latin-only one, so Hangul is the CJK fallback's"
    );

    let mut b_first = card.ctx_with("yy-font", vec![second.clone(), first.clone(), assets()]);
    assert_eq!(
        b_first.fonts.resolve('가'),
        Some(FontId(0)),
        "the order is the caller's, and the first directory holding the name is the answer"
    );
}

// ---------------------------------------------------------------- what a pack may not ask for

#[test]
fn unusable_preferences_keep_the_default_chain() {
    let card = Card::new("unsafe");
    // Every place an unchecked join would land, with a real font sitting there: if the name
    // were used, the chain would grow a slot and Hangul would come from it.
    let outside = card.root.join("System").join("Escape.ttf");
    copy(&asset(NOTO), &outside);
    fs::create_dir_all(card.fonts_dir().join("sub")).unwrap();
    copy(
        &asset(NOTO),
        &card.fonts_dir().join("sub").join("Escape.ttf"),
    );

    let cases: Vec<(&str, String)> = vec![
        ("plain-missing", "Missing.ttf".to_string()),
        ("slash", "sub/Escape.ttf".to_string()),
        ("backslash", "sub\\Escape.ttf".to_string()),
        ("parent", "../Escape.ttf".to_string()),
        ("parent-backslash", "..\\Escape.ttf".to_string()),
        ("dot", ".".to_string()),
        ("dotdot", "..".to_string()),
        ("absolute", outside.to_string_lossy().into_owned()),
    ];

    for (code, name) in &cases {
        card.pack(code, Some(name));
        let mut ctx = card.ctx(code);
        assert_eq!(ctx.i18n.code(), *code);
        assert_eq!(
            ctx.fonts.len(),
            2,
            "{name:?}: a preference outside the font directories was registered"
        );
        assert_eq!(
            ctx.fonts.resolve('A'),
            Some(FontId(0)),
            "{name:?}: the default chain is not in front"
        );
        assert_eq!(
            ctx.fonts.resolve('가'),
            Some(FontId(1)),
            "{name:?}: Hangul came from the card's escape file"
        );
    }

    // The escape files are still there, untouched: refusing a name is not a filesystem act.
    assert!(outside.is_file());
    assert!(card.fonts_dir().join("sub").join("Escape.ttf").is_file());
}

#[test]
fn a_corrupt_preference_falls_through_without_stopping_the_context() {
    let card = Card::new("corrupt");
    card.pack("zz-broken", Some("Broken.ttf"));
    fs::write(card.fonts_dir().join("Broken.ttf"), b"this is not a font\n").unwrap();

    let mut ctx = card.ctx("zz-broken");
    assert_eq!(ctx.i18n.code(), "zz-broken");
    assert_eq!(
        ctx.fonts.len(),
        3,
        "a file that will not parse is still just a slot"
    );
    assert!(
        !ctx.fonts.is_loaded(FontId(0)),
        "the broken file was read at context creation"
    );

    // The first glyph that reaches it skips it and draws from the next font that has the
    // glyph: Latin from the embedded UI font, Hangul from the CJK fallback.
    assert_eq!(ctx.fonts.resolve('A'), Some(FontId(1)));
    assert_eq!(ctx.fonts.resolve('가'), Some(FontId(2)));
    assert!(
        !ctx.fonts.is_loaded(FontId(0)),
        "a file that failed to parse was parsed again"
    );

    // A failed slot costs its own glyphs and nothing else: the characters still resolve, to
    // the fonts behind it — Latin to the embedded UI font, Hangul to the CJK fallback — and
    // the line box and the drawn pixels come from those same fonts. Nothing here is allowed to
    // hide the broken slot instead: it is registered as the pack asked, and the chain's own
    // fallback is what makes the screen readable.
    let metrics = ctx.fonts.measure("Hello 세계", 16.0);
    assert!(
        metrics.line_height > 0,
        "a broken first slot flattened the line box"
    );
    assert!(metrics.ascent > 0.0 && metrics.descent > 0.0, "{metrics:?}");
    assert!(metrics.width > 0.0, "the line measured as nothing at all");
    let line = ctx.fonts.rasterize("Hello 세계", 16.0);
    assert_eq!(line.height, metrics.line_height);
    assert!(line.ink() > 0, "nothing was drawn behind the broken slot");
}

#[test]
fn a_request_that_did_not_load_uses_english_fonts_not_its_own() {
    let card = Card::new("fallback");
    card.bring_font("CardPreferred.ttf", NOTO);
    // The pack names a font this card really carries, and then will not parse.
    fs::write(
        card.lang_dir().join("zz-bad.ftl"),
        "lang-font = CardPreferred.ttf\nthis is not = = valid\n{{{\n",
    )
    .unwrap();

    let mut ctx = card.ctx("zz-bad");
    assert_eq!(ctx.i18n.code(), "en", "the request did not load");
    assert_eq!(
        ctx.i18n.font(),
        None,
        "the preference of a pack nobody is reading"
    );
    assert_eq!(
        ctx.fonts.len(),
        2,
        "the failed pack's font was registered anyway"
    );
    assert_eq!(ctx.fonts.resolve('A'), Some(FontId(0)));
    assert_eq!(ctx.fonts.resolve('가'), Some(FontId(1)));
}
