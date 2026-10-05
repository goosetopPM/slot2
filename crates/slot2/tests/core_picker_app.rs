//! The in-game Core row: opening the picker, choosing a core, and living through every way
//! that choice can go wrong.
//!
//! This is the one place in the frontend that stops a running core and starts another, so the
//! fixtures are real cores. Both libraries are copied into a directory of their own and the app
//! is pointed at that, which is what lets a test take a library away, put an unopenable one in
//! its place, or block a write without touching the repository's own files. Without mGBA and
//! gpSP the file skips loudly.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_platform::by_target;
use slot2_retro::CoreId;
use slot2_store::{Card, Cart, GameSettings, Platform, ScaleMode, StateKind, StateNamespace};
use slot2_ui::core_picker::{BADGE_INSET, BOX_W, CURRENT_KEY, PAD, ROW_H};
use slot2_ui::insert::EJECT_S;
use slot2_ui::toast::TOAST_S;
use slot2_ui::{face, CorePicker, InGameChoice, UiCtx, PX_BODY, PX_TITLE};

use slot2::app::{App, Screen, SinkRequest};

/// A libretro core is a library loaded once per process, so these tests take turns.
static SERIAL: Mutex<()> = Mutex::new(());
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// Which row of the in-game menu the Core row is. Asserted against the menu itself rather
/// than trusted: a row added above it would otherwise make every test here open something else.
const CORE_ROW: usize = 4;

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Where the real cores live. Both are needed for the switch tests, and neither may be
/// replaced by a stand-in: what is under test is a second core being started for real.
fn vendor() -> Option<PathBuf> {
    let d = repo().join("vendor");
    for core in [CoreId::Mgba, CoreId::Gpsp] {
        if !d.join(core.file_name()).is_file() {
            eprintln!(
                "no {} in {} — skipping (run build/cores.ps1)",
                core.base_name(),
                d.display()
            );
            return None;
        }
    }
    Some(d)
}

