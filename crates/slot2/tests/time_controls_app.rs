//! The R2 time controls over a real session: holding is momentary, a double tap latches, and L2
//! outranks both. Skips loudly without `vendor/mgba_libretro.*`.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};
use slot2_ui::hud::{HUD_H, HUD_MARGIN};
use slot2_ui::insert::EJECT_S;
use slot2_ui::{InGameMenu, PowerMenu, UiCtx, PX_HINT};

use slot2::app::{App, Screen, FAST_FORWARD};

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

/// Two R2 presses inside the gesture layer's window. The latch action fires on the second
/// press; the caller releases it afterwards, as the player eventually would.
fn double_tap_r2(a: &mut App, at: Instant) {
    a.feed(&ev(Button::R2, true, at));
    a.feed(&ev(Button::R2, false, at + ms(40)));
    a.feed(&ev(Button::R2, true, at + ms(100)));
    a.tick(at + ms(120));
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
    let root = std::env::temp_dir().join(format!("slot2-ff-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// An app with the test ROM on the shelf, sitting at the game.
fn playing(cores: PathBuf, tag: &str) -> App {
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    fs::copy(
        repo().join("assets/test/arm.gba"),
        card.games_dir(Platform::Gba).join("arm.gba"),
    )
    .unwrap();
    let mut a = App::with_card(
        card,
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
            return a;
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

fn speed(a: &App) -> u32 {
    a.session().expect("no session").speed()
}

// ------------------------------------------------------------------ the badge

/// A context the badge can be measured in.
fn badge_ctx() -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(slot2_platform::detect().profile, "en", vec![fonts], None)
}

/// How wide the badge's word is, from the message itself.
fn want_width(key: &str, speed: Option<i64>) -> f32 {
    let mut ctx = badge_ctx();
    let text = match speed {
        Some(n) => ctx.i18n.t_args(key, &[("speed", n.into())]),
        None => ctx.i18n.t(key),
    };
    slot2_ui::face::measure(&mut ctx, &text, PX_HINT)
}

/// The time-control badge as Task 43 draws it: one plate `HUD_H` tall at the top margin,
/// centred on the panel, with one line of text on it. Returns `(word width, plate index, text
/// index)`, or `None` when the frame wears no badge. The badge is identified by that geometry
/// rather than by a texture id, which belongs to whichever canvas drew it.
fn badge(c: &RecordingCanvas, panel_w: u32) -> Option<(f32, usize, usize)> {
    let ops = c.frame();
    let plate = ops.iter().enumerate().find_map(|(i, o)| match o {
        Op::Rect { x, y, w, h, .. }
            if (*h - HUD_H).abs() < 0.01
                && (*y - HUD_MARGIN).abs() < 0.01
                && ((x + w / 2.0) - panel_w as f32 / 2.0).abs() < 0.51 =>
        {
            Some(i)
        }
        _ => None,
    })?;
    let (text, width) = ops
        .iter()
        .enumerate()
        .skip(plate)
        .find_map(|(i, o)| match o {
            Op::Image { y, w, .. } if (*y - HUD_MARGIN).abs() < HUD_H => Some((i, *w)),
            _ => None,
        })?;
    Some((width, plate, text))
}

/// Where the game's own frame was drawn: the quad across the whole panel, through the effect the
/// GBA gets by default when its game says nothing about shaders. The badge's own plate and line
/// are plain draws and are deliberately not matched here.
fn game_frame_at(c: &RecordingCanvas) -> Option<usize> {
    c.frame().iter().position(|o| match o {
        Op::ImageEffect {
            x, y, w, h, effect, ..
        } => {
            *x == 0.0
                && *y == 0.0
                && (*w - 720.0).abs() < 0.5
                && (*h - 480.0).abs() < 0.5
                && *effect == slot2_gfx::ShaderEffect::Lcd3x
        }
        _ => false,
    })
}

fn uploads(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count()
}

#[test]
fn a_hold_is_momentary_and_a_double_tap_latches() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let mut a = playing(cores, "latch");
    let mut now = Instant::now();

    // Held: fast, and only while it is held.
    a.feed(&ev(Button::R2, true, now));
    now = run(&mut a, now, 0.1);
    assert_eq!(speed(&a), FAST_FORWARD, "a hold did not accelerate");
    a.feed(&ev(Button::R2, false, now));
    now = run(&mut a, now, 0.1);
    assert_eq!(speed(&a), 1, "letting go did not restore the speed");

    // Double tap: the same fast forward, with the button out of it.
    double_tap_r2(&mut a, now);
    a.feed(&ev(Button::R2, false, now + ms(200)));
    now = run(&mut a, now + ms(300), 0.1);
    assert_eq!(speed(&a), FAST_FORWARD, "the double tap did not latch");

    // And a second double tap lets it go.
    double_tap_r2(&mut a, now);
    a.feed(&ev(Button::R2, false, now + ms(200)));
    now = run(&mut a, now + ms(300), 0.1);
    assert_eq!(speed(&a), 1, "the latch would not let go");

    // The game really is running faster while it is on: four core frames a frame.
    double_tap_r2(&mut a, now);
    a.feed(&ev(Button::R2, false, now + ms(200)));
    let frames = a.session().unwrap().frames_run();
    run(&mut a, now + ms(300), 0.1);
    let per_frame = (a.session().unwrap().frames_run() - frames) as f32 / 6.0;
    assert!(
        per_frame > 3.0,
        "a latched fast forward ran {per_frame} frames a frame"
    );
}

#[test]
fn rewind_outranks_the_latch_and_the_menu_keeps_it() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let mut a = playing(cores, "rewind");
    let mut now = Instant::now();

    double_tap_r2(&mut a, now);
    a.feed(&ev(Button::R2, false, now + ms(200)));
    now = run(&mut a, now + ms(300), 0.3);
    assert_eq!(speed(&a), FAST_FORWARD);

    // L2: the core does not run forward, latch or no latch.
    let frames = a.session().unwrap().frames_run();
    a.feed(&ev(Button::L2, true, now));
    now = run(&mut a, now, 0.2);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "rewind let the core run on"
    );
    a.feed(&ev(Button::L2, false, now));
    now = run(&mut a, now, 0.1);
    assert!(
        a.session().unwrap().frames_run() > frames,
        "letting go of rewind did not start the game again"
    );
    assert_eq!(speed(&a), FAST_FORWARD, "rewind lost the latch");

    // The menu pauses, and does not lose the latch.
    tap(&mut a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    let frames = a.session().unwrap().frames_run();
    now = run(&mut a, now + ms(200), 0.2);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran under the menu"
    );
    tap(&mut a, Button::B, now);
    assert_eq!(a.screen, Screen::Playing);
    run(&mut a, now + ms(200), 0.1);
    assert_eq!(speed(&a), FAST_FORWARD, "the menu lost the latch");
}

#[test]
fn the_badge_follows_the_buttons_and_the_latch() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let mut a = playing(cores, "badge");
    // A frame of play first: the session draws nothing until the core has produced a picture.
    let t = run(&mut a, Instant::now(), 0.2);
    let mut ctx = badge_ctx();
    let mut c = RecordingCanvas::new(720, 480);
    let fast = want_width("time-fast-forward", Some(4));
    let rewind = want_width("time-rewind", None);

    // A plain game: the frame, and no badge on it.
    a.draw(&mut c, &mut ctx, t);
    assert!(game_frame_at(&c).is_some(), "the game frame was not drawn");
    assert!(badge(&c, 720).is_none(), "a plain frame wore a time badge");

    // R2 held: the badge is there at once, over the game.
    a.feed(&ev(Button::R2, true, t + ms(100)));
    a.draw(&mut c, &mut ctx, t + ms(120));
    let (width, plate, text) = badge(&c, 720).expect("a held R2 showed no badge");
    assert!(
        (width - fast).abs() < 0.51,
        "the badge said {width}, not {fast}"
    );
    let game = game_frame_at(&c).expect("the game went missing under the badge");
    assert!(
        game < plate && plate < text,
        "the badge went down before the game ({game} {plate} {text})"
    );
    // A second, identical frame: the word is already rasterised.
    let warm = c.ops.len();
    a.draw(&mut c, &mut ctx, t + ms(140));
    assert_eq!(
        uploads(&c.ops[warm..]),
        0,
        "a warm badge uploaded something"
    );
    assert!(
        badge(&c, 720).is_some(),
        "the badge went out with the second frame"
    );

    // Let go: gone again.
    a.feed(&ev(Button::R2, false, t + ms(200)));
    a.draw(&mut c, &mut ctx, t + ms(220));
    assert!(badge(&c, 720).is_none(), "letting go left the badge up");

    // A double tap latches it: the badge stays after the release.
    double_tap_r2(&mut a, t + ms(300));
    a.feed(&ev(Button::R2, false, t + ms(500)));
    a.draw(&mut c, &mut ctx, t + ms(520));
    let (width, ..) = badge(&c, 720).expect("the latch showed no badge");
    assert!(
        (width - fast).abs() < 0.51,
        "the latched badge said {width}"
    );

    // L2 takes it over, and hands it back when it is let go.
    a.feed(&ev(Button::L2, true, t + ms(600)));
    a.draw(&mut c, &mut ctx, t + ms(620));
    let (width, ..) = badge(&c, 720).expect("rewind showed no badge");
    assert!(
        (width - rewind).abs() < 0.51,
        "rewind said {width}, not {rewind}"
    );
    a.feed(&ev(Button::L2, false, t + ms(700)));
    a.draw(&mut c, &mut ctx, t + ms(720));
    let (width, ..) = badge(&c, 720).expect("the latch did not come back");
    assert!(
        (width - fast).abs() < 0.51,
        "the latch came back as {width}"
    );

    // And the second double tap takes it away for good.
    double_tap_r2(&mut a, t + ms(800));
    a.feed(&ev(Button::R2, false, t + ms(1000)));
    a.draw(&mut c, &mut ctx, t + ms(1020));
    assert!(badge(&c, 720).is_none(), "unlatching left the badge up");
}

