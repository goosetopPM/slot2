//! Which core's states a game sees, from the shelf all the way down.
//!
//! A libretro state is the core's own serialization, so one numbered slot means two different
//! sets of bytes depending on the core that wrote it. These tests drive one GBA cart through
//! both of the cores this build ships for it — mGBA, the platform's own, and gpSP, the
//! alternative a settings file can name — and check that no product path ever shows one
//! core's Resume or numbered states to the other. Needs `vendor/mgba_libretro.*` and
//! `vendor/gpsp_libretro.*` (build/cores.ps1) and the MIT test ROM; without either core the
//! whole file skips loudly.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use slot2_audio::Volume;
use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_retro::CoreId;
use slot2_store::{Card, Cart, GameSettings, Platform, StateKind, StateNamespace};
use slot2_ui::insert::EJECT_S;
use slot2_ui::layout::SafeArea;
use slot2_ui::toast::TOAST_S;
use slot2_ui::UiCtx;

use slot2::app::{App, Screen};
use slot2::session::Session;

/// A libretro core is a library loaded once per process, so these tests take turns.
static SERIAL: Mutex<()> = Mutex::new(());
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Both cores this file needs. gpSP is a build product of this repository, and both it and
/// mGBA have to be there for a comparison between them to prove anything.
fn cores_dir() -> Option<PathBuf> {
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

fn scratch(tag: &str) -> PathBuf {
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("slot2-coreroute-{tag}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A card with the test ROM on it, and the cart that names it.
fn card_with_rom(tag: &str) -> (Card, Cart, PathBuf) {
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    let rom = card.games_dir(Platform::Gba).join("arm.gba");
    fs::copy(repo().join("assets/test/arm.gba"), &rom).unwrap();
    let cart = Cart {
        platform: Platform::Gba,
        stem: "arm".into(),
        title: "arm".into(),
        rom,
    };
    (card, cart, root)
}

fn app_for(card: Card, cores: PathBuf) -> App {
    App::with_card(card, cores, 48_000, tuning(), false, Screen::List)
}

/// The core this game's settings file names, or the platform's own when it names none.
fn name_core(card: &Card, cart: &Cart, core: Option<&str>) {
    let settings = GameSettings {
        core: core.map(str::to_owned),
        ..Default::default()
    };
    card.write_settings(cart, &settings).unwrap();
}

/// The namespace a core this frontend ships keeps its states in, spelled the way the store
/// spells it. Every test checks a session's own `state_namespace()` against these, so the
/// fixture cannot quietly drift from what the product does.
fn ns(core: CoreId) -> StateNamespace {
    StateNamespace::new(core.base_name()).expect("a core's base name is a namespace")
}

/// This core's numbered slots, in order: what its switcher would list, without the resume.
fn numbered(card: &Card, cart: &Cart, core: CoreId) -> Vec<u32> {
    let mut ns: Vec<u32> = card
        .scoped_list_states(cart, &ns(core))
        .iter()
        .filter_map(|slot| match slot.kind {
            StateKind::Numbered(n) => Some(n),
            StateKind::Resume => None,
        })
        .collect();
    ns.sort_unstable();
    ns
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

/// Insert the test cart and run until the game is on screen, draining the insert's pending
/// requests the way the loop does every frame.
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
            return now;
        }
    }
    panic!("the cart never reached the game: {:?}", a.screen);
}

/// Play for a moment at the seat, then leave with the MENU hold: stopping writes this core's
/// resume state, which is what the tests below launch from.
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

/// The Save State row, then A: the switcher as the player reaches it.
fn open_switcher(a: &mut App, at: Instant) -> Instant {
    let mut now = at;
    tap(a, Button::Menu, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    now += ms(200);
    tap(a, Button::Down, now);
    now += ms(200);
    tap(a, Button::A, now);
    now + ms(200)
}

/// Out of the switcher, through the in-game menu, back to the game.
fn close_switcher(a: &mut App, at: Instant) -> Instant {
    let mut now = at;
    tap(a, Button::B, now);
    assert!(matches!(a.screen, Screen::InGame(_)), "{:?}", a.screen);
    now += ms(200);
    tap(a, Button::Up, now);
    now += ms(200);
    tap(a, Button::A, now);
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
    now + ms(200)
}

/// How many resume hints the shelf is wearing. The hint is drawn from the scan's cached
/// answer, so counting the caps it paints is the only way to see it from outside; with one
/// cart on the shelf, one cap is the cart and two is the cart plus its hint.
fn resume_hints(a: &mut App) -> usize {
    let safe = SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    ctx.safe = safe;
    a.draw(&mut canvas, &mut ctx, Instant::now());
    let cap = slot2_ui::splash::INK_DIM.with_alpha(0.25);
    canvas
        .frame()
        .iter()
        .filter(|o| matches!(o, Op::Rect { color, .. } if *color == cap))
        .count()
}

// ------------------------------------------------------------------ the sessions themselves

#[test]
fn two_cores_keep_two_sets_of_states_for_one_game() {
    let _serial = serial();
    let Some(cores) = cores_dir() else { return };
    let (card, cart, _root) = card_with_rom("twocores");
    let volume = Volume::default();

    // mGBA, the platform's own core: a numbered state and a resume, written by the session.
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.core_id(), Some(CoreId::Mgba));
    assert_eq!(s.state_namespace(), &ns(CoreId::Mgba));
    for _ in 0..20 {
        s.run_frame(&[], &volume);
    }
    s.save_state(&card, StateKind::Numbered(1)).unwrap();
    let mgba_slot = card
        .scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1))
        .unwrap();
    s.stop(&card);
    let mgba_resume = card
        .scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
        .unwrap();

    // gpSP, named by this game's settings file: the same cart, its own namespace, and no
    // sight of anything mGBA wrote.
    name_core(&card, &cart, Some("gpsp"));
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.core_id(), Some(CoreId::Gpsp));
    assert_eq!(s.state_namespace(), &ns(CoreId::Gpsp));
    assert_ne!(s.state_namespace(), &ns(CoreId::Mgba));
    assert!(
        card.scoped_list_states(&cart, &ns(CoreId::Gpsp)).is_empty(),
        "the alternative core was shown the other core's states"
    );
    assert_eq!(
        card.scoped_next_state_number(&cart, &ns(CoreId::Gpsp)),
        1,
        "the slot numbers are not this core's own"
    );
    assert!(
        s.load_state(&card, StateKind::Resume).is_err(),
        "mGBA's resume was handed to gpSP"
    );

    for _ in 0..20 {
        s.run_frame(&[], &volume);
    }
    s.save_state(&card, StateKind::Numbered(1)).unwrap();
    let gpsp_slot = card
        .scoped_read_state(&cart, &ns(CoreId::Gpsp), StateKind::Numbered(1))
        .unwrap();
    assert_ne!(
        gpsp_slot, mgba_slot,
        "the same slot number held the same bytes"
    );
    s.stop(&card);
    let gpsp_resume = card
        .scoped_read_state(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
        .unwrap();
    assert_ne!(
        gpsp_resume, mgba_resume,
        "one core's resume is not the other's"
    );

    // Both sets are on the card at once, each where it was written, and neither moved.
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1))
            .unwrap(),
        mgba_slot
    );
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
            .unwrap(),
        mgba_resume
    );
    assert_eq!(numbered(&card, &cart, CoreId::Mgba), vec![1]);
    assert_eq!(numbered(&card, &cart, CoreId::Gpsp), vec![1]);

    // Back to mGBA: its own resume loads again, and its next slot is the one after its own.
    name_core(&card, &cart, None);
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.core_id(), Some(CoreId::Mgba));
    s.load_state(&card, StateKind::Resume).unwrap();
    assert_eq!(card.scoped_next_state_number(&cart, &ns(CoreId::Mgba)), 2);
    s.save_state(&card, StateKind::Numbered(2)).unwrap();
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Gpsp), StateKind::Numbered(1))
            .unwrap(),
        gpsp_slot,
        "mGBA's save reached into gpSP's namespace"
    );
    s.stop(&card);
}