fn tuning() -> slot2_retro::Tuning {
    slot2::tuning_for(&slot2_platform::detect().profile)
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

/// A card with one GBA cart on it, and a directory of real core libraries of its own.
///
/// The copies are the point: several tests take a library away or block a write, and the
/// repository's `vendor/` is not a test's to damage.
fn fixture(tag: &str, vendor: &Path) -> (Card, Cart, PathBuf) {
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("slot2-corepicker-{tag}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let card = Card::new(root.join("card"));
    card.ensure_layout();
    let rom = card.games_dir(Platform::Gba).join("arm.gba");
    fs::copy(repo().join("assets/test/arm.gba"), &rom).unwrap();
    let cores = root.join("cores");
    fs::create_dir_all(&cores).unwrap();
    for core in [CoreId::Mgba, CoreId::Gpsp] {
        fs::copy(vendor.join(core.file_name()), cores.join(core.file_name())).unwrap();
    }
    let cart = Cart {
        platform: Platform::Gba,
        stem: "arm".into(),
        title: "arm".into(),
        rom,
    };
    (card, cart, cores)
}

/// A card with one NES cart on it. FCEUmm is the only core that runs it, which is the point.
fn nes_fixture(tag: &str, vendor: &Path) -> (Card, Cart, PathBuf) {
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("slot2-corepicker-{tag}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let card = Card::new(root.join("card"));
    card.ensure_layout();
    let rom = card.games_dir(Platform::Nes).join("loop.nes");
    fs::write(&rom, nes_rom()).unwrap();
    let cores = root.join("cores");
    fs::create_dir_all(&cores).unwrap();
    fs::copy(
        vendor.join(CoreId::Fceumm.file_name()),
        cores.join(CoreId::Fceumm.file_name()),
    )
    .unwrap();
    let cart = Cart {
        platform: Platform::Nes,
        stem: "loop".into(),
        title: "loop".into(),
        rom,
    };
    (card, cart, cores)
}

/// An iNES file: 16-byte header, one 16 KiB PRG bank, one 8 KiB CHR bank, mapper 0.
///
/// The same image `slot2-retro`'s own suite builds for its NES tests, written out here because
/// that builder lives in another crate's test support and is not reachable from this one.
/// Making it beats depending on a cartridge being dropped into `assets/test/local`: the
/// single-candidate case has to be provable on any machine that can run the rest of the file.
fn nes_rom() -> Vec<u8> {
    let mut rom = Vec::with_capacity(16 + 16384 + 8192);
    rom.extend_from_slice(b"NES\x1A");
    rom.push(1); // 16 KiB of PRG
    rom.push(1); // 8 KiB of CHR
    rom.extend_from_slice(&[0; 10]); // flags 6..15: mapper 0, no battery, no trainer

    // One PRG bank is mirrored into both $8000 and $C000, so the 6502 vectors are the last six
    // bytes of it: RESET points at the start, where `SEI; CLD; JMP $C000` parks the CPU.
    let mut prg = vec![0u8; 16384];
    prg[0..5].copy_from_slice(&[0x78, 0xD8, 0x4C, 0x00, 0xC0]);
    prg[0x3FFA] = 0x00;
    prg[0x3FFB] = 0xC0;
    prg[0x3FFC] = 0x00;
    prg[0x3FFD] = 0xC0;
    prg[0x3FFE] = 0x00;
    prg[0x3FFF] = 0xC0;
    rom.extend_from_slice(&prg);
    rom.extend_from_slice(&[0u8; 8192]); // blank pattern tables
    rom
}

/// The external core filename the fixtures write: what product resolution means on this host.
///
/// The setting names the core `mystery`, and `session::core_file_name` resolves that to the
/// `mystery_libretro` base name plus this platform's DLL extension: `dll` on Windows, `dylib`
/// on macOS, `so` on Linux and the other Unix targets. The extension comes from the standard
/// library's own DLL extension constant, so the fixture cannot pin a Windows-only suffix that a
/// Linux runner would then fail to find. No `lib` prefix: SLOT2 core filenames have none.
fn mystery_external_core() -> String {
    format!("mystery_libretro.{}", std::env::consts::DLL_EXTENSION)
}

/// A card whose game runs on a library this frontend does not ship, in a cores directory with
/// no official core in it at all. What the picker has to offer a session that is already
/// running, without taking a library away from a core that has it open.
fn external_fixture(tag: &str, vendor: &Path) -> (Card, Cart, PathBuf) {
    let (card, cart, cores) = fixture(tag, vendor);
    fs::remove_file(cores.join(CoreId::Mgba.file_name())).unwrap();
    fs::remove_file(cores.join(CoreId::Gpsp.file_name())).unwrap();
    fs::copy(
        vendor.join(CoreId::Mgba.file_name()),
        cores.join(mystery_external_core()),
    )
    .unwrap();
    card.write_settings(
        &cart,
        &GameSettings {
            core: Some("mystery".into()),
            ..Default::default()
        },
    )
    .unwrap();
    (card, cart, cores)
}

fn app_for(card: Card, cores: PathBuf) -> App {
    App::with_card(card, cores, 48_000, tuning(), false, Screen::List)
}

/// The namespace a core this frontend ships keeps its states in, spelled the way the store
/// spells it: every test checks a session's own `state_namespace()` against these.
fn ns(core: CoreId) -> StateNamespace {
    StateNamespace::new(core.base_name()).expect("a core's base name is a namespace")
}

fn tap(a: &mut App, b: Button, at: Instant) {
    a.feed(&ev(b, true, at));
    a.feed(&ev(b, false, at + ms(40)));
    a.tick(at + ms(60));
}

/// SELECT held, then the shoulder: what `Gestures` turns into `Action::Chord`.
fn chord(a: &mut App, shoulder: Button, at: Instant) {
    a.feed(&ev(Button::Select, true, at));
    a.feed(&ev(shoulder, true, at + ms(20)));
    a.feed(&ev(shoulder, false, at + ms(60)));
    a.feed(&ev(Button::Select, false, at + ms(80)));
    a.tick(at + ms(100));
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

/// Insert the cart and run until the game is on screen, draining the insert's pending requests
/// the way the loop does every frame.
fn reach_the_game(a: &mut App, from: Instant) -> Instant {
    let mut now = from;
    tap(a, Button::A, now);
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            return run(a, now, 0.2);
        }
    }
    panic!("the test cart never reached the game: {:?}", a.screen);
}

/// Tick the insert through to the game **without** running a core frame: the state the launch
/// landed on is the state the first Playing frame starts from, and a test that ran frames on
/// the way would be looking at a game that had already moved on.
fn seat_to_playing(a: &mut App, from: Instant) -> Instant {
    let mut now = from;
    for _ in 0..600 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Playing {
            let _ = a.take_sink_request();
            let _ = a.take_consumer();
            return now;
        }
    }
    panic!("the cart never reached the game: {:?}", a.screen);
}

/// Play for a moment at the seat, then leave with the MENU hold: stopping writes this core's
/// Resume and flushes its save RAM, which is what a switch has to checkpoint later.
fn play_then_leave(a: &mut App, from: Instant) -> Instant {
    let mut now = run(a, from, 0.3);
    a.feed(&ev(Button::Menu, true, now));
    for _ in 0..180 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen != Screen::Playing {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "the hold did not eject");
    let now = run(a, now, EJECT_S + 0.1);
    assert_eq!(a.screen, Screen::List, "the cart never came back out");
    now
}

/// The in-game menu, down to the Core row, then A: the picker as the player reaches it.
fn open_core_row(a: &mut App, at: Instant) -> Instant {
    let mut now = at;
    tap(a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    for _ in 0..CORE_ROW {
        now += ms(200);
        tap(a, Button::Down, now);
    }
    match a.screen {
        Screen::InGame(menu) => assert_eq!(menu.choice(), InGameChoice::Core, "the Core row moved"),
        other => panic!("the in-game menu is not open: {other:?}"),
    }
    now += ms(200);
    tap(a, Button::A, now);
    now
}

/// A panel and a context to draw the picker into.
fn ui() -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target("rgsp").unwrap(), "en", vec![fonts], None)
}

