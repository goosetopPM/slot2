//! The Display submenu inside the running app: the scale a game is drawn with, and the shader
//! submenu its last row opens. Both are changed from the in-game menu and saved on the card.
//!
//! Needs `vendor/mgba_libretro.*` for the session half and skips loudly without it; the
//! no-session behavior runs without a core.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::Op;
use slot2_gfx::RecordingCanvas;
use slot2_input::{Button, Event};
use slot2_retro::CoreId;
use slot2_store::{Card, Cart, GameSettings, Platform, ScaleMode, ShaderPreset};
use slot2_ui::display_menu::{BOX_H, BOX_H_CROPPING, CROPPING_ROWS, ROWS};
use slot2_ui::{DisplayChoice, DisplayMenu, InGameMenu};

use slot2::app::{App, Screen};
use slot2::session::Session;

static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn core_dir() -> Option<PathBuf> {
    let d = repo().join("vendor");
    let name = if cfg!(windows) {
        "mgba_libretro.dll"
    } else if cfg!(target_os = "macos") {
        "mgba_libretro.dylib"
    } else {
        "mgba_libretro.so"
    };
    if d.join(name).is_file() {
        Some(d)
    } else {
        eprintln!(
            "no core in {} — skipping (run build/cores.ps1)",
            d.display()
        );
        None
    }
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

fn tap(a: &mut App, b: Button, at: Instant) {
    a.feed(&ev(b, true, at));
    a.feed(&ev(b, false, at + ms(40)));
    a.tick(at + ms(60));
}

fn run(a: &mut App, from: Instant, secs: f32) -> Instant {
    let mut now = from;
    for _ in 0..(secs * 60.0).ceil() as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        a.run_frame();
    }
    now
}

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("slot2-display-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card with the test ROM on it, and the app sitting at the game with a frame drawn.
fn playing(tag: &str) -> Option<(App, Card, Cart)> {
    let cores = core_dir()?;
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    fs::copy(
        repo().join("assets/test/arm.gba"),
        card.games_dir(Platform::Gba).join("arm.gba"),
    )
    .unwrap();
    let cart = Cart {
        platform: Platform::Gba,
        stem: "arm".into(),
        title: "arm".into(),
        rom: card.games_dir(Platform::Gba).join("arm.gba"),
    };
    let mut a = App::with_card(
        card.clone(),
        cores,
        48_000,
        slot2::tuning_for(&slot2_platform::detect().profile),
        false,
        Screen::List,
    );
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let mut now = t + ms(60);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            run(&mut a, now, 0.2); // a frame for the session to draw
            let _ = a.take_sink_request();
            return Some((a, card, cart));
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

/// The in-game menu, open and parked on the Display row.
fn open_display_row(a: &mut App, at: Instant) -> Instant {
    tap(a, Button::Menu, at);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    let mut now = at + ms(200);
    for _ in 0..3 {
        tap(a, Button::Down, now);
        now += ms(200);
    }
    let menu = match a.screen {
        Screen::InGame(m) => m,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        menu.choice(),
        slot2_ui::InGameChoice::Display,
        "the menu is not on the Display row"
    );
    now
}

/// The row the Display menu — or the shader screen that was opened from it — is on. The choice
/// rather than the scale alone: the shader row has no scale, and a helper that folded the two
/// together would let a test pass on the wrong row.
fn choice_of(a: &App) -> DisplayChoice {
    match a.screen {
        Screen::Display(_, menu) => menu.choice(),
        // The parent menu is still on its Shader or Overscan row while that screen is open: that
        // is what closing it lands back on.
        Screen::Shader(_, display, _) => display.choice(),
        Screen::Overscan(_, display, _) => display.choice(),
        other => panic!("no display menu is open: {other:?}"),
    }
}

/// The Display row that chooses the platform's own scale.
fn platform_default() -> DisplayChoice {
    DisplayChoice::Scale(None)
}

/// The card's setting for this game, as the launcher would read it.
fn write_shader(card: &Card, cart: &Cart, shader: Option<ShaderPreset>) {
    let mut settings = card.read_settings(cart);
    settings.shader = shader;
    card.write_settings(cart, &settings).unwrap();
}

/// The Display submenu, open and parked on its Shader row.
fn open_shader_row(a: &mut App, at: Instant) -> Instant {
    let mut now = open_display_row(a, at);
    tap(a, Button::A, now);
    assert!(matches!(a.screen, Screen::Display(..)), "{:?}", a.screen);
    for _ in 0..slot2_ui::display_menu::ROWS.len() {
        if choice_of(a) == DisplayChoice::Shader {
            return now + ms(200);
        }
        now += ms(200);
        tap(a, Button::Down, now);
    }
    panic!("the Shader row was never reached: {:?}", choice_of(a));
}

/// The shader screen, open on the card's own setting.
fn open_shader(a: &mut App, at: Instant) -> Instant {
    let now = open_shader_row(a, at);
    tap(a, Button::A, now);
    assert!(matches!(a.screen, Screen::Shader(..)), "{:?}", a.screen);
    now + ms(200)
}

/// The row the shader screen is on.
fn shader_row(a: &App) -> Option<ShaderPreset> {
    match a.screen {
        Screen::Shader(_, _, menu) => menu.selected(),
        other => panic!("the shader menu is not open: {other:?}"),
    }
}

/// The game's own draw in the frame, as the effect it went through (`None` for the plain path).
/// Identified by the whole panel, so a menu label in the same frame is not mistaken for it.
fn game_effect(c: &RecordingCanvas) -> Option<Option<slot2_gfx::ShaderEffect>> {
    let panel = |x: &f32, y: &f32, w: &f32, h: &f32| {
        *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
    };
    c.frame().iter().find_map(|o| match o {
        Op::Image { x, y, w, h, .. } if panel(x, y, w, h) => Some(None),
        Op::ImageEffect {
            x, y, w, h, effect, ..
        } if panel(x, y, w, h) => Some(Some(*effect)),
        _ => None,
    })
}

/// `(uploads, updates, frees)` over everything recorded so far: the game texture's whole life.
fn texture_ops(c: &RecordingCanvas) -> (usize, usize, usize) {
    let count = |f: fn(&Op) -> bool| c.ops.iter().filter(|o| f(o)).count();
    (
        count(|o| matches!(o, Op::UploadRgba8 { .. })),
        count(|o| matches!(o, Op::UpdateRgba8 { .. })),
        count(|o| matches!(o, Op::Free(_))),
    )
}

#[test]
fn the_display_row_opens_on_the_stored_scale() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("open") else {
        return;
    };
    let t = Instant::now();

    // No file: the platform default is the row.
    let now = open_display_row(&mut a, t);
    tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Display(..)), "{:?}", a.screen);
    assert_eq!(
        choice_of(&a),
        platform_default(),
        "a game with no override opened elsewhere"
    );

    // With an override, that row, and nothing was written by looking.
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Fill),
            ..Default::default()
        },
    )
    .unwrap();
    let now = now + ms(400);
    tap(&mut a, Button::B, now);
    let now = now + ms(200);
    tap(&mut a, Button::A, now); // the same Display row, straight back in
    assert_eq!(choice_of(&a), DisplayChoice::Scale(Some(ScaleMode::Fill)));
}

