//! The Overlay submenu inside the running app: the row that opens it, the card it writes, and
//! the picture it retargets on the frame already on screen.
//!
//! Needs `vendor/mgba_libretro.*` for the GBA half and `vendor/fceumm_libretro.*` for the NES
//! half. Both are in this checkout; a missing one fails here rather than skipping, because what
//! is under test is the row set a real platform earns and the frame a real core produced.
//!
//! One canvas is used per test and drawn into repeatedly, so a texture id means the same texture
//! in every frame; the ops a single frame added are the slice `draw_frame` hands back.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas, TexId};
use slot2_input::{Button, Event};
use slot2_store::{Card, Cart, GameSettings, Platform, ScaleMode, ShaderPreset};
use slot2_ui::display_menu::{CROPPING_ROWS, ROWS};
use slot2_ui::{DisplayChoice, InGameChoice, InGameMenu, UiCtx};

use slot2::app::{App, Screen};
use slot2::overlay::{card_overlay_path, geometry_for_panel};

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

/// Where a core this file needs lives. A missing one fails rather than skipping.
fn vendor() -> PathBuf {
    let d = repo().join("vendor");
    for name in [core_name("mgba"), core_name("fceumm")] {
        assert!(
            d.join(&name).is_file(),
            "no {name} in {}: the overlay menu tests need real cores (run build/cores.ps1)",
            d.display()
        );
    }
    d
}

fn tuning() -> slot2_retro::Tuning {
    slot2::tuning_for(&slot2_platform::detect().profile)
}

/// The panel the app is running on, which is also the size an overlay has to be.
fn panel() -> (u32, u32) {
    tuning().geometry
}

fn scratch(tag: &str) -> PathBuf {
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "slot2-overlay-menu-{tag}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
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

fn canvas() -> RecordingCanvas {
    let (pw, ph) = panel();
    RecordingCanvas::new(pw, ph)
}

// ------------------------------------------------------------------ pictures

/// A panel-sized RGBA picture: black, with one marker and one fully transparent pixel, so no two
/// fixtures share their bytes.
fn panel_pixels(w: u32, h: u32, seed: u8) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    let marker = ((20 * w + 10) * 4) as usize;
    out[marker] = seed;
    out[marker + 1] = 0x11;
    out[marker + 2] = 0x22;
    out[marker + 3] = 0xFF;
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

/// What the card holds at the overlay's path.
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

/// The cart, the setting and the picture, all in place before the app is built — the launch
/// boundary is where the overlay is resolved, so nothing written after it counts.
fn fixture(tag: &str, platform: Platform, overlay: Option<bool>, picture: Picture) -> Fixture {
    let cores = vendor();
    let root = scratch(tag);
    let card = Card::new(root.join("card"));
    card.ensure_layout();
    let (stem, rom) = match platform {
        Platform::Gba => {
            let rom = card.games_dir(Platform::Gba).join("arm.gba");
            fs::copy(repo().join("assets/test/arm.gba"), &rom).unwrap();
            ("arm", rom)
        }
        Platform::Nes => {
            let rom = card.games_dir(Platform::Nes).join("loop.nes");
            fs::write(&rom, nes_rom()).unwrap();
            ("loop", rom)
        }
        other => panic!("no fixture for {other:?}"),
    };
    let cart = Cart {
        platform,
        stem: stem.into(),
        title: stem.into(),
        rom,
    };

    if let Some(overlay) = overlay {
        let mut settings = card.read_settings(&cart);
        settings.overlay = Some(overlay);
        card.write_settings(&cart, &settings).unwrap();
    }
    write_picture(&card, platform, picture);

    Fixture { card, cart, cores }
}

fn write_picture(card: &Card, platform: Platform, picture: Picture) {
    let geometry = geometry_for_panel(panel()).expect("the host panel is one of the three");
    let path = card_overlay_path(card, platform, geometry);
    let (pw, ph) = panel();
    match picture {
        Picture::None => {}
        Picture::Panel => {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, png_rgba(pw, ph, &panel_pixels(pw, ph, 0xE1))).unwrap();
        }
        Picture::Corrupt => {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"not a picture").unwrap();
        }
        Picture::WrongPanel => {
            let other = if geometry == slot2_platform::Geometry::W640H480 {
                slot2_platform::Geometry::W720H480
            } else {
                slot2_platform::Geometry::W640H480
            };
            let (ow, oh) = other.size();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, png_rgba(ow, oh, &panel_pixels(ow, oh, 0xE2))).unwrap();
        }
    }
}

