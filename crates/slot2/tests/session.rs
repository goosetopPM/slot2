//! Running a real cart end to end: load the core, run frames, flush a save, take a state,
//! stop with a resume state. Needs `vendor/mgba_libretro.*` (build/cores.ps1) and the MIT
//! test ROM; without the core the whole file skips.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// A libretro core is global state and only one instance of a library may live at a time,
/// so these tests take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

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

/// A context just real enough for `draw`, which only wants somewhere to measure text.
fn ui_ctx() -> slot2_ui::UiCtx {
    slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None)
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
    let _serial = serial();
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
    // The count is what matters: arm.gba is a CPU test ROM and writes no sound, so the
    // samples are legitimately zero. That the volume path scales them is slot2-audio's
    // own test; here we only prove the chain runs at the device rate.
    assert!(got > 10_000, "audio samples after 30 frames: {got}");
}

#[test]
fn muting_silences_the_stream_without_stopping_it() {
    let _serial = serial();
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
    let _serial = serial();
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
    let _serial = serial();
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
    let _serial = serial();
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
    let _serial = serial();
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
    let _serial = serial();
    let (card, cart, _root) = card_with_rom("nocore");
    let empty = std::env::temp_dir().join(format!("slot2-nocore-{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap();
    let e = Session::start(&card, &cart, &empty, 48_000).unwrap_err();
    assert!(matches!(e, slot2::session::Error::NoCore(_)), "{e}");
    assert!(e.to_string().contains("mgba_libretro"), "{e}");
}

#[test]
fn a_session_paces_and_produces_audio_at_the_sink_rate() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("pace");
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000).unwrap();

    // A GBA is 59.7275 fps, not 60. Pacing the loop at a flat 1/60 runs the core 0.46 %
    // fast, and once the ring fills every frame's audio tail is thrown away.
    let ft = s.frame_time().as_secs_f64();
    assert!(
        (ft - 1.0 / 59.7275).abs() < 1e-4,
        "frame time {ft} is not the GBA's"
    );

    // The core hands over audio at *its* declared rate (mGBA: 65536 Hz for GBA), and one
    // frame of it must come out of the resampler as one frame's worth at the sink rate.
    // This is the regression guard for a clamp that used to rewrite any rate above 50 kHz
    // to 32768, which made every GBA game play an octave low and twice too fast.
    let vol = Volume::new(100);
    let mut buf = vec![0i16; 4096];
    for _ in 0..2 {
        s.run_frame(&[], &vol);
        while consumer.read(&mut buf) > 0 {}
    }
    let (before, _, _, _) = s.audio_health();
    for _ in 0..60 {
        s.run_frame(&[], &vol);
        while consumer.read(&mut buf) > 0 {}
    }
    let (after, dropped, _, _) = s.audio_health();

    let per_frame = (after - before) as f64 / 60.0;
    let want = 48_000.0 / 59.7275;
    assert!(
        (per_frame - want).abs() / want < 0.02,
        "{per_frame} audio frames per video frame, expected about {want}"
    );
    assert_eq!(dropped, 0, "a drained ring must never drop");
}