#[test]
fn navigation_wraps_and_closing_returns_to_the_same_row() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("nav") else {
        return;
    };
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Integer),
            ..Default::default()
        },
    )
    .unwrap();
    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::Integer))
    );

    // The rows are Platform default, Integer, Aspect fit, Fill, Shader, Overlay: one up from
    // Integer is the platform default, and one more wraps past it to the last row.
    let mut now = t + ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(
        choice_of(&a),
        platform_default(),
        "up from the Integer row was not the platform default"
    );
    now += ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Overlay,
        "up from the first row did not wrap to the last row"
    );
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        choice_of(&a),
        platform_default(),
        "down from the last row did not wrap"
    );
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::Integer))
    );
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::AspectFit))
    );
    now += ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::Integer))
    );

    // Looking around is not choosing: the card and the running game are where they were.
    assert_eq!(
        card.read_settings(&cart).scale,
        Some(ScaleMode::Integer),
        "navigation wrote a setting"
    );
    assert_eq!(
        a.session().unwrap().scale(),
        Session::policy_for(Some(ScaleMode::Integer)),
        "navigation moved the picture"
    );

    // B and MENU both hand back the row it came from, and the card was not touched.
    for closer in [Button::B, Button::Menu] {
        let at = now + ms(200);
        tap(&mut a, closer, at);
        match a.screen {
            Screen::InGame(m) => assert_eq!(
                m.choice(),
                slot2_ui::InGameChoice::Display,
                "{closer:?} lost the row"
            ),
            other => panic!("{closer:?} left on {other:?}"),
        }
        assert_eq!(
            card.read_settings(&cart).scale,
            Some(ScaleMode::Integer),
            "{closer:?} changed the card"
        );
        assert_eq!(
            a.session().unwrap().scale(),
            Session::policy_for(Some(ScaleMode::Integer)),
            "{closer:?} changed the game's scale"
        );
        // The row it handed back is the Display row, so A goes straight back in.
        let back = at + ms(200);
        tap(&mut a, Button::A, back);
        assert_eq!(
            choice_of(&a),
            DisplayChoice::Scale(Some(ScaleMode::Integer)),
            "{closer:?} reopened on another row"
        );
    }
}

#[test]
fn the_submenu_pauses_the_session_and_keeps_it() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("pause") else {
        return;
    };
    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);
    assert!(a.session().is_some(), "the submenu dropped the session");
    assert!(
        a.audio_paused(),
        "the game kept its sound under the submenu"
    );
    assert!(
        a.take_sink_request().is_none(),
        "the submenu touched the audio"
    );

    let frames = a.session().unwrap().frames_run();
    let now = run(&mut a, t, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran under the submenu"
    );
    let _ = now;
}

/// Whether this op is the game's own frame: the whole panel, drawn through the effect the GBA
/// gets by default when its game says nothing about shaders.
fn game_frame(o: &Op) -> bool {
    matches!(o, Op::ImageEffect { x, y, w, h, effect, .. }
        if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
            && *effect == slot2_gfx::ShaderEffect::Lcd3x)
}

#[test]
fn drawing_shows_the_game_frame_under_the_submenu() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("draw") else {
        return;
    };
    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);

    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, t + ms(100));
    let ops = c.frame();

    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the submenu cleared the game frame"
    );
    let game = ops
        .iter()
        .position(game_frame)
        .expect("the game frame was not drawn");
    let dim = ops
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, w, h, color }
            if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
                && *color == slot2_ui::display_menu::DIM)
        })
        .expect("the submenu was not drawn");
    assert!(game < dim, "the submenu went down before the game");
    assert_eq!(
        ops.iter()
            .find(|o| matches!(
                o,
                Op::Rect { .. } | Op::Image { .. } | Op::ImageEffect { .. }
            ))
            .map(game_frame),
        Some(true),
        "the first mark was not the game frame"
    );

    // Not the parent menu, not the switcher, not the corner HUD, not the time badge.
    let (bx, by) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "the parent menu was drawn under the submenu"
    );
    let (cx, cy, ..) = slot2_ui::StateSwitcher::card_rect(&ctx, 0);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - cx).abs() < 0.5 && (*y - cy).abs() < 0.5)),
        "a switcher card was drawn under the submenu"
    );
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { y, h, .. }
            if (*y - slot2_ui::hud::HUD_MARGIN).abs() < 0.5
                && (*h - slot2_ui::hud::HUD_H).abs() < 0.01)),
        "the top band was drawn over the game"
    );
}

#[test]
fn choosing_a_scale_saves_it_and_puts_it_on_the_game() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("choose") else {
        return;
    };
    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);
    let mut now = t + ms(200);
    let frames = a.session().unwrap().frames_run();

    // Down past Integer and Aspect fit to Fill, committing each one as the player would.
    for want in [ScaleMode::Integer, ScaleMode::AspectFit, ScaleMode::Fill] {
        tap(&mut a, Button::Down, now);
        now += ms(200);
        tap(&mut a, Button::A, now);
        now += ms(200);
        let case = format!("{want:?}");
        assert_eq!(
            choice_of(&a),
            DisplayChoice::Scale(Some(want)),
            "{case}: the row moved"
        );
        assert_eq!(
            card.read_settings(&cart).scale,
            Some(want),
            "{case}: the card does not have it"
        );
        assert_eq!(
            a.session().unwrap().scale(),
            Session::policy_for(Some(want)),
            "{case}: the session was not told"
        );
        assert!(
            matches!(a.screen, Screen::Display(..)),
            "{case}: the menu closed"
        );
    }
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "committing a scale ran the core"
    );
    assert!(a.take_sink_request().is_none());
}

#[test]
fn platform_default_removes_only_the_scale_override() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("default") else {
        return;
    };
    // A card with a hand-written unknown key, and the other known settings set.
    card.write_settings(
        &cart,
        &GameSettings {
            overscan: Some(false),
            rewind: Some(false),
            scale: Some(ScaleMode::Fill),
            ..Default::default()
        },
    )
    .unwrap();
    let path = card.game_settings_path(&cart);
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("shutdown = fast\n");
    fs::write(&path, text).unwrap();

    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);
    assert_eq!(choice_of(&a), DisplayChoice::Scale(Some(ScaleMode::Fill)));
    // Fill is not the last row any more: down from it is the shader row, then the overlay row,
    // and down from there wraps to the platform default.
    let now = t + ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Shader,
        "down from Fill was not the shader row"
    );
    let now = now + ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Overlay,
        "down from the shader row was not the overlay row"
    );
    let now = now + ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        choice_of(&a),
        platform_default(),
        "down from the last row did not wrap to the platform default"
    );
    let now = now + ms(200);
    tap(&mut a, Button::A, now);

    // What the App's own commit did, with nothing touching the store in between.
    let settings = card.read_settings(&cart);
    assert_eq!(settings.scale, None, "the override is still on the card");
    assert_eq!(settings.overscan, Some(false), "overscan was lost");
    assert_eq!(settings.rewind, Some(false), "rewind was lost");
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("shutdown = fast"),
        "the unknown key was lost"
    );
    assert_eq!(
        a.session().unwrap().scale(),
        Session::policy_for(None),
        "the session did not go back to the launch policy for no override"
    );
    assert_eq!(
        choice_of(&a),
        platform_default(),
        "the menu left the committed row"
    );

    // And a file holding nothing but the scale goes away entirely, again through the App:
    // the row the menu is on is the platform default, and A commits it.
    let _ = fs::remove_file(&path);
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Integer),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        path.is_file(),
        "the scale-only file was not there to remove"
    );
    let now = now + ms(200);
    tap(&mut a, Button::A, now);
    assert_eq!(a.toast_key(), None, "removing the override failed");
    assert!(!path.exists(), "the App left an all-default file behind");
    assert_eq!(
        a.session().unwrap().scale(),
        Session::policy_for(None),
        "the session left the launch policy"
    );
}

#[test]
fn a_write_that_fails_leaves_the_game_alone() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("failed") else {
        return;
    };
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Integer),
            ..Default::default()
        },
    )
    .unwrap();
    let path = card.game_settings_path(&cart);

    // The menu opens on what the card really says: the file is right there and readable.
    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::Integer))
    );

    // A directory where the atomic temporary file belongs: the ini stays readable and only
    // the write can fail.
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    let blocker = PathBuf::from(tmp);
    fs::create_dir(&blocker).unwrap();

    let now = t + ms(200);
    tap(&mut a, Button::Down, now); // Integer → Aspect fit
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::AspectFit))
    );
    let now = now + ms(200);
    tap(&mut a, Button::A, now);

    assert_eq!(
        a.toast_key(),
        Some("display-save-failed"),
        "a failed save said nothing"
    );
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::AspectFit)),
        "the menu did not stay on the attempted row"
    );
    assert_eq!(
        a.session().unwrap().scale(),
        Session::policy_for(Some(ScaleMode::Integer)),
        "the session followed a setting the card refused"
    );
    assert_eq!(
        card.read_settings(&cart).scale,
        Some(ScaleMode::Integer),
        "the card's own setting moved on a write that failed"
    );
    assert!(path.is_file(), "the original settings file is gone");
    assert!(matches!(a.screen, Screen::Display(..)));
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());

    // Left in place, the blocker would keep the next write to this card from landing.
    fs::remove_dir(&blocker).unwrap();
}