/// An iNES file: 16-byte header, one 16 KiB PRG bank, one 8 KiB CHR bank, mapper 0, with the CPU
/// parked at `SEI; CLD; JMP $C000` so the core produces a steady 256x240 frame.
fn nes_rom() -> Vec<u8> {
    let mut rom = Vec::with_capacity(16 + 16384 + 8192);
    rom.extend_from_slice(b"NES\x1A");
    rom.push(1);
    rom.push(1);
    rom.extend_from_slice(&[0; 10]);

    let mut prg = vec![0u8; 16384];
    prg[0..5].copy_from_slice(&[0x78, 0xD8, 0x4C, 0x00, 0xC0]);
    for v in [0x3FFA, 0x3FFC, 0x3FFE] {
        prg[v] = 0x00;
        prg[v + 1] = 0xC0;
    }
    rom.extend_from_slice(&prg);
    rom.extend_from_slice(&[0u8; 8192]);
    rom
}

impl Fixture {
    fn app(&self) -> App {
        App::with_card(
            self.card.clone(),
            self.cores.clone(),
            48_000,
            tuning(),
            false,
            Screen::List,
        )
    }

    /// The app at the game, with a frame of the core drawn.
    fn playing(&self) -> App {
        let mut a = self.app();
        let t = Instant::now();
        if self.cart.platform != Platform::Gba {
            // The shelf opens on the GBA, so any other cart is one shelf along.
            tap(&mut a, Button::R1, t);
            assert_eq!(a.platform(), self.cart.platform, "the shelf did not change");
        }
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
                return a;
            }
        }
        panic!("the test cart never reached the game: {:?}", a.screen);
    }
}

// ------------------------------------------------------------------ navigation helpers

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
    assert_eq!(menu.choice(), InGameChoice::Display, "not the Display row");
    now
}

/// The Display submenu, open. The in-game menu has been walked to Display and A pressed.
fn open_display(a: &mut App, at: Instant) -> Instant {
    let now = open_display_row(a, at);
    tap(a, Button::A, now);
    assert!(matches!(a.screen, Screen::Display(..)), "{:?}", a.screen);
    now + ms(200)
}

/// The row the Display menu — or the Overlay screen that was opened from it — is on.
fn display_choice(a: &App) -> DisplayChoice {
    match a.screen {
        Screen::Display(_, menu) => menu.choice(),
        // The parent Display menu keeps the row the screen was opened from: that is what closing
        // it lands back on.
        Screen::Overlay(_, display, _) => display.choice(),
        other => panic!("no Display menu is open: {other:?}"),
    }
}

/// The row count of the Display menu on show, which is its platform's answer.
fn display_rows(a: &App) -> usize {
    match a.screen {
        Screen::Display(_, menu) => menu.choices().len(),
        Screen::Overlay(_, display, _) => display.choices().len(),
        other => panic!("no Display menu is open: {other:?}"),
    }
}

/// Walk the open Display menu to its last row: the Overlay row, in either row set.
fn walk_to_display_overlay(a: &mut App, mut now: Instant) -> Instant {
    for _ in 0..display_rows(a) {
        if display_choice(a) == DisplayChoice::Overlay {
            return now;
        }
        tap(a, Button::Down, now);
        now += ms(200);
    }
    panic!("the Overlay row was never reached: {:?}", display_choice(a));
}

/// The Overlay screen, open on the card's own setting, from the app sitting on the game.
fn open_overlay(a: &mut App, at: Instant) -> Instant {
    let now = open_display(a, at);
    let now = walk_to_display_overlay(a, now);
    tap(a, Button::A, now);
    assert!(matches!(a.screen, Screen::Overlay(..)), "{:?}", a.screen);
    now + ms(200)
}

/// Back to the running game from any of the menus, so a test can walk in again through the
/// in-game menu a second time.
fn back_to_playing(a: &mut App, mut now: Instant) -> Instant {
    for _ in 0..3 {
        if a.screen == Screen::Playing {
            return now;
        }
        tap(a, Button::B, now);
        now += ms(200);
    }
    assert_eq!(a.screen, Screen::Playing, "the walk back ended elsewhere");
    now
}

/// The row the Overlay screen is on.
fn overlay_row(a: &App) -> Option<bool> {
    match a.screen {
        Screen::Overlay(_, _, menu) => menu.selected(),
        other => panic!("the overlay menu is not open: {other:?}"),
    }
}

/// Walk the Overlay screen to the row this test is about.
fn walk_to_row(a: &mut App, want: Option<bool>, mut now: Instant) -> Instant {
    let first = overlay_row(a);
    if first == want {
        return now;
    }
    // Up reaches `Some(true)` from `Some(false)` and the other way round, so both directions are
    // tried; one of them gets there inside one lap.
    for _ in 0..slot2_ui::overlay_menu::ROWS.len() {
        now += ms(200);
        tap(a, Button::Down, now);
        if overlay_row(a) == want {
            return now;
        }
    }
    for _ in 0..slot2_ui::overlay_menu::ROWS.len() {
        now += ms(200);
        tap(a, Button::Up, now);
        if overlay_row(a) == want {
            return now;
        }
    }
    panic!("{want:?} was not reachable from {first:?}");
}