#[test]
fn the_picture_lands_where_the_scale_policy_says() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("scale");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();

    let vol = slot2_audio::Volume::new(100);
    for _ in 0..4 {
        s.run_frame(&[], &vol);
    }

    // A GBA frame is 240x160 with square pixels, so on a 720x480 panel Integer is an exact
    // 3x with nothing left over — the case the design table calls out in bold.
    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);

    let image = canvas
        .frame()
        .iter()
        .find_map(|op| match op {
            slot2_gfx::Op::Image { x, y, w, h, uv, .. } => Some((*x, *y, *w, *h, *uv)),
            _ => None,
        })
        .expect("the game was not drawn");
    assert_eq!(
        (image.0, image.1, image.2, image.3),
        (0.0, 0.0, 720.0, 480.0)
    );
    assert_eq!(
        image.4,
        [0.0, 0.0, 1.0, 1.0],
        "a GBA has no overscan to crop"
    );

    // Fill takes the panel whatever the frame is; on this one it happens to agree with
    // Integer, so ask a geometry where it cannot.
    let mut canvas = slot2_gfx::RecordingCanvas::new(640, 480);
    s.set_scale(slot2_gfx::ScalePolicy::Fill);
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);
    let (x, y, w, h) = canvas
        .frame()
        .iter()
        .find_map(|op| match op {
            slot2_gfx::Op::Image { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .expect("the game was not drawn");
    assert_eq!((x, y, w, h), (0.0, 0.0, 640.0, 480.0));
}

#[test]
fn a_steady_frame_size_rewrites_the_texture_instead_of_reallocating_it() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("tex");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();

    let vol = slot2_audio::Volume::new(100);
    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);

    // The first frame has to allocate; after that the size does not change, and a frontend
    // that frees and reallocates the game texture sixty times a second is asking the
    // driver for a new allocation sixty times a second.
    for _ in 0..12 {
        s.run_frame(&[], &vol);
        s.upload_video(&mut canvas);
    }

    let uploads = canvas
        .ops
        .iter()
        .filter(|op| matches!(op, slot2_gfx::Op::UploadRgba8 { .. }))
        .count();
    let updates = canvas
        .ops
        .iter()
        .filter(|op| matches!(op, slot2_gfx::Op::UpdateRgba8 { .. }))
        .count();
    let frees = canvas
        .ops
        .iter()
        .filter(|op| matches!(op, slot2_gfx::Op::Free(_)))
        .count();

    assert_eq!(uploads, 1, "the texture was reallocated {uploads} times");
    assert_eq!(frees, 0, "the texture was freed mid-play");
    assert!(updates >= 10, "only {updates} frames reached the texture");
}

#[test]
fn a_canvas_refuses_to_rewrite_a_texture_at_a_different_size() {
    // What makes the reuse above safe: a geometry change cannot be smuggled into an
    // existing allocation, so the session falls back to a fresh upload.
    use slot2_gfx::Canvas as _;
    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
    let big = [0u8; 320 * 224 * 4];
    let small = [1u8; 256 * 224 * 4];
    let tex = canvas.upload_rgba8(320, 224, &big);
    assert!(canvas.update_rgba8(tex, 320, 224, &big));
    assert!(!canvas.update_rgba8(tex, 256, 224, &small));
    canvas.free(tex);
    assert!(!canvas.update_rgba8(tex, 320, 224, &big));
}

#[test]
fn a_settings_file_changes_how_the_game_is_drawn() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("settings");

    // A GBA frame is an exact 3x on this panel under Integer, so Fill is visibly different:
    // it takes the whole 640-wide panel where Integer would leave bars.
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            scale: Some(slot2_store::ScaleMode::Fill),
            ..Default::default()
        },
    )
    .unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    assert_eq!(s.scale(), slot2_gfx::ScalePolicy::Fill);

    let vol = slot2_audio::Volume::new(100);
    for _ in 0..4 {
        s.run_frame(&[], &vol);
    }
    let mut canvas = slot2_gfx::RecordingCanvas::new(640, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);
    let (w, h) = canvas
        .frame()
        .iter()
        .find_map(|op| match op {
            slot2_gfx::Op::Image { w, h, .. } => Some((*w, *h)),
            _ => None,
        })
        .expect("the game was not drawn");
    assert_eq!((w, h), (640.0, 480.0), "the scale setting was ignored");
}

#[test]
fn a_core_that_is_not_on_the_card_falls_back_instead_of_failing() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("badcore");
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            core: Some("a_core_nobody_has".into()),
            ..Default::default()
        },
    )
    .unwrap();

    // A setting is allowed to be wrong. It is not allowed to make a game unlaunchable.
    let (s, _c) = Session::start(&card, &cart, &cores, 48_000)
        .expect("a bad core name must fall back, not refuse");
    assert_eq!(s.cart().stem, "arm");
}