#[test]
fn an_unreadable_settings_file_is_not_overwritten_by_the_menu() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("unreadable") else {
        return;
    };
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, [0xff, 0xfe, b's', b'=', 0x80, b'\n']).unwrap();
    let damaged = fs::read(&path).unwrap();

    // Reading stays forgiving, so the row is the platform default rather than a refusal.
    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);
    assert_eq!(
        choice_of(&a),
        platform_default(),
        "an unreadable file hid the row"
    );
    let launched = a.session().unwrap().scale();

    // Choosing anything has to refuse: this file was never read.
    let now = t + ms(200);
    tap(&mut a, Button::Down, now); // Platform default → Integer
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::Integer))
    );
    let now = now + ms(200);
    tap(&mut a, Button::A, now);

    assert_eq!(
        a.toast_key(),
        Some("display-save-failed"),
        "a refused save said nothing"
    );
    assert_eq!(
        choice_of(&a),
        DisplayChoice::Scale(Some(ScaleMode::Integer)),
        "the menu left the attempted row"
    );
    assert_eq!(
        a.session().unwrap().scale(),
        launched,
        "the session followed a setting the card refused"
    );
    assert!(matches!(a.screen, Screen::Display(..)));
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());
    assert_eq!(
        fs::read(&path).unwrap(),
        damaged,
        "the damaged file was rewritten"
    );
}

#[test]
fn the_display_row_does_nothing_without_a_session() {
    // No core, no session: the row stays a row, and nothing is read or written.
    let root = scratch("nosession");
    let card = Card::new(&root);
    card.ensure_layout();
    fs::write(card.games_dir(Platform::Gba).join("arm.gba"), b"rom").unwrap();
    let mut a = App::with_card(
        card.clone(),
        PathBuf::from(".").join("no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    let mut menu = InGameMenu::default();
    for _ in 0..3 {
        menu.down();
    }
    assert_eq!(menu.choice(), slot2_ui::InGameChoice::Display);
    a.screen = Screen::InGame(menu);
    tap(&mut a, Button::A, Instant::now());
    assert!(
        matches!(a.screen, Screen::InGame(_)),
        "the Display row opened something with no game: {:?}",
        a.screen
    );
    assert_eq!(DisplayMenu::new(None, false).choice(), platform_default());
}

// -------------------------------------------------------------- the shader submenu

#[test]
fn the_shader_row_opens_on_the_cards_own_setting_and_hands_the_row_back() {
    let _serial = serial();
    for (tag, setting) in [
        ("plain", None),
        ("off", Some(ShaderPreset::Off)),
        ("explicit", Some(ShaderPreset::ZfastCrt)),
    ] {
        let Some((mut a, card, cart)) = playing(&format!("shader-open-{tag}")) else {
            return;
        };
        if let Some(shader) = setting {
            write_shader(&card, &cart, Some(shader));
        }
        let path = card.game_settings_path(&cart);
        let before = fs::read(&path).ok();

        // The Display menu walks to its last row without touching the card, and A opens the
        // shader screen on what the card says.
        let mut now = open_shader_row(&mut a, Instant::now());
        assert_eq!(choice_of(&a), DisplayChoice::Shader);
        assert_eq!(
            fs::read(&path).ok(),
            before,
            "opening the row wrote a setting"
        );
        tap(&mut a, Button::A, now);
        assert_eq!(
            shader_row(&a),
            setting,
            "{setting:?} was not the opening row"
        );

        // B and MENU both hand back the Display screen, still on its Shader row, and A goes
        // straight back into the shader menu on the same row.
        for closer in [Button::B, Button::Menu] {
            now += ms(200);
            tap(&mut a, closer, now);
            match a.screen {
                Screen::Display(_, display) => assert_eq!(
                    display.choice(),
                    DisplayChoice::Shader,
                    "{closer:?} lost the Shader row"
                ),
                other => panic!("{closer:?} left on {other:?}"),
            }
            assert_eq!(
                fs::read(&path).ok(),
                before,
                "{closer:?} changed the card on the way out"
            );
            now += ms(200);
            tap(&mut a, Button::A, now);
            assert_eq!(
                shader_row(&a),
                setting,
                "{closer:?} reopened on another row"
            );
        }
    }
}

#[test]
fn the_shader_row_does_nothing_without_a_session() {
    // No core, no session: the row stays a row and no card is read.
    let root = scratch("shader-nosession");
    let card = Card::new(&root);
    card.ensure_layout();
    fs::write(card.games_dir(Platform::Gba).join("arm.gba"), b"rom").unwrap();
    let mut a = App::with_card(
        card.clone(),
        PathBuf::from(".").join("no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::Display(InGameMenu::default(), DisplayMenu::new(None, false)),
    );
    // Straight onto the Shader row, which is what walking to it would leave in place.
    let mut now = Instant::now();
    for _ in 0..slot2_ui::display_menu::ROWS.len() {
        if choice_of(&a) == DisplayChoice::Shader {
            break;
        }
        tap(&mut a, Button::Down, now);
        now += ms(200);
    }
    assert_eq!(choice_of(&a), DisplayChoice::Shader);
    tap(&mut a, Button::A, now);
    assert!(
        matches!(a.screen, Screen::Display(..)),
        "the Shader row opened a screen with no game: {:?}",
        a.screen
    );
    assert_eq!(choice_of(&a), DisplayChoice::Shader);
}

#[test]
fn walking_the_shader_rows_and_closing_changes_nothing() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("shader-walk") else {
        return;
    };
    write_shader(&card, &cart, Some(ShaderPreset::ZfastCrt));
    let path = card.game_settings_path(&cart);
    let bytes = fs::read(&path).unwrap();
    // The game was launched while the card said nothing about shaders, so it is drawn through
    // the GBA's own default. The file written above is what the menu opens on, not what the
    // running game was started with.
    let effect = a.session().unwrap().shader_effect();
    assert_eq!(effect, Some(slot2_gfx::ShaderEffect::Lcd3x));

    let mut now = open_shader(&mut a, Instant::now());
    assert_eq!(shader_row(&a), Some(ShaderPreset::ZfastCrt));

    // Up the rows above it, then one more up wraps from the first row to the last, and one
    // down wraps back.
    for want in [
        Some(ShaderPreset::Lcd3x),
        Some(ShaderPreset::SharpBilinear),
        Some(ShaderPreset::Off),
        None,
        Some(ShaderPreset::Scanline),
    ] {
        now += ms(200);
        tap(&mut a, Button::Up, now);
        assert_eq!(shader_row(&a), want, "up from {:?}", a.screen);
    }
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        shader_row(&a),
        None,
        "down from the last row did not wrap to the platform default"
    );

    // One lap of the six rows comes back to where it was.
    let start = shader_row(&a);
    for _ in 0..slot2_ui::shader_menu::ROWS.len() {
        now += ms(200);
        tap(&mut a, Button::Down, now);
    }
    assert_eq!(shader_row(&a), start, "one lap per row count");

    // Closing hands the Display screen its Shader row back, and nothing moved: not the card,
    // not the picture, not the audio.
    now += ms(200);
    tap(&mut a, Button::B, now);
    match a.screen {
        Screen::Display(_, display) => {
            assert_eq!(display.choice(), DisplayChoice::Shader)
        }
        other => panic!("B left on {other:?}"),
    }
    assert_eq!(fs::read(&path).unwrap(), bytes, "looking wrote a setting");
    assert_eq!(
        a.session().unwrap().shader_effect(),
        effect,
        "looking moved the picture"
    );
    assert!(a.take_sink_request().is_none());
    assert_eq!(a.toast_key(), None, "looking said something");
}

#[test]
fn the_platform_default_choice_removes_only_the_shader_override() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("shader-default") else {
        return;
    };
    write_shader(&card, &cart, Some(ShaderPreset::Scanline));
    let path = card.game_settings_path(&cart);
    assert!(path.is_file(), "the shader-only file was not there");

    let mut now = open_shader(&mut a, Instant::now());
    assert_eq!(shader_row(&a), Some(ShaderPreset::Scanline));
    // Applied first, so that going back to the platform default is a change of picture rather
    // than the one the game already had: the session launched before this file existed.
    now += ms(200);
    tap(&mut a, Button::A, now);
    assert_eq!(
        a.session().unwrap().shader_effect(),
        Some(slot2_gfx::ShaderEffect::Scanline)
    );
    assert!(
        path.is_file(),
        "the choice removed the file it shares keys with"
    );
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        shader_row(&a),
        None,
        "down from the last row did not wrap to the platform default"
    );
    now += ms(200);
    tap(&mut a, Button::A, now);

    // The choice is the absence of the key, so a file that held nothing else goes, and the
    // running game is drawn the way a game that never had a shader setting is drawn: a GBA is
    // Lcd3x.
    assert_eq!(a.toast_key(), None, "removing the override failed");
    assert!(!path.exists(), "the shader-only file was left behind");
    assert_eq!(card.read_settings(&cart).shader, None);
    assert_eq!(
        a.session().unwrap().shader_effect(),
        Some(slot2_gfx::ShaderEffect::Lcd3x),
        "the session did not go back to the platform default"
    );
    assert_eq!(shader_row(&a), None, "the screen left the committed row");
}