fn draw(a: &mut App, ctx: &mut UiCtx, at: Instant) -> RecordingCanvas {
    let mut canvas = RecordingCanvas::new(720, 480);
    a.draw(&mut canvas, ctx, at);
    canvas
}

/// Every text or image quad in a frame, as `(x, y, w)`.
fn images(ops: &[Op]) -> Vec<(f32, f32, f32)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Image { x, y, w, .. } => Some((*x, *y, *w)),
            _ => None,
        })
        .collect()
}

/// The width a line of text draws at, which is how a drawn quad is identified without
/// depending on texture ids.
fn width(ctx: &mut UiCtx, key: &str, px: f32) -> f32 {
    let spans = ctx.i18n.spans(key, &[]);
    face::spans_width(ctx, &spans, px)
}

/// The row a message is drawn on, as `(y, width)`, when it is the label of a candidate row.
fn row_of(ops: &[Op], ctx: &mut UiCtx, core: CoreId) -> Option<f32> {
    let w = width(ctx, CorePicker::key(core), PX_TITLE);
    images(ops)
        .iter()
        .find(|(_, _, iw)| (*iw - w).abs() < 0.5)
        .map(|(_, y, _)| *y)
}

/// Whether the `Current` badge is drawn on the row at `row_y`.
fn badge_at(ops: &[Op], ctx: &mut UiCtx, bx: f32, row_y: f32) -> bool {
    let w = width(ctx, CURRENT_KEY, PX_BODY);
    let x = bx + BOX_W - PAD - BADGE_INSET - w;
    images(ops).iter().any(|(ix, iy, iw)| {
        (*ix - x).abs() < 0.5 && (*iw - w).abs() < 0.5 && *iy >= row_y && *iy < row_y + ROW_H
    })
}

/// Whether the highlighted row's backing rect is at `row_y`.
fn highlighted(ops: &[Op], row_y: f32, bx: f32) -> bool {
    ops.iter().any(|o| {
        matches!(o, Op::Rect { x, y, w, h, color }
        if (*x - (bx + PAD)).abs() < 0.5
            && (*y - row_y).abs() < 0.5
            && (*w - (BOX_W - 2.0 * PAD)).abs() < 0.5
            && (*h - ROW_H).abs() < 0.5
            && *color == slot2_ui::splash::INK.with_alpha(0.15))
    })
}

/// The settings file's path with `.tmp` appended — where `atomic_write` builds its temporary
/// file. A directory there is a write that cannot happen, on every platform, with the real
/// file left exactly as it was.
fn block_write(path: &Path) {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    fs::create_dir_all(PathBuf::from(tmp)).unwrap();
}

// ------------------------------------------------------------------ opening the picker

