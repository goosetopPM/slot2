//! The language picker inside the app: which languages the machine can really speak, what the
//! player chooses, and the one moment a language actually changes.
//!
//! No core, ROM or GL window is involved. The card lives in a temporary folder, the context is
//! built exactly as the two backends build theirs, and the change is applied through the same
//! `service_language_request` they call once per frame.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_i18n::I18n;
use slot2_input::{Button, Event};
use slot2_platform::by_target;
use slot2_store::{Card, Ini};
use slot2_ui::hud::{HUD_H, HUD_MARGIN};
use slot2_ui::language_picker::{BOX_H as PICKER_H, BOX_W as PICKER_W};
use slot2_ui::shelf_menu::{BOX_H as SHELF_H, BOX_W as SHELF_W};
use slot2_ui::splash::BACKDROP;
use slot2_ui::{face, ShelfAvailability, ShelfChoice, ShelfMenu, UiCtx, PX_BODY};

use slot2::app::{App, Screen};

/// A valid card-only pack, with a name of its own.
const CARD_PACK: &str = "lang-name = Card Only\npower-off = Card Power\n";
/// A pack that will not parse.
const BROKEN_PACK: &str = "this is not = = valid\n{{{\n";

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

fn tap(a: &mut App, b: Button, at: Instant) -> Instant {
    a.feed(&ev(b, true, at));
    let released = at + ms(40);
    a.feed(&ev(b, false, released));
    a.tick(released + ms(20));
    released
}

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-langapp-{tag}-{}", std::process::id()));
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

fn lang_dir(card: &Card) -> PathBuf {
    card.root().join("System").join("Lang")
}

/// A pack the player put on the card by hand.
fn pack(card: &Card, code: &str, text: &str) -> PathBuf {
    let dir = lang_dir(card);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{code}.ftl"));
    fs::write(&path, text).unwrap();
    path
}

/// An app on the shelf, with no core directory and no cart: the picker is the shelf's own.
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

/// The build's own font directory, as `crate::font_dirs` places it after the card's.
fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts")
}

/// The startup context, built the way both backends build it.
fn ui_ctx(card: &Card, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(
        by_target("rgsp").unwrap(),
        lang,
        vec![fonts],
        Some(&lang_dir(card)),
    )
}

/// What the backends do once at startup: tell the app which language the context really loaded.
fn start(card: &Card, requested: &str) -> (App, UiCtx) {
    let mut app = app(card);
    let ctx = ui_ctx(card, requested);
    app.set_current_language(ctx.i18n.code());
    (app, ctx)
}

fn frame(a: &mut App, ctx: &mut UiCtx) -> RecordingCanvas {
    let (w, h) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    a.draw(&mut c, ctx, Instant::now());
    c
}

fn shelf_of(a: &App) -> Option<ShelfMenu> {
    match a.screen {
        Screen::Shelf(menu) => Some(menu),
        Screen::Language(parent) => Some(parent),
        Screen::Timezone(parent, _) => Some(parent),
        Screen::About(parent, _) => Some(parent),
        _ => None,
    }
}

/// How many rects of that size and colour the frame draws.
fn panels(ops: &[Op], w: f32, h: f32) -> usize {
    ops.iter()
        .filter(|o| {
            matches!(o, Op::Rect { w: rw, h: rh, color, .. }
                if (*rw - w).abs() < 0.5 && (*rh - h).abs() < 0.5 && *color == BACKDROP)
        })
        .count()
}

/// The quads drawn in the band the corner furniture is pinned to.
fn hud_quads(c: &RecordingCanvas) -> usize {
    c.ops
        .iter()
        .filter(|o| match o {
            Op::Rect { y, h, .. } | Op::Image { y, h, .. } => {
                *y >= HUD_MARGIN - 1.0 && y + h <= HUD_MARGIN + HUD_H + 1.0
            }
            _ => false,
        })
        .count()
}

fn uploads(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count()
}