#[test]
fn setting_and_clearing_a_shader_keeps_every_other_key() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("shader-keys") else {
        return;
    };
    // Everything else a game can have set, plus a key this version has never heard of.
    card.write_settings(
        &cart,
        &GameSettings {
            core: Some("mgba".into()),
            scale: Some(ScaleMode::Fill),
            overscan: Some(false),
            rewind: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    let path = card.game_settings_path(&cart);
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("shutdown = fast\n");
    fs::write(&path, text).unwrap();

    // On, then off: Down from the platform default is the explicit off.
    let mut now = open_shader(&mut a, Instant::now());
    assert_eq!(shader_row(&a), None);
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(shader_row(&a), Some(ShaderPreset::Off));
    now += ms(200);
    tap(&mut a, Button::A, now);

    let settings = card.read_settings(&cart);
    assert_eq!(settings.shader, Some(ShaderPreset::Off));
    assert_eq!(settings.core.as_deref(), Some("mgba"), "core was lost");
    assert_eq!(settings.scale, Some(ScaleMode::Fill), "scale was lost");
    assert_eq!(settings.overscan, Some(false), "overscan was lost");
    assert_eq!(settings.rewind, Some(false), "rewind was lost");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("shader = none"), "{text}");
    assert!(
        text.contains("shutdown = fast"),
        "the unknown key was lost: {text}"
    );
    assert_eq!(
        a.session().unwrap().shader_effect(),
        None,
        "Off is the plain draw"
    );

    // And back to the platform default, which takes the shader key and nothing else.
    now += ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(shader_row(&a), None);
    now += ms(200);
    tap(&mut a, Button::A, now);

    let settings = card.read_settings(&cart);
    assert_eq!(settings.shader, None);
    assert_eq!(settings.core.as_deref(), Some("mgba"));
    assert_eq!(settings.scale, Some(ScaleMode::Fill));
    assert_eq!(settings.overscan, Some(false));
    assert_eq!(settings.rewind, Some(false));
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        !text.contains("shader"),
        "the shader key is still there: {text}"
    );
    assert!(text.contains("shutdown = fast"), "{text}");
}

#[test]
fn every_shader_choice_is_saved_exactly_and_applied_at_once() {
    let _serial = serial();
    // The six meanings a card can hold, and what each one draws on a GBA: the absence of a key
    // is the platform's own default, and "off" is the plain path.
    let cases: [(Option<ShaderPreset>, Option<slot2_gfx::ShaderEffect>); 6] = [
        (None, Some(slot2_gfx::ShaderEffect::Lcd3x)),
        (Some(ShaderPreset::Off), None),
        (
            Some(ShaderPreset::SharpBilinear),
            Some(slot2_gfx::ShaderEffect::SharpBilinear),
        ),
        (
            Some(ShaderPreset::Lcd3x),
            Some(slot2_gfx::ShaderEffect::Lcd3x),
        ),
        (
            Some(ShaderPreset::ZfastCrt),
            Some(slot2_gfx::ShaderEffect::ZfastCrt),
        ),
        (
            Some(ShaderPreset::Scanline),
            Some(slot2_gfx::ShaderEffect::Scanline),
        ),
    ];
    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);

    for (i, (want, effect)) in cases.into_iter().enumerate() {
        let tag = format!("shader-choose{i}");
        let Some((mut a, card, cart)) = playing(&tag) else {
            return;
        };
        let path = card.game_settings_path(&cart);
        let mut now = open_shader(&mut a, Instant::now());

        // Walk to the row this case is about; the six are in the order the menu draws them.
        for _ in 0..slot2_ui::shader_menu::ROWS.len() {
            if shader_row(&a) == want {
                break;
            }
            now += ms(200);
            tap(&mut a, Button::Down, now);
        }
        assert_eq!(shader_row(&a), want, "{want:?} was not reachable");

        // One warm draw, so the texture the next draw reuses is already there.
        let mut c = RecordingCanvas::new(720, 480);
        a.draw(&mut c, &mut ctx, now);
        let frames = a.session().unwrap().frames_run();
        let frame_before = a
            .session()
            .unwrap()
            .last_frame()
            .map(|(w, h, d)| (w, h, d.to_vec()));
        let audio_before = a.session().unwrap().audio_health();
        let texture_before = texture_ops(&c);

        now += ms(200);
        tap(&mut a, Button::A, now);
        let case = format!("{want:?}");

        // Saved first: the card holds the game's own meaning, not a computed effect.
        assert_eq!(a.toast_key(), None, "{case}: saving failed");
        assert_eq!(
            card.read_settings(&cart).shader,
            want,
            "{case}: the card has something else"
        );
        if want.is_none() {
            assert!(
                !path.exists(),
                "{case}: a file was left holding nothing but the default"
            );
        }

        // And put on the running session at once, through the session's own boundary.
        assert_eq!(
            a.session().unwrap().shader_effect(),
            effect,
            "{case}: the session was not told"
        );
        assert_eq!(shader_row(&a), want, "{case}: the screen left the row");

        // The next draw is the only thing that changed. `frame()` is everything since the
        // session's own clear, so the first mark is the game and the panel comes after it.
        a.draw(&mut c, &mut ctx, now);
        assert_eq!(game_effect(&c), Some(effect), "{case}: the next draw");
        assert!(
            matches!(c.frame()[0], Op::Image { .. } | Op::ImageEffect { .. }),
            "{case}: the first mark was not the game frame: {:?}",
            c.frame()[0]
        );
        let dim_at = c
            .frame()
            .iter()
            .position(|o| {
                matches!(o, Op::Rect { x, y, w, h, color }
                if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
                    && *color == slot2_ui::shader_menu::DIM)
            })
            .expect("{case}: the shader menu was not drawn");
        assert_eq!(
            game_effect(&c).map(|_| 0),
            Some(0),
            "{case}: the game frame is not the first thing drawn"
        );
        assert!(dim_at > 0, "{case}: the menu went down before the game");
        // Nothing else moved: no frame, no new picture, no texture work, no audio, no sink.
        assert_eq!(
            a.session().unwrap().frames_run(),
            frames,
            "{case}: a frame ran"
        );
        assert_eq!(
            a.session()
                .unwrap()
                .last_frame()
                .map(|(w, h, d)| (w, h, d.to_vec())),
            frame_before,
            "{case}: the core's picture moved"
        );
        assert_eq!(
            texture_ops(&c),
            texture_before,
            "{case}: a draw uploaded, rewrote or freed the game texture"
        );
        assert_eq!(
            a.session().unwrap().audio_health(),
            audio_before,
            "{case}: a draw touched the audio"
        );
        assert!(
            a.take_sink_request().is_none(),
            "{case}: a sink was asked for"
        );

        // Closing and reopening lands on the value that was saved.
        now += ms(200);
        tap(&mut a, Button::B, now);
        now += ms(200);
        tap(&mut a, Button::A, now);
        assert_eq!(
            shader_row(&a),
            want,
            "{case}: reopening did not show what was saved"
        );
    }
}