/// A on the Overlay screen, from wherever it is parked.
fn commit(a: &mut App, at: Instant) {
    tap(a, Button::A, at);
    assert!(
        matches!(a.screen, Screen::Overlay(..)),
        "the overlay screen closed on A: {:?}",
        a.screen
    );
}

// ------------------------------------------------------------------ draw helpers

/// Draw one frame and hand back the span of ops it added.
fn draw_frame(a: &mut App, c: &mut RecordingCanvas, at: Instant) -> (usize, usize) {
    let from = c.ops.len();
    let mut ctx = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    a.draw(c, &mut ctx, at);
    (from, c.ops.len())
}

fn frame_ops(c: &RecordingCanvas, frame: (usize, usize)) -> &[Op] {
    &c.ops[frame.0..frame.1]
}

/// The game's own draw: the first picture after the session's own clear.
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
/// and nothing else in these frames.
fn overlay_upload(ops: &[Op]) -> Option<TexId> {
    let (pw, ph) = panel();
    ops.iter().find_map(|o| match o {
        Op::UploadRgba8 { id, w, h } if *w == pw && *h == ph => Some(*id),
        _ => None,
    })
}

/// Whether this frame drew the overlay picture: a plain panel-sized image of its texture.
fn overlay_at(ops: &[Op], tex: TexId) -> Option<usize> {
    let (pw, ph) = panel();
    ops.iter().rposition(|o| {
        matches!(o, Op::Image { tex: t, x, y, w, h, uv, tint }
        if *t == tex
            && *x == 0.0 && *y == 0.0
            && (*w - pw as f32).abs() < 0.5 && (*h - ph as f32).abs() < 0.5
            && *uv == [0.0, 0.0, 1.0, 1.0]
            && *tint == slot2_gfx::Color::WHITE)
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

/// `(uploads, updates, frees)` of the game's own texture: panel-sized uploads are the overlay's
/// and are left out, and so is nothing else, because a setting change has no business touching
/// the game's place in the canvas at all.
fn game_texture_ops(c: &RecordingCanvas) -> (usize, usize, usize) {
    let (pw, ph) = panel();
    let uploads = c
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadRgba8 { w, h, .. } if !(*w == pw && *h == ph)))
        .count();
    let updates = c
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UpdateRgba8 { .. }))
        .count();
    let frees = c.ops.iter().filter(|o| matches!(o, Op::Free(_))).count();
    (uploads, updates, frees)
}

/// The Overlay menu's own dim: the whole panel at its own colour.
fn menu_dim_at(ops: &[Op]) -> Option<usize> {
    let (pw, ph) = panel();
    ops.iter().position(|o| {
        matches!(o, Op::Rect { x, y, w, h, color }
        if *x == 0.0 && *y == 0.0
            && (*w - pw as f32).abs() < 0.5 && (*h - ph as f32).abs() < 0.5
            && *color == slot2_ui::overlay_menu::DIM)
    })
}

/// The card's hand-written settings file as bytes, or `None` when there is no file.
fn card_bytes(f: &Fixture) -> Option<Vec<u8>> {
    let path = f.card.game_settings_path(&f.cart);
    path.is_file().then(|| fs::read(&path).unwrap())
}

fn card_text(f: &Fixture) -> String {
    fs::read_to_string(f.card.game_settings_path(&f.cart)).unwrap()
}

/// A directory where the settings file's atomic temporary belongs, so only the write can fail.
fn block_writes(f: &Fixture) -> PathBuf {
    let path = f.card.game_settings_path(&f.cart);
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    let blocker = PathBuf::from(tmp);
    fs::create_dir(&blocker).unwrap();
    blocker
}

// ------------------------------------------------------------------ the row and the screen

#[test]
fn the_overlay_row_is_last_in_both_row_sets_and_walking_wraps() {
    let _serial = serial();

    // A GBA has nothing to crop: six rows, the overlay row last.
    let f = fixture("rows-gba", Platform::Gba, None, Picture::None);
    let mut a = f.playing();
    let now = open_display(&mut a, Instant::now());
    assert_eq!(display_rows(&a), ROWS.len());
    assert_eq!(ROWS.last(), Some(&DisplayChoice::Overlay));
    assert_eq!(
        ROWS[..ROWS.len() - 1].last(),
        Some(&DisplayChoice::Shader),
        "the row before the overlay row moved"
    );

    // Down from the last row wraps to the platform default, and up from it reaches the overlay
    // row again: the same walk the other screens offer.
    let now = walk_to_display_overlay(&mut a, now);
    assert_eq!(display_choice(&a), DisplayChoice::Overlay);
    let now = now + ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(
        display_choice(&a),
        DisplayChoice::Scale(None),
        "down from the overlay row did not wrap"
    );
    let now = now + ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(
        display_choice(&a),
        DisplayChoice::Overlay,
        "up from the first row did not wrap to the overlay row"
    );

    // A NES has something to crop: seven rows, the overscan row before the overlay one, and the
    // rows a player already knows in the same place.
    let f = fixture("rows-nes", Platform::Nes, None, Picture::None);
    let mut a = f.playing();
    let now = open_display(&mut a, Instant::now());
    assert_eq!(display_rows(&a), CROPPING_ROWS.len());
    assert_eq!(CROPPING_ROWS.last(), Some(&DisplayChoice::Overlay));
    assert_eq!(
        &CROPPING_ROWS[..ROWS.len() - 1],
        &ROWS[..ROWS.len() - 1],
        "the shared rows moved when the platform gained the overscan row"
    );
    let now = walk_to_display_overlay(&mut a, now);
    assert_eq!(display_choice(&a), DisplayChoice::Overlay);
    let now = now + ms(200);
    tap(&mut a, Button::Up, now);
    assert_eq!(
        display_choice(&a),
        DisplayChoice::Overscan,
        "the overlay row is not next to the overscan row"
    );
}