fn marks(ops: &[Op]) -> Vec<Op> {
    ops.iter()
        .filter(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
        .cloned()
        .collect()
}

/// The bytes and the last modification time of the settings file, or of whatever is in its place.
fn settings_stamp(card: &Card) -> (Option<Vec<u8>>, Option<std::time::SystemTime>) {
    let path = card.settings_path();
    if path.is_dir() {
        return (None, None);
    }
    let bytes = fs::read(&path).ok();
    let mtime = fs::metadata(&path).ok().and_then(|m| m.modified().ok());
    (bytes, mtime)
}

/// Whether a text run is drawn, whole, in the band — at the width it measures.
fn text_in(ops: &[Op], ctx: &mut UiCtx, text: &str, px: f32, band: (f32, f32)) -> bool {
    let want = face::measure(ctx, text, px);
    ops.iter().any(|o| match o {
        Op::Image { y, w, .. } => (*w - want).abs() < 0.5 && *y >= band.0 && *y < band.0 + band.1,
        _ => false,
    })
}

/// Open the settings menu and then the language row.
fn open_language(a: &mut App, at: Instant) -> Instant {
    let at = tap(a, Button::Menu, at);
    tap(a, Button::A, at)
}

// ---------------------------------------------------------------- discovery

#[test]
fn the_language_row_lists_what_a_card_can_really_speak() {
    let card = card_with("discover", Some("volume = 30\n"));
    pack(&card, "xx-card", CARD_PACK);
    let (mut a, ctx) = start(&card, "en");
    let mut now = Instant::now();

    // 1. The settings menu offers the language row, the time zone and About.
    assert_eq!(a.screen, Screen::List);
    now = tap(&mut a, Button::Menu, now);
    assert_eq!(
        a.screen,
        Screen::Shelf(ShelfMenu::new(ShelfAvailability {
            language: true,
            time_zone: true,
            about: true,
            ..Default::default()
        })),
        "the settings menu is not this build's rows"
    );
    assert_eq!(
        shelf_of(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::Language),
        "the menu does not open on its first openable row"
    );

    // 2. A opens the picker on the languages that actually load, in the order `available` gave
    // them, each named by its own pack.
    now = tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Language(_)), "{:?}", a.screen);
    let dir = lang_dir(&card);
    let codes = I18n::available(Some(&dir));
    assert_eq!(
        codes,
        vec!["en".to_string(), "ko".to_string(), "xx-card".to_string()],
        "the built-ins and the card's own pack are not what is on offer"
    );
    let picker = a.language_picker();
    assert_eq!(
        picker
            .options()
            .iter()
            .map(|o| o.code().to_owned())
            .collect::<Vec<String>>(),
        codes,
        "the picker's order is not the discovery order"
    );
    assert_eq!(picker.options()[0].name(), "English");
    assert_eq!(picker.options()[1].name(), "한국어");
    assert_eq!(picker.options()[2].name(), "Card Only", "a pack's own name");
    // The badge is the language the context really loaded, which the backend told the app.
    assert_eq!(
        picker.current().map(|o| o.code().to_owned()),
        Some(ctx.i18n.code().to_owned())
    );
    assert_eq!(a.current_language(), ctx.i18n.code());

    // 3. A pack that will not parse, and a file that is not a pack at all, are not languages.
    pack(&card, "xx-broken", BROKEN_PACK);
    fs::write(lang_dir(&card).join("notes.txt"), "ignored\n").unwrap();
    now = tap(&mut a, Button::B, now);
    assert!(matches!(a.screen, Screen::Shelf(_)), "B left the picker");
    now = tap(&mut a, Button::A, now);
    let codes: Vec<String> = a
        .language_picker()
        .options()
        .iter()
        .map(|o| o.code().to_owned())
        .collect();
    assert!(
        !codes.contains(&"xx-broken".to_string()),
        "a pack that will not parse is on offer: {codes:?}"
    );
    assert!(!codes.contains(&"notes".to_string()));

    // 4. A card's broken copy of English is not a reason to lose English: the built-in one is
    // what the frontend can always speak.
    pack(&card, "en", BROKEN_PACK);
    now = tap(&mut a, Button::B, now);
    tap(&mut a, Button::A, now);
    let picker = a.language_picker();
    let en = picker
        .options()
        .iter()
        .find(|o| o.code() == "en")
        .expect("English was lost with the card's broken copy of it");
    assert_eq!(en.name(), "English");
    assert_eq!(a.current_language(), "en");

    // 5. What the app guesses is the running language is what the backend told it, not the code
    // the card asks for.
    let (mut b, ctx) = start(&card_with("guessed", Some("language = ko\n")), "en");
    b.set_current_language(ctx.i18n.code());
    assert_eq!(b.current_language(), "en");
    let now = Instant::now();
    open_language(&mut b, now);
    assert_eq!(
        b.language_picker().current().map(|o| o.code().to_owned()),
        Some("en".to_string()),
        "the badge is the requested language rather than the running one"
    );
    assert!(!b.language_picker().changed());
}

