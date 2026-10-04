//! The overlay inside the running app: where it sits in the frame, when its texture is made,
//! and when it goes.
//!
//! These tests run a real core — mGBA, and gpSP for the core-change cases — because what is
//! under test is the order the app draws a game's frame and its overlay in, and the frame is the
//! core's. A missing core is a failure here rather than a skip: the file would otherwise pass
//! without ever drawing anything.
//!
//! One canvas is used per test and drawn into repeatedly, so a texture id means the same texture
//! in every frame; the ops a single frame added are the slice `draw_frame` hands back.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Color, Op, RecordingCanvas, TexId};
use slot2_input::{Button, Event};
use slot2_platform::Geometry;
use slot2_store::{Card, Cart, GameSettings, Platform};
use slot2_ui::{InGameMenu, PowerMenu, UiCtx};

use slot2::app::{App, Screen};
use slot2::overlay::{card_overlay_path, geometry_for_panel, overlay_enabled};

/// A libretro core is a library loaded once per process, so these tests take turns.
static SERIAL: Mutex<()> = Mutex::new(());
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn core_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}_libretro.dll")
    } else if cfg!(target_os = "macos") {
        format!("{base}_libretro.dylib")
    } else {
        format!("{base}_libretro.so")
    }
}

/// The library this file needs. A missing one fails the test rather than skipping it.
fn vendor() -> PathBuf {
    let d = repo().join("vendor");
    let name = core_name("mgba");
    assert!(
        d.join(&name).is_file(),
        "no {name} in {}: the overlay tests need a real core (run build/cores.ps1)",
        d.display()
    );
    d
}

fn tuning() -> slot2_retro::Tuning {
    slot2::tuning_for(&slot2_platform::detect().profile)
}

/// The panel the app is running on, which is also the size an overlay has to be.
fn panel() -> (u32, u32) {
    tuning().geometry
}

fn geometry() -> Geometry {
    geometry_for_panel(panel()).expect("the host panel is one of the three geometries")
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
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "slot2-overlay-app-{tag}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

// ------------------------------------------------------------------ pictures

/// A panel-sized RGBA picture: black, with one marker that carries `seed` and one fully
/// transparent pixel. Quick to encode, and never two fixtures the same bytes.
fn panel_pixels(w: u32, h: u32, seed: u8) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    let marker = ((20 * w + 10) * 4) as usize;
    out[marker] = seed;
    out[marker + 1] = 0x11;
    out[marker + 2] = 0x22;
    out[marker + 3] = 0xFF;
    let clear = ((30 * w + 40) * 4) as usize;
    out[clear] = 0x30;
    out[clear + 1] = 0x40;
    out[clear + 2] = 0x50;
    out[clear + 3] = 0x00;
    out
}

fn png_rgba(w: u32, h: u32, pixels: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(pixels).unwrap();
    }
    out
}

/// What the card holds at the overlay's path, written before the launch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Picture {
    None,
    Panel,
    Corrupt,
    /// A valid PNG of another supported panel: the picture is fine, the pair is wrong.
    WrongPanel,
}

// ------------------------------------------------------------------ fixtures

struct Fixture {
    card: Card,
    cart: Cart,
    cores: PathBuf,
}

/// A card with the test ROM on it, a game setting and a picture, all in place before the app is
/// built — the launch boundary is where the overlay is resolved, so nothing after it counts.
fn fixture(tag: &str, overlay: Option<bool>, picture: Picture) -> Fixture {
    let cores = vendor();
    let root = scratch(tag);
    let card = Card::new(root.join("card"));
    card.ensure_layout();
    let rom = card.games_dir(Platform::Gba).join("arm.gba");
    fs::copy(repo().join("assets/test/arm.gba"), &rom).unwrap();
    let cart = Cart {
        platform: Platform::Gba,
        stem: "arm".into(),
        title: "arm".into(),
        rom,
    };

    if let Some(overlay) = overlay {
        let mut settings = card.read_settings(&cart);
        settings.overlay = Some(overlay);
        if settings != GameSettings::default() {
            card.write_settings(&cart, &settings).unwrap();
        }
    }

    let (pw, ph) = panel();
    let path = card_overlay_path(&card, Platform::Gba, geometry());
    match picture {
        Picture::None => {}
        Picture::Panel => {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, png_rgba(pw, ph, &panel_pixels(pw, ph, 0xC1))).unwrap();
        }
        Picture::Corrupt => {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"not a picture").unwrap();
        }
        Picture::WrongPanel => {
            let other = if geometry() == Geometry::W640H480 {
                Geometry::W720H480
            } else {
                Geometry::W640H480
            };
            let (ow, oh) = other.size();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, png_rgba(ow, oh, &panel_pixels(ow, oh, 0xC2))).unwrap();
        }
    }

    Fixture { card, cart, cores }
}