#[test]
fn a_menu_hides_the_badge_without_losing_the_latch() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let mut a = playing(cores, "hidden");
    let t = run(&mut a, Instant::now(), 0.2);
    let mut ctx = badge_ctx();
    let mut c = RecordingCanvas::new(720, 480);

    double_tap_r2(&mut a, t);
    a.feed(&ev(Button::R2, false, t + ms(200)));
    a.draw(&mut c, &mut ctx, t + ms(220));
    assert!(badge(&c, 720).is_some(), "the latch showed no badge");

    // Into the in-game menu: the latch is still set, and out of sight.
    tap(&mut a, Button::Menu, t + ms(300));
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    a.draw(&mut c, &mut ctx, t + ms(400));
    assert!(badge(&c, 720).is_none(), "a menu wore the badge");

    // And back out to the game: there it is again, still fast.
    tap(&mut a, Button::B, t + ms(500));
    assert_eq!(a.screen, Screen::Playing);
    a.draw(&mut c, &mut ctx, t + ms(600));
    assert!(
        badge(&c, 720).is_some(),
        "the latch was lost under the menu"
    );
    a.run_frame();
    assert_eq!(speed(&a), FAST_FORWARD, "the menu lost the speed");
}

#[test]
fn drawing_the_badge_changes_nothing() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let mut a = playing(cores, "quiet");
    let t = run(&mut a, Instant::now(), 0.2);
    let mut ctx = badge_ctx();
    let mut c = RecordingCanvas::new(720, 480);

    // Both paths live: latched fast forward, and L2 held on top of it.
    double_tap_r2(&mut a, t);
    a.feed(&ev(Button::L2, true, t + ms(200)));
    let frames = a.session().unwrap().frames_run();
    let rewind = a.session().unwrap().rewind_state();
    let speed_before = speed(&a);

    for _ in 0..4 {
        a.draw(&mut c, &mut ctx, t + ms(300));
    }
    assert!(badge(&c, 720).is_some(), "nothing was drawn to look at");
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "drawing advanced the core"
    );
    assert_eq!(speed(&a), speed_before, "drawing changed the speed");
    assert_eq!(
        a.session().unwrap().rewind_state(),
        rewind,
        "drawing ate rewind history"
    );
    assert_eq!(a.screen, Screen::Playing, "drawing changed the screen");
    assert!(a.take_sink_request().is_none(), "drawing asked for a sink");
}