#[test]
fn the_core_row_shows_the_platforms_cores_with_the_running_one_badged() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("open", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));

    let at = open_core_row(&mut a, now);
    assert!(matches!(a.screen, Screen::Core(_)), "{:?}", a.screen);
    match a.screen {
        Screen::Core(menu) => assert_eq!(menu.selected_index(), CORE_ROW),
        other => panic!("{other:?}"),
    }
    // Opening the picker is not a change: no setting, no state, no sink.
    assert!(a.audio_paused());
    assert!(a.take_sink_request().is_none());
    assert!(!card.game_settings_path(&cart).exists());
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));

    // The frame on screen is the game's last one under the picker.
    let mut ctx = ui();
    let canvas = draw(&mut a, &mut ctx, at);
    let ops = canvas.frame();
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the picker cleared the game frame"
    );
    match ops.iter().find(|o| {
        matches!(
            o,
            Op::Rect { .. } | Op::Image { .. } | Op::ImageEffect { .. }
        )
    }) {
        // The game's own frame, first and across the whole panel, drawn through the effect the
        // GBA gets by default when its game says nothing about shaders.
        Some(Op::ImageEffect {
            x, y, w, h, effect, ..
        }) => {
            assert_eq!((*x, *y, *w, *h), (0.0, 0.0, 720.0, 480.0));
            assert_eq!(*effect, slot2_gfx::ShaderEffect::Lcd3x);
        }
        other => panic!("the game frame is not under the picker: {other:?}"),
    }

    // Both of the platform's cores, in the registry's order, with the running one badged and
    // the highlight on it.
    let (bx, _) = CorePicker::box_origin(&ctx);
    let mgba = row_of(ops, &mut ctx, CoreId::Mgba).expect("no mGBA row");
    let gpsp = row_of(ops, &mut ctx, CoreId::Gpsp).expect("no gpSP row");
    assert!(mgba < gpsp, "the rows are not in the registry's order");
    let first = CorePicker::row_y(&ctx, 0);
    let second = CorePicker::row_y(&ctx, 1);
    assert!(
        badge_at(ops, &mut ctx, bx, first),
        "the running core is not badged"
    );
    assert!(
        !badge_at(ops, &mut ctx, bx, second),
        "the idle core is badged"
    );
    assert!(
        highlighted(ops, first, bx),
        "the highlight is not on the running core"
    );
    assert!(!highlighted(ops, second, bx));
}

#[test]
fn a_platform_with_one_core_says_so_and_stays_in_the_menu() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    if !vendor.join(CoreId::Fceumm.file_name()).is_file() {
        eprintln!(
            "no fceumm core in {} — skipping this case",
            vendor.display()
        );
        return;
    }
    let (card, cart, cores) = nes_fixture("single", &vendor);
    let mut a = app_for(card.clone(), cores);
    let t = Instant::now();
    // The shelf opens on the GBA, so the NES cart is one shelf along.
    tap(&mut a, Button::R1, t);
    assert_eq!(a.platform(), Platform::Nes, "the shelf did not change");
    let now = reach_the_game(&mut a, t + ms(200));
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Fceumm));

    let _ = open_core_row(&mut a, now);
    assert!(
        matches!(a.screen, Screen::InGame(_)),
        "a picker opened for a platform with one core: {:?}",
        a.screen
    );
    assert_eq!(a.toast_key(), Some("core-no-alternatives"));
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Fceumm));
    assert!(!card.game_settings_path(&cart).exists());
    assert!(a.take_sink_request().is_none());
}

#[test]
fn the_picker_pauses_the_game_and_hands_back_the_same_row() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("pause", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());

    let at = open_core_row(&mut a, now);
    let frames = a.session().unwrap().frames_run();
    let mut now = run(&mut a, at, 0.3);
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the core ran on under the picker"
    );

    // Out of the picker and the menu, back to the game: the rounds below open the menu from
    // `Playing`, which is where every one of them starts.
    now += ms(300);
    tap(&mut a, Button::B, now);
    now += ms(400);
    tap(&mut a, Button::B, now);
    assert_eq!(a.screen, Screen::Playing, "the picker would not close");

    for closer in [Button::B, Button::Menu] {
        now += ms(500);
        now = open_core_row(&mut a, now);
        now += ms(300);
        tap(&mut a, closer, now);
        match a.screen {
            Screen::InGame(menu) => assert_eq!(
                menu.selected_index(),
                CORE_ROW,
                "{closer:?} lost the row the picker was opened from"
            ),
            other => panic!("{closer:?} left the picker on {other:?}"),
        }
        assert!(
            a.take_sink_request().is_none(),
            "{closer:?} touched the session's audio"
        );
        assert!(!card.game_settings_path(&cart).exists());
        // Back to the game, which is where the next round opens the menu from.
        now += ms(400);
        tap(&mut a, Button::B, now);
        assert_eq!(a.screen, Screen::Playing, "{closer:?} left the menu open");
    }
}

// ------------------------------------------------------------------ choosing