#[test]
fn the_overlay_screen_opens_on_the_cards_own_setting() {
    let _serial = serial();
    for (platform, tag) in [(Platform::Gba, "open-gba"), (Platform::Nes, "open-nes")] {
        for setting in [None, Some(true), Some(false)] {
            let f = fixture(
                &format!("{tag}-{setting:?}"),
                platform,
                setting,
                Picture::None,
            );
            let mut a = f.playing();
            let now = open_overlay(&mut a, Instant::now());
            assert_eq!(
                overlay_row(&a),
                setting,
                "{platform:?}: the screen did not open on the card's own setting"
            );
            assert_eq!(
                display_choice(&a),
                DisplayChoice::Overlay,
                "{platform:?}: the parent menu is not on the row it was opened from"
            );
            assert!(
                a.toast_key().is_none(),
                "{platform:?}: opening said something"
            );
            assert!(a.take_sink_request().is_none());
            let _ = now;
        }
    }
}

#[test]
fn the_overlay_row_does_nothing_without_a_session() {
    // No core, no session: the row stays a row, and nothing is read or written.
    let root = scratch("nosession");
    let card = Card::new(&root);
    card.ensure_layout();
    fs::write(card.games_dir(Platform::Gba).join("arm.gba"), b"rom").unwrap();
    let mut a = App::with_card(
        card.clone(),
        PathBuf::from("no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    let mut menu = InGameMenu::default();
    for _ in 0..3 {
        menu.down();
    }
    assert_eq!(menu.choice(), InGameChoice::Display);
    a.screen = Screen::InGame(menu);
    tap(&mut a, Button::A, Instant::now());
    assert!(
        matches!(a.screen, Screen::InGame(_)),
        "the Display row opened something with no game: {:?}",
        a.screen
    );
}

#[test]
fn closing_the_overlay_screen_hands_the_same_display_row_back() {
    let _serial = serial();
    let f = fixture("close", Platform::Gba, Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the launch resolved the picture");
    let bytes = card_bytes(&f).expect("the setting was written");

    let now = open_overlay(&mut a, t + ms(100));
    assert_eq!(overlay_row(&a), Some(true));

    // B goes back one step, to the Display menu still on its Overlay row.
    let now = now + ms(200);
    tap(&mut a, Button::B, now);
    assert!(matches!(a.screen, Screen::Display(..)), "{:?}", a.screen);
    assert_eq!(display_choice(&a), DisplayChoice::Overlay);

    // And A there opens the same screen again: the row is a way in, not a setting.
    let now = now + ms(200);
    tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Overlay(..)), "{:?}", a.screen);
    assert_eq!(overlay_row(&a), Some(true));

    // MENU is the same step back as B.
    let now = now + ms(200);
    tap(&mut a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::Display(..)), "{:?}", a.screen);
    assert_eq!(display_choice(&a), DisplayChoice::Overlay);

    // Nothing was written and nothing was read again: the picture on screen is the same texture.
    assert_eq!(card_bytes(&f).as_deref(), Some(bytes.as_slice()));
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    let ops = frame_ops(&c, r);
    assert!(
        overlay_at(ops, tex).is_some(),
        "closing the screen took the picture away"
    );
    assert_eq!(
        overlay_upload(ops),
        None,
        "closing the screen read the card"
    );
    assert_eq!(frees(&c, tex), 0, "closing the screen freed the picture");
    assert_eq!(a.toast_key(), None);
}