// ---------------------------------------------------------------- walking it

#[test]
fn walking_the_picker_writes_nothing_until_a_language_is_chosen() {
    let card = card_with("walk", Some("volume = 30\n"));
    pack(&card, "xx-card", CARD_PACK);
    let (mut a, mut ctx) = start(&card, "en");
    let stamp = settings_stamp(&card);
    let mut now = Instant::now();

    now = open_language(&mut a, now);
    assert!(matches!(a.screen, Screen::Language(_)));
    // Up and down move the highlight and nothing else: no request, no write, no new context.
    now = tap(&mut a, Button::Down, now);
    now = tap(&mut a, Button::Up, now);
    now = tap(&mut a, Button::Up, now);
    assert!(a.pending_language().is_none(), "walking queued a request");
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(a.current_language(), "en");
    assert_eq!(settings_stamp(&card), stamp, "walking touched the card");
    assert!(!slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));

    // B leaves the picker with nothing written and nothing queued.
    now = tap(&mut a, Button::B, now);
    assert_eq!(
        shelf_of(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::Language),
        "B did not land back on the language row"
    );
    assert!(a.pending_language().is_none());
    assert_eq!(settings_stamp(&card), stamp, "leaving touched the card");

    // And a language that is already running is not a change: A goes straight back.
    now = tap(&mut a, Button::A, now);
    assert!(!a.language_picker().changed());
    tap(&mut a, Button::A, now);
    assert!(
        matches!(a.screen, Screen::Shelf(_)),
        "the running language was treated as a change: {:?}",
        a.screen
    );
    assert!(a.pending_language().is_none());
    assert_eq!(
        settings_stamp(&card),
        stamp,
        "an unchanged choice wrote the card"
    );
    assert_eq!(a.toast_key(), None);
}

// ---------------------------------------------------------------- changing it

#[test]
fn choosing_a_language_loads_saves_and_swaps_in_that_order() {
    let card = card_with("swap", Some("volume = 30\nfuture_key = something\n"));
    let (mut a, mut ctx) = start(&card, "en");
    let fonts = ctx.font_dirs.clone();
    let profile = ctx.profile;
    let mut now = Instant::now();

    now = open_language(&mut a, now);
    now = tap(&mut a, Button::Down, now); // en → ko
    assert_eq!(
        a.language_picker()
            .highlighted()
            .map(|o| o.code().to_owned()),
        Some("ko".to_string())
    );
    assert!(a.language_picker().changed());

    // A queues the choice and leaves the screen alone: the backend applies it before the frame
    // that would draw the new language.
    now = tap(&mut a, Button::A, now);
    assert_eq!(a.pending_language(), Some("ko"));
    assert!(matches!(a.screen, Screen::Language(_)), "{:?}", a.screen);
    assert_eq!(
        a.current_language(),
        "en",
        "the choice ran ahead of the context"
    );
    assert_eq!(ctx.i18n.code(), "en");

    // The service builds the candidate, saves, and only then hands the context over.
    assert!(slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));
    assert_eq!(ctx.i18n.code(), "ko", "the context did not change");
    assert_eq!(
        a.current_language(),
        "ko",
        "the app still thinks it is English"
    );
    assert_eq!(a.pending_language(), None, "the request was not consumed");
    assert_eq!(
        shelf_of(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::Language),
        "the app did not land back on the language row"
    );
    assert_eq!(card.read_language(), "ko");
    assert_eq!(a.toast_key(), None, "a change that worked said something");

    // The machine's own facts are carried over, and the old language's faces are gone: the new
    // context was built fresh rather than re-pointed.
    assert_eq!(
        ctx.profile, profile,
        "the panel profile changed with the language"
    );
    assert_eq!(
        ctx.font_dirs, fonts,
        "the font directories changed with the language"
    );
    assert!(
        ctx.faces.is_empty(),
        "the previous language's face cache came along"
    );
    assert_eq!(
        ctx.i18n.t("power-off"),
        "전원 끄기",
        "the new context is not Korean"
    );

    // Applied once: a second service call has nothing to do.
    assert!(!slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));

    // The saved file kept every key this screen does not own, and the language is its own.
    let ini = Ini::load(&card.settings_path()).unwrap();
    assert_eq!(ini.get("language"), Some("ko"));
    assert_eq!(ini.get("volume"), Some("30"));
    assert_eq!(ini.get("future_key"), Some("something"));

    // Going back to English removes its key and nothing else. The settings menu is already on
    // the language row, so A is what opens the picker again.
    now = tap(&mut a, Button::A, now);
    now = tap(&mut a, Button::Up, now); // ko → en
    tap(&mut a, Button::A, now);
    assert!(slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(a.current_language(), "en");
    let ini = Ini::load(&card.settings_path()).unwrap();
    assert_eq!(
        ini.get("language"),
        None,
        "English is the absence of the key"
    );
    assert_eq!(ini.get("volume"), Some("30"));
    assert_eq!(ini.get("future_key"), Some("something"));
}