impl Fixture {
    fn app_with(&self, cores: PathBuf, screen: Screen) -> App {
        App::with_card(self.card.clone(), cores, 48_000, tuning(), false, screen)
    }

    fn app(&self, screen: Screen) -> App {
        self.app_with(self.cores.clone(), screen)
    }

    /// The app at the game, with a frame of the core drawn.
    fn playing(&self) -> App {
        let mut a = self.app(Screen::List);
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
                return a;
            }
        }
        panic!("the test cart never reached the game: {:?}", a.screen);
    }
}

// ------------------------------------------------------------------ draw helpers

fn canvas() -> RecordingCanvas {
    let (pw, ph) = panel();
    RecordingCanvas::new(pw, ph)
}

/// Draw one frame and hand back the span of ops it added. The canvas is the test's own, drawn
/// into again and again, so a texture id keeps meaning the same texture.
fn draw_frame(a: &mut App, c: &mut RecordingCanvas, at: Instant) -> (usize, usize) {
    let from = c.ops.len();
    let mut ctx = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    a.draw(c, &mut ctx, at);
    (from, c.ops.len())
}

/// The ops one frame added, as `draw_frame` reported them.
fn frame_ops(c: &RecordingCanvas, frame: (usize, usize)) -> &[Op] {
    &c.ops[frame.0..frame.1]
}

/// The game's own draw: the first picture after the session's own clear. The shelf's wallpaper
/// is a picture too, so the clear is what separates a game frame from one.
fn game_at(ops: &[Op]) -> Option<usize> {
    let start = ops
        .iter()
        .rposition(|o| matches!(o, Op::Clear(_)))
        .map_or(0, |i| i + 1);
    ops[start..]
        .iter()
        .position(|o| matches!(o, Op::Image { .. } | Op::ImageEffect { .. }))
        .map(|i| i + start)
}

/// The overlay's texture, as the upload that made it: a panel-sized RGBA upload is the overlay
/// and nothing else in these frames — the core's picture is the console's own size, and the
/// shelf's gradient is one texel wide.
fn overlay_upload(ops: &[Op]) -> Option<TexId> {
    let (pw, ph) = panel();
    ops.iter().find_map(|o| match o {
        Op::UploadRgba8 { id, w, h } if *w == pw && *h == ph => Some(*id),
        _ => None,
    })
}

/// Whether this frame drew the overlay: a plain image of its texture over the whole panel, full
/// UV, white tint. Not an effect — an overlay is a picture, not something the shader runs
/// through.
fn overlay_at(ops: &[Op], tex: TexId) -> Option<usize> {
    let (pw, ph) = panel();
    ops.iter().rposition(|o| {
        matches!(o, Op::Image { tex: t, x, y, w, h, uv, tint }
        if *t == tex
            && *x == 0.0 && *y == 0.0
            && (*w - pw as f32).abs() < 0.5 && (*h - ph as f32).abs() < 0.5
            && *uv == [0.0, 0.0, 1.0, 1.0]
            && *tint == Color::WHITE)
    })
}

fn frees(c: &RecordingCanvas, tex: TexId) -> usize {
    c.ops
        .iter()
        .filter(|o| matches!(o, Op::Free(t) if *t == tex))
        .count()
}

fn panel_uploads(c: &RecordingCanvas) -> usize {
    let (pw, ph) = panel();
    c.ops
        .iter()
        .filter(|o| matches!(o, Op::UploadRgba8 { w, h, .. } if *w == pw && *h == ph))
        .count()
}

// ------------------------------------------------------------------ helpers alone

#[test]
fn the_setting_helper_reads_its_three_states_in_the_open() {
    assert!(
        !overlay_enabled(None),
        "a game that said nothing turned it on"
    );
    assert!(overlay_enabled(Some(true)));
    assert!(!overlay_enabled(Some(false)));
}