#[test]
fn reopening_the_screen_shows_the_last_successful_save() {
    let _serial = serial();
    let f = fixture("reopen", Platform::Gba, None, Picture::Panel);
    let mut a = f.playing();
    let now = open_overlay(&mut a, Instant::now());
    assert_eq!(overlay_row(&a), None);
    let now = walk_to_row(&mut a, Some(true), now);
    let now = now + ms(200);
    commit(&mut a, now);

    let now = now + ms(200);
    tap(&mut a, Button::B, now);
    let now = now + ms(200);
    tap(&mut a, Button::A, now);
    assert!(matches!(a.screen, Screen::Overlay(..)), "{:?}", a.screen);
    assert_eq!(
        overlay_row(&a),
        Some(true),
        "the screen did not open on what was saved"
    );

    // And once more, through the in-game menu: the card is the only memory this screen has.
    let now = back_to_playing(&mut a, now + ms(200));
    let now = open_overlay(&mut a, now + ms(200));
    assert_eq!(overlay_row(&a), Some(true));
    let _ = now;
}

// ------------------------------------------------------------------ saving

#[test]
fn every_overlay_choice_is_saved_exactly() {
    let _serial = serial();
    let f = fixture("save", Platform::Gba, None, Picture::Panel);
    let mut a = f.playing();
    let path = f.card.game_settings_path(&f.cart);
    let mut now = open_overlay(&mut a, Instant::now());
    assert_eq!(overlay_row(&a), None, "the screen ignored the card");

    // None → On: the key is written as the one spelling a save uses.
    now = walk_to_row(&mut a, Some(true), now);
    let now = now + ms(200);
    commit(&mut a, now);
    assert_eq!(overlay_row(&a), Some(true));
    assert_eq!(f.card.read_settings(&f.cart).overlay, Some(true));
    assert!(
        card_text(&f).contains("overlay = on"),
        "the on choice is not on the card: {}",
        card_text(&f)
    );

    // On → Off.
    let now = now + ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(overlay_row(&a), Some(false));
    let now = now + ms(200);
    commit(&mut a, now);
    assert_eq!(f.card.read_settings(&f.cart).overlay, Some(false));
    assert!(
        card_text(&f).contains("overlay = off"),
        "the off choice is not on the card: {}",
        card_text(&f)
    );

    // Off → None: the absence of a setting, which the store keeps by removing the key — and with
    // nothing else in the file, the file itself.
    let now = now + ms(200);
    tap(&mut a, Button::Down, now);
    assert_eq!(overlay_row(&a), None, "down from Off did not wrap");
    let now = now + ms(200);
    commit(&mut a, now);
    assert_eq!(f.card.read_settings(&f.cart).overlay, None);
    assert_eq!(a.toast_key(), None, "removing the key failed");
    assert!(
        !path.exists(),
        "the App left an all-default settings file behind"
    );
}

#[test]
fn setting_and_clearing_an_overlay_keeps_every_other_key() {
    let _serial = serial();
    let f = fixture("keys", Platform::Gba, None, Picture::Panel);
    let mut a = f.playing();
    let path = f.card.game_settings_path(&f.cart);

    f.card
        .write_settings(
            &f.cart,
            &GameSettings {
                core: Some("mgba".into()),
                scale: Some(ScaleMode::Fill),
                overscan: Some(false),
                rewind: Some(false),
                shader: Some(ShaderPreset::Scanline),
                overlay: None,
            },
        )
        .unwrap();
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("future_filter = keep\n");
    fs::write(&path, text).unwrap();

    let now = open_overlay(&mut a, Instant::now());
    assert_eq!(overlay_row(&a), None);
    let now = walk_to_row(&mut a, Some(true), now);
    let now = now + ms(200);
    commit(&mut a, now);

    let settings = f.card.read_settings(&f.cart);
    assert_eq!(settings.overlay, Some(true));
    assert_eq!(settings.core.as_deref(), Some("mgba"), "core was lost");
    assert_eq!(settings.scale, Some(ScaleMode::Fill), "scale was lost");
    assert_eq!(settings.overscan, Some(false), "overscan was lost");
    assert_eq!(settings.rewind, Some(false), "rewind was lost");
    assert_eq!(
        settings.shader,
        Some(ShaderPreset::Scanline),
        "shader was lost"
    );
    assert!(
        card_text(&f).contains("future_filter = keep"),
        "the unknown key was lost"
    );

    // And clearing it again leaves exactly the same keys behind.
    let now = walk_to_row(&mut a, None, now + ms(200));
    let now = now + ms(200);
    commit(&mut a, now);

    let settings = f.card.read_settings(&f.cart);
    assert_eq!(settings.overlay, None, "the override is still on the card");
    assert_eq!(settings.core.as_deref(), Some("mgba"), "core was lost");
    assert_eq!(settings.scale, Some(ScaleMode::Fill), "scale was lost");
    assert_eq!(settings.overscan, Some(false), "overscan was lost");
    assert_eq!(settings.rewind, Some(false), "rewind was lost");
    assert_eq!(
        settings.shader,
        Some(ShaderPreset::Scanline),
        "shader was lost"
    );
    assert!(
        !card_text(&f).contains("overlay"),
        "the overlay key is still in the file: {}",
        card_text(&f)
    );
    assert!(card_text(&f).contains("future_filter = keep"));
}