#[test]
fn a_card_only_language_can_be_chosen_too() {
    let card = card_with("card-only", Some("volume = 30\n"));
    pack(&card, "xx-card", CARD_PACK);
    let (mut a, mut ctx) = start(&card, "en");
    let mut now = Instant::now();

    now = open_language(&mut a, now);
    now = tap(&mut a, Button::Down, now); // en → ko
    now = tap(&mut a, Button::Down, now); // ko → xx-card
    assert_eq!(
        a.language_picker()
            .highlighted()
            .map(|o| o.code().to_owned()),
        Some("xx-card".to_string())
    );
    tap(&mut a, Button::A, now);
    assert!(slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));
    assert_eq!(ctx.i18n.code(), "xx-card");
    assert_eq!(a.current_language(), "xx-card");
    assert_eq!(card.read_language(), "xx-card");
    assert_eq!(ctx.i18n.t("power-off"), "Card Power");
    assert_eq!(
        ctx.i18n.t("resume"),
        "Resume",
        "a key the card pack does not define comes from English"
    );
    assert!(matches!(a.screen, Screen::Shelf(_)));
}

// ---------------------------------------------------------------- the card's own language

/// The M5 Acceptance scenario, at the App boundary: a card that ships one `System/Lang/ja.ftl`
/// of three messages gets a language row that really speaks, and choosing it swaps the running
/// context and writes the code to the card.
#[test]
fn a_card_only_japanese_pack_is_offered_chosen_and_running() {
    let card = card_with("ja-acceptance", Some("volume = 30\n"));
    // Exactly the three messages a translator would put on a card, and nothing else.
    pack(
        &card,
        "ja",
        "lang-name = 日本語\npower-off = 電源を切る\nresume = 続ける\n",
    );
    let (mut a, mut ctx) = start(&card, "en");
    let mut now = Instant::now();

    // 1. The picker lists both built-ins and the card's own language, in the plain code order,
    // and the row is the name the pack gives itself.
    now = open_language(&mut a, now);
    assert!(matches!(a.screen, Screen::Language(_)), "{:?}", a.screen);
    let codes: Vec<String> = a
        .language_picker()
        .options()
        .iter()
        .map(|o| o.code().to_owned())
        .collect();
    assert_eq!(
        codes,
        vec!["en".to_string(), "ja".to_string(), "ko".to_string()],
        "the card's pack is not on the list where it belongs"
    );
    let ja_name = a
        .language_picker()
        .options()
        .iter()
        .find(|o| o.code() == "ja")
        .map(|o| o.name());
    assert_eq!(
        ja_name,
        Some("日本語"),
        "the row is not the pack's own name"
    );

    // 2. Choosing it queues one request, and the shared service the two backends call applies
    // it before the next frame.
    now = tap(&mut a, Button::Down, now); // en → ja
    assert_eq!(
        a.language_picker()
            .highlighted()
            .map(|o| o.code().to_owned()),
        Some("ja".to_string())
    );
    tap(&mut a, Button::A, now);
    assert_eq!(a.pending_language(), Some("ja"));
    assert!(slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));

    // 3. The context, the app and the card all agree on the language.
    assert_eq!(ctx.i18n.code(), "ja", "the context did not change");
    assert_eq!(a.current_language(), "ja");
    assert_eq!(card.read_language(), "ja");

    // 4. What the pack defines is Japanese; what it does not define comes from English.
    assert_eq!(ctx.i18n.t("power-off"), "電源を切る");
    assert_eq!(ctx.i18n.t("resume"), "続ける");
    assert_eq!(
        ctx.i18n.t("shelf-language"),
        "Language",
        "a key the pack lacks did not fall back to English"
    );

    // 5. The success contract: back on the language row, and nothing said about a failure.
    assert_eq!(
        shelf_of(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::Language),
        "the app did not land back on the language row: {:?}",
        a.screen
    );
    assert_eq!(a.toast_key(), None, "a change that worked said something");
}