#[test]
fn the_geometry_helper_knows_the_three_panels_and_nothing_else() {
    assert_eq!(geometry_for_panel((640, 480)), Some(Geometry::W640H480));
    assert_eq!(geometry_for_panel((720, 480)), Some(Geometry::W720H480));
    assert_eq!(geometry_for_panel((720, 720)), Some(Geometry::W720H720));
    for other in [
        (0, 0),
        (1, 1),
        (320, 240),
        (640, 481),
        (800, 600),
        (720, 719),
    ] {
        assert_eq!(
            geometry_for_panel(other),
            None,
            "{other:?} was taken for a panel"
        );
    }
    // The three variants keep their own sizes: a geometry that lied about its panel would key
    // the wrong picture.
    assert_eq!(Geometry::W640H480.size(), (640, 480));
    assert_eq!(Geometry::W720H480.size(), (720, 480));
    assert_eq!(Geometry::W720H720.size(), (720, 720));
}

// ------------------------------------------------------------------ the playing path

#[test]
fn an_enabled_game_draws_its_overlay_over_the_frame() {
    let _serial = serial();
    let f = fixture("enabled", Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let r = draw_frame(&mut a, &mut c, Instant::now());
    let ops = frame_ops(&c, r);

    let tex = overlay_upload(ops).expect("the overlay was not uploaded");
    let game = game_at(ops).expect("the game frame was not drawn");
    let overlay = overlay_at(ops, tex).expect("the overlay was not drawn");
    assert!(overlay > game, "the overlay went down before the game");
    assert!(
        matches!(ops[game], Op::ImageEffect { .. }),
        "the game is not drawn through its effect: {:?}",
        ops[game]
    );
    assert!(
        matches!(ops[overlay], Op::Image { .. }),
        "the overlay was drawn through an effect: {:?}",
        ops[overlay]
    );
    assert_eq!(panel_uploads(&c), 1, "the overlay was not uploaded once");
    assert_eq!(a.toast_key(), None, "the overlay said something");
    assert!(a.take_sink_request().is_none());
}

#[test]
fn the_next_frame_draws_the_overlay_from_the_texture_it_has() {
    let _serial = serial();
    let f = fixture("cached", Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");
    assert_eq!(panel_uploads(&c), 1);

    let frames = a.session().unwrap().frames_run();
    for i in 1..=3 {
        let r = draw_frame(&mut a, &mut c, t + ms(100 * i));
        let ops = frame_ops(&c, r);
        assert!(
            overlay_at(ops, tex).is_some(),
            "frame {i}: the overlay went missing"
        );
        assert_eq!(
            overlay_upload(ops),
            None,
            "frame {i}: the overlay was uploaded again"
        );
        assert_eq!(frees(&c, tex), 0, "frame {i}: the overlay was freed");
    }
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "a draw ran the core"
    );
}

#[test]
fn a_game_that_never_asked_for_an_overlay_draws_none() {
    let _serial = serial();
    // The picture is on the card either way: only the setting decides.
    for setting in [None, Some(false)] {
        let f = fixture(&format!("off-{setting:?}"), setting, Picture::Panel);
        let mut a = f.playing();
        let mut c = canvas();
        let r = draw_frame(&mut a, &mut c, Instant::now());
        let ops = frame_ops(&c, r);
        assert!(
            game_at(ops).is_some(),
            "{setting:?}: the game frame went missing"
        );
        assert_eq!(
            overlay_upload(ops),
            None,
            "{setting:?}: an overlay was uploaded for a game that did not ask"
        );
        assert_eq!(panel_uploads(&c), 0, "{setting:?}: a texture was made");
        drop(a);
    }
}

#[test]
fn a_picture_that_is_missing_broken_or_the_wrong_panel_draws_no_overlay() {
    let _serial = serial();
    for picture in [Picture::None, Picture::Corrupt, Picture::WrongPanel] {
        let f = fixture(&format!("broken-{picture:?}"), Some(true), picture);
        let mut a = f.playing();
        let mut c = canvas();
        let r = draw_frame(&mut a, &mut c, Instant::now());
        let ops = frame_ops(&c, r);
        assert!(
            game_at(ops).is_some(),
            "{picture:?}: a bad overlay stopped the game frame"
        );
        assert_eq!(
            panel_uploads(&c),
            0,
            "{picture:?}: a texture was made for it"
        );

        // The game is still a game: frames and sound carry on.
        let frames = a.session().unwrap().frames_run();
        let now = run(&mut a, Instant::now(), 0.1);
        assert!(a.session().unwrap().frames_run() > frames);
        assert!(a.take_sink_request().is_none());
        let _ = now;
        drop(a);
    }
}

#[test]
fn the_launch_resolves_the_setting_once_and_not_per_frame() {
    let _serial = serial();
    let f = fixture("once", Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");

    // Take both the decision and the picture away: a launch-boundary resolution keeps drawing
    // what it resolved, where a per-frame one would quietly stop.
    fs::remove_file(card_overlay_path(&f.card, Platform::Gba, geometry())).unwrap();
    fs::remove_file(f.card.game_settings_path(&f.cart)).unwrap();

    let r = draw_frame(&mut a, &mut c, t + ms(200));
    let ops = frame_ops(&c, r);
    assert!(
        overlay_at(ops, tex).is_some(),
        "the overlay went missing between frames"
    );
    assert_eq!(overlay_upload(ops), None, "the card was read again");
    assert_eq!(frees(&c, tex), 0);
}

#[test]
fn the_overlay_goes_under_the_menu_it_was_opened_through() {
    let _serial = serial();
    let f = fixture("menus", Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let mut now = Instant::now();
    let r = draw_frame(&mut a, &mut c, now);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");

    // The in-game menu first: game, overlay, then the menu's own panel and words.
    tap(&mut a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    now += ms(100);
    let r = draw_frame(&mut a, &mut c, now);
    let ops = frame_ops(&c, r);
    let game = game_at(ops).expect("the game frame");
    let overlay = overlay_at(ops, tex).expect("the overlay");
    let ctx = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let (bx, by) = InGameMenu::box_origin(&ctx);
    let panel_at = ops
        .iter()
        .position(|o| {
            matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)
        })
        .expect("the in-game menu panel");
    assert!(game < overlay && overlay < panel_at, "{ops:?}");

    // Then the Display submenu, over the same frame.
    now += ms(200);
    for _ in 0..3 {
        tap(&mut a, Button::Down, now);
        now += ms(200);
    }
    tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Display(..)), "{:?}", a.screen);
    now += ms(100);
    let r = draw_frame(&mut a, &mut c, now);
    let ops = frame_ops(&c, r);
    let game = game_at(ops).expect("the game frame");
    let overlay = overlay_at(ops, tex).expect("the overlay");
    let dim = ops
        .iter()
        .position(|o| matches!(o, Op::Rect { x, y, .. } if *x == 0.0 && *y == 0.0))
        .expect("the Display menu's dim");
    assert!(game < overlay && overlay < dim, "{ops:?}");
    assert!(
        matches!(ops[overlay], Op::Image { .. }),
        "the overlay was drawn through an effect"
    );
    assert_eq!(
        overlay_upload(ops),
        None,
        "the menu re-uploaded the overlay"
    );
}

#[test]
fn the_overlay_goes_under_the_power_menu_too() {
    let _serial = serial();
    let f = fixture("power", Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");

    // The power menu over a running game: the frame and its overlay stay where they are, and
    // the menu comes down on both.
    a.screen = Screen::Power(PowerMenu::default());
    let r = draw_frame(&mut a, &mut c, t + ms(100));
    let ops = frame_ops(&c, r);
    let game = game_at(ops).expect("the game frame");
    let overlay = overlay_at(ops, tex).expect("the overlay");
    let dim = ops
        .iter()
        .position(|o| matches!(o, Op::Rect { x, y, .. } if *x == 0.0 && *y == 0.0))
        .expect("the power menu's dim");
    assert!(game < overlay && overlay < dim, "{ops:?}");
    assert_eq!(overlay_upload(ops), None, "the power menu re-uploaded it");
    assert_eq!(frees(&c, tex), 0, "the power menu freed the overlay");
    assert!(a.session().is_some(), "the power menu dropped the session");
}

#[test]
fn a_cart_on_its_way_in_has_no_overlay_yet() {
    let _serial = serial();
    let f = fixture("inserting", Some(true), Picture::Panel);
    let mut a = f.app(Screen::List);
    let mut c = canvas();
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let mut now = t + ms(60);
    let mut seen = false;
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.session().is_some() && a.screen == Screen::Inserting {
            seen = true;
            break;
        }
    }
    assert!(seen, "the session never came up under the insert");

    // The core is ready and the cart is still going in: neither the frame nor its overlay
    // belongs over that animation yet.
    let r = draw_frame(&mut a, &mut c, now);
    let ops = frame_ops(&c, r);
    assert_eq!(
        panel_uploads(&c),
        0,
        "the overlay's texture was made for the insert"
    );
    assert!(
        !ops.iter().any(|o| matches!(o, Op::ImageEffect { .. })),
        "the game frame was drawn over the insert"
    );

    // And it appears on the first frame that really shows the game.
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    let _ = a.take_sink_request();
    let r = draw_frame(&mut a, &mut c, now);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay never appeared");
    assert!(overlay_at(frame_ops(&c, r), tex).is_some());
    assert_eq!(panel_uploads(&c), 1);
}

#[test]
fn ending_the_game_frees_the_overlay_texture_exactly_once() {
    let _serial = serial();
    let f = fixture("free", Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");

    // The MENU hold takes the cart out, and the overlay goes with the game.
    let now = t + ms(100);
    a.feed(&ev(Button::Menu, true, now));
    let mut now = now;
    for _ in 0..180 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen != Screen::Playing {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "the hold did not eject");
    assert!(a.session().is_none());

    let from = c.ops.len();
    let r = draw_frame(&mut a, &mut c, now + ms(100));
    assert_eq!(
        frees(&c, tex),
        1,
        "the texture was not freed exactly once: {:?}",
        &c.ops[from..]
    );
    assert_eq!(
        overlay_at(frame_ops(&c, r), tex),
        None,
        "the overlay outlived the game"
    );

    // The frame after that is not a second free.
    let frees_before = frees(&c, tex);
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    assert_eq!(frees(&c, tex), frees_before, "the texture was freed twice");
    assert_eq!(overlay_at(frame_ops(&c, r), tex), None);
}

#[test]
fn a_menu_with_no_game_draws_no_overlay_of_its_own() {
    let _serial = serial();
    // The card has a picture and the setting is on, but nothing was launched: the overlay is
    // resolved at a launch, so there is nothing to draw and nothing may be drawn anyway.
    let f = fixture("no-session", Some(true), Picture::Panel);
    let mut a = f.app(Screen::InGame(InGameMenu::default()));
    assert!(a.session().is_none());
    let mut c = canvas();
    let r = draw_frame(&mut a, &mut c, Instant::now());
    let ops = frame_ops(&c, r);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::ImageEffect { .. })),
        "a game frame appeared with no game"
    );
    assert_eq!(
        panel_uploads(&c),
        0,
        "a texture was made for a screen with no game on it"
    );
    // The menu itself is still there.
    assert!(ops.iter().any(|o| matches!(o, Op::Rect { .. })));

    // The same for a shelf: no session, no overlay, and no picture of one.
    let mut a = f.app(Screen::List);
    let mut c = canvas();
    draw_frame(&mut a, &mut c, Instant::now());
    assert_eq!(panel_uploads(&c), 0, "an overlay was uploaded with no game");
}

