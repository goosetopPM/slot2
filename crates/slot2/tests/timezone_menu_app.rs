//! The shelf settings menu and the time zone screen inside the app: each one opening the other,
//! the preview on the running clock, and the three ways out of the offset screen — a cancel, an
//! apply, and a save that could not land.
//!
//! The clock offset is process-global, so this file holds exactly one `#[test]`: two of them
//! would race over it. Every scenario runs in order inside it, and the offset the process started
//! with is put back at the end.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_i18n::I18n;
use slot2_input::{Button, Event};
use slot2_platform::clock;
use slot2_store::{Card, Ini, DEFAULT_VOLUME_LEVEL};
use slot2_ui::hud::{HUD_H, HUD_MARGIN};
use slot2_ui::shelf_menu::{BOX_H as SHELF_H, BOX_W as SHELF_W};
use slot2_ui::splash::BACKDROP;
use slot2_ui::timezone_menu::{BOX_H as TZ_H, BOX_W as TZ_W, DIM};
use slot2_ui::{ShelfAvailability, ShelfChoice, ShelfMenu, TimezoneMenu, UiCtx};

use slot2::app::{App, Screen};

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

/// Press and release `b`. The release is the moment the app acts on — the two shelf menus are
/// taps — and the tick afterwards lets the app finish what the release started.
fn tap(a: &mut App, b: Button, at: Instant) -> Instant {
    a.feed(&ev(b, true, at));
    let released = at + ms(40);
    a.feed(&ev(b, false, released));
    a.tick(released + ms(20));
    released
}

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-tzmenu-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card in a fresh temporary folder, holding the bytes a hand-edited settings file would.
fn card_with(tag: &str, file: Option<&[u8]>) -> Card {
    let card = Card::new(scratch(tag));
    card.ensure_layout();
    if let Some(bytes) = file {
        fs::write(card.settings_path(), bytes).unwrap();
    }
    card
}

/// An app on the shelf, with no core directory and no cart: neither menu belongs to a game, so
/// the shelf is all either of them needs.
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

fn ctx() -> UiCtx {
    UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None)
}

fn frame(a: &mut App, ctx: &mut UiCtx) -> RecordingCanvas {
    let (w, h) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    a.draw(&mut c, ctx, Instant::now());
    c
}

/// The settings menu the app is on: its own, or the one the offset screen was opened from.
fn parent_menu(a: &App) -> Option<ShelfMenu> {
    match a.screen {
        Screen::Shelf(menu) => Some(menu),
        Screen::Timezone(parent, _) => Some(parent),
        _ => None,
    }
}

fn offset_menu(a: &App) -> Option<TimezoneMenu> {
    match a.screen {
        Screen::Timezone(_, menu) => Some(menu),
        _ => None,
    }
}

/// The frame's marks: what the player sees, without the uploads that made a face or a mask
/// available. An upload is bookkeeping — the first frame of a screen has some and the next does
/// not — and only the marks are the picture.
fn marks(ops: &[Op]) -> Vec<&Op> {
    ops.iter()
        .filter(|o| {
            matches!(
                o,
                Op::Rect { .. } | Op::Image { .. } | Op::ImageEffect { .. } | Op::Clear(_)
            )
        })
        .collect()
}

/// How many menu panels of that size the frame draws. The two menus share a width and differ in
/// height, which is what tells one panel from the other.
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