// ---------------------------------------------------------------- the pack's own font

/// A chosen language brings the font its pack asks for, and a preference the card does not
/// have is not a reason to refuse the change.
#[test]
fn a_change_takes_the_pack_s_font_and_survives_a_missing_one() {
    let card = card_with("pack-font", Some("volume = 30\n"));
    pack(
        &card,
        "xx-font",
        "lang-name = Card Only\npower-off = Card Power\nlang-font = CardPreferred.ttf\n",
    );
    pack(
        &card,
        "zz-nofont",
        "lang-name = No Font\npower-off = No Font Power\nlang-font = Missing.ttf\n",
    );
    // The card carries the font the first pack names, and nothing for the second.
    let fonts = card.root().join("System").join("Fonts");
    fs::create_dir_all(&fonts).unwrap();
    let noto = assets_dir().join("NotoSansKR-Regular.otf");
    assert!(noto.is_file(), "the build's CJK font is missing");
    fs::copy(&noto, fonts.join("CardPreferred.ttf")).unwrap();

    // The context a backend builds: the card's own `System/Fonts` first, the assets after.
    let mut a = app(&card);
    let mut ctx = UiCtx::new(
        by_target("rgsp").unwrap(),
        "en",
        vec![fonts.clone(), assets_dir()],
        Some(&lang_dir(&card)),
    );
    a.set_current_language(ctx.i18n.code());
    let mut now = Instant::now();
    now = open_language(&mut a, now);
    now = tap(&mut a, Button::Down, now); // en → ko
    now = tap(&mut a, Button::Down, now); // ko → xx-font
    now = tap(&mut a, Button::A, now);
    // The context this change replaces has faces of its own, and it does not keep them.
    let _ = frame(&mut a, &mut ctx);
    assert!(!ctx.faces.is_empty(), "the old context cached nothing");

    assert!(slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));
    assert_eq!(ctx.i18n.code(), "xx-font");
    assert_eq!(ctx.i18n.font().as_deref(), Some("CardPreferred.ttf"));
    assert!(ctx.faces.is_empty(), "the old language's faces came along");
    assert_eq!(
        ctx.fonts.len(),
        3,
        "the card's font, the UI font and the CJK fallback"
    );
    assert!(
        !ctx.fonts.is_loaded(slot2_text::FontId(0)),
        "the change parsed the card's font instead of registering it"
    );
    // The card's font is the chain's first slot, so it is what draws: the Hangul a default
    // chain would have sent to the CJK fallback does not fall through.
    let latin = ctx.fonts.resolve('A');
    let hangul = ctx.fonts.resolve('가');
    assert_eq!(hangul, latin, "the card's font is not the first slot");

    // And a pack whose font the card does not have still changes the language: the new context
    // is the default chain rather than the change being refused. The settings menu is still on
    // the language row, so A opens the picker again.
    now = tap(&mut a, Button::A, now);
    now = tap(&mut a, Button::Down, now); // xx-font → zz-nofont
    tap(&mut a, Button::A, now);
    assert_eq!(a.pending_language(), Some("zz-nofont"));
    assert!(slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));
    assert_eq!(ctx.i18n.code(), "zz-nofont");
    assert_eq!(ctx.i18n.t("power-off"), "No Font Power");
    assert!(ctx.faces.is_empty());
    assert_eq!(ctx.fonts.len(), 2, "a missing preference became a slot");
    let latin = ctx.fonts.resolve('A');
    let hangul = ctx.fonts.resolve('가');
    assert_ne!(
        hangul, latin,
        "Hangul must be the CJK fallback's again, not the first slot's"
    );
}