// ------------------------------------------------------------------ the live picture

#[test]
fn choosing_on_draws_the_picture_at_once_between_the_game_and_the_menu() {
    let _serial = serial();
    let f = fixture("live-on", Platform::Gba, None, Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();

    // The launch resolved "nothing said", so the first frame has no picture to draw.
    let r = draw_frame(&mut a, &mut c, t);
    assert!(game_at(frame_ops(&c, r)).is_some());
    assert_eq!(panel_uploads(&c), 0);
    let frames = a.session().unwrap().frames_run();

    let mut now = open_overlay(&mut a, t + ms(100));
    now = walk_to_row(&mut a, Some(true), now);
    let now = now + ms(200);
    commit(&mut a, now);

    // The very next frame draws it, in the one order the screen may use.
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    let ops = frame_ops(&c, r);
    let tex = overlay_upload(ops).expect("the picture was not taken up");
    let game = game_at(ops).expect("the game frame went missing");
    let overlay = overlay_at(ops, tex).expect("the picture was not drawn");
    let dim = menu_dim_at(ops).expect("the overlay menu was not drawn");
    assert!(game < overlay, "the picture went down before the game");
    assert!(overlay < dim, "the picture went over the menu");
    assert_eq!(panel_uploads(&c), 1, "the picture was uploaded twice");
    assert_eq!(frees(&c, tex), 0, "a picture that never existed was freed");
    assert_eq!(a.toast_key(), None, "a save that worked said something");
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the draw ran the core"
    );
    assert!(a.take_sink_request().is_none());
}