#[test]
fn the_shelf_menu_and_the_time_zone_screen_are_one_round_trip() {
    let started_at = clock::utc_offset_min();

    // A card that already holds a level and a key from a version this one does not know. Both
    // belong to somebody else, and neither may move because the time zone was set.
    let card = card_with(
        "shelf",
        Some(b"language = ko\nvolume = 30\nfuture_key = something\n"),
    );
    let original = fs::read(card.settings_path()).unwrap();
    clock::set_utc_offset_min(0).unwrap();

    let mut a = app(&card);
    let mut now = Instant::now();

    // 1. The tap opens the settings menu, which is where the language, the time zone and About
    // are openable and the other three rows are not.
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
        "the tap did not open the settings menu"
    );
    // The menu opens on its first openable row, and up and down walk the three of them: the
    // three rows between and after them are never landed on.
    assert_eq!(
        parent_menu(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::Language),
        "the menu did not open on its first openable row"
    );
    now = tap(&mut a, Button::Up, now);
    assert_eq!(
        parent_menu(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::About),
        "up did not wrap to the last openable row"
    );
    now = tap(&mut a, Button::Down, now);
    now = tap(&mut a, Button::Down, now);
    assert_eq!(
        parent_menu(&a).and_then(|m| m.selected()),
        Some(ShelfChoice::TimeZone),
        "down did not walk back to the time zone"
    );

    // 2. The hold is still the power menu, and it does not leave the settings menu behind it: a
    // hold that has fired is not also a tap.
    {
        let mut power = app(&card);
        let t = Instant::now();
        power.feed(&ev(Button::Menu, true, t));
        power.tick(t + ms(700));
        assert!(
            matches!(power.screen, Screen::Power(_)),
            "the hold opened {:?}",
            power.screen
        );
        power.feed(&ev(Button::Menu, false, t + ms(800)));
        power.tick(t + ms(900));
        assert!(
            matches!(power.screen, Screen::Power(_)),
            "letting go of a held Menu opened {:?}",
            power.screen
        );
    }

    // 3. B and the Menu tap both come back to the shelf.
    for closer in [Button::B, Button::Menu] {
        let mut s = app(&card);
        let mut at = Instant::now();
        at = tap(&mut s, Button::Menu, at);
        assert!(matches!(s.screen, Screen::Shelf(_)), "{closer:?}: setup");
        at = tap(&mut s, closer, at);
        assert_eq!(s.screen, Screen::List, "{closer:?} left the settings menu");
    }

    // 4. A on the Time zone row opens the offset screen on the value the clock is running at.
    // The running value is set after the app is built: construction puts the card's own offset
    // on the clock, which is what a machine booting on an untouched card looks like.
    let mut s = app(&card);
    clock::set_utc_offset_min(540).unwrap();
    now = tap(&mut s, Button::Menu, now);
    now = tap(&mut s, Button::Down, now); // Language → TimeZone
    now = tap(&mut s, Button::A, now);
    match s.screen {
        Screen::Timezone(parent, menu) => {
            assert_eq!(parent.selected(), Some(ShelfChoice::TimeZone));
            assert_eq!(menu.original(), 540);
            assert_eq!(menu.selected(), 540);
            assert!(!menu.changed());
        }
        other => panic!("A on the Time zone row opened {other:?}"),
    }

    // 5. Every direction moves the menu and the running clock by the same amount, and the card
    // is not written to: what is on the clock is a preview and nothing else yet.
    for (button, want) in [
        (Button::Right, 555),
        (Button::Down, 495),
        (Button::Up, 555),
        (Button::Left, 540),
    ] {
        now = tap(&mut s, button, now);
        let menu = offset_menu(&s).unwrap_or_else(|| panic!("{button:?} left the offset screen"));
        assert_eq!(menu.selected(), want, "{button:?} moved the menu elsewhere");
        assert_eq!(menu.original(), 540, "{button:?} moved what the card holds");
        assert_eq!(
            clock::utc_offset_min(),
            want,
            "{button:?} did not reach the running clock"
        );
        assert_eq!(
            fs::read(card.settings_path()).unwrap(),
            original,
            "{button:?} wrote the card"
        );
    }
    assert_eq!(clock::utc_offset_min(), 540, "the walk did not come back");

    // 6. Leaving with B or Menu puts the clock back where the card is, writes nothing, and lands
    // on the settings menu still holding the Time zone row.
    for closer in [Button::B, Button::Menu] {
        now = tap(&mut s, Button::Up, now);
        assert_eq!(
            clock::utc_offset_min(),
            600,
            "the preview is not on the clock"
        );
        now = tap(&mut s, closer, now);
        assert_eq!(
            clock::utc_offset_min(),
            540,
            "{closer:?} left the preview standing on the clock"
        );
        assert_eq!(
            parent_menu(&s).and_then(|m| m.selected()),
            Some(ShelfChoice::TimeZone),
            "{closer:?} did not land back on the Time zone row"
        );
        assert_eq!(
            fs::read(card.settings_path()).unwrap(),
            original,
            "{closer:?} wrote the card"
        );
        assert_eq!(
            s.toast_key(),
            None,
            "{closer:?} said something about a cancel"
        );
        now = tap(&mut s, Button::A, now);
        assert!(offset_menu(&s).is_some(), "{closer:?}: could not reopen");
    }

    // 7. A without a change writes nothing, says nothing, and goes back.
    now = tap(&mut s, Button::A, now);
    assert!(
        matches!(s.screen, Screen::Shelf(_)),
        "an unchanged A left the screen: {:?}",
        s.screen
    );
    assert_eq!(s.toast_key(), None, "an unchanged A said something");
    assert_eq!(
        fs::read(card.settings_path()).unwrap(),
        original,
        "an unchanged A wrote the card"
    );
    assert_eq!(clock::utc_offset_min(), 540);

    // 8. A change lands on the card and stays on the clock, and everything this screen does not
    // own is still in the file afterwards.
    now = tap(&mut s, Button::A, now);
    now = tap(&mut s, Button::Right, now);
    assert_eq!(clock::utc_offset_min(), 555);
    tap(&mut s, Button::A, now);
    assert!(
        matches!(s.screen, Screen::Shelf(_)),
        "apply did not go back to the settings menu: {:?}",
        s.screen
    );
    assert_eq!(s.toast_key(), None, "a save that worked said something");
    assert_eq!(
        clock::utc_offset_min(),
        555,
        "the value on the card is not the one running"
    );
    assert_eq!(card.read_utc_offset_minutes(), 555);
    let saved = Ini::load(&card.settings_path()).unwrap();
    assert_eq!(saved.get("utc_offset_minutes"), Some("555"));
    assert_eq!(saved.get("volume"), Some("30"), "the level was lost");
    assert_eq!(saved.get("language"), Some("ko"), "the language was lost");
    assert_eq!(
        saved.get("future_key"),
        Some("something"),
        "a stranger's key was lost"
    );
    assert_eq!(card.read_global_settings().volume, 30);
    assert_eq!(s.volume.level(), 30, "the offset screen moved the volume");

    // 9. A save that cannot land: a settings file the store cannot read. Nothing is written, the
    // clock goes back to what the card holds, the menu goes back with it, and the screen stays
    // open on the offset with the message that says so.
    let damaged = [0xffu8, 0xfe, b'v', b'=', 0x80, b'\n'];
    let broken = card_with("broken", Some(&damaged));
    let system = broken.settings_path().parent().unwrap().to_path_buf();
    let entries = fs::read_dir(&system).unwrap().count();
    clock::set_utc_offset_min(0).unwrap();
    let mut f = app(&broken);
    let mut at = Instant::now();
    assert_eq!(
        clock::utc_offset_min(),
        0,
        "the app did not start on the card's unreadable value"
    );
    assert_eq!(f.volume.level(), DEFAULT_VOLUME_LEVEL);
    at = tap(&mut f, Button::Menu, at);
    at = tap(&mut f, Button::Down, at); // Language → TimeZone
    at = tap(&mut f, Button::A, at);
    at = tap(&mut f, Button::Right, at);
    assert_eq!(
        clock::utc_offset_min(),
        15,
        "the preview is not on the clock"
    );
    at = tap(&mut f, Button::A, at);
    match f.screen {
        Screen::Timezone(parent, menu) => {
            assert_eq!(parent.selected(), Some(ShelfChoice::TimeZone));
            assert_eq!(menu.original(), 0, "the failure moved what the card holds");
            assert_eq!(
                menu.selected(),
                0,
                "the value that failed is still on screen"
            );
            assert!(!menu.changed());
        }
        other => panic!("a save that failed left the screen: {other:?}"),
    }
    assert_eq!(
        clock::utc_offset_min(),
        0,
        "a save that failed left the preview on the clock"
    );
    assert_eq!(f.toast_key(), Some("timezone-save-failed"));
    assert_eq!(
        f.volume.level(),
        DEFAULT_VOLUME_LEVEL,
        "the failure moved the volume"
    );
    assert_eq!(
        fs::read(broken.settings_path()).unwrap(),
        damaged,
        "the unreadable file was rewritten"
    );
    assert!(broken.settings_path().is_file());
    assert_eq!(broken.read_utc_offset_minutes(), 0);
    assert_eq!(
        fs::read_dir(&system).unwrap().count(),
        entries,
        "the settings folder gained or lost an entry"
    );

    // The screen is still live: another press previews again, and leaving puts it back.
    at = tap(&mut f, Button::Right, at);
    assert_eq!(clock::utc_offset_min(), 15);
    tap(&mut f, Button::B, at);
    assert_eq!(clock::utc_offset_min(), 0);
    assert!(matches!(f.screen, Screen::Shelf(_)));

    // 10. The message a failed save shows, in both languages, exactly as the packs have it.
    assert_eq!(
        I18n::embedded("en").unwrap().t("timezone-save-failed"),
        "Could not save the time zone; restored the previous value"
    );
    assert_eq!(
        I18n::embedded("ko").unwrap().t("timezone-save-failed"),
        "시간대를 저장하지 못해 이전 값으로 돌아갔습니다"
    );

    // 11. Both screens draw their own layer over the shelf the list draws, and neither draws the
    // other's panel.
    let draw_card = card_with("draw", Some(b"volume = 30\n"));
    clock::set_utc_offset_min(0).unwrap();
    let mut d = app(&draw_card);
    let mut ctx = ctx();
    let (pw, ph) = ctx.profile.geometry.size();
    let (pw, ph) = (pw as f32, ph as f32);
    d.tick(Instant::now());
    let list = frame(&mut d, &mut ctx);
    let at = Instant::now();
    tap(&mut d, Button::Menu, at);
    let shelf = frame(&mut d, &mut ctx);
    tap(&mut d, Button::Down, at + ms(200)); // Language → TimeZone
    tap(&mut d, Button::A, at + ms(400));
    let timezone = frame(&mut d, &mut ctx);

    for (case, c) in [("list", &list), ("shelf", &shelf), ("timezone", &timezone)] {
        assert!(
            !c.ops.iter().any(|o| matches!(o, Op::Clear(_))),
            "{case}: the frame cleared the panel"
        );
    }

    // The menu dims the whole panel, and it is not the first mark drawn: the shelf is under it.
    // The two frames are compared mark for mark, so the uploads the first one needed do not
    // count as a difference the player could see.
    let list_marks = marks(&list.ops);
    let shelf_marks = marks(&shelf.ops);
    let tz_marks = marks(&timezone.ops);
    let dim_at = |m: &[&Op]| {
        m.iter()
            .position(|o| {
                matches!(o, Op::Rect { x, y, w, h, color }
                    if *x == 0.0 && *y == 0.0 && (*w - pw).abs() < 0.5 && (*h - ph).abs() < 0.5
                        && *color == DIM)
            })
            .unwrap_or_else(|| panic!("no dim in the menu's frame"))
    };
    let shelf_dim = dim_at(&shelf_marks);
    assert!(shelf_dim > 0, "the settings menu was the first thing drawn");
    assert_eq!(
        list_marks[..shelf_dim],
        shelf_marks[..shelf_dim],
        "the shelf under the settings menu is not the list's own shelf"
    );
    let tz_dim = dim_at(&tz_marks);
    assert!(tz_dim > 0, "the offset screen was the first thing drawn");
    assert_eq!(
        list_marks[..tz_dim],
        tz_marks[..tz_dim],
        "the shelf under the offset screen is not the list's own shelf"
    );

    // Each screen draws its own panel, and only its own: the settings menu is not stacked behind
    // the offset screen.
    assert_eq!(panels(&shelf.ops, SHELF_W, SHELF_H), 1, "no settings panel");
    assert_eq!(
        panels(&shelf.ops, TZ_W, TZ_H),
        0,
        "an offset panel on the shelf"
    );
    assert_eq!(panels(&timezone.ops, TZ_W, TZ_H), 1, "no offset panel");
    assert_eq!(
        panels(&timezone.ops, SHELF_W, SHELF_H),
        0,
        "the parent settings panel is drawn behind the offset screen"
    );

    // The HUD keeps its corner on all three screens, and it is drawn after the menus so the
    // previewed offset is the one it shows.
    assert!(hud_quads(&list) > 0, "no HUD on the list");
    assert_eq!(
        hud_quads(&shelf),
        hud_quads(&list),
        "the settings menu hid the HUD"
    );
    assert_eq!(
        hud_quads(&timezone),
        hud_quads(&list),
        "the offset screen hid the HUD"
    );

    // 12. The volume this machine is running and the level on the card are not the offset
    // screen's business: nothing above moved either.
    assert_eq!(d.volume.level(), 30, "the menus moved the volume");
    assert_eq!(draw_card.read_global_settings().volume, 30);
    assert_eq!(
        fs::read(draw_card.settings_path()).unwrap(),
        b"volume = 30\n",
        "the offset screens wrote to a card that had nothing set"
    );

    let _ = clock::set_utc_offset_min(started_at);
}