#[test]
fn only_a_live_game_wears_the_badge() {
    // No core, no session: every screen the app can be on draws no badge, and neither does a
    // `Playing` screen with nothing running.
    let screens = [
        Screen::Splash,
        Screen::List,
        Screen::Inserting,
        Screen::Ejecting,
        Screen::Playing,
        Screen::InGame(InGameMenu::default()),
        Screen::Switcher(InGameMenu::default()),
        Screen::Power(PowerMenu::default()),
    ];
    let mut ctx = badge_ctx();
    for screen in screens {
        let mut a = App::new(false);
        a.screen = screen;
        // Held and latched as far as a screen with no game can be.
        a.feed(&ev(Button::R2, true, Instant::now()));
        let mut c = RecordingCanvas::new(720, 480);
        a.draw(&mut c, &mut ctx, Instant::now());
        assert!(
            badge(&c, 720).is_none(),
            "{screen:?} wore a time-control badge"
        );
    }
}

#[test]
fn a_new_session_starts_unlatched() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let mut a = playing(cores, "reset");
    let mut now = Instant::now();

    double_tap_r2(&mut a, now);
    a.feed(&ev(Button::R2, false, now + ms(200)));
    now = run(&mut a, now + ms(300), 0.1);
    assert_eq!(speed(&a), FAST_FORWARD);

    // Eject with the MENU hold, and launch the cart again.
    a.feed(&ev(Button::Menu, true, now));
    for _ in 0..180 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen != Screen::Playing {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "the hold did not eject");
    now = run(&mut a, now, EJECT_S + 0.1);
    assert_eq!(a.screen, Screen::List);

    tap(&mut a, Button::A, now);
    let mut now = now + ms(60);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            break;
        }
    }
    assert_eq!(a.screen, Screen::Playing, "the cart never came back in");
    run(&mut a, now, 0.1);
    assert_eq!(
        speed(&a),
        1,
        "the new session inherited the last one's fast forward"
    );
}