#[test]
fn the_shader_screen_pauses_the_session_and_keeps_it() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("shader-pause") else {
        return;
    };
    let t = open_shader(&mut a, Instant::now());
    assert!(
        a.session().is_some(),
        "the shader screen dropped the session"
    );
    assert!(a.audio_paused(), "the game kept its sound under the screen");
    assert!(
        a.take_sink_request().is_none(),
        "the shader screen touched the audio"
    );

    let frames = a.session().unwrap().frames_run();
    let audio = a.session().unwrap().audio_health();
    let now = run(&mut a, t, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran under the shader screen"
    );
    assert_eq!(
        a.session().unwrap().audio_health(),
        audio,
        "audio advanced under the shader screen"
    );
    let _ = now;
}

#[test]
fn drawing_shows_the_game_frame_under_the_shader_menu() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("shader-draw") else {
        return;
    };
    let t = open_shader(&mut a, Instant::now());

    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, t + ms(100));
    assert_eq!(
        game_effect(&c),
        Some(Some(slot2_gfx::ShaderEffect::Lcd3x)),
        "the game's own frame is not under the shader menu"
    );

    let ops = c.frame();
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the shader screen cleared the game frame"
    );
    let dim = ops
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, w, h, color }
            if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
                && *color == slot2_ui::shader_menu::DIM)
        })
        .expect("the shader menu was not drawn");
    // The game frame is the first mark of the frame, so no wallpaper went down first, and the
    // menu's dim is the first whole-panel rect after it.
    assert!(
        matches!(ops[0], Op::ImageEffect { .. } | Op::Image { .. }),
        "the first mark was not the game frame: {:?}",
        ops[0]
    );
    assert!(dim > 0, "the menu went down before the game");

    // Not the parent Display menu, not the in-game menu, not the switcher, not the badge.
    let Screen::Shader(_, display, _) = a.screen else {
        panic!("the shader screen is not open: {:?}", a.screen)
    };
    let (dx, dy) = display.box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - dx).abs() < 0.5 && (*y - dy).abs() < 0.5)),
        "the Display menu was drawn under the shader menu"
    );
    let (bx, by) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "the in-game menu was drawn under the shader menu"
    );
    let (cx, cy, ..) = slot2_ui::StateSwitcher::card_rect(&ctx, 0);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - cx).abs() < 0.5 && (*y - cy).abs() < 0.5)),
        "a switcher card was drawn under the shader menu"
    );
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { y, h, .. }
            if (*y - slot2_ui::hud::HUD_MARGIN).abs() < 0.5
                && (*h - slot2_ui::hud::HUD_H).abs() < 0.01)),
        "the top band was drawn over the game"
    );
}

#[test]
fn a_shader_write_that_fails_leaves_the_card_and_the_picture_alone() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("shader-fail") else {
        return;
    };
    // A file that is there and readable, holding something other than a shader: the write that
    // fails has a card to write to. The game was launched before it existed, so it is drawn the
    // way a card that says nothing about shaders is drawn.
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Integer),
            ..Default::default()
        },
    )
    .unwrap();
    let path = card.game_settings_path(&cart);
    let bytes = fs::read(&path).unwrap();
    let effect = a.session().unwrap().shader_effect();
    assert_eq!(effect, Some(slot2_gfx::ShaderEffect::Lcd3x));

    // A directory where the atomic temporary file belongs: the ini stays readable and only the
    // write can fail.
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    let blocker = PathBuf::from(tmp);
    fs::create_dir(&blocker).unwrap();

    let mut now = open_shader(&mut a, Instant::now());
    assert_eq!(shader_row(&a), None, "the card has no shader to open on");
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(shader_row(&a), Some(ShaderPreset::Off));
    now += ms(200);
    tap(&mut a, Button::A, now);

    assert_eq!(
        a.toast_key(),
        Some("shader-save-failed"),
        "a failed save said nothing"
    );
    assert!(matches!(a.screen, Screen::Shader(..)), "{:?}", a.screen);
    assert_eq!(
        shader_row(&a),
        Some(ShaderPreset::Off),
        "the screen left the attempted row"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        bytes,
        "the card moved on a write that failed"
    );
    // The refused choice was the explicit off, which would have made the draw plain: the game
    // is still on the effect it was launched with.
    assert_eq!(
        a.session().unwrap().shader_effect(),
        effect,
        "the session followed a setting the card refused"
    );
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());

    // Left in place, the blocker would keep the next write to this card from landing.
    fs::remove_dir(&blocker).unwrap();
}

#[test]
fn an_unreadable_shader_file_is_not_overwritten_by_the_shader_menu() {
    let _serial = serial();
    let Some((mut a, card, cart)) = playing("shader-unreadable") else {
        return;
    };
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, [0xff, 0xfe, b's', b'=', 0x80, b'\n']).unwrap();
    let damaged = fs::read(&path).unwrap();
    let effect = a.session().unwrap().shader_effect();

    // Reading stays forgiving, so the screen opens on the platform default rather than
    // refusing.
    let mut now = open_shader(&mut a, Instant::now());
    assert_eq!(shader_row(&a), None, "an unreadable file hid the row");
    now += ms(200);
    tap(&mut a, Button::Down, now); // → the explicit off
    assert_eq!(shader_row(&a), Some(ShaderPreset::Off));
    now += ms(200);
    tap(&mut a, Button::A, now);

    assert_eq!(
        a.toast_key(),
        Some("shader-save-failed"),
        "a refused save said nothing"
    );
    assert_eq!(
        shader_row(&a),
        Some(ShaderPreset::Off),
        "the screen left the attempted row"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        damaged,
        "the damaged file was rewritten"
    );
    assert_eq!(
        a.session().unwrap().shader_effect(),
        effect,
        "the session followed a setting the card refused"
    );
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());
}

// -------------------------------------------------------------- the overscan submenu

/// The NES frame the synthetic cart produces. Fixed by the core, and the source the crop and the
/// UVs below are counted from.
const NES_FRAME: (u32, u32) = (256, 240);

/// Where the FCEUmm library is. Only the overscan cases need it, and only because the row is
/// offered by platform: a GBA tells nothing about whether a NES reaches the screen.
fn fceumm() -> Option<PathBuf> {
    let d = repo().join("vendor");
    if d.join(CoreId::Fceumm.file_name()).is_file() {
        Some(d)
    } else {
        eprintln!(
            "no {} in {} — skipping the NES overscan cases (run build/cores.ps1)",
            CoreId::Fceumm.base_name(),
            d.display()
        );
        None
    }
}

/// An iNES file: 16-byte header, one 16 KiB PRG bank, one 8 KiB CHR bank, mapper 0.
///
/// Copied from the core picker's suite rather than shared: test support in another crate is not
/// reachable from here, and the row this file is about is decided by the registry, not by the
/// cart. The image parks the CPU at `SEI; CLD; JMP $C000` and produces a steady 256x240 frame.
fn nes_rom() -> Vec<u8> {
    let mut rom = Vec::with_capacity(16 + 16384 + 8192);
    rom.extend_from_slice(b"NES\x1A");
    rom.push(1); // 16 KiB of PRG
    rom.push(1); // 8 KiB of CHR
    rom.extend_from_slice(&[0; 10]); // flags 6..15: mapper 0, no battery, no trainer

    let mut prg = vec![0u8; 16384];
    prg[0..5].copy_from_slice(&[0x78, 0xD8, 0x4C, 0x00, 0xC0]);
    for v in [0x3FFA, 0x3FFC, 0x3FFE] {
        prg[v] = 0x00;
        prg[v + 1] = 0xC0;
    }
    rom.extend_from_slice(&prg);
    rom.extend_from_slice(&[0u8; 8192]); // blank pattern tables
    rom
}