// ------------------------------------------------------------------ the shelf's scan

#[test]
fn older_flat_states_move_to_the_platforms_own_core_once() {
    let _serial = serial();
    let Some(cores) = cores_dir() else { return };
    let (card, cart, _root) = card_with_rom("legacy");

    // A card from before a game could be given a core: a flat resume and a flat numbered slot,
    // each with the picture that belongs beside it.
    let flat_resume = card.state_path(&cart, StateKind::Resume);
    let flat_one = card.state_path(&cart, StateKind::Numbered(1));
    fs::create_dir_all(flat_resume.parent().unwrap()).unwrap();
    fs::write(&flat_resume, b"the resume from the old version").unwrap();
    fs::write(&flat_one, b"the slot from the old version").unwrap();
    fs::write(flat_one.with_extension("png"), b"its picture").unwrap();

    // The player has since named gpSP for this game. The flat states are still mGBA's: they
    // were written before an alternative could be chosen at all.
    name_core(&card, &cart, Some("gpsp"));
    let mut a = app_for(card.clone(), cores);
    a.rescan();

    assert!(
        !flat_resume.exists() && !flat_one.exists(),
        "the flat states were copied rather than moved, or left behind"
    );
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
            .unwrap(),
        b"the resume from the old version"
    );
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1))
            .unwrap(),
        b"the slot from the old version"
    );
    assert!(
        card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1))
            .with_extension("png")
            .is_file(),
        "the picture was left behind"
    );

    // gpSP sees none of it: not in a listing, and not as a resume the shelf offers, because
    // the core that opens this game cannot read what mGBA wrote.
    assert!(card.scoped_list_states(&cart, &ns(CoreId::Gpsp)).is_empty());
    assert_eq!(
        resume_hints(&mut a),
        1,
        "the shelf offered mGBA's resume to gpSP"
    );

    // The same cart under the core that owns them does offer it.
    name_core(&card, &cart, None);
    a.rescan();
    assert_eq!(
        resume_hints(&mut a),
        2,
        "the shelf lost the resume it moved"
    );

    // A second scan finds nothing left to move and touches nothing.
    a.rescan();
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
            .unwrap(),
        b"the resume from the old version"
    );
}