#[test]
fn the_overlay_does_not_change_the_game_it_is_drawn_over() {
    let _serial = serial();
    let with = fixture("compare-with", Some(true), Picture::Panel);
    let without = fixture("compare-without", None, Picture::Panel);

    let mut a = with.playing();
    let mut ca = canvas();
    let warm = Instant::now();
    let warm_range = draw_frame(&mut a, &mut ca, warm); // the game's own texture upload
    let tex_a = overlay_upload(frame_ops(&ca, warm_range)).expect("the overlay's texture");
    let ra = draw_frame(&mut a, &mut ca, warm + ms(100));
    let ops_a = frame_ops(&ca, ra);
    let game_a = game_at(ops_a).map(|i| ops_a[i].clone());
    let op_count_a = ops_a.len();
    let frames_a = a.session().unwrap().frames_run();
    let (gw, gh) = a
        .session()
        .unwrap()
        .last_frame()
        .map(|(w, h, _)| (w, h))
        .unwrap();
    assert!(overlay_at(ops_a, tex_a).is_some());
    drop(a); // one core at a time in this process

    let mut b = without.playing();
    let mut cb = canvas();
    let warm = Instant::now();
    draw_frame(&mut b, &mut cb, warm);
    let rb = draw_frame(&mut b, &mut cb, warm + ms(100));
    let ops_b = frame_ops(&cb, rb);
    let game_b = game_at(ops_b).map(|i| ops_b[i].clone());
    let op_count_b = ops_b.len();
    let frames_b = b.session().unwrap().frames_run();

    // The same game, moments apart, with and without an overlay: the frame the core put down,
    // the effect it went through and where it landed are the same draw.
    assert_eq!(game_a, game_b, "the overlay changed what the game draws");
    match game_a {
        Some(Op::ImageEffect { effect, uv, .. }) => {
            assert_eq!(effect, slot2_gfx::ShaderEffect::Lcd3x, "the GBA's default");
            assert_eq!(uv, [0.0, 0.0, 1.0, 1.0], "the crop moved");
        }
        other => panic!("the game is not drawn through its effect: {other:?}"),
    }
    assert_eq!(
        op_count_a,
        op_count_b + 1,
        "the overlay did not add exactly one draw"
    );
    assert_eq!(frames_a, frames_b, "the overlay changed the frame count");

    // The game texture is still the core's own, uploaded once in each canvas: the overlay is
    // the only extra texture there is.
    let game_uploads = |c: &RecordingCanvas| {
        c.ops
            .iter()
            .filter(|o| matches!(o, Op::UploadRgba8 { w, h, .. } if *w == gw && *h == gh))
            .count()
    };
    assert_eq!(game_uploads(&ca), 1, "the game texture was uploaded again");
    assert_eq!(game_uploads(&cb), 1);
    assert_eq!(panel_uploads(&ca), 1, "the overlay's own texture");
    assert_eq!(panel_uploads(&cb), 0, "a texture with no overlay to draw");
    assert!(b.take_sink_request().is_none());
}