/// A card with the synthetic NES cart on it, and the app sitting at the game with a frame drawn.
///
/// The shelf opens on the GBA, so the cart is one shelf along — the same walk the core picker's
/// NES cases make.
fn nes_playing(tag: &str) -> Option<(App, Card, Cart)> {
    let cores = fceumm()?;
    let root = scratch(&format!("overscan-{tag}"));
    let card = Card::new(&root);
    card.ensure_layout();
    let rom = card.games_dir(Platform::Nes).join("loop.nes");
    fs::write(&rom, nes_rom()).unwrap();
    let cart = Cart {
        platform: Platform::Nes,
        stem: "loop".into(),
        title: "loop".into(),
        rom,
    };
    let mut a = App::with_card(
        card.clone(),
        cores,
        48_000,
        slot2::tuning_for(&slot2_platform::detect().profile),
        false,
        Screen::List,
    );
    let t = Instant::now();
    tap(&mut a, Button::R1, t);
    assert_eq!(a.platform(), Platform::Nes, "the shelf did not change");
    let t = t + ms(200);
    tap(&mut a, Button::A, t);
    let mut now = t + ms(60);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            run(&mut a, now, 0.2); // a frame for the session to draw
            let _ = a.take_sink_request();
            assert_eq!(
                a.session().unwrap().last_frame().map(|(w, h, _)| (w, h)),
                Some(NES_FRAME),
                "the core's frame is not the one the crop is counted from"
            );
            return Some((a, card, cart));
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

/// The Display submenu, walked to its Overscan row. There is no stored value that lands on the
/// row — it is a way in — so the walk is how a player reaches it.
fn open_overscan_row(a: &mut App, at: Instant) -> Instant {
    let mut now = open_display_row(a, at);
    tap(a, Button::A, now);
    assert!(matches!(a.screen, Screen::Display(..)), "{:?}", a.screen);
    for _ in 0..CROPPING_ROWS.len() {
        if choice_of(a) == DisplayChoice::Overscan {
            return now + ms(200);
        }
        now += ms(200);
        tap(a, Button::Down, now);
    }
    panic!("the Overscan row was never reached: {:?}", choice_of(a));
}

/// The overscan screen, open on the card's own setting.
fn open_overscan(a: &mut App, at: Instant) -> Instant {
    let now = open_overscan_row(a, at);
    tap(a, Button::A, now);
    assert!(matches!(a.screen, Screen::Overscan(..)), "{:?}", a.screen);
    now + ms(200)
}

/// The row the overscan screen is on.
fn overscan_row(a: &App) -> Option<bool> {
    match a.screen {
        Screen::Overscan(_, _, menu) => menu.selected(),
        other => panic!("the overscan menu is not open: {other:?}"),
    }
}

/// The card's setting for this game, as the launcher would read it.
fn write_overscan(card: &Card, cart: &Cart, overscan: Option<bool>) {
    let mut settings = card.read_settings(cart);
    settings.overscan = overscan;
    card.write_settings(cart, &settings).unwrap();
}

/// The game's own draw in the frame, as `(x, y, w, h, uv)`. The session draws one quad before
/// anything else, so the first image in the frame is the game.
fn game_quad(c: &RecordingCanvas) -> Option<(f32, f32, f32, f32, [f32; 4])> {
    c.frame().iter().find_map(|o| match o {
        Op::Image { x, y, w, h, uv, .. } | Op::ImageEffect { x, y, w, h, uv, .. } => {
            Some((*x, *y, *w, *h, *uv))
        }
        _ => None,
    })
}

/// Where the game picture belongs for a visible size: the same placement function the session
/// uses, asked with the frame the crop leaves and the console's own aspect.
fn nes_place(shown: (u32, u32)) -> (f32, f32, f32, f32) {
    let aspect = slot2_retro::def(slot2_retro::Platform::Nes)
        .aspect
        .display(shown);
    let r = slot2_gfx::place(slot2_gfx::ScalePolicy::default(), shown, aspect, (720, 480));
    (r.x as f32, r.y as f32, r.w as f32, r.h as f32)
}

#[test]
fn a_platform_with_nothing_to_crop_has_no_overscan_row() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = playing("no-overscan-row") else {
        return;
    };
    let t = open_display_row(&mut a, Instant::now());
    tap(&mut a, Button::A, t);
    let Screen::Display(_, menu) = a.screen else {
        panic!("the Display menu is not open: {:?}", a.screen)
    };
    // A GBA crops nothing, so the row is not in the set and the box is the five-row one.
    assert_eq!(menu.choices(), &ROWS[..]);
    assert_eq!(menu.box_h(), BOX_H);

    // Two laps of the rows: the Overscan row is not merely unwalked to, it is not there.
    let mut now = t + ms(200);
    for _ in 0..2 * ROWS.len() {
        tap(&mut a, Button::Down, now);
        now += ms(200);
        assert_ne!(
            choice_of(&a),
            DisplayChoice::Overscan,
            "a platform with nothing to crop reached the overscan row"
        );
    }
    for _ in 0..2 * ROWS.len() {
        tap(&mut a, Button::Up, now);
        now += ms(200);
        assert_ne!(
            choice_of(&a),
            DisplayChoice::Overscan,
            "a platform with nothing to crop reached the overscan row"
        );
    }
    assert!(matches!(a.screen, Screen::Display(..)));

    // The NES, for contrast, does offer it: six rows and the taller box.
    let Some((mut n, _, _)) = nes_playing("crop-row") else {
        return;
    };
    let t = open_display_row(&mut n, Instant::now());
    tap(&mut n, Button::A, t);
    let Screen::Display(_, menu) = n.screen else {
        panic!("the Display menu is not open: {:?}", n.screen)
    };
    assert_eq!(menu.choices(), &CROPPING_ROWS[..]);
    assert_eq!(menu.box_h(), BOX_H_CROPPING);
}

#[test]
fn the_overscan_row_opens_on_the_cards_own_setting_and_hands_the_row_back() {
    let _serial = serial();
    for (tag, setting) in [("plain", None), ("crop", Some(true)), ("full", Some(false))] {
        let Some((mut a, card, cart)) = nes_playing(&format!("overscan-open-{tag}")) else {
            return;
        };
        if let Some(overscan) = setting {
            write_overscan(&card, &cart, Some(overscan));
        }
        let path = card.game_settings_path(&cart);
        let before = fs::read(&path).ok();
        let crop = a.session().unwrap().overscan();

        // The Display menu walks to its Overscan row without touching the card, and A opens the
        // overscan screen on what the card says.
        let mut now = open_overscan_row(&mut a, Instant::now());
        assert_eq!(choice_of(&a), DisplayChoice::Overscan);
        assert_eq!(
            fs::read(&path).ok(),
            before,
            "opening the row wrote a setting"
        );
        tap(&mut a, Button::A, now);
        assert_eq!(
            overscan_row(&a),
            setting,
            "{setting:?} was not the opening row"
        );

        // B and MENU both hand back the Display screen, still on its Overscan row, and A goes
        // straight back into the overscan menu on the same row.
        for closer in [Button::B, Button::Menu] {
            now += ms(200);
            tap(&mut a, closer, now);
            match a.screen {
                Screen::Display(_, display) => assert_eq!(
                    display.choice(),
                    DisplayChoice::Overscan,
                    "{closer:?} lost the Overscan row"
                ),
                other => panic!("{closer:?} left on {other:?}"),
            }
            assert_eq!(
                fs::read(&path).ok(),
                before,
                "{closer:?} changed the card on the way out"
            );
            assert_eq!(
                a.session().unwrap().overscan(),
                crop,
                "{closer:?} changed the crop on the way out"
            );
            now += ms(200);
            tap(&mut a, Button::A, now);
            assert_eq!(
                overscan_row(&a),
                setting,
                "{closer:?} reopened on another row"
            );
        }
    }
}