#[test]
fn a_flat_state_that_cannot_move_is_left_where_it_is_and_never_merged() {
    let _serial = serial();
    let Some(cores) = cores_dir() else { return };
    let (card, cart, _root) = card_with_rom("conflict");
    let mut a = app_for(card.clone(), cores);

    // A real mGBA resume, written the way the product writes one.
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    let now = play_then_leave(&mut a, now);
    let scoped = card
        .scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
        .unwrap();

    // And an older flat resume in the place the move would write to. The move refuses rather
    // than choosing one of them: which is newer is not something the frontend can know.
    let flat = card.state_path(&cart, StateKind::Resume);
    fs::create_dir_all(flat.parent().unwrap()).unwrap();
    fs::write(&flat, b"an older, flat resume").unwrap();
    a.rescan();

    assert_eq!(
        fs::read(&flat).unwrap(),
        b"an older, flat resume",
        "the flat original was changed by a move that could not happen"
    );
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
            .unwrap(),
        scoped,
        "the scoped resume was overwritten"
    );
    assert_eq!(card.scoped_list_states(&cart, &ns(CoreId::Mgba)).len(), 1);

    // The launch reads the scoped file, which is the one the hint was about: the game comes
    // back where it was, and a save straight afterwards writes exactly what was loaded.
    assert_eq!(resume_hints(&mut a), 2);
    tap(&mut a, Button::A, now);
    let now = seat_to_playing(&mut a, now + ms(60));
    assert_eq!(a.toast_key(), None, "the resume could not be read");
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));
    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1))
            .unwrap(),
        scoped,
        "the launch did not land on the scoped resume"
    );
    assert_eq!(fs::read(&flat).unwrap(), b"an older, flat resume");
}

// ------------------------------------------------------------------ launching