#[test]
fn choosing_off_frees_the_texture_once_and_draws_no_picture() {
    let _serial = serial();
    let f = fixture("live-off", Platform::Gba, Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the launch resolved the picture");
    assert_eq!(frees(&c, tex), 0);

    let mut now = open_overlay(&mut a, t + ms(100));
    assert_eq!(overlay_row(&a), Some(true));
    now = walk_to_row(&mut a, Some(false), now);
    let now = now + ms(200);
    commit(&mut a, now);

    // The next frame lets the texture go exactly once, and shows the game and the menu without
    // a picture between them.
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    let ops = frame_ops(&c, r);
    assert!(game_at(ops).is_some(), "the game frame went missing");
    assert!(menu_dim_at(ops).is_some(), "the menu went missing");
    assert!(
        overlay_at(ops, tex).is_none(),
        "the picture was still drawn after being turned off"
    );
    assert_eq!(frees(&c, tex), 1, "the texture was not freed exactly once");
    assert_eq!(overlay_upload(ops), None, "a picture was taken up anyway");

    // And once is enough: the frame after it frees nothing and uploads nothing.
    let r = draw_frame(&mut a, &mut c, now + ms(400));
    let ops = frame_ops(&c, r);
    assert!(game_at(ops).is_some());
    assert_eq!(frees(&c, tex), 1, "the texture was freed again");
    assert_eq!(panel_uploads(&c), 1, "another picture was taken up");
}

#[test]
fn choosing_on_again_uploads_once_and_resaving_the_same_source_changes_nothing() {
    let _serial = serial();
    let f = fixture("live-again", Platform::Gba, Some(false), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    draw_frame(&mut a, &mut c, t);
    assert_eq!(panel_uploads(&c), 0, "an off setting took a picture up");

    let mut now = open_overlay(&mut a, t + ms(100));
    assert_eq!(overlay_row(&a), Some(false));
    now = walk_to_row(&mut a, Some(true), now);
    let now = now + ms(200);
    commit(&mut a, now);

    let r = draw_frame(&mut a, &mut c, now + ms(200));
    let tex = overlay_upload(frame_ops(&c, r)).expect("the picture was not taken up");
    assert_eq!(panel_uploads(&c), 1);

    // The same row saved again is the same source: the layer's own no-op, not a new texture.
    let now = now + ms(400);
    commit(&mut a, now);
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    let ops = frame_ops(&c, r);
    assert!(
        overlay_at(ops, tex).is_some(),
        "the picture went away on a second save"
    );
    assert_eq!(overlay_upload(ops), None, "the source was decoded again");
    assert_eq!(frees(&c, tex), 0, "the source change freed the texture");
    assert_eq!(panel_uploads(&c), 1, "a second picture was taken up");
    assert_eq!(a.toast_key(), None);
}

#[test]
fn a_setting_change_leaves_the_core_the_session_and_the_game_texture_alone() {
    let _serial = serial();
    let f = fixture("untouched", Platform::Gba, Some(false), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    draw_frame(&mut a, &mut c, t);

    let frames = a.session().unwrap().frames_run();
    let audio = a.session().unwrap().audio_health();
    let core = a.session().unwrap().core_id();
    let cart = a.session().unwrap().cart().clone();
    let frame = a
        .session()
        .unwrap()
        .last_frame()
        .map(|(w, h, d)| (w, h, d.to_vec()));
    let texture = game_texture_ops(&c);

    let mut now = open_overlay(&mut a, t + ms(100));
    now = walk_to_row(&mut a, Some(true), now);
    let now = now + ms(200);
    commit(&mut a, now);
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    assert!(game_at(frame_ops(&c, r)).is_some());

    assert_eq!(a.session().unwrap().frames_run(), frames, "the core ran");
    assert_eq!(a.session().unwrap().audio_health(), audio, "audio advanced");
    assert_eq!(a.session().unwrap().core_id(), core, "the core changed");
    assert_eq!(
        a.session().unwrap().cart(),
        &cart,
        "the session swapped cart"
    );
    assert_eq!(
        a.session()
            .unwrap()
            .last_frame()
            .map(|(w, h, d)| (w, h, d.to_vec())),
        frame,
        "the core's frame changed"
    );
    assert_eq!(
        game_texture_ops(&c),
        texture,
        "the game texture was touched"
    );
    assert!(a.take_sink_request().is_none(), "the sink was asked about");
    assert!(a.session().is_some());
}

#[test]
fn a_picture_that_is_missing_broken_or_the_wrong_panel_keeps_the_choice() {
    let _serial = serial();
    for picture in [Picture::None, Picture::Corrupt, Picture::WrongPanel] {
        let f = fixture(&format!("broken-{picture:?}"), Platform::Gba, None, picture);
        let mut a = f.playing();
        let mut c = canvas();
        let t = Instant::now();
        let now = open_overlay(&mut a, t);
        let now = walk_to_row(&mut a, Some(true), now);
        let now = now + ms(200);
        commit(&mut a, now);

        // The choice is a decision about this game, and there is nothing here to check it
        // against: it is saved, it stays on the row, and it is not a failure.
        assert_eq!(overlay_row(&a), Some(true), "{picture:?}");
        assert_eq!(
            f.card.read_settings(&f.cart).overlay,
            Some(true),
            "{picture:?}"
        );
        assert!(
            card_text(&f).contains("overlay = on"),
            "{picture:?}: the choice is not on the card"
        );
        assert_eq!(a.toast_key(), None, "{picture:?}: a picture is not a write");

        // The game and the menu draw as they always did, with no picture between them.
        let r = draw_frame(&mut a, &mut c, now + ms(200));
        let ops = frame_ops(&c, r);
        assert!(game_at(ops).is_some(), "{picture:?}: the game went missing");
        assert!(
            menu_dim_at(ops).is_some(),
            "{picture:?}: the menu went missing"
        );
        assert_eq!(panel_uploads(&c), 0, "{picture:?}: a texture was made");

        // And the game is still a game once the screen is closed: it resumes where it was.
        let frames = a.session().unwrap().frames_run();
        let now = back_to_playing(&mut a, now + ms(200));
        let _ = run(&mut a, now + ms(200), 0.1);
        assert!(
            a.session().unwrap().frames_run() > frames,
            "{picture:?}: the core did not resume"
        );
        assert!(a.take_sink_request().is_none());
        drop(a);
    }
}

// ------------------------------------------------------------------ failures

#[test]
fn a_write_that_fails_leaves_the_card_and_the_picture_alone() {
    let _serial = serial();
    let f = fixture("fail", Platform::Gba, Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the launch resolved the picture");
    let bytes = card_bytes(&f).unwrap();

    let mut now = open_overlay(&mut a, t + ms(100));
    now = walk_to_row(&mut a, Some(false), now);
    let blocker = block_writes(&f);

    let now = now + ms(200);
    commit(&mut a, now);

    assert_eq!(
        a.toast_key(),
        Some("overlay-save-failed"),
        "a refused save said nothing"
    );
    assert_eq!(
        overlay_row(&a),
        Some(false),
        "the menu left the attempted row"
    );
    assert!(matches!(a.screen, Screen::Overlay(..)));
    assert_eq!(
        card_bytes(&f).as_deref(),
        Some(bytes.as_slice()),
        "the card moved on a write that failed"
    );
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());

    // The picture the running game resolved is still the one on screen, and nothing was freed
    // or taken up on the way.
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    let ops = frame_ops(&c, r);
    assert!(
        overlay_at(ops, tex).is_some(),
        "a refused write took the picture away"
    );
    assert_eq!(frees(&c, tex), 0, "a refused write freed the texture");
    assert_eq!(panel_uploads(&c), 1, "a refused write took a picture up");

    fs::remove_dir(&blocker).unwrap();
}

#[test]
fn an_unreadable_overlay_file_is_not_overwritten_by_the_overlay_menu() {
    let _serial = serial();
    let f = fixture("unreadable", Platform::Gba, None, Picture::Panel);
    let mut a = f.playing();
    let path = f.card.game_settings_path(&f.cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, [0xff, 0xfe, b'o', b'=', 0x80, b'\n']).unwrap();
    let damaged = fs::read(&path).unwrap();

    // Reading stays forgiving, so the row is the platform default rather than a refusal.
    let now = open_overlay(&mut a, Instant::now());
    assert_eq!(overlay_row(&a), None, "an unreadable file hid the row");
    let now = walk_to_row(&mut a, Some(true), now);
    let now = now + ms(200);
    commit(&mut a, now);

    assert_eq!(
        a.toast_key(),
        Some("overlay-save-failed"),
        "a refused save said nothing"
    );
    assert_eq!(
        overlay_row(&a),
        Some(true),
        "the menu left the attempted row"
    );
    assert!(matches!(a.screen, Screen::Overlay(..)));
    assert_eq!(
        fs::read(&path).unwrap(),
        damaged,
        "the damaged file was rewritten"
    );

    // Nothing was retargeted either: the running game still has no picture.
    let mut c = canvas();
    let r = draw_frame(&mut a, &mut c, now + ms(200));
    assert_eq!(panel_uploads(&c), 0, "a refused write took a picture up");
    assert!(game_at(frame_ops(&c, r)).is_some());
    assert!(a.session().is_some());
    assert!(a.take_sink_request().is_none());
}

#[test]
fn both_packs_have_the_overlay_failure_words() {
    // The key the App toasts on a refused write, spelled out in both packs: a ko pack that lacked
    // it would fall back to English and read as half-translated rather than as missing.
    for (lang, want) in [
        ("en", "Could not save the overlay setting"),
        ("ko", "오버레이 설정을 저장하지 못했습니다"),
    ] {
        let ctx = UiCtx::new(slot2_platform::detect().profile, lang, Vec::new(), None);
        assert_eq!(ctx.i18n.t("overlay-save-failed"), want, "{lang}");
    }
}

// ------------------------------------------------------------------ pause and draw

#[test]
fn the_overlay_screen_pauses_the_session_and_keeps_it() {
    let _serial = serial();
    let f = fixture("pause", Platform::Gba, Some(true), Picture::Panel);
    let mut a = f.playing();
    let t = open_overlay(&mut a, Instant::now());
    assert!(
        a.session().is_some(),
        "the overlay screen dropped the session"
    );
    assert!(a.audio_paused(), "the game kept its sound under the screen");
    assert!(
        a.take_sink_request().is_none(),
        "the overlay screen touched the audio"
    );

    let frames = a.session().unwrap().frames_run();
    let audio = a.session().unwrap().audio_health();
    let _ = run(&mut a, t, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran under the overlay screen"
    );
    assert_eq!(
        a.session().unwrap().audio_health(),
        audio,
        "audio advanced under the overlay screen"
    );
}

#[test]
fn drawing_shows_the_game_its_picture_and_the_menu_and_nothing_else() {
    let _serial = serial();
    let f = fixture("draw", Platform::Gba, Some(true), Picture::Panel);
    let mut a = f.playing();
    let mut c = canvas();
    let t = Instant::now();
    let r = draw_frame(&mut a, &mut c, t);
    let tex = overlay_upload(frame_ops(&c, r)).expect("the launch resolved the picture");

    let now = open_overlay(&mut a, t + ms(100));
    let mut ctx = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    let mut c = canvas();
    a.draw(&mut c, &mut ctx, now);
    let ops = c.frame();

    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the overlay screen cleared the frame"
    );
    assert!(
        matches!(ops[0], Op::Image { .. } | Op::ImageEffect { .. }),
        "the first mark was not the game frame: {:?}",
        ops[0]
    );
    let overlay = overlay_at(ops, tex).expect("the picture was not drawn");
    let dim = menu_dim_at(ops).expect("the overlay menu was not drawn");
    assert!(overlay < dim, "the picture went over the menu");

    // Not the parent menus: the overlay screen is the whole of what is on screen.
    let Screen::Overlay(_, display, _) = a.screen else {
        panic!("the overlay screen is not open: {:?}", a.screen)
    };
    let (dx, dy) = display.box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - dx).abs() < 0.5 && (*y - dy).abs() < 0.5)),
        "the Display menu was drawn under the overlay menu"
    );
    let (bx, by) = InGameMenu::box_origin(&ctx);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
        "the in-game menu was drawn under the overlay menu"
    );
    let (cx, cy, ..) = slot2_ui::StateSwitcher::card_rect(&ctx, 0);
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
            if (*x - cx).abs() < 0.5 && (*y - cy).abs() < 0.5)),
        "a switcher card was drawn under the overlay menu"
    );
    // The time-control badge is `Playing`-only, and the corner furniture is not on a game screen.
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Rect { y, h, .. }
            if (*y - slot2_ui::hud::HUD_MARGIN).abs() < 0.5
                && (*h - slot2_ui::hud::HUD_H).abs() < 0.01)),
        "the top band was drawn over the game"
    );
}