/// One case of the overscan menu: the choice the player makes, the choice the card held before
/// it, the crop that choice means on a NES, the visible size it leaves of the core's 256x240
/// frame, and the UVs that frame is sampled with.
type OverscanCase = (
    Option<bool>,
    Option<bool>,
    slot2_retro::Overscan,
    (u32, u32),
    [f32; 4],
);

#[test]
fn every_overscan_choice_is_saved_exactly_and_applied_at_once() {
    let _serial = serial();
    // The three meanings a card can hold, what each draws on a NES, and how the frame behind the
    // menu must change: the platform default and an explicit crop are the registry's own eight
    // rows, and "show the whole image" is the full frame. The UVs are worked out from the core's
    // own 256x240 frame rather than read off the code.
    let cases: [OverscanCase; 3] = [
        (
            None,
            Some(false),
            slot2_retro::Overscan::rows(8),
            (256, 224),
            [0.0, 8.0 / 240.0, 1.0, 232.0 / 240.0],
        ),
        (
            Some(true),
            Some(false),
            slot2_retro::Overscan::rows(8),
            (256, 224),
            [0.0, 8.0 / 240.0, 1.0, 232.0 / 240.0],
        ),
        (
            Some(false),
            Some(true),
            slot2_retro::Overscan::NONE,
            (256, 240),
            [0.0, 0.0, 1.0, 1.0],
        ),
    ];
    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let nes = slot2_retro::Platform::Nes;

    for (i, (want, before, crop, shown, uv)) in cases.into_iter().enumerate() {
        let tag = format!("overscan-choose{i}");
        let Some((mut a, card, cart)) = nes_playing(&tag) else {
            return;
        };
        // The card starts on the other choice, so the file is there and the row the screen opens
        // on is read from it rather than from the launch.
        write_overscan(&card, &cart, before);
        let path = card.game_settings_path(&cart);
        let mut now = open_overscan(&mut a, Instant::now());
        assert_eq!(overscan_row(&a), before, "the screen ignored the card");

        // Walk to the row this case is about.
        for _ in 0..slot2_ui::overscan_menu::ROWS.len() {
            if overscan_row(&a) == want {
                break;
            }
            now += ms(200);
            tap(&mut a, Button::Down, now);
        }
        assert_eq!(overscan_row(&a), want, "{want:?} was not reachable");

        // One warm draw, so the texture the next draw reuses is already there. The game was
        // launched while the card said nothing about overscan, so it is on the platform's own
        // crop: the file written above is what the menu opens on, not what the game started with.
        let mut c = RecordingCanvas::new(720, 480);
        a.draw(&mut c, &mut ctx, now);
        assert_eq!(
            a.session().unwrap().overscan(),
            Session::overscan_for(nes, None),
            "the launch crop is not the platform's own"
        );
        let frames = a.session().unwrap().frames_run();
        let frame_before = a
            .session()
            .unwrap()
            .last_frame()
            .map(|(w, h, d)| (w, h, d.to_vec()));
        let audio_before = a.session().unwrap().audio_health();
        let texture_before = texture_ops(&c);

        now += ms(200);
        tap(&mut a, Button::A, now);
        let case = format!("{want:?}");

        // Saved first: the card holds the game's own meaning, not a computed crop.
        assert_eq!(a.toast_key(), None, "{case}: saving failed");
        assert_eq!(
            card.read_settings(&cart).overscan,
            want,
            "{case}: the card has something else"
        );
        match want {
            None => assert!(
                !path.exists(),
                "{case}: a file was left holding nothing but the default"
            ),
            Some(true) => assert!(
                fs::read_to_string(&path).unwrap().contains("overscan = on"),
                "{case}: an explicit crop is not spelled on"
            ),
            Some(false) => assert!(
                fs::read_to_string(&path)
                    .unwrap()
                    .contains("overscan = off"),
                "{case}: an explicit full image is not spelled off"
            ),
        }
        assert_eq!(overscan_row(&a), want, "{case}: the screen left the row");

        // And put on the running session at once: the same picture, cropped differently, with its
        // placement worked out again for the visible size the crop leaves.
        assert_eq!(
            a.session().unwrap().overscan(),
            crop,
            "{case}: the session was not told"
        );
        assert_eq!(
            slot2_gfx::cropped_size(NES_FRAME, crop.left, crop.top, crop.right, crop.bottom),
            shown,
            "{case}: the visible size is not the one the crop leaves"
        );
        a.draw(&mut c, &mut ctx, now);
        let quad = game_quad(&c).expect("{case}: the game was not drawn");
        assert_eq!(quad.4, uv, "{case}: the next draw sampled the wrong rows");
        assert_eq!(
            (quad.0, quad.1, quad.2, quad.3),
            nes_place(shown),
            "{case}: the picture was not placed for its visible size"
        );
        assert_eq!(game_quad(&c), Some(quad), "{case}: the picture moved");
        // The two crops are not the same picture: the placement above is a real answer, not one
        // that happened to hold for either.
        if crop == slot2_retro::Overscan::NONE {
            assert_ne!(nes_place((256, 240)), nes_place((256, 224)));
        }

        // Nothing else moved: no frame, no new picture, no texture work, no audio, no sink.
        assert_eq!(
            a.session().unwrap().frames_run(),
            frames,
            "{case}: a frame ran"
        );
        assert_eq!(
            a.session()
                .unwrap()
                .last_frame()
                .map(|(w, h, d)| (w, h, d.to_vec())),
            frame_before,
            "{case}: the core's picture moved"
        );
        assert_eq!(
            texture_ops(&c),
            texture_before,
            "{case}: a draw uploaded, rewrote or freed the game texture"
        );
        assert_eq!(
            a.session().unwrap().audio_health(),
            audio_before,
            "{case}: a draw touched the audio"
        );
        assert!(
            a.take_sink_request().is_none(),
            "{case}: a sink was asked for"
        );

        // Closing and reopening lands on the value that was saved.
        now += ms(200);
        tap(&mut a, Button::B, now);
        now += ms(200);
        tap(&mut a, Button::A, now);
        assert_eq!(
            overscan_row(&a),
            want,
            "{case}: reopening did not show what was saved"
        );
    }
}

