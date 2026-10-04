//! The About sticker inside the app: the shelf's menu opening it, the build information the
//! frontend hands it, and the ways back.
//!
//! No core, ROM or GL window is involved: the sticker belongs to the shelf, so the whole file runs
//! against `RecordingCanvas` and an empty card. Nothing here needs a clock of its own — the offset
//! is only read to prove the sticker does not touch it.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas, TexId};
use slot2_i18n::Arg;
use slot2_input::{Button, Event};
use slot2_platform::by_target;
use slot2_store::Card;
use slot2_ui::about_sticker::{BOX_H, BOX_W, TARGET_KEY, VERSION_KEY};
use slot2_ui::hud::{HUD_H, HUD_MARGIN};
use slot2_ui::shelf_menu::{BOX_H as SHELF_H, BOX_W as SHELF_W};
use slot2_ui::splash::BACKDROP;
use slot2_ui::timezone_menu::{BOX_H as TZ_H, BOX_W as TZ_W, DIM};
use slot2_ui::{face, AboutSticker, ShelfAvailability, ShelfChoice, ShelfMenu, UiCtx, PX_BODY};

use slot2::app::{App, Screen};

/// The rows this build can open, which is the one literal `App::shelf_availability` holds.
fn availability() -> ShelfAvailability {
    ShelfAvailability {
        language: true,
        time_zone: true,
        about: true,
        ..Default::default()
    }
}