#[test]
fn an_empty_picker_says_so_and_changes_nothing() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = external_fixture("empty", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());
    assert_eq!(
        a.session().unwrap().core_id(),
        None,
        "the fixture did not land on the external library"
    );

    let at = open_core_row(&mut a, now);
    assert!(matches!(a.screen, Screen::Core(_)), "{:?}", a.screen);
    tap(&mut a, Button::A, at);
    assert_eq!(a.toast_key(), Some("core-picker-empty"));
    assert!(
        matches!(a.screen, Screen::Core(_)),
        "an empty picker closed itself: {:?}",
        a.screen
    );
    assert_eq!(a.session().unwrap().core_id(), None);
    assert!(
        a.take_sink_request().is_none(),
        "an empty picker touched the sink"
    );
    assert_eq!(
        card.read_settings(&cart).core.as_deref(),
        Some("mystery"),
        "an empty picker changed the setting"
    );
    assert!(
        !card
            .scoped_state_path(
                &cart,
                &StateNamespace::new("mystery_libretro").unwrap(),
                StateKind::Resume
            )
            .exists(),
        "an empty picker checkpointed the game"
    );
}

#[test]
fn choosing_the_core_that_is_already_running_only_closes_the_picker() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("same", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());

    let at = open_core_row(&mut a, now);
    let frames = a.session().unwrap().frames_run();
    tap(&mut a, Button::A, at);

    match a.screen {
        Screen::InGame(menu) => assert_eq!(menu.selected_index(), CORE_ROW),
        other => panic!("{other:?}"),
    }
    assert!(
        a.take_sink_request().is_none(),
        "no change asked for a sink"
    );
    assert!(a.take_consumer().is_none());
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the game was restarted for a core that was already running"
    );
    assert!(
        !card.game_settings_path(&cart).exists(),
        "a setting was written for a core that was already running"
    );
    assert!(
        !card
            .scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume)
            .exists(),
        "a core that was already running was checkpointed"
    );
}

#[test]
fn switching_to_the_other_core_keeps_the_old_playthrough_and_saves_the_choice() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("switch", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());

    // One numbered state under mGBA, so the old core's own states can be shown to survive.
    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"));
    let mgba_one = card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1));
    let mgba_state = fs::read(&mgba_one).unwrap();

    // The save's message has to expire before a switch that says nothing can be told apart
    // from one that said something. Nothing is lost by letting the game run on.
    let now = run(&mut a, now, TOAST_S + 0.5);
    assert_eq!(
        a.toast_key(),
        None,
        "an old message outlived its three seconds"
    );

    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Down, at);
    tap(&mut a, Button::A, at + ms(200));

    // The game is running again, on the other core, and said nothing: a fresh target is not a
    // failure.
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    assert_eq!(a.toast_key(), None);
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Gpsp));
    assert_eq!(a.session().unwrap().state_namespace(), &ns(CoreId::Gpsp));
    assert_eq!(
        a.session().unwrap().frames_run(),
        0,
        "the new core ran before the player saw it"
    );

    // Exactly one sink, for the new consumer: the loop replaces the old sink rather than
    // stacking a second one.
    assert_eq!(a.take_sink_request(), Some(SinkRequest::Open));
    assert!(a.take_consumer().is_some(), "no consumer for the new sink");
    assert_eq!(a.take_sink_request(), None, "a second sink request");
    assert!(a.take_consumer().is_none());

    // The card: the choice, this core's own checkpoint, and the old core's states untouched.
    assert_eq!(
        card.read_settings(&cart).core.as_deref(),
        Some("gpsp_libretro")
    );
    assert!(
        card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume)
            .is_file(),
        "the checkpoint did not write the old core's resume"
    );
    assert_eq!(
        fs::read(&mgba_one).unwrap(),
        mgba_state,
        "the new core's save reached into the old core's states"
    );
    assert!(
        !card
            .scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
            .exists(),
        "the switch invented a resume for a core that had none"
    );
}