#[test]
fn setting_and_clearing_an_overscan_keeps_every_other_key() {
    let _serial = serial();
    let Some((mut a, card, cart)) = nes_playing("overscan-keys") else {
        return;
    };
    // Everything else a game can have set, plus a key this version has never heard of.
    card.write_settings(
        &cart,
        &GameSettings {
            core: Some("fceumm".into()),
            scale: Some(ScaleMode::Fill),
            overscan: Some(false),
            rewind: Some(false),
            shader: Some(ShaderPreset::Lcd3x),
            overlay: None,
        },
    )
    .unwrap();
    let path = card.game_settings_path(&cart);
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("shutdown = fast\n");
    fs::write(&path, text).unwrap();

    // On: up from the explicit off is the explicit crop.
    let mut now = open_overscan(&mut a, Instant::now());
    assert_eq!(overscan_row(&a), Some(false));
    now += ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(overscan_row(&a), Some(true));
    now += ms(200);
    tap(&mut a, Button::A, now);

    let settings = card.read_settings(&cart);
    assert_eq!(settings.overscan, Some(true));
    assert_eq!(settings.core.as_deref(), Some("fceumm"), "core was lost");
    assert_eq!(settings.scale, Some(ScaleMode::Fill), "scale was lost");
    assert_eq!(settings.rewind, Some(false), "rewind was lost");
    assert_eq!(
        settings.shader,
        Some(ShaderPreset::Lcd3x),
        "shader was lost"
    );
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("overscan = on"), "{text}");
    assert!(
        text.contains("shutdown = fast"),
        "the unknown key was lost: {text}"
    );

    // And back to the platform default, which takes the overscan key and nothing else. The file
    // stays: the other settings still live in it.
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(overscan_row(&a), Some(false));
    now += ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(overscan_row(&a), None);
    now += ms(200);
    tap(&mut a, Button::A, now);

    let settings = card.read_settings(&cart);
    assert_eq!(settings.overscan, None);
    assert_eq!(settings.core.as_deref(), Some("fceumm"));
    assert_eq!(settings.scale, Some(ScaleMode::Fill));
    assert_eq!(settings.rewind, Some(false));
    assert_eq!(settings.shader, Some(ShaderPreset::Lcd3x));
    let text = fs::read_to_string(&path).unwrap();
    assert!(!text.contains("overscan"), "the key is still there: {text}");
    assert!(text.contains("shutdown = fast"), "{text}");

    // A file holding nothing but the overscan goes away entirely, again through the App.
    fs::remove_file(&path).unwrap();
    card.write_settings(
        &cart,
        &GameSettings {
            overscan: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(path.is_file(), "the overscan-only file was not there");
    now += ms(200);
    tap(&mut a, Button::A, now);
    assert_eq!(a.toast_key(), None, "removing the override failed");
    assert!(!path.exists(), "the App left an all-default file behind");
}

#[test]
fn an_overscan_write_that_fails_leaves_the_card_and_the_picture_alone() {
    let _serial = serial();
    let Some((mut a, card, cart)) = nes_playing("overscan-fail") else {
        return;
    };
    // The card asks for the crop that is what the game already launched with, so the choice that
    // would move the picture is the one refused below.
    write_overscan(&card, &cart, Some(true));
    let path = card.game_settings_path(&cart);
    let bytes = fs::read(&path).unwrap();
    let crop = a.session().unwrap().overscan();
    assert_eq!(crop, slot2_retro::Overscan::rows(8), "the launch crop");

    // A directory where the atomic temporary file belongs: the ini stays readable and only the
    // write can fail.
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    let blocker = PathBuf::from(tmp);
    fs::create_dir(&blocker).unwrap();

    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut now = open_overscan(&mut a, Instant::now());
    assert_eq!(
        overscan_row(&a),
        Some(true),
        "the card has nothing to open on"
    );
    now += ms(200);
    tap(&mut a, Button::Down, now); // on → off, the choice that would drop the crop
    assert_eq!(overscan_row(&a), Some(false));
    now += ms(200);
    tap(&mut a, Button::A, now);

    assert_eq!(
        a.toast_key(),
        Some("overscan-save-failed"),
        "a failed save said nothing"
    );
    assert!(matches!(a.screen, Screen::Overscan(..)), "{:?}", a.screen);
    assert_eq!(
        overscan_row(&a),
        Some(false),
        "the screen left the attempted row"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        bytes,
        "the card moved on a write that failed"
    );
    assert_eq!(
        a.session().unwrap().overscan(),
        crop,
        "the session followed a setting the card refused"
    );
    // And the picture is still the cropped one: the refused choice would have shown the whole
    // frame, and it did not.
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, now);
    assert_eq!(
        game_quad(&c).expect("the game was not drawn").4,
        [0.0, 8.0 / 240.0, 1.0, 232.0 / 240.0],
        "the refused choice reached the picture"
    );
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());

    // Left in place, the blocker would keep the next write to this card from landing.
    fs::remove_dir(&blocker).unwrap();
}

#[test]
fn an_unreadable_overscan_file_is_not_overwritten_by_the_overscan_menu() {
    let _serial = serial();
    let Some((mut a, card, cart)) = nes_playing("overscan-unreadable") else {
        return;
    };
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, [0xff, 0xfe, b'o', b'=', 0x80, b'\n']).unwrap();
    let damaged = fs::read(&path).unwrap();
    let crop = a.session().unwrap().overscan();
    assert_eq!(crop, slot2_retro::Overscan::rows(8), "the launch crop");

    // Reading stays forgiving, so the screen opens on the platform default rather than refusing.
    // The choice that would move the picture is the whole frame, two rows down.
    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut now = open_overscan(&mut a, Instant::now());
    assert_eq!(overscan_row(&a), None, "an unreadable file hid the row");
    now += ms(200);
    tap(&mut a, Button::Down, now); // → the explicit crop
    assert_eq!(overscan_row(&a), Some(true));
    now += ms(200);
    tap(&mut a, Button::Down, now); // → the whole frame
    assert_eq!(overscan_row(&a), Some(false));
    now += ms(200);
    tap(&mut a, Button::A, now);

    assert_eq!(
        a.toast_key(),
        Some("overscan-save-failed"),
        "a refused save said nothing"
    );
    assert_eq!(
        overscan_row(&a),
        Some(false),
        "the screen left the attempted row"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        damaged,
        "the damaged file was rewritten"
    );
    assert_eq!(
        a.session().unwrap().overscan(),
        crop,
        "the session followed a setting the card refused"
    );
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, now);
    assert_eq!(
        game_quad(&c).expect("the game was not drawn").4,
        [0.0, 8.0 / 240.0, 1.0, 232.0 / 240.0],
        "the refused choice reached the picture"
    );
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());
}

#[test]
fn the_overscan_screen_pauses_the_session_and_keeps_it() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = nes_playing("overscan-pause") else {
        return;
    };
    let t = open_overscan(&mut a, Instant::now());
    assert!(
        a.session().is_some(),
        "the overscan screen dropped the session"
    );
    assert!(a.audio_paused(), "the game kept its sound under the screen");
    assert!(
        a.take_sink_request().is_none(),
        "the overscan screen touched the audio"
    );

    let frames = a.session().unwrap().frames_run();
    let audio = a.session().unwrap().audio_health();
    let now = run(&mut a, t, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran under the overscan screen"
    );
    assert_eq!(
        a.session().unwrap().audio_health(),
        audio,
        "audio advanced under the overscan screen"
    );
    let _ = now;
}

#[test]
fn drawing_shows_the_game_frame_under_the_overscan_menu() {
    let _serial = serial();
    let Some((mut a, _card, _cart)) = nes_playing("overscan-draw") else {
        return;
    };
    let t = open_overscan(&mut a, Instant::now());

    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = RecordingCanvas::new(720, 480);
    a.draw(&mut c, &mut ctx, t + ms(100));
    let ops = c.frame();

    // The game's own frame is the first mark: no wallpaper under it, and the menu's dim is the
    // first whole-panel rect after it.
    let quad = game_quad(&c).expect("the game frame was not drawn");
    assert_eq!(quad.4, [0.0, 8.0 / 240.0, 1.0, 232.0 / 240.0]);
    let dim = ops
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, w, h, color }
            if *x == 0.0 && *y == 0.0 && (*w - 720.0).abs() < 0.5 && (*h - 480.0).abs() < 0.5
                && *color == slot2_ui::overscan_menu::DIM)
        })
        .expect("the overscan menu was not drawn");
    assert!(dim > 0, "the menu went down before the game");
    assert!(
        matches!(ops[0], Op::Image { .. } | Op::ImageEffect { .. }),
        "the first mark was not the game frame: {:?}",
        ops[0]
    );

    // Not the parent Display menu, not the in-game menu, not the switcher, not the badge.
    let Screen::Overscan(_, display, _) = a.screen else {
        panic!("the overscan screen is not open: {:?}", a.screen)
    };
    let (dx, dy) = display.box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - dx).abs() < 0.5 && (*y - dy).abs() < 0.5)),
        "the Display menu was drawn under the overscan menu"
    );
    let (bx, by) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "the in-game menu was drawn under the overscan menu"
    );
    let (cx, cy, ..) = slot2_ui::StateSwitcher::card_rect(&ctx, 0);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - cx).abs() < 0.5 && (*y - cy).abs() < 0.5)),
        "a switcher card was drawn under the overscan menu"
    );
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { y, h, .. }
            if (*y - slot2_ui::hud::HUD_MARGIN).abs() < 0.5
                && (*h - slot2_ui::hud::HUD_H).abs() < 0.01)),
        "the top band was drawn over the game"
    );
}

#[test]
fn both_packs_have_the_overscan_failure_words() {
    // The key the App toasts on a refused write, spelled out in both packs: a ko pack that lacked
    // it would fall back to English and read as half-translated rather than as missing.
    for (lang, want) in [
        ("en", "Could not save the overscan setting"),
        ("ko", "오버스캔 설정을 저장하지 못했습니다"),
    ] {
        let ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, lang, Vec::new(), None);
        assert_eq!(ctx.i18n.t("overscan-save-failed"), want, "{lang}");
    }
}