// ------------------------------------------------------------------ the core change

/// The in-game menu, down to the Core row, then A: the picker as the player reaches it.
fn open_core_row(a: &mut App, at: Instant) -> Instant {
    let mut now = at;
    tap(a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    for _ in 0..4 {
        now += ms(200);
        tap(a, Button::Down, now);
    }
    match a.screen {
        Screen::InGame(menu) => assert_eq!(
            menu.choice(),
            slot2_ui::InGameChoice::Core,
            "the Core row moved"
        ),
        other => panic!("the in-game menu is not open: {other:?}"),
    }
    now += ms(200);
    tap(a, Button::A, now);
    now
}

/// Play the cart through the insert animations, leaving the app at `Playing`.
fn leat_to_playing(a: &mut App, tag: &str) {
    let t = Instant::now();
    tap(a, Button::A, t);
    let mut now = t + ms(60);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            run(a, now, 0.2);
            let _ = a.take_sink_request();
            return;
        }
    }
    panic!("{tag}: the cart never reached the game: {:?}", a.screen);
}

/// A card, cart and core directory of its own, with both GBA cores copied in so they can be
/// broken without touching the repository's `vendor/`.
fn switch_fixture(tag: &str, overlay: Option<bool>) -> (Fixture, PathBuf) {
    let f = fixture(tag, overlay, Picture::Panel);
    let cores = scratch(&format!("{tag}-cores"));
    let vendor = vendor();
    for base in ["mgba", "gpsp"] {
        let name = core_name(base);
        assert!(
            vendor.join(&name).is_file(),
            "no {name} in {}: the core-change cases need both cores",
            vendor.display()
        );
        fs::copy(vendor.join(&name), cores.join(&name)).unwrap();
    }
    (f, cores)
}