#[test]
fn going_back_to_the_platforms_own_core_loads_its_own_resume_and_keeps_the_rest() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("default", &vendor);
    let mut a = app_for(card.clone(), cores);

    // A playthrough on the platform's own core, left the way a player leaves one: this is the
    // Resume the switch below has to come back to.
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    let now = play_then_leave(&mut a, now);
    let mgba_resume = card
        .scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
        .expect("leaving the game wrote no resume");

    // A card with a real choice on it, plus everything else a game can have set, plus a key
    // this version has never heard of.
    let ini = card.game_settings_path(&cart);
    fs::create_dir_all(ini.parent().unwrap()).unwrap();
    fs::write(
        &ini,
        "core = gpsp_libretro\nscale = fill\noverscan = off\nfuture_thing = keep me\n",
    )
    .unwrap();

    // The other core runs the game on from there, so the two playthroughs are different
    // moments rather than the same one twice.
    let now = reach_the_game(&mut a, now);
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(CoreId::Gpsp),
        "the setting did not choose the core"
    );
    let now = run(&mut a, now, 0.4);

    // Up from the running core, which is the second row, is the platform's own.
    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Up, at);
    tap(&mut a, Button::A, at + ms(200));

    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));
    assert_eq!(
        a.toast_key(),
        None,
        "coming back to the default core said something"
    );
    assert_eq!(
        a.session().unwrap().frames_run(),
        0,
        "the new core ran before the player saw it"
    );

    // Only the target core's own resume was read. Nothing has run since the load, so saving now
    // writes exactly what was loaded — the resume this core left before the switch.
    chord(&mut a, Button::R1, at + ms(400));
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1))
            .unwrap(),
        mgba_resume,
        "the switch did not load the target core's own resume"
    );

    // The settings: the platform's own core is the absence of an override, and every other key
    // in the file survived.
    let settings = card.read_settings(&cart);
    assert_eq!(
        settings.core, None,
        "the platform's own core was left as an override"
    );
    assert_eq!(settings.scale, Some(ScaleMode::Fill));
    assert_eq!(settings.overscan, Some(false));
    let text = fs::read_to_string(&ini).unwrap();
    assert!(
        !text.lines().any(|l| l.trim_start().starts_with("core")),
        "the core key is still in the file: {text}"
    );
    assert!(text.contains("scale = fill"), "{text}");
    assert!(text.contains("overscan = off"), "{text}");
    assert!(text.contains("future_thing = keep me"), "{text}");

    // And the core that was left keeps its own playthrough, written by the checkpoint.
    let gpsp_resume = card
        .scoped_read_state(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
        .expect("the checkpoint wrote no resume for the core it left");
    assert_ne!(
        gpsp_resume, mgba_resume,
        "the two cores' resumes came out as the same bytes"
    );
}

// ------------------------------------------------------------------ failures

#[test]
fn a_checkpoint_that_cannot_be_written_leaves_everything_alone() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("checkpoint", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());
    let frames = a.session().unwrap().frames_run();
    assert!(
        frames > 0,
        "the game never ran, so this test proves nothing"
    );

    // The resume write is the half of the checkpoint that can be blocked on this card; the save
    // RAM half is covered by the test that shows it running even when this one fails.
    let resume = card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume);
    block_write(&resume);

    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Down, at);
    tap(&mut a, Button::A, at + ms(200));

    assert_eq!(a.toast_key(), Some("core-checkpoint-failed"));
    assert!(
        matches!(a.screen, Screen::Core(_)),
        "a failed checkpoint left the picker: {:?}",
        a.screen
    );
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(CoreId::Mgba),
        "the session was let go for a checkpoint that failed"
    );
    assert_eq!(
        a.session().unwrap().frames_run(),
        frames,
        "the session was restarted or the core ran under the picker"
    );
    assert!(
        a.take_sink_request().is_none(),
        "a failed checkpoint touched the sink"
    );
    assert!(
        !card.game_settings_path(&cart).exists(),
        "the setting was written before the switch was safe"
    );
    assert!(
        !resume.is_file(),
        "a resume appeared where the write could not go"
    );
    assert!(!card
        .scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
        .exists());
}

#[test]
fn a_checkpoint_that_cannot_write_its_resume_still_flushes_the_save() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("flush", &vendor);
    let mut a = app_for(card.clone(), cores);

    // Leaving a game flushes save RAM, so the card holds the save this switch has to write
    // again. mGBA hands over a save region for every GBA cartridge (the libretro wrapper maps
    // `GBA_SIZE_FLASH1M` bytes of it before the ROM is even loaded), so this is real save RAM
    // and not an empty call.
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    let now = play_then_leave(&mut a, now);
    let save = card.save_path(&cart);
    let saved = fs::read(&save).expect("leaving the game wrote no save RAM");
    assert!(!saved.is_empty(), "the save RAM that was written is empty");

    // Both halves of the checkpoint start from nothing on the card: the resume the switch would
    // write, and the save RAM it would flush.
    let resume = card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume);
    fs::remove_file(&save).unwrap();
    fs::remove_file(&resume).unwrap();

    // Back in, with the resume write blocked: the half of the checkpoint that cannot happen.
    let now = reach_the_game(&mut a, now);
    block_write(&resume);

    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Down, at);
    tap(&mut a, Button::A, at + ms(200));

    assert_eq!(a.toast_key(), Some("core-checkpoint-failed"));
    assert!(
        matches!(a.screen, Screen::Core(_)),
        "the picker did not stay open: {:?}",
        a.screen
    );
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));
    assert!(
        !resume.is_file(),
        "a resume appeared where the write could not go"
    );

    // The other half ran anyway, and the switch did not start because the checkpoint as a whole
    // failed: the save RAM is on the card again, byte for byte.
    let again = fs::read(&save).expect("a failed checkpoint wrote no save RAM");
    assert_eq!(
        again, saved,
        "the flush wrote something other than the save RAM it holds"
    );
    assert!(
        !card
            .scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
            .exists(),
        "a checkpoint that failed still moved the game to another core"
    );
}