#[test]
fn a_tap_resumes_from_the_core_that_opens_and_a_hold_drops_only_that_cores() {
    let _serial = serial();
    let Some(cores) = cores_dir() else { return };
    let (card, cart, _root) = card_with_rom("launching");
    let mut a = app_for(card.clone(), cores);

    // Play under mGBA and leave: that core's resume is on the card.
    let t = Instant::now();
    tap(&mut a, Button::A, t);
    let now = seat_to_playing(&mut a, t + ms(60));
    let now = play_then_leave(&mut a, now);
    let mgba_resume_path = card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Resume);
    let mgba_resume = fs::read(&mgba_resume_path).unwrap();
    let mgba_thumb = fs::read(mgba_resume_path.with_extension("png")).unwrap();

    // The player names gpSP. Its resume is not mGBA's, so a tap starts fresh rather than
    // handing mGBA's bytes to a core that cannot read them.
    name_core(&card, &cart, Some("gpsp"));
    a.rescan();
    assert_eq!(resume_hints(&mut a), 1, "gpSP was offered mGBA's resume");
    tap(&mut a, Button::A, now);
    let now = seat_to_playing(&mut a, now + ms(60));
    assert_eq!(a.toast_key(), None, "another core's resume was a failure");
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Gpsp));

    // Leave again: gpSP has a resume of its own now, and mGBA's is untouched.
    let now = run(&mut a, now, 0.3);
    let now = play_then_leave(&mut a, now);
    let gpsp_resume = card
        .scoped_read_state(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
        .unwrap();
    assert_ne!(gpsp_resume, mgba_resume);
    assert_eq!(fs::read(&mgba_resume_path).unwrap(), mgba_resume);
    assert_eq!(
        fs::read(mgba_resume_path.with_extension("png")).unwrap(),
        mgba_thumb
    );

    // Only gpSP's own resume is offered to gpSP, and resuming lands on it: nothing has run
    // since the load, so a save now writes exactly what was loaded.
    a.rescan();
    assert_eq!(resume_hints(&mut a), 2, "gpSP's own resume was not offered");
    tap(&mut a, Button::A, now);
    let now = seat_to_playing(&mut a, now + ms(60));
    assert_eq!(a.toast_key(), None);
    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Gpsp), StateKind::Numbered(1))
            .unwrap(),
        gpsp_resume,
        "the tap did not resume from gpSP's own state"
    );

    // A hold starts over: gpSP's resume goes, mGBA's stays.
    let now = run(&mut a, now, 0.3);
    let now = play_then_leave(&mut a, now);
    a.rescan();
    a.feed(&ev(Button::A, true, now));
    a.tick(now + ms(700));
    assert_eq!(
        a.screen,
        Screen::Inserting,
        "the hold did not start an insert"
    );
    assert!(card
        .scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
        .is_file());
    let now = seat_to_playing(&mut a, now + ms(800));
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Gpsp));
    assert!(
        !card
            .scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Resume)
            .exists(),
        "a fresh launch left gpSP's resume behind"
    );
    assert_eq!(
        fs::read(&mgba_resume_path).unwrap(),
        mgba_resume,
        "a fresh gpSP launch took mGBA's resume with it"
    );
    assert_eq!(
        fs::read(mgba_resume_path.with_extension("png")).unwrap(),
        mgba_thumb
    );

    // And the release does not start a second insert.
    a.feed(&ev(Button::A, false, now + ms(200)));
    a.tick(now + ms(300));
    assert_eq!(a.screen, Screen::Playing, "{:?}", a.screen);
}

// ------------------------------------------------------------------ playing