#[test]
fn a_core_change_keeps_the_overlay_it_already_has() {
    let _serial = serial();
    let (f, cores) = switch_fixture("switch", Some(true));
    let mut a = f.app_with(cores, Screen::List);
    leat_to_playing(&mut a, "switch");
    let mut c = canvas();
    let r = draw_frame(&mut a, &mut c, Instant::now());
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");

    // The same game on gpSP: a real second core, the same cart and the same picture.
    let now = open_core_row(&mut a, Instant::now() + ms(200));
    tap(&mut a, Button::Down, now);
    tap(&mut a, Button::A, now + ms(200));
    assert_eq!(a.toast_key(), None, "the switch failed");
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(slot2_retro::CoreId::Gpsp),
        "the other core is not running"
    );
    let _ = a.take_sink_request();
    let _ = a.take_consumer();

    // Another core is not another game: the overlay it already had is the overlay it keeps.
    let r = draw_frame(&mut a, &mut c, now + ms(400));
    let ops = frame_ops(&c, r);
    assert!(overlay_at(ops, tex).is_some(), "the overlay went missing");
    assert_eq!(overlay_upload(ops), None, "the overlay was uploaded again");
    assert_eq!(frees(&c, tex), 0, "the overlay was freed by the switch");
}

#[test]
fn a_core_change_that_falls_back_keeps_the_overlay_too() {
    let _serial = serial();
    let (f, cores) = switch_fixture("recover", Some(true));
    // gpSP is a candidate by name and not a library by content, so the switch refuses it and
    // the app goes back to the core the settings still name.
    fs::write(cores.join(core_name("gpsp")), b"not a library").unwrap();

    let mut a = f.app_with(cores, Screen::List);
    leat_to_playing(&mut a, "recover");
    let mut c = canvas();
    let r = draw_frame(&mut a, &mut c, Instant::now());
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");

    let now = open_core_row(&mut a, Instant::now() + ms(200));
    tap(&mut a, Button::Down, now);
    tap(&mut a, Button::A, now + ms(200));
    assert_eq!(a.toast_key(), Some("core-switch-failed"));
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(slot2_retro::CoreId::Mgba),
        "the old core did not come back"
    );
    let _ = a.take_sink_request();
    let _ = a.take_consumer();

    // The playthrough came back, and so did its overlay: same texture, no re-upload.
    let r = draw_frame(&mut a, &mut c, now + ms(400));
    let ops = frame_ops(&c, r);
    assert!(overlay_at(ops, tex).is_some(), "the overlay went missing");
    assert_eq!(overlay_upload(ops), None, "the overlay was uploaded again");
    assert_eq!(frees(&c, tex), 0, "the overlay was freed by the fallback");
}