#[test]
fn a_target_that_will_not_load_comes_back_to_the_core_and_the_playthrough() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("bogus", &vendor);
    // The name of a real core and none of its contents: the name is what makes it a candidate,
    // and the file is what makes it refuse to open.
    fs::write(cores.join(CoreId::Gpsp.file_name()), b"not a library").unwrap();

    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));

    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Down, at);
    tap(&mut a, Button::A, at + ms(200));

    assert_eq!(a.toast_key(), Some("core-switch-failed"));
    assert!(
        matches!(a.screen, Screen::Core(_)),
        "a failed switch left the picker: {:?}",
        a.screen
    );
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(CoreId::Mgba),
        "the old core did not come back"
    );
    assert_eq!(a.session().unwrap().state_namespace(), &ns(CoreId::Mgba));
    assert_eq!(a.take_sink_request(), Some(SinkRequest::Open));
    assert!(
        a.take_consumer().is_some(),
        "no consumer for the recovered core"
    );
    assert_eq!(a.take_sink_request(), None);
    assert!(
        !card.game_settings_path(&cart).exists(),
        "the setting was written for a core that never started"
    );

    // The playthrough came back with it: out of the picker, out of the menu, and save at once.
    // Nothing has run since the load, so what is written is what was loaded.
    let resume = card
        .scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
        .expect("the checkpoint wrote no resume");
    tap(&mut a, Button::B, at + ms(400));
    tap(&mut a, Button::B, at + ms(600));
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    chord(&mut a, Button::R1, at + ms(800));
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1))
            .unwrap(),
        resume,
        "the recovered core was not on the checkpointed playthrough"
    );
}

#[test]
fn a_setting_that_will_not_save_comes_back_to_the_core() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("savefail", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());

    // The ini is readable and the temporary file beside it cannot be created, which is the
    // deterministic way to fail exactly this one step.
    let ini = card.game_settings_path(&cart);
    fs::create_dir_all(ini.parent().unwrap()).unwrap();
    block_write(&ini);

    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Down, at);
    tap(&mut a, Button::A, at + ms(200));

    assert_eq!(a.toast_key(), Some("core-setting-save-failed"));
    assert!(
        matches!(a.screen, Screen::Core(_)),
        "a choice that could not be saved left the picker: {:?}",
        a.screen
    );
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(CoreId::Mgba),
        "the target core was kept for a choice that could not be saved"
    );
    assert_eq!(a.take_sink_request(), Some(SinkRequest::Open));
    assert!(a.take_consumer().is_some());
    assert!(
        card.read_settings(&cart).core.is_none(),
        "the settings came out half-written"
    );
    assert!(
        !ini.is_file(),
        "a settings file appeared where the write could not go"
    );
    // The core that started and then had to be thrown away left its own namespace alone: no
    // other core's resume was read, written or removed.
    assert!(!card
        .scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
        .exists());
    assert!(
        card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume)
            .is_file(),
        "the checkpoint's resume went missing"
    );
}

#[test]
fn a_broken_target_resume_starts_the_new_core_fresh() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("targetresume", &vendor);
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());

    // The other core's resume is there and is not a state: the switch still happens, and the
    // game starts over rather than being handed bytes it cannot read.
    let gpsp_resume = card.scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume);
    fs::create_dir_all(gpsp_resume.parent().unwrap()).unwrap();
    fs::write(&gpsp_resume, b"not a state").unwrap();

    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Down, at);
    tap(&mut a, Button::A, at + ms(200));

    assert_eq!(a.toast_key(), Some("resume-load-failed"));
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Gpsp));
    assert_eq!(
        card.read_settings(&cart).core.as_deref(),
        Some("gpsp_libretro"),
        "the choice was not saved because the resume was unreadable"
    );
    assert_eq!(
        fs::read(&gpsp_resume).unwrap(),
        b"not a state",
        "the state it could not read was removed"
    );
    // The core it left keeps its own resume: the switch reads and removes nothing of another
    // core's.
    assert!(card
        .scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume)
        .is_file());
}