#[test]
fn quick_state_and_the_switcher_only_see_the_opened_cores_states() {
    let _serial = serial();
    let Some(cores) = cores_dir() else { return };
    let (card, cart, _root) = card_with_rom("playing");
    let mut a = app_for(card.clone(), cores);

    // mGBA: one quick save, with the picture the switcher shows.
    let now = reach_the_game(&mut a, Instant::now());
    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"));
    let mgba_one = card.scoped_state_path(&cart, &ns(CoreId::Mgba), StateKind::Numbered(1));
    let mgba_slot = fs::read(&mgba_one).unwrap();
    let mgba_thumb = fs::read(mgba_one.with_extension("png")).unwrap();

    // Out, and back in under the other core.
    let now = play_then_leave(&mut a, now);
    let mgba_resume = card
        .scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Resume)
        .unwrap();
    name_core(&card, &cart, Some("gpsp"));
    let now = reach_the_game(&mut a, now);
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Gpsp));
    assert_eq!(a.session().unwrap().state_namespace(), &ns(CoreId::Gpsp));

    // The switcher offers nothing: mGBA's slot 1 is not a state this core can read.
    let at = open_switcher(&mut a, now);
    tap(&mut a, Button::A, at);
    assert_eq!(
        a.toast_key(),
        Some("states-empty"),
        "gpSP was offered mGBA's slot"
    );
    assert!(
        matches!(a.screen, Screen::Switcher(_)),
        "an empty view closed itself: {:?}",
        a.screen
    );
    let mut now = close_switcher(&mut a, at + ms(200));

    // The quick save numbers from this core's own list: slot 1, not slot 2.
    now = run(&mut a, now, 0.3);
    chord(&mut a, Button::R1, now);
    assert_eq!(a.toast_key(), Some("state-saved"));
    let gpsp_one = card.scoped_state_path(&cart, &ns(CoreId::Gpsp), StateKind::Numbered(1));
    assert!(gpsp_one.is_file(), "gpSP did not write its own slot 1");
    let gpsp_slot = fs::read(&gpsp_one).unwrap();
    assert_ne!(gpsp_slot, mgba_slot);
    assert_eq!(
        fs::read(&mgba_one).unwrap(),
        mgba_slot,
        "gpSP's save reached into mGBA's slot 1"
    );

    // X deletes what this core's switcher was showing, and nothing else.
    let at = open_switcher(&mut a, now);
    tap(&mut a, Button::X, at);
    assert_eq!(a.toast_key(), Some("state-deleted"));
    assert!(!gpsp_one.exists(), "the state stayed on the card");
    assert!(!gpsp_one.with_extension("png").exists());
    assert_eq!(fs::read(&mgba_one).unwrap(), mgba_slot);
    assert_eq!(
        fs::read(mgba_one.with_extension("png")).unwrap(),
        mgba_thumb
    );

    // Y puts gpSP's own back, with its bytes.
    let at = at + ms(200);
    tap(&mut a, Button::Y, at);
    assert_eq!(a.toast_key(), Some("state-restored"));
    assert_eq!(fs::read(&gpsp_one).unwrap(), gpsp_slot);
    assert_eq!(fs::read(&mgba_one).unwrap(), mgba_slot);

    // A loads it: saving straight afterwards writes the same bytes into this core's slot 2.
    // Nothing runs a core frame in between, or this would be a different moment.
    let at = at + ms(200);
    tap(&mut a, Button::A, at);
    assert_eq!(a.toast_key(), Some("state-loaded"));
    assert_eq!(a.screen, Screen::Playing);
    chord(&mut a, Button::R1, at + ms(200));
    assert_eq!(a.toast_key(), Some("state-saved"));
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Gpsp), StateKind::Numbered(2))
            .unwrap(),
        gpsp_slot,
        "the switcher loaded a different state"
    );
    assert_eq!(fs::read(&mgba_one).unwrap(), mgba_slot);

    // Back to mGBA: its own resume and its own slot are what it sees again.
    let now = play_then_leave(&mut a, at + ms(200));
    name_core(&card, &cart, None);
    a.rescan();
    assert_eq!(numbered(&card, &cart, CoreId::Mgba), vec![1]);
    // The earlier save's message has to be gone before a launch that says nothing can be told
    // apart from one that was never asked to resume. Nothing runs on the shelf, so this is free.
    let now = run(&mut a, now, TOAST_S + 0.5);
    assert_eq!(
        a.toast_key(),
        None,
        "the shelf was still wearing an old message"
    );
    tap(&mut a, Button::A, now);
    let now = seat_to_playing(&mut a, now + ms(60));
    assert_eq!(a.session().unwrap().core_id(), Some(CoreId::Mgba));
    assert_eq!(a.toast_key(), None, "mGBA's own resume could not be read");
    chord(&mut a, Button::R1, now);
    assert_eq!(
        card.scoped_read_state(&cart, &ns(CoreId::Mgba), StateKind::Numbered(2))
            .unwrap(),
        mgba_resume,
        "mGBA did not come back to its own resume"
    );
    assert_eq!(fs::read(&gpsp_one).unwrap(), gpsp_slot);
}