#[test]
fn a_game_that_cannot_come_back_takes_its_overlay_with_it() {
    let _serial = serial();
    let (f, cores) = switch_fixture("recovery-failed", Some(true));

    let mut a = f.app_with(cores, Screen::List);
    leat_to_playing(&mut a, "recovery-failed");
    let mut c = canvas();
    let r = draw_frame(&mut a, &mut c, Instant::now());
    let tex = overlay_upload(frame_ops(&c, r)).expect("the overlay's texture");

    // A cheat file the card will not hand over, written now: the core that is running stays up,
    // but every session started after this — the new core and the recovery alike — is refused,
    // which is the one way to reach "no game at all" without taking a library away mid-flight.
    let cheats = f.card.cheat_path(&f.cart);
    fs::create_dir_all(cheats.parent().unwrap()).unwrap();
    fs::write(
        &cheats,
        "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"7E007C9A\"\n",
    )
    .unwrap();

    let now = open_core_row(&mut a, Instant::now() + ms(200));
    tap(&mut a, Button::Down, now);
    tap(&mut a, Button::A, now + ms(200));

    assert_eq!(a.toast_key(), Some("core-recovery-failed"));
    assert!(
        a.session().is_none(),
        "a game survived a recovery that failed"
    );
    assert_eq!(a.screen, Screen::Ejecting, "{:?}", a.screen);
    let _ = a.take_sink_request();

    // No game, no overlay: the texture goes on the next frame with a canvas, once.
    let r = draw_frame(&mut a, &mut c, now + ms(400));
    assert_eq!(frees(&c, tex), 1, "the texture was not freed exactly once");
    assert_eq!(
        overlay_at(frame_ops(&c, r), tex),
        None,
        "an overlay outlived its game"
    );
    let frees_before = frees(&c, tex);
    let r = draw_frame(&mut a, &mut c, now + ms(600));
    assert_eq!(frees(&c, tex), frees_before, "the texture was freed twice");
    assert_eq!(overlay_at(frame_ops(&c, r), tex), None);
}