// ---------------------------------------------------------------- what can go wrong

#[test]
fn a_pack_that_vanishes_before_the_change_keeps_the_running_language() {
    let card = card_with("vanished", Some("language = en\nvolume = 30\n"));
    let path = pack(&card, "xx-card", CARD_PACK);
    let (mut a, mut ctx) = start(&card, "en");
    let stamp = settings_stamp(&card);
    let mut now = Instant::now();

    now = open_language(&mut a, now);
    now = tap(&mut a, Button::Down, now); // en → ko
    now = tap(&mut a, Button::Down, now); // ko → xx-card

    // The card changed under the picker: the pack is gone before the choice is applied.
    fs::remove_file(&path).unwrap();
    now = tap(&mut a, Button::A, now);
    assert_eq!(a.pending_language(), Some("xx-card"));

    // While the backend has the request in hand the picker takes no more input: the presses
    // are dropped rather than queued behind a change that is already on its way.
    now = tap(&mut a, Button::B, now);
    now = tap(&mut a, Button::Up, now);
    assert_eq!(a.pending_language(), Some("xx-card"));
    assert!(
        matches!(a.screen, Screen::Language(_)),
        "a press closed a screen whose change was in flight: {:?}",
        a.screen
    );
    assert_eq!(
        a.language_picker()
            .highlighted()
            .map(|o| o.code().to_owned()),
        Some("xx-card".to_string()),
        "the picker moved under a request that was in flight"
    );

    assert!(
        !slot2::service_language_request(&mut a, &mut ctx, &lang_dir(&card)),
        "a language with no pack was applied"
    );
    assert_eq!(ctx.i18n.code(), "en", "the running context changed");
    assert_eq!(a.current_language(), "en");
    assert_eq!(settings_stamp(&card), stamp, "a failed load wrote the card");
    assert_eq!(
        a.toast_key(),
        Some("language-load-failed"),
        "a failed load said nothing"
    );
    match a.screen {
        Screen::Language(parent) => assert_eq!(parent.selected(), Some(ShelfChoice::Language)),
        other => panic!("a failed load left the screen: {other:?}"),
    }
    assert!(
        a.language_picker().current().is_some(),
        "the badge was lost with the failure"
    );
    // The request is gone, so the screen takes input again.
    assert!(a.pending_language().is_none());
    tap(&mut a, Button::Down, now);
    assert!(
        matches!(a.screen, Screen::Language(_)),
        "the picker stopped taking input"
    );
}

#[test]
fn a_save_that_cannot_land_keeps_the_running_language() {
    // Bytes that are not UTF-8: the store cannot read the file, so it will not write it either.
    let card = card_with("save-fails", None);
    fs::write(
        card.settings_path(),
        [0xffu8, 0xfe, b'l', b'=', 0x80, b'\n'],
    )
    .unwrap();
    let stamp = settings_stamp(&card);
    let (mut a, mut ctx) = start(&card, "en");
    let mut now = Instant::now();

    now = open_language(&mut a, now);
    now = tap(&mut a, Button::Down, now); // en → ko
    tap(&mut a, Button::A, now);

    assert!(
        !slot2::service_language_request(&mut a, &mut ctx, &lang_dir(&card)),
        "a change was applied although it could not be saved"
    );
    assert_eq!(
        ctx.i18n.code(),
        "en",
        "the context changed under a failed save"
    );
    assert_eq!(a.current_language(), "en");
    assert_eq!(
        settings_stamp(&card),
        stamp,
        "the damaged file was rewritten"
    );
    assert_eq!(
        a.toast_key(),
        Some("language-save-failed"),
        "a failed save said nothing"
    );
    assert!(matches!(a.screen, Screen::Language(_)));
    assert!(a.pending_language().is_none());
}