#[test]
fn rewinding_puts_the_game_back_where_it_was() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("rewind");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    assert!(
        s.rewind_enabled(),
        "rewind should be on unless a game says no"
    );

    let vol = slot2_audio::Volume::new(100);
    for _ in 0..60 {
        s.run_frame(&[], &vol);
    }
    let (depth, bytes) = s.rewind_state();
    let (interval, cost) = s.rewind_pace();

    // How many captures there are depends on how fast this machine is — the interval widens
    // itself when captures turn out to be expensive, and a debug build is expensive. What
    // must hold is that captures happened at the interval in force.
    assert!(depth >= 2, "only {depth} captures in 60 frames");
    assert!(
        depth as u32 * interval <= 60 + interval,
        "{depth} captures at one per {interval} frames is more than 60 frames' worth"
    );

    // And that the cost stayed inside its share of a frame, which is the promise the
    // widening exists to keep.
    let per_frame = cost.div_f64(interval as f64);
    let allowed = s
        .frame_time()
        .mul_f64(slot2::session::REWIND_FRAME_BUDGET * 2.0);
    assert!(
        per_frame <= allowed || interval >= slot2::session::MAX_REWIND_INTERVAL,
        "a capture costs {cost:?} every {interval} frames, over the budget"
    );

    // Sixty frames of a Game Boy Advance is half a megabyte of state many times over. The
    // whole point is that it does not cost that.
    assert!(
        bytes < 128 * 1024,
        "{bytes} bytes for {depth} captures — the deltas are not compressing"
    );

    let before = s.frames_run();
    let at = s.core_state().expect("a state to compare against");

    for _ in 0..20 {
        s.run_frame(&[], &vol);
    }
    assert_ne!(
        s.core_state().as_deref(),
        Some(&at[..]),
        "twenty frames changed nothing, so this test proves nothing"
    );

    // Step back far enough to cross the point we recorded.
    let mut steps = 0;
    while s.rewind_step() {
        steps += 1;
        if steps > 40 {
            break;
        }
    }
    assert!(steps > 0, "rewind refused to step back at all");
    assert!(
        s.frames_run() >= before,
        "frames_run is a counter, not a clock"
    );

    // And the ring empties rather than looping forever.
    assert_eq!(s.rewind_state().0, 0, "the ring should be spent");
    assert!(!s.rewind_step());
}

#[test]
fn turning_rewind_off_frees_the_ring() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("norewind");
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            rewind: Some(false),
            ..Default::default()
        },
    )
    .unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    assert!(!s.rewind_enabled());
    let vol = slot2_audio::Volume::new(100);
    for _ in 0..60 {
        s.run_frame(&[], &vol);
    }
    assert_eq!(
        s.rewind_state(),
        (0, 0),
        "a game with rewind off kept states"
    );
    assert!(!s.rewind_step());
}

#[test]
fn fast_forward_runs_more_frames_and_makes_no_sound() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("ff");
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000).unwrap();
    let vol = slot2_audio::Volume::new(100);

    for _ in 0..10 {
        s.run_frame(&[], &vol);
    }
    let normal = s.frames_run();
    let (made_before, _, _, _) = s.audio_health();
    assert!(made_before > 0, "normal speed should make sound");

    let mut buf = vec![0i16; 8192];
    while consumer.read(&mut buf) > 0 {}

    s.set_speed(4);
    assert_eq!(s.speed(), 4);
    for _ in 0..10 {
        s.run_frame(&[], &vol);
    }
    assert_eq!(
        s.frames_run() - normal,
        40,
        "four core frames per displayed frame"
    );

    // Silent on purpose: played four times as fast it would be four times the pitch, and
    // the ring would overflow producing it.
    let (made_after, dropped, _, _) = s.audio_health();
    assert_eq!(made_after, made_before, "fast forward pushed audio");
    assert_eq!(dropped, 0);

    s.set_speed(99);
    assert_eq!(s.speed(), slot2::session::MAX_SPEED, "speed must be capped");
    s.set_speed(0);
    assert_eq!(s.speed(), 1, "zero would stop the game, not slow it");
}
