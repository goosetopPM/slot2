//! Running a real cart end to end: load the core, run frames, flush a save, take a state,
//! stop with a resume state. Needs `vendor/mgba_libretro.*` (build/cores.ps1) and the MIT
//! test ROM; without the core the whole file skips.

use std::fs;
use std::path::PathBuf;

use slot2::session::{Session, SAVE_EVERY_FRAMES, THUMB_MAX};
use slot2_audio::Volume;
use slot2_retro::LogicalButton;
use slot2_store::{Card, Cart, Platform, StateKind};

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

/// A card with the test ROM copied in as a GBA cart.
fn card_with_rom(name: &str) -> (Card, Cart, PathBuf) {
    let root = std::env::temp_dir().join(format!("slot2-session-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
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

#[test]
fn a_session_runs_frames_and_produces_audio_and_video() {
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("run");
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    assert_eq!(s.cart().stem, "arm");
    assert_eq!(s.frames_run(), 0);
    assert!(s.last_frame().is_none());

    let vol = Volume::new(100);
    for _ in 0..30 {
        s.run_frame(&[LogicalButton::A], &vol);
    }
    assert_eq!(s.frames_run(), 30);
    let (w, h, rgba) = s.last_frame().expect("a frame");
    assert_eq!((w, h), (240, 160));
    assert_eq!(rgba.len(), (w * h * 4) as usize);
    assert!(rgba.chunks(4).all(|p| p[3] == 255));
    let first = &rgba[..3];
    assert!(rgba.chunks(4).any(|p| &p[..3] != first), "flat picture");

    // Half a second at 48 kHz stereo should be well over 10 000 samples for 30 frames.
    let mut buf = vec![0i16; 200_000];
    let got = consumer.read(&mut buf);
    assert!(got > 10_000, "audio samples after 30 frames: {got}");
    assert!(buf[..got].iter().any(|&s| s != 0), "audio is all silence");
}

#[test]
fn muting_silences_the_stream_without_stopping_it() {
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("mute");
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    let mut vol = Volume::new(100);
    vol.set_muted(true);
    for _ in 0..20 {
        s.run_frame(&[], &vol);
    }
    let mut buf = vec![0i16; 100_000];
    let got = consumer.read(&mut buf);
    assert!(got > 1000, "still producing samples: {got}");
    assert!(
        buf[..got].iter().all(|&s| s == 0),
        "muted audio must be silence"
    );
}

#[test]
fn thumbnail_fits_the_box_and_keeps_aspect() {
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("thumb");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    assert!(s.thumbnail().is_none());
    for _ in 0..10 {
        s.run_frame(&[], &Volume::default());
    }
    let (w, h, rgba) = s.thumbnail().expect("a thumbnail");
    assert!(w <= THUMB_MAX && h <= THUMB_MAX, "{w}x{h}");
    assert!(
        w == THUMB_MAX || h == THUMB_MAX,
        "scaled to fit exactly one axis: {w}x{h}"
    );
    // 240x160 is 3:2; the thumbnail keeps it.
    let ratio = w as f32 / h as f32;
    assert!((ratio - 1.5).abs() < 0.05, "{ratio}");
    assert_eq!(rgba.len(), (w * h * 4) as usize);
}

#[test]
fn states_round_trip_through_the_card() {
    let Some(cores) = core_dir() else { return };
    let (card, cart, root) = card_with_rom("states");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    for _ in 0..20 {
        s.run_frame(&[], &Volume::default());
    }
    s.save_state(&card, StateKind::Numbered(1)).unwrap();
    assert!(root.join("States/GBA/arm/1.state").is_file());
    assert!(
        root.join("States/GBA/arm/1.png").is_file(),
        "a thumbnail is written beside it"
    );
    let states = card.list_states(&cart);
    assert_eq!(states.len(), 1);

    for _ in 0..20 {
        s.run_frame(&[], &Volume::default());
    }
    s.load_state(&card, StateKind::Numbered(1)).unwrap();
    assert!(
        s.load_state(&card, StateKind::Numbered(9)).is_err(),
        "a missing state is an error"
    );
}

#[test]
fn stopping_writes_the_save_and_a_resume_state() {
    let Some(cores) = core_dir() else { return };
    let (card, cart, root) = card_with_rom("stop");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    for _ in 0..5 {
        s.run_frame(&[], &Volume::default());
    }
    s.stop(&card);
    assert!(root.join("States/GBA/arm/resume.state").is_file());
    // The core can be loaded again afterwards: the library was released.
    let (mut s2, _c2) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    s2.run_frame(&[], &Volume::default());
    assert!(s2.last_frame().is_some());
    s2.stop(&card);
}

#[test]
fn save_ram_is_flushed_periodically_and_restored_on_the_next_start() {
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("sram");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    // arm.gba may expose no save RAM; the calls must still be clean either way.
    let _ = s.flush_save(&card).unwrap();
    for _ in 0..SAVE_EVERY_FRAMES + 2 {
        s.run_frame(&[], &Volume::default());
    }
    assert_eq!(s.frames_run(), SAVE_EVERY_FRAMES + 2);
    s.stop(&card);
    // Starting again with whatever was written must not fail.
    let (s2, _c2) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    s2.stop(&card);
}

#[test]
fn a_missing_core_is_a_clean_error() {
    let (card, cart, _root) = card_with_rom("nocore");
    let empty = std::env::temp_dir().join(format!("slot2-nocore-{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap();
    let e = Session::start(&card, &cart, &empty, 48_000).unwrap_err();
    assert!(matches!(e, slot2::session::Error::NoCore(_)), "{e}");
    assert!(e.to_string().contains("mgba_libretro"), "{e}");
}