#[test]
fn a_folder_where_the_settings_file_belongs_is_a_save_failure_too() {
    let card = card_with("folder", None);
    fs::create_dir(card.settings_path()).unwrap();
    let (mut a, mut ctx) = start(&card, "en");
    let mut now = Instant::now();

    now = open_language(&mut a, now);
    now = tap(&mut a, Button::Down, now);
    tap(&mut a, Button::A, now);
    assert!(!slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(a.current_language(), "en");
    assert!(card.settings_path().is_dir(), "the folder was replaced");
    assert_eq!(a.toast_key(), Some("language-save-failed"));
}

#[test]
fn a_language_the_store_will_not_write_is_a_save_failure_and_not_a_panic() {
    // A pack whose file name is a language the frontend can load but the store will not store:
    // the code has a space in it, so it could never be a settings value.
    let card = card_with("unsafe", Some("volume = 30\n"));
    pack(&card, "x y", CARD_PACK);
    let stamp = settings_stamp(&card);
    let (mut a, mut ctx) = start(&card, "en");
    let mut now = Instant::now();

    now = open_language(&mut a, now);
    let codes: Vec<String> = a
        .language_picker()
        .options()
        .iter()
        .map(|o| o.code().to_owned())
        .collect();
    assert!(codes.contains(&"x y".to_string()), "{codes:?}");
    // Walk to it: en → ko → x y.
    now = tap(&mut a, Button::Down, now);
    now = tap(&mut a, Button::Down, now);
    assert_eq!(
        a.language_picker()
            .highlighted()
            .map(|o| o.code().to_owned()),
        Some("x y".to_string())
    );
    tap(&mut a, Button::A, now);

    assert!(!slot2::service_language_request(
        &mut a,
        &mut ctx,
        &lang_dir(&card)
    ));
    assert_eq!(ctx.i18n.code(), "en");
    assert_eq!(a.current_language(), "en");
    assert_eq!(
        settings_stamp(&card),
        stamp,
        "a refused code touched the card"
    );
    assert_eq!(a.toast_key(), Some("language-save-failed"));
    assert!(matches!(a.screen, Screen::Language(_)));
}

// ---------------------------------------------------------------- the screen itself

#[test]
fn the_language_screen_draws_its_own_panel_over_the_shelf() {
    let card = card_with("draw", Some("volume = 30\n"));
    pack(&card, "xx-card", CARD_PACK);
    let (mut a, mut ctx) = start(&card, "en");
    let mut now = Instant::now();

    now = tap(&mut a, Button::Menu, now);
    let shelf = frame(&mut a, &mut ctx);
    tap(&mut a, Button::A, now);
    let language = frame(&mut a, &mut ctx);
    assert!(matches!(a.screen, Screen::Language(_)));

    for (case, c) in [("shelf", &shelf), ("language", &language)] {
        assert!(
            !c.ops.iter().any(|o| matches!(o, Op::Clear(_))),
            "{case}: the frame cleared the panel"
        );
    }
    assert_eq!(
        panels(&language.ops, SHELF_W, SHELF_H),
        0,
        "the parent panel is drawn behind the picker"
    );
    assert_eq!(
        panels(&language.ops, PICKER_W, PICKER_H),
        1,
        "no picker panel"
    );
    assert_eq!(panels(&shelf.ops, SHELF_W, SHELF_H), 1, "no settings panel");
    assert_eq!(panels(&shelf.ops, PICKER_W, PICKER_H), 0);

    // The corner furniture stays, and a warm redraw of the same screen uploads nothing.
    assert!(hud_quads(&language) > 0, "the picker hid the HUD");
    let again = frame(&mut a, &mut ctx);
    assert_eq!(uploads(&again.ops), 0, "a warm redraw uploaded something");
    assert_eq!(
        marks(&again.ops),
        marks(&language.ops),
        "the picker redrew differently"
    );

    // The shelf under it is the list's own shelf, and the picker's title is the row's own name.
    let title = ctx.i18n.t("shelf-language");
    let (_, by) = picker_box_origin(&ctx);
    assert!(
        text_in(&language.ops, &mut ctx, &title, PX_BODY, (by + 16.0, 40.0)),
        "the picker has no title"
    );
}

/// The picker panel's own origin, asked of the UI crate rather than worked out here.
fn picker_box_origin(ctx: &UiCtx) -> (f32, f32) {
    slot2_ui::LanguagePicker::box_origin(ctx)
}