/// The settings menu as it is left after the player walks to the About row, which is one press up
/// from where it opens.
fn on_about_row() -> ShelfMenu {
    let mut menu = ShelfMenu::new(availability());
    menu.up();
    menu
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

fn tap(a: &mut App, b: Button, at: Instant) -> Instant {
    a.feed(&ev(b, true, at));
    let released = at + ms(40);
    a.feed(&ev(b, false, released));
    a.tick(released + ms(20));
    released
}

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-about-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card in a fresh temporary folder holding the bytes a hand-edited settings file would.
fn card_with(tag: &str, file: Option<&[u8]>) -> Card {
    let card = Card::new(scratch(tag));
    card.ensure_layout();
    if let Some(bytes) = file {
        fs::write(card.settings_path(), bytes).unwrap();
    }
    card
}

/// An app on the shelf, with no core directory and no cart: the sticker is the shelf's own.
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

/// A context for a panel. The app owns none: `draw` is handed the profile of the machine it is
/// drawing for, which is where the sticker's target comes from.
fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn frame(a: &mut App, ctx: &mut UiCtx) -> RecordingCanvas {
    let (w, h) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    a.draw(&mut c, ctx, Instant::now());
    c
}

/// The settings menu the app is on, whichever shelf screen carries it.
fn parent_menu(a: &App) -> Option<ShelfMenu> {
    match a.screen {
        Screen::Shelf(menu) => Some(menu),
        Screen::Timezone(parent, _) | Screen::About(parent, _) => Some(parent),
        _ => None,
    }
}

/// Everything the frame draws, without the one-off texture uploads, in order: what the player
/// sees. An upload is bookkeeping — the first frame of a screen has some and the next does not.
fn marks(ops: &[Op]) -> Vec<Op> {
    ops.iter()
        .filter(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
        .cloned()
        .collect()
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

/// The texture ids the frame drew, in order.
fn tex_ids(ops: &[Op]) -> Vec<TexId> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Image { tex, .. } => Some(*tex),
            _ => None,
        })
        .collect()
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

#[test]
fn the_about_row_opens_the_sticker_the_build_actually_is() {
    let card = card_with("shelf", Some(b"volume = 30\n"));
    let original = fs::read(card.settings_path()).unwrap();

    // 1. The tap opens the settings menu, and this build can open the language picker, the time
    // zone and About — nothing else.
    let mut a = app(&card);
    let mut now = Instant::now();
    assert_eq!(a.screen, Screen::List);
    now = tap(&mut a, Button::Menu, now);
    assert_eq!(
        a.screen,
        Screen::Shelf(ShelfMenu::new(availability())),
        "the tap did not open the settings menu on this build's rows"
    );
    for choice in [
        ShelfChoice::Language,
        ShelfChoice::DisplayDefaults,
        ShelfChoice::BootLogo,
        ShelfChoice::Sync,
        ShelfChoice::TimeZone,
        ShelfChoice::About,
    ] {
        let want = matches!(
            choice,
            ShelfChoice::Language | ShelfChoice::TimeZone | ShelfChoice::About
        );
        assert_eq!(
            parent_menu(&a).unwrap().is_available(choice),
            want,
            "{choice:?} is available the wrong way round"
        );
    }
    assert_eq!(
        parent_menu(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::Language),
        "the first openable row is not the one the menu opens on"
    );

    // 2. Up and down walk the three openable rows and nothing else, both ways: up from the first
    // row wraps to the last, and the walk comes back through the middle.
    for (button, want) in [
        (Button::Up, ShelfChoice::About),
        (Button::Up, ShelfChoice::TimeZone),
        (Button::Down, ShelfChoice::About),
        (Button::Down, ShelfChoice::Language),
    ] {
        now = tap(&mut a, button, now);
        let menu = parent_menu(&a).expect("the settings menu is on screen");
        let selected = menu.selected().expect("there are openable rows");
        assert_eq!(selected, want, "{button:?} landed on the wrong row");
        assert!(
            menu.is_available(selected),
            "{button:?} highlighted a row with nothing behind it"
        );
    }

    // 3. A on the About row opens the sticker, with the settings menu it came from still on that
    // row.
    now = tap(&mut a, Button::Up, now); // Language → About
    now = tap(&mut a, Button::A, now);
    assert_eq!(
        a.screen,
        Screen::About(on_about_row(), AboutSticker::new()),
        "A on the About row did not open the sticker"
    );
    match a.screen {
        Screen::About(parent, _) => assert_eq!(parent.selected(), Some(ShelfChoice::About)),
        other => panic!("A on the About row opened {other:?}"),
    }

    // 4. B and the Menu tap both come back to the same About row.
    for closer in [Button::B, Button::Menu] {
        now = tap(&mut a, closer, now);
        match a.screen {
            Screen::Shelf(menu) => assert_eq!(
                menu.selected(),
                Some(ShelfChoice::About),
                "{closer:?} lost the row it was opened from"
            ),
            other => panic!("{closer:?} landed on {other:?}"),
        }
        now = tap(&mut a, Button::A, now);
        assert!(
            matches!(a.screen, Screen::About(..)),
            "{closer:?}: could not reopen the sticker"
        );
    }

    // 5. Nothing on the sticker is a control: A and the four directions change no screen, no
    // clock, no volume, no file and nothing said.
    let clock_before = slot2_platform::clock::utc_offset_min();
    let volume_before = a.volume.level();
    for button in [
        Button::A,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
    ] {
        now = tap(&mut a, button, now);
        assert!(
            matches!(a.screen, Screen::About(..)),
            "{button:?} left the sticker"
        );
        assert_eq!(a.toast_key(), None, "{button:?} said something");
        assert_eq!(
            slot2_platform::clock::utc_offset_min(),
            clock_before,
            "{button:?} moved the clock"
        );
        assert_eq!(
            a.volume.level(),
            volume_before,
            "{button:?} moved the volume"
        );
        assert_eq!(
            fs::read(card.settings_path()).unwrap(),
            original,
            "{button:?} wrote the card"
        );
    }

    // 6. The shelf under the sticker is the list's own shelf, and the sticker is the only panel
    // over it.
    let mut d = app(&card);
    let mut ctx = ctx("rgsp", "en");
    let (pw, ph) = ctx.profile.geometry.size();
    let (pw, ph) = (pw as f32, ph as f32);
    d.tick(Instant::now());
    d.screen = Screen::List;
    let list = frame(&mut d, &mut ctx);
    let mut at = Instant::now();
    at = tap(&mut d, Button::Menu, at);
    at = tap(&mut d, Button::Up, at); // TimeZone → About
    let shelf = frame(&mut d, &mut ctx);
    tap(&mut d, Button::A, at);
    let about = frame(&mut d, &mut ctx);

    for (case, c) in [("list", &list), ("shelf", &shelf), ("about", &about)] {
        assert!(
            !c.ops.iter().any(|o| matches!(o, Op::Clear(_))),
            "{case}: the frame cleared the panel"
        );
    }
    let list_marks = marks(&list.ops);
    let about_marks = marks(&about.ops);
    let dim_at = about_marks
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, w, h, color }
                if *x == 0.0 && *y == 0.0 && (*w - pw).abs() < 0.5 && (*h - ph).abs() < 0.5
                    && *color == DIM)
        })
        .expect("the sticker has no dim");
    assert!(dim_at > 0, "the sticker was the first thing drawn");
    assert_eq!(
        list_marks[..dim_at],
        about_marks[..dim_at],
        "the shelf under the sticker is not the list's own shelf"
    );
    assert_eq!(panels(&about.ops, BOX_W, BOX_H), 1, "no sticker panel");
    assert_eq!(
        panels(&about.ops, SHELF_W, SHELF_H),
        0,
        "the parent settings panel is drawn behind the sticker"
    );
    assert_eq!(
        panels(&about.ops, TZ_W, TZ_H),
        0,
        "an offset panel is drawn behind the sticker"
    );

    // 7. What the sticker prints is the frontend's own package version and the profile it was
    // drawn for. A face is cached under its exact string, so asking the same cache for the
    // expected sentence finds it already there; a placeholder, or the UI crate's own version, or
    // some other machine's target, would have left that sentence undrawn and the cache longer.
    let mut probe = RecordingCanvas::new(1, 1);
    let drawn = tex_ids(&about.ops);
    let mut cached = |ctx: &mut UiCtx, key: &str, name: &str, value: &str| {
        let text = ctx.i18n.t_args(key, &[(name, Arg::from(value))]);
        let before = ctx.faces.len();
        let f = face::face(&mut probe, ctx, &text, PX_BODY);
        (ctx.faces.len() == before, f.tex)
    };

    let target = ctx.profile.target;
    let (was_drawn, tex) = cached(&mut ctx, VERSION_KEY, "version", env!("CARGO_PKG_VERSION"));
    assert!(
        was_drawn,
        "the sticker did not print this package's version"
    );
    assert!(
        drawn.contains(&tex),
        "the version on screen is not the sentence the pack makes of it"
    );
    assert!(
        !cached(&mut ctx, VERSION_KEY, "version", "9.9.9").0,
        "a version nobody passed is on screen"
    );
    let (was_drawn, tex) = cached(&mut ctx, TARGET_KEY, "target", target);
    assert!(
        was_drawn,
        "the sticker did not print the target it was drawn for"
    );
    assert!(
        drawn.contains(&tex),
        "the target on screen is not the sentence the pack makes of it"
    );
    assert!(
        !cached(&mut ctx, TARGET_KEY, "target", "rg-some-other").0,
        "a target nobody passed is on screen"
    );
    assert!(
        !cached(
            &mut ctx,
            TARGET_KEY,
            "target",
            by_target("rg35xxsp").unwrap().target
        )
        .0,
        "the sticker printed a panel other than the one it was drawn for"
    );

    // 8. The corner furniture stays on all three screens, and the sticker is not a game menu: it
    // has no sound to pause.
    assert!(hud_quads(&list) > 0, "no HUD on the list");
    assert_eq!(
        hud_quads(&shelf),
        hud_quads(&list),
        "the settings menu hid the HUD"
    );
    assert_eq!(
        hud_quads(&about),
        hud_quads(&list),
        "the sticker hid the HUD"
    );
    assert!(!d.audio_paused(), "the sticker paused a game's sound");
}