#[test]
fn a_recovery_that_cannot_read_the_old_playthrough_says_so() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("recoverstate", &vendor);

    // The game runs on the core the card names.
    let settings = GameSettings {
        core: Some("gpsp".into()),
        ..Default::default()
    };
    card.write_settings(&cart, &settings).unwrap();
    let mut a = app_for(card.clone(), cores);
    let now = reach_the_game(&mut a, Instant::now());
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Gpsp));

    // An unreadable resume in the platform's own namespace: the core the resolver falls back
    // to when it cannot read what the card says.
    let fallback = card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume);
    fs::create_dir_all(fallback.parent().unwrap()).unwrap();
    fs::write(&fallback, b"not a state").unwrap();
    // The settings file becomes something that can neither be read nor written — a card fault,
    // and the deterministic way to make what the settings mean change under a running session
    // without touching a library the core has open.
    let ini = card.game_settings_path(&cart);
    fs::remove_file(&ini).unwrap();
    fs::create_dir_all(&ini).unwrap();

    // Up from the running core, which is the second row: the platform's own.
    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Up, at);
    tap(&mut a, Button::A, at + ms(200));

    assert_eq!(
        a.toast_key(),
        Some("core-recovery-state-failed"),
        "the worse failure was not the one reported"
    );
    assert!(matches!(a.screen, Screen::Core(_)), "{:?}", a.screen);
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(CoreId::Mgba),
        "the core the settings now mean did not open"
    );
    assert_eq!(a.take_sink_request(), Some(SinkRequest::Open));
    assert!(a.take_consumer().is_some());
    // Nothing was written over the settings, and what the checkpoint wrote for the core that
    // was running is still there.
    assert!(
        ini.is_dir(),
        "a settings file appeared where none could be written"
    );
    assert!(card
        .scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
        .is_file());
    assert_eq!(fs::read(&fallback).unwrap(), b"not a state");
}

#[test]
fn a_recovery_that_cannot_open_the_old_core_takes_the_cart_out() {
    let _serial = serial();
    let Some(vendor) = vendor() else { return };
    let (card, cart, cores) = fixture("recoverfail", &vendor);
    // The core the target choice needs will not open...
    fs::write(cores.join(CoreId::Gpsp.file_name()), b"not a library").unwrap();
    // ...and the library the card names is not there yet, so the game starts on the platform's
    // own instead. It turns up, unopenable, before the choice is made: the recovery has nothing
    // left to open, and no session is better than an overlay over a game that is not there.
    card.write_settings(
        &cart,
        &GameSettings {
            core: Some("mystery".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let mut a = app_for(card.clone(), cores.clone());
    let now = reach_the_game(&mut a, Instant::now());
    assert_eq!(
        a.session().unwrap().core_id(),
        Some(CoreId::Mgba),
        "the fixture did not start on the fallback core"
    );

    fs::write(cores.join(mystery_external_core()), b"not a library").unwrap();
    let at = open_core_row(&mut a, now);
    tap(&mut a, Button::Down, at);
    // A is taken without the usual trailing tick: the request the failed switch leaves behind is
    // the one thing the next tick is about to replace. Every eject makes a sound, and the sound
    // needs a sink of its own, so the close is what the loop is told at this moment and the
    // sound's own open follows immediately afterwards — exactly as a MENU-hold eject does it.
    a.feed(&ev(Button::A, true, at + ms(200)));
    a.feed(&ev(Button::A, false, at + ms(240)));
    assert_eq!(
        a.take_sink_request(),
        Some(SinkRequest::Close),
        "the dead session's sink was left open"
    );
    assert!(
        a.take_consumer().is_none(),
        "the dead session's consumer was installed as a new sink"
    );
    a.tick(at + ms(260));

    assert_eq!(a.toast_key(), Some("core-recovery-failed"));
    assert!(
        a.session().is_none(),
        "a session appeared from a core that could not be opened"
    );
    assert_eq!(
        a.screen,
        Screen::Ejecting,
        "the cart was left on a screen with no game behind it"
    );

    // And the eject finishes the way every other one does, with nothing else asked of the
    // loop: the shelf is where a cart goes when there is no game to go back to.
    let mut now = at + ms(260);
    for _ in 0..((EJECT_S + 0.3) * 60.0).ceil() as u32 {
        now += Duration::from_micros(16_667);
        a.tick(now);
    }
    assert_eq!(a.screen, Screen::List, "the failed cart never came out");
    assert!(!a.audio_paused(), "the shelf kept the game's audio paused");
    assert_eq!(
        card.read_settings(&cart).core.as_deref(),
        Some("mystery"),
        "the settings were changed by a switch that did not happen"
    );
}
