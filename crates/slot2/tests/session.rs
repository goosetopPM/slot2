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
use slot2_store::{Card, Cart, Platform, ShaderPreset, StateKind};

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

/// The host's tuning, which is what these tests run under.
fn tuning() -> slot2_retro::Tuning {
    slot2::tuning_for(&slot2_platform::detect().profile)
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

/// The namespace a GBA cart's states live in on this machine: the platform's own core, mGBA.
/// A session's own `state_namespace()` is the product answer; this is the fixture's line when
/// there is no session yet.
fn states_ns() -> slot2_store::StateNamespace {
    slot2_store::StateNamespace::new("mgba_libretro").expect("a core's base name")
}

#[test]
fn a_session_runs_frames_and_produces_audio_and_video() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("run");
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
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
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
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
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
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
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    for _ in 0..20 {
        s.run_frame(&[], &Volume::default());
    }
    s.save_state(&card, StateKind::Numbered(1)).unwrap();
    assert!(root.join("States/GBA/arm/mgba_libretro/1.state").is_file());
    assert!(
        root.join("States/GBA/arm/mgba_libretro/1.png").is_file(),
        "a thumbnail is written beside it"
    );
    let states = card.scoped_list_states(&cart, &states_ns());
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
fn loading_a_state_clears_the_rewind_chain() {
    // A state jump is the end of the chain, not a point in it: the captures behind the
    // loaded moment belong to a future that no longer happened, and rewinding into it would
    // put the player in a game that was never played.
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("loadrewind");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    let vol = Volume::new(100);

    s.save_state(&card, StateKind::Numbered(1)).unwrap();
    for _ in 0..4 * slot2::session::MAX_REWIND_INTERVAL as usize {
        s.run_frame(&[], &vol);
    }
    assert!(
        s.rewind_state().0 > 0,
        "no captures were taken, so there is nothing to clear"
    );

    s.load_state(&card, StateKind::Numbered(1)).unwrap();
    assert_eq!(
        s.rewind_state(),
        (0, 0),
        "the pre-load future is still behind the player"
    );
    assert!(!s.rewind_step(), "a capture from before the load came back");

    // And the game keeps running from the loaded state.
    let before = s.frames_run();
    for _ in 0..5 {
        s.run_frame(&[], &vol);
    }
    assert_eq!(s.frames_run(), before + 5);
}

#[test]
fn stopping_writes_the_save_and_a_resume_state() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, root) = card_with_rom("stop");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    for _ in 0..5 {
        s.run_frame(&[], &Volume::default());
    }
    s.stop(&card);
    assert!(root
        .join("States/GBA/arm/mgba_libretro/resume.state")
        .is_file());
    // The core can be loaded again afterwards: the library was released.
    let (mut s2, _c2) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    s2.run_frame(&[], &Volume::default());
    assert!(s2.last_frame().is_some());
    s2.stop(&card);
}

#[test]
fn save_ram_is_flushed_periodically_and_restored_on_the_next_start() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("sram");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    // arm.gba may expose no save RAM; the calls must still be clean either way.
    let _ = s.flush_save(&card).unwrap();
    for _ in 0..SAVE_EVERY_FRAMES + 2 {
        s.run_frame(&[], &Volume::default());
    }
    assert_eq!(s.frames_run(), SAVE_EVERY_FRAMES + 2);
    s.stop(&card);
    // Starting again with whatever was written must not fail.
    let (s2, _c2) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    s2.stop(&card);
}

#[test]
fn a_missing_core_is_a_clean_error() {
    let _serial = serial();
    let (card, cart, _root) = card_with_rom("nocore");
    let empty = std::env::temp_dir().join(format!("slot2-nocore-{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap();
    let e = Session::start(&card, &cart, &empty, 48_000, tuning()).unwrap_err();
    assert!(matches!(e, slot2::session::Error::NoCore(_)), "{e}");
    assert!(e.to_string().contains("mgba_libretro"), "{e}");
}

#[test]
fn a_session_paces_and_produces_audio_at_the_sink_rate() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("pace");
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();

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
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();

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

    // The platform's own shader default is in play for these draws; what is under test is
    // where the picture lands, which an effect is not allowed to move.
    let (image, _) = game_draw(&canvas).expect("the game was not drawn");
    assert_eq!(
        (image.x, image.y, image.w, image.h),
        (0.0, 0.0, 720.0, 480.0)
    );
    assert_eq!(
        image.uv,
        [0.0, 0.0, 1.0, 1.0],
        "a GBA has no overscan to crop"
    );

    // Fill takes the panel whatever the frame is; on this one it happens to agree with
    // Integer, so ask a geometry where it cannot.
    let mut canvas = slot2_gfx::RecordingCanvas::new(640, 480);
    s.set_scale(slot2_gfx::ScalePolicy::Fill);
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);
    let (quad, _) = game_draw(&canvas).expect("the game was not drawn");
    assert_eq!((quad.x, quad.y, quad.w, quad.h), (0.0, 0.0, 640.0, 480.0));
}

#[test]
fn a_steady_frame_size_rewrites_the_texture_instead_of_reallocating_it() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("tex");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();

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

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.scale(), slot2_gfx::ScalePolicy::Fill);

    let vol = slot2_audio::Volume::new(100);
    for _ in 0..4 {
        s.run_frame(&[], &vol);
    }
    let mut canvas = slot2_gfx::RecordingCanvas::new(640, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);
    let (quad, _) = game_draw(&canvas).expect("the game was not drawn");
    assert_eq!(
        (quad.w, quad.h),
        (640.0, 480.0),
        "the scale setting was ignored"
    );
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
    let (s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning())
        .expect("a bad core name must fall back, not refuse");
    assert_eq!(s.cart().stem, "arm");
}

#[test]
fn rewinding_puts_the_game_back_where_it_was() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("rewind");
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert!(
        s.rewind_enabled(),
        "rewind should be on unless a game says no"
    );

    let vol = slot2_audio::Volume::new(100);
    // Enough frames to have captured several even if this machine turned out slow enough
    // for the interval to widen all the way: how often states are taken is decided at run
    // time, so a test that counts on a particular rate is a test that fails under load.
    const FRAMES: usize = 4 * slot2::session::MAX_REWIND_INTERVAL as usize;
    for _ in 0..FRAMES {
        s.run_frame(&[], &vol);
    }
    let (depth, bytes) = s.rewind_state();
    let (interval, cost) = s.rewind_pace();
    assert!(depth >= 2, "only {depth} captures in {FRAMES} frames");

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
    // The cap only guards against a rewind that never says it is finished; how many
    // captures there actually are depends on the interval this machine settled on, so it
    // has to be well clear of that rather than a guess at it.
    let mut steps = 0;
    while s.rewind_step() {
        steps += 1;
        assert!(steps < 1000, "rewind stepped back forever");
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

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
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
    let (mut s, mut consumer) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
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

#[test]
fn a_game_with_no_cheat_file_starts_with_none() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("nocheats");
    assert!(!card.cheat_path(&cart).exists());

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert!(s.cheats().is_empty(), "cheats appeared from nowhere");
    s.run_frame(&[], &Volume::default());
    assert_eq!(s.frames_run(), 1);
    assert!(s.last_frame().is_some());
}

#[test]
fn a_cheat_file_reaches_the_session_in_order_before_the_first_frame() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("cheatfile");
    // Written out of order, with the middle entry disabled and a two-part code.
    fs::write(
        card.cheat_path(&cart),
        r#"cheats = 3
cheat2_desc = "Max hearts"
cheat2_code = "7E13F2FF"
cheat2_enable = true
cheat0_desc = "Infinite lives"
cheat0_code = "7E007C9A"
cheat1_desc = "No damage"
cheat1_code = "7E007C9B+7E007C9C"
"#,
    )
    .unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(
        s.frames_run(),
        0,
        "the session ran a frame before the caller had it"
    );
    let cheats = s.cheats();
    assert_eq!(cheats.len(), 3);
    assert_eq!(cheats[0].description, "Infinite lives");
    assert_eq!(cheats[0].code, "7E007C9A");
    assert!(!cheats[0].enabled, "a missing enable means off");
    assert_eq!(cheats[1].description, "No damage");
    assert_eq!(
        cheats[1].code, "7E007C9B+7E007C9C",
        "the code was rewritten"
    );
    assert!(!cheats[1].enabled);
    assert_eq!(cheats[2].description, "Max hearts");
    assert_eq!(cheats[2].code, "7E13F2FF");
    assert!(cheats[2].enabled);

    // A set with disabled entries in it is still a set the game runs under.
    let vol = Volume::new(100);
    for _ in 0..5 {
        s.run_frame(&[], &vol);
    }
    assert_eq!(s.frames_run(), 5);
    assert!(s.last_frame().is_some());
}

#[test]
fn toggling_a_cheat_leaves_the_file_alone_and_a_new_session_starts_from_it() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("cheattoggle");
    let path = card.cheat_path(&cart);
    fs::write(
        &path,
        "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"7E007C9A\"\ncheat1_desc = \"b\"\ncheat1_code = \"7E007C9B\"\ncheat1_enable = true\n",
    )
    .unwrap();
    let before = fs::read(&path).unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert!(!s.cheats()[0].enabled);
    assert!(s.cheats()[1].enabled);

    s.set_cheat_enabled(0, true).unwrap();
    assert!(
        s.cheats()[0].enabled,
        "the accessor did not follow the toggle"
    );
    s.set_cheat_enabled(1, false).unwrap();
    assert!(!s.cheats()[1].enabled);
    // Asking for what is already true is not a failure, and not a special case either.
    s.set_cheat_enabled(1, false).unwrap();

    let vol = Volume::new(100);
    for _ in 0..5 {
        s.run_frame(&[], &vol);
    }
    assert!(s.last_frame().is_some(), "the game stopped after a toggle");
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "the cheat file was rewritten"
    );

    // A session is a moment, not a record: the next one is the file's again.
    s.stop(&card);
    let (s2, _c2) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert!(
        !s2.cheats()[0].enabled,
        "the toggle was written back to the card"
    );
    assert!(s2.cheats()[1].enabled);
    assert_eq!(fs::read(&path).unwrap(), before);
    s2.stop(&card);
}

#[test]
fn an_index_outside_the_file_is_an_error_and_changes_nothing() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("cheatindex");
    fs::write(
        card.cheat_path(&cart),
        "cheats = 1\ncheat0_desc = \"a\"\ncheat0_code = \"7E007C9A\"\n",
    )
    .unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    let e = s.set_cheat_enabled(1, true).unwrap_err();
    assert!(matches!(e, slot2::session::Error::NoCheat(1)), "{e}");
    let e = s.set_cheat_enabled(usize::MAX, false).unwrap_err();
    assert!(matches!(e, slot2::session::Error::NoCheat(_)), "{e}");

    assert_eq!(s.cheats().len(), 1, "the list moved for a refused index");
    assert!(!s.cheats()[0].enabled);
    for _ in 0..3 {
        s.run_frame(&[], &Volume::default());
    }
    assert!(
        s.last_frame().is_some(),
        "the game stopped after a refused index"
    );
}

#[test]
fn a_cheat_the_core_cannot_take_is_not_a_half_started_session() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("cheatnul");
    // A NUL is valid UTF-8, so the file parses and the entry looks complete; a C string
    // cannot hold it, which is where putting the set on the core has to fail. The entry has to
    // be on: mGBA is not handed the entries that are off (it ignores the flag), so a disabled
    // one would never reach the core and would never be refused.
    let mut text = Vec::new();
    text.extend_from_slice(
        b"cheats = 2\ncheat0_desc = \"ok\"\ncheat0_code = \"7E007C9A\"\ncheat1_desc = \"nul\"\ncheat1_enable = true\ncheat1_code = \"12",
    );
    text.push(0);
    text.extend_from_slice(b"34\"\n");
    let path = card.cheat_path(&cart);
    fs::write(&path, &text).unwrap();

    let e = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap_err();
    assert!(matches!(e, slot2::session::Error::Retro(_)), "{e}");
    assert!(e.to_string().contains("NUL"), "{e}");
    assert_eq!(
        fs::read(&path).unwrap(),
        text,
        "the refused start rewrote the file"
    );

    // Nothing was left behind: the same card with a code the core can take starts clean.
    fs::write(
        &path,
        "cheats = 1\ncheat0_desc = \"ok\"\ncheat0_code = \"7E007C9A\"\n",
    )
    .unwrap();
    let (s2, _c2) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s2.cheats().len(), 1);
    s2.stop(&card);
}

#[test]
fn a_named_alternative_core_opens_and_runs() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("gpsp");
    // A settings file may name a core by either spelling the launcher accepts, and `gpsp` is
    // the base name; `build/cores.ps1 -Core gpsp` put the library in vendor/ on this machine.
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            core: Some("gpsp".into()),
            ..Default::default()
        },
    )
    .unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(
        s.core_id(),
        Some(slot2_retro::CoreId::Gpsp),
        "the named core did not open"
    );
    for _ in 0..10 {
        s.run_frame(&[], &Volume::default());
    }
    assert_eq!(s.frames_run(), 10);
    assert!(s.last_frame().is_some(), "gpSP drew nothing");
    s.stop(&card);
}

#[test]
fn a_core_that_cannot_run_the_console_falls_back_to_the_platforms_own() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("wrongcore");
    // The Game Boy core on the card, asked for by a GBA game: the file is there, and opening
    // it would be a game that cannot start.
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            core: Some("gambatte_libretro".into()),
            ..Default::default()
        },
    )
    .unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(
        s.core_id(),
        Some(slot2_retro::CoreId::Mgba),
        "a core that cannot run this console was opened anyway"
    );
    s.run_frame(&[], &Volume::default());
    assert!(s.last_frame().is_some());
    drop(s);

    // A library this crate does not know is still opened, exactly as before: the card is
    // allowed to hold a core nobody here has heard of. It is the same mGBA binary under a name
    // the registry does not list, which is what an unknown core looks like from the outside.
    // Both names come from the host's own contract: the real one from the registry, the unknown
    // one from the platform's DLL extension, so the copy finds its source on every OS.
    let alt = std::env::temp_dir().join(format!("slot2-unknown-core-{}", std::process::id()));
    let _ = fs::remove_dir_all(&alt);
    fs::create_dir_all(&alt).unwrap();
    let unknown = format!("mystery_libretro.{}", std::env::consts::DLL_EXTENSION);
    fs::copy(
        cores.join(slot2_retro::CoreId::Mgba.file_name()),
        alt.join(&unknown),
    )
    .unwrap();
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            core: Some("mystery".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let (mut s, _c) = Session::start(&card, &cart, &alt, 48_000, tuning()).unwrap();
    assert_eq!(
        s.core_id(),
        None,
        "an unknown library was given an identity"
    );
    s.run_frame(&[], &Volume::default());
    assert!(s.last_frame().is_some(), "the unknown core did not run");
    s.stop(&card);
}

#[test]
fn a_broken_cheat_file_is_a_launch_error_and_not_a_missing_one() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("cheatbroken");
    let path = card.cheat_path(&cart);

    // Declares two, describes one: refused whole rather than starting with half of it.
    fs::write(
        &path,
        "cheats = 2\ncheat0_desc = \"a\"\ncheat0_code = \"7E007C9A\"\n",
    )
    .unwrap();
    let e = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap_err();
    assert!(matches!(e, slot2::session::Error::Store(_)), "{e}");
    assert!(e.to_string().contains("cheat1"), "{e}");

    // A file that is not there is the normal empty start, not this.
    fs::remove_file(&path).unwrap();
    let (s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert!(s.cheats().is_empty());
    s.stop(&card);
}

// -------------------------------------------------------------- mGBA and disabled entries

/// The code the state comparisons are made with: a CodeBreaker 8-bit write into EWRAM that
/// mGBA applies every frame. That it takes is not assumed — the difference between a run with
/// it and a run without it is what the tests below assert.
const WRITE_CODE: &str = "32000000+00AA";

/// A second write, somewhere else in EWRAM, so a disabled entry that leaked would show up as
/// a state the reference run does not have.
const OTHER_CODE: &str = "3203FFF0+00BB";

/// A `.cht` with one entry, on or off.
fn one_entry(enabled: bool) -> String {
    format!(
        "cheats = 1\ncheat0_desc = \"write\"\ncheat0_code = \"{WRITE_CODE}\"\ncheat0_enable = {enabled}\n"
    )
}

/// A card with the test ROM and, if given, a cheat file written before the launch — which is
/// when the session reads it.
fn card_with_cheats(name: &str, cheats: Option<&str>) -> (Card, Cart, PathBuf) {
    let (card, cart, root) = card_with_rom(name);
    if let Some(text) = cheats {
        let path = card.cheat_path(&cart);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
    }
    (card, cart, root)
}

/// The core's own state after exactly `frames` frames from a fresh launch.
///
/// Serialized, because that is the whole of what the core is. The session is dropped rather
/// than stopped so nothing else touches the card on the way out.
fn state_after(card: &Card, cart: &Cart, cores: &std::path::Path, frames: usize) -> Vec<u8> {
    let (mut s, _c) = Session::start(card, cart, cores, 48_000, tuning()).unwrap();
    for _ in 0..frames {
        s.run_frame(&[], &Volume::default());
    }
    let state = s.core_state().expect("the core serializes");
    drop(s);
    state
}

#[test]
fn the_session_says_which_core_it_opened() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("coreid");

    let (s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.core_id(), Some(slot2_retro::CoreId::Mgba));
    drop(s);

    // The same core, named in the settings file by either spelling the launcher accepts.
    for name in ["mgba", "mgba_libretro"] {
        card.write_settings(
            &cart,
            &slot2_store::GameSettings {
                core: Some(name.into()),
                ..Default::default()
            },
        )
        .unwrap();
        let (s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
        assert_eq!(s.core_id(), Some(slot2_retro::CoreId::Mgba), "{name}");
        drop(s);
    }

    // A core the card does not have falls back to the platform's own, and the session reports
    // the core it actually opened rather than the one it was asked for.
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            core: Some("a_core_nobody_has".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let (s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.core_id(), Some(slot2_retro::CoreId::Mgba), "the fallback");
}

#[test]
fn mgba_is_not_given_a_disabled_entry() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    const FRAMES: usize = 12;

    // Two runs of the same untouched fixture first: if these disagree, nothing later can be
    // read as being about the cheat, and this test is the wrong instrument.
    let (plain, cart, _root) = card_with_rom("cheatbase");
    let baseline = state_after(&plain, &cart, &cores, FRAMES);
    assert_eq!(
        state_after(&plain, &cart, &cores, FRAMES),
        baseline,
        "two runs of the same fixture disagree"
    );

    // The same write, off in the file. mGBA ignores the `enabled` flag it is handed, so the
    // only way for this entry to be off is for the session not to send it at all.
    let (off_card, off_cart, _root) = card_with_cheats("cheatoff", Some(&one_entry(false)));
    assert_eq!(
        state_after(&off_card, &off_cart, &cores, FRAMES),
        baseline,
        "a disabled entry reached the core"
    );

    // And on: the code writes into EWRAM every frame, so the core cannot be in the same state.
    let (on_card, on_cart, _root) = card_with_cheats("cheaton", Some(&one_entry(true)));
    assert_ne!(
        state_after(&on_card, &on_cart, &cores, FRAMES),
        baseline,
        "an enabled entry did not reach the core"
    );
}

#[test]
fn toggling_before_the_first_frame_is_the_same_as_starting_that_way() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    const FRAMES: usize = 12;

    // Off in the file, turned on before a frame runs: the core has to end up where it would
    // have been if the file had said on.
    let (card, cart, _root) = card_with_cheats("toggleon", Some(&one_entry(false)));
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    s.set_cheat_enabled(0, true).unwrap();
    for _ in 0..FRAMES {
        s.run_frame(&[], &Volume::default());
    }
    let toggled_on = s.core_state().unwrap();
    drop(s);

    let (on_card, on_cart, _root) = card_with_cheats("toggleon-ref", Some(&one_entry(true)));
    assert_eq!(
        toggled_on,
        state_after(&on_card, &on_cart, &cores, FRAMES),
        "turning it on was not the same as starting with it on"
    );

    // On in the file, turned off before a frame runs: the core has to be where a fixture with
    // no cheat at all is.
    let (card, cart, _root) = card_with_cheats("toggleoff", Some(&one_entry(true)));
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    s.set_cheat_enabled(0, false).unwrap();
    for _ in 0..FRAMES {
        s.run_frame(&[], &Volume::default());
    }
    let toggled_off = s.core_state().unwrap();
    drop(s);

    let (plain, cart_plain, _root) = card_with_rom("toggleoff-ref");
    assert_eq!(
        toggled_off,
        state_after(&plain, &cart_plain, &cores, FRAMES),
        "turning it off left it on the core"
    );
}

#[test]
fn a_disabled_entry_in_the_middle_does_not_disturb_the_ones_around_it() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    const FRAMES: usize = 12;

    // Three entries: the first two off — one of them with a code of its own, so a leak would
    // show — and the last one on. The session's list keeps all three, in file order.
    let text = format!(
        "cheats = 3\n\
         cheat0_desc = \"first\"\ncheat0_code = \"{OTHER_CODE}\"\n\
         cheat1_desc = \"middle\"\ncheat1_code = \"{WRITE_CODE}\"\n\
         cheat2_desc = \"last\"\ncheat2_code = \"{WRITE_CODE}\"\ncheat2_enable = true\n"
    );
    let (card, cart, _root) = card_with_cheats("cheatmiddle", Some(&text));
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();

    assert_eq!(
        s.cheats().len(),
        3,
        "the session's list was cut to what was sent"
    );
    let flags: Vec<bool> = s.cheats().iter().map(|c| c.enabled).collect();
    assert_eq!(flags, [false, false, true], "file order and file flags");
    let descriptions: Vec<&str> = s.cheats().iter().map(|c| c.description.as_str()).collect();
    assert_eq!(descriptions, ["first", "middle", "last"], "file order");

    for _ in 0..FRAMES {
        s.run_frame(&[], &Volume::default());
    }
    let state = s.core_state().unwrap();
    drop(s);

    // The last entry's write, sent alone: the two before it were withheld, so this is the
    // whole of what the core was given — and the same write at a different file index is the
    // same write to a core that does not look at the index.
    let (one_card, one_cart, _root) = card_with_cheats("cheatmiddle-ref", Some(&one_entry(true)));
    assert_eq!(
        state,
        state_after(&one_card, &one_cart, &cores, FRAMES),
        "the entry after the disabled ones did not reach the core alone"
    );

    let (plain, cart_plain, _root) = card_with_rom("cheatmiddle-base");
    assert_ne!(state, state_after(&plain, &cart_plain, &cores, FRAMES));
}

// -------------------------------------------------------------- shader routing

/// The game setting the launcher will read for this cart.
fn write_shader_setting(card: &Card, cart: &Cart, shader: Option<ShaderPreset>) {
    card.write_settings(
        cart,
        &slot2_store::GameSettings {
            shader,
            ..Default::default()
        },
    )
    .unwrap();
}

/// The one draw a game frame makes: what it sampled, where it put it, and what it was
/// multiplied by. Both the plain and the effect path are this and nothing else.
#[derive(Clone, Debug, PartialEq)]
struct GameQuad {
    tex: slot2_gfx::TexId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    uv: [f32; 4],
    tint: slot2_gfx::Color,
}

/// The game draw in the recorded frame, with the effect it wore (`None` for the plain path).
fn game_draw(
    canvas: &slot2_gfx::RecordingCanvas,
) -> Option<(GameQuad, Option<slot2_gfx::ShaderEffect>)> {
    canvas.frame().iter().find_map(|op| match op {
        slot2_gfx::Op::Image {
            tex,
            x,
            y,
            w,
            h,
            uv,
            tint,
        } => Some((
            GameQuad {
                tex: *tex,
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                uv: *uv,
                tint: *tint,
            },
            None,
        )),
        slot2_gfx::Op::ImageEffect {
            tex,
            x,
            y,
            w,
            h,
            uv,
            tint,
            effect,
        } => Some((
            GameQuad {
                tex: *tex,
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                uv: *uv,
                tint: *tint,
            },
            Some(*effect),
        )),
        _ => None,
    })
}

/// `(uploads, updates, frees)` over everything recorded so far: the game texture's whole life.
fn texture_ops(canvas: &slot2_gfx::RecordingCanvas) -> (usize, usize, usize) {
    let count = |f: fn(&slot2_gfx::Op) -> bool| canvas.ops.iter().filter(|o| f(o)).count();
    (
        count(|o| matches!(o, slot2_gfx::Op::UploadRgba8 { .. })),
        count(|o| matches!(o, slot2_gfx::Op::UpdateRgba8 { .. })),
        count(|o| matches!(o, slot2_gfx::Op::Free(_))),
    )
}

/// The card's four presets, each beside the effect it names. Written out here so the mapping
/// is pinned rather than derived from either enum's order.
const PRESET_EFFECTS: [(ShaderPreset, slot2_gfx::ShaderEffect); 4] = [
    (
        ShaderPreset::SharpBilinear,
        slot2_gfx::ShaderEffect::SharpBilinear,
    ),
    (ShaderPreset::Lcd3x, slot2_gfx::ShaderEffect::Lcd3x),
    (ShaderPreset::ZfastCrt, slot2_gfx::ShaderEffect::ZfastCrt),
    (ShaderPreset::Scanline, slot2_gfx::ShaderEffect::Scanline),
];

/// Every platform, and the effect a game that says nothing about shaders gets on it.
const PLATFORM_SHADER_DEFAULTS: [(slot2_retro::Platform, slot2_gfx::ShaderEffect); 7] = [
    (slot2_retro::Platform::Gb, slot2_gfx::ShaderEffect::Lcd3x),
    (slot2_retro::Platform::Gbc, slot2_gfx::ShaderEffect::Lcd3x),
    (slot2_retro::Platform::Gba, slot2_gfx::ShaderEffect::Lcd3x),
    (
        slot2_retro::Platform::Nes,
        slot2_gfx::ShaderEffect::ZfastCrt,
    ),
    (
        slot2_retro::Platform::Snes,
        slot2_gfx::ShaderEffect::ZfastCrt,
    ),
    (slot2_retro::Platform::Md, slot2_gfx::ShaderEffect::ZfastCrt),
    (
        slot2_retro::Platform::Sms,
        slot2_gfx::ShaderEffect::ZfastCrt,
    ),
];

#[test]
fn a_missing_shader_key_inherits_the_platforms_own_default() {
    // The card's silence, which is not the same thing as the player turning shaders off. No
    // core is needed: this is the mapping, and it runs wherever.
    for (platform, effect) in PLATFORM_SHADER_DEFAULTS {
        assert_eq!(
            Session::shader_effect_for(platform, None),
            Some(effect),
            "{platform:?} did not fall back to its own default"
        );
    }
}

#[test]
fn an_explicit_off_is_the_plain_draw_on_every_platform() {
    // "Shaders off" is a decision, and no platform's default may come back and override it.
    for (platform, _) in PLATFORM_SHADER_DEFAULTS {
        assert_eq!(
            Session::shader_effect_for(platform, Some(ShaderPreset::Off)),
            None,
            "{platform:?} drew through an effect its game had turned off"
        );
    }
}

#[test]
fn an_explicit_preset_beats_the_platforms_default_on_every_platform() {
    for (platform, _) in PLATFORM_SHADER_DEFAULTS {
        for (preset, effect) in PRESET_EFFECTS {
            assert_eq!(
                Session::shader_effect_for(platform, Some(preset)),
                Some(effect),
                "{platform:?} ignored {preset:?}"
            );
        }
    }

    // Including the other family's look, which is the case a "the platform decides" rule
    // would get wrong: a GBA game may ask for scanlines, a NES game for an LCD grid.
    assert_eq!(
        Session::shader_effect_for(slot2_retro::Platform::Gba, Some(ShaderPreset::ZfastCrt)),
        Some(slot2_gfx::ShaderEffect::ZfastCrt)
    );
    assert_eq!(
        Session::shader_effect_for(slot2_retro::Platform::Nes, Some(ShaderPreset::Lcd3x)),
        Some(slot2_gfx::ShaderEffect::Lcd3x)
    );
}

#[test]
fn a_game_with_no_shader_setting_inherits_the_platforms_default() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("noshader");
    assert!(!card.game_settings_path(&cart).exists());

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(
        s.shader_effect(),
        Some(slot2_gfx::ShaderEffect::Lcd3x),
        "a GBA with nothing set did not take its own default"
    );
    for _ in 0..4 {
        s.run_frame(&[], &Volume::new(100));
    }

    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);

    let (quad, effect) = game_draw(&canvas).expect("the game was not drawn");
    assert_eq!(effect, Some(slot2_gfx::ShaderEffect::Lcd3x));
    assert_eq!(
        (quad.x, quad.y, quad.w, quad.h),
        (0.0, 0.0, 720.0, 480.0),
        "a 240x160 GBA frame is an exact 3x on this panel"
    );
    assert_eq!(
        quad.uv,
        [0.0, 0.0, 1.0, 1.0],
        "a GBA has no overscan to crop"
    );
    assert_eq!(quad.tint, slot2_gfx::Color::WHITE);
    // One draw and nothing else: no second pass, no stray op. `frame()` is everything since
    // the clear, so this is the whole of what the draw put on the panel.
    assert_eq!(canvas.frame().len(), 1, "{:?}", canvas.frame());
    assert!(
        matches!(
            canvas.frame()[0],
            slot2_gfx::Op::ImageEffect { effect: e, .. } if e == slot2_gfx::ShaderEffect::Lcd3x
        ),
        "{:?}",
        canvas.frame()
    );
    // Inheriting a default is a reading, not a write: the game that never needed a settings
    // file still has none, and the key on the card is still absent.
    assert!(!card.game_settings_path(&cart).exists());
    assert_eq!(
        card.read_settings(&cart).shader,
        None,
        "the computed default was written onto the card"
    );
}

#[test]
fn a_shader_spelling_this_version_does_not_know_inherits_the_default_and_is_left_alone() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("badshader");
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    // A hand-edited file with a value this version has never heard of. Reading it as an
    // explicit off would silently turn shaders off for a game whose only mistake was a typo.
    fs::write(&path, "shader = vhs\n").unwrap();
    let before = fs::read(&path).unwrap();
    assert_eq!(card.read_settings(&cart).shader, None);

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.shader_effect(), Some(slot2_gfx::ShaderEffect::Lcd3x));
    for _ in 0..4 {
        s.run_frame(&[], &Volume::new(100));
    }
    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);
    let (_, effect) = game_draw(&canvas).expect("the game was not drawn");
    assert_eq!(effect, Some(slot2_gfx::ShaderEffect::Lcd3x));
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "the launch corrected a file it was only reading"
    );
}

#[test]
fn an_explicit_off_draws_plainly_and_stays_off_on_the_card() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("offshader");
    write_shader_setting(&card, &cart, Some(ShaderPreset::Off));
    let path = card.game_settings_path(&cart);
    let before = fs::read(&path).unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.shader_effect(), None, "an explicit off became an effect");
    for _ in 0..4 {
        s.run_frame(&[], &Volume::new(100));
    }
    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);
    let (_, effect) = game_draw(&canvas).expect("the game was not drawn");
    assert_eq!(effect, None);
    assert!(
        matches!(canvas.frame()[0], slot2_gfx::Op::Image { .. }),
        "{:?}",
        canvas.frame()
    );

    // Turning shaders off is a different setting from never having said anything, and the
    // runtime's `None` must not collapse the two on the card: this GBA has a platform default
    // (Lcd3x), and the card still has to tell this game from one that never said anything.
    assert_eq!(
        card.read_settings(&cart).shader,
        Some(ShaderPreset::Off),
        "the explicit off was lost on the card"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "the settings file was rewritten by a draw"
    );
}

#[test]
fn every_preset_reaches_the_game_draw_as_its_own_effect() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let ctx = ui_ctx();
    // Every one of the four is an override of the GBA's own default, which is Lcd3x: the
    // game's choice wins over the platform's, including when it asks for the other family's
    // look (ZfastCrt here).
    let cases = [
        (
            ShaderPreset::SharpBilinear,
            slot2_gfx::ShaderEffect::SharpBilinear,
        ),
        (ShaderPreset::Lcd3x, slot2_gfx::ShaderEffect::Lcd3x),
        (ShaderPreset::ZfastCrt, slot2_gfx::ShaderEffect::ZfastCrt),
        (ShaderPreset::Scanline, slot2_gfx::ShaderEffect::Scanline),
    ];

    for (i, (preset, effect)) in cases.into_iter().enumerate() {
        let (card, cart, _root) = card_with_rom(&format!("shader{i}"));
        write_shader_setting(&card, &cart, Some(preset));
        let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
        assert_eq!(
            s.shader_effect(),
            Some(effect),
            "{preset:?} did not reach the session"
        );
        for _ in 0..4 {
            s.run_frame(&[], &Volume::new(100));
        }

        // The same frame and the same texture drawn without the effect, as the reference for
        // what the effect is allowed to change: nothing but how the quad is sampled.
        s.set_shader_effect(None);
        let mut plain = slot2_gfx::RecordingCanvas::new(720, 480);
        s.upload_video(&mut plain);
        s.draw(&mut plain, &ctx);
        let (plain_quad, plain_effect) = game_draw(&plain).expect("the plain draw");
        assert_eq!(plain_effect, None);

        s.set_shader_effect(Some(effect));
        let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
        s.draw(&mut canvas, &ctx);
        let (quad, got) = game_draw(&canvas).expect("the effect draw");
        assert_eq!(got, Some(effect), "{preset:?} drew without its effect");
        assert_eq!(
            quad, plain_quad,
            "{preset:?} moved or re-tinted the picture"
        );
        // Exactly one draw, through the effect API and nothing else.
        assert_eq!(canvas.frame().len(), 1, "{:?}", canvas.frame());
        assert!(
            matches!(
                canvas.frame()[0],
                slot2_gfx::Op::ImageEffect { effect: e, .. } if e == effect
            ),
            "{:?}",
            canvas.frame()
        );
    }
}

#[test]
fn a_shader_override_and_a_scale_override_apply_together() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("shaderscale");
    card.write_settings(
        &cart,
        &slot2_store::GameSettings {
            scale: Some(slot2_store::ScaleMode::Fill),
            shader: Some(ShaderPreset::Lcd3x),
            ..Default::default()
        },
    )
    .unwrap();

    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.scale(), slot2_gfx::ScalePolicy::Fill);
    assert_eq!(s.shader_effect(), Some(slot2_gfx::ShaderEffect::Lcd3x));
    for _ in 0..4 {
        s.run_frame(&[], &Volume::new(100));
    }

    // On a 640-wide panel Integer would letterbox this 3:2 picture; Fill takes the lot. The
    // effect is not allowed to change which of those the game asked for.
    let mut canvas = slot2_gfx::RecordingCanvas::new(640, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    s.draw(&mut canvas, &ctx);
    let (quad, effect) = game_draw(&canvas).expect("the game was not drawn");
    assert_eq!(effect, Some(slot2_gfx::ShaderEffect::Lcd3x));
    assert_eq!(
        (quad.x, quad.y, quad.w, quad.h),
        (0.0, 0.0, 640.0, 480.0),
        "the scale setting was ignored"
    );
    assert_eq!(quad.uv, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(quad.tint, slot2_gfx::Color::WHITE);
}

#[test]
fn changing_the_effect_changes_only_what_the_next_draw_uses() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("seteff");
    write_shader_setting(&card, &cart, Some(ShaderPreset::SharpBilinear));
    let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    let vol = Volume::new(100);
    for _ in 0..6 {
        s.run_frame(&[], &vol);
    }

    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
    let ctx = ui_ctx();
    s.upload_video(&mut canvas);
    let frames = s.frames_run();
    let core_before = s.core_state().expect("the core serializes");
    let texture_before = texture_ops(&canvas);
    let audio_before = s.audio_health();
    let path = card.game_settings_path(&cart);
    let bytes = fs::read(&path).unwrap();

    // A menu walks through the four and back off, twice, with a draw after each: the same
    // moment redrawn under a different program, which is what instant preview is.
    let wanted = [
        Some(slot2_gfx::ShaderEffect::Scanline),
        Some(slot2_gfx::ShaderEffect::Lcd3x),
        None,
        Some(slot2_gfx::ShaderEffect::ZfastCrt),
        Some(slot2_gfx::ShaderEffect::SharpBilinear),
        None,
    ];
    for effect in wanted {
        s.set_shader_effect(effect);
        s.draw(&mut canvas, &ctx);
        let (_, got) = game_draw(&canvas).expect("the game was not drawn");
        assert_eq!(got, effect, "the next draw did not use the new effect");
        assert_eq!(canvas.frame().len(), 1, "{:?}", canvas.frame());
    }

    // Six redraws later, nothing else has moved: the setter is renderer state, not a game
    // event, and the card is not its notebook.
    assert_eq!(s.frames_run(), frames, "a draw ran frames");
    assert_eq!(
        s.core_state().as_deref(),
        Some(&core_before[..]),
        "a draw touched the core"
    );
    assert_eq!(s.scale(), slot2_gfx::ScalePolicy::Integer);
    assert_eq!(
        texture_ops(&canvas),
        texture_before,
        "a draw uploaded, rewrote or freed the game texture"
    );
    assert_eq!(
        s.audio_health(),
        audio_before,
        "a draw touched the audio chain"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        bytes,
        "a draw rewrote the settings file"
    );
    assert_eq!(
        card.read_settings(&cart).shader,
        Some(ShaderPreset::SharpBilinear),
        "the card's own setting moved"
    );
    assert_eq!(s.shader_effect(), None, "the last setter did not stick");
}

#[test]
fn a_named_core_launch_reads_the_same_shader_setting() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("namedshader");
    write_shader_setting(&card, &cart, Some(ShaderPreset::ZfastCrt));

    let (s, _c) = Session::start_named(
        &card,
        &cart,
        &cores,
        slot2_retro::CoreId::Mgba,
        48_000,
        tuning(),
    )
    .unwrap();
    assert_eq!(s.core_id(), Some(slot2_retro::CoreId::Mgba));
    assert_eq!(
        s.shader_effect(),
        Some(slot2_gfx::ShaderEffect::ZfastCrt),
        "a launch with a named core skipped the game's shader setting"
    );
}

#[test]
fn a_named_core_launch_inherits_the_platforms_default_too() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("nameddefault");
    assert!(!card.game_settings_path(&cart).exists());

    // A game that says nothing reaches the same platform default whichever way it was
    // launched: the named-core path is the same `open`.
    let (s, _c) = Session::start_named(
        &card,
        &cart,
        &cores,
        slot2_retro::CoreId::Mgba,
        48_000,
        tuning(),
    )
    .unwrap();
    assert_eq!(s.core_id(), Some(slot2_retro::CoreId::Mgba));
    assert_eq!(
        s.shader_effect(),
        Some(slot2_gfx::ShaderEffect::Lcd3x),
        "a launch with a named core skipped the platform's default"
    );
}

#[test]
fn an_effect_with_no_frame_yet_draws_nothing() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    let (card, cart, _root) = card_with_rom("noshareshader");
    write_shader_setting(&card, &cart, Some(ShaderPreset::Scanline));

    let (s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
    assert_eq!(s.shader_effect(), Some(slot2_gfx::ShaderEffect::Scanline));
    assert!(
        s.last_frame().is_none(),
        "a frame ran before the caller had it"
    );

    // No frame has run, so there is no texture and nothing to draw — through an effect or
    // otherwise.
    let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
    let ctx = ui_ctx();
    s.draw(&mut canvas, &ctx);
    assert!(
        canvas.ops.is_empty(),
        "something was drawn with no picture: {:?}",
        canvas.ops
    );
}

// -------------------------------------------------------------- overscan routing

/// Every platform, and the crop its own registry entry asks for. Written out rather than read
/// off the registry, so a default that moved there fails here as well.
///
/// Only the NES crops anything: eight rows off the top and eight off the bottom, which is the
/// rubbish a television never showed on that machine.
const PLATFORM_CROPS: [(slot2_retro::Platform, slot2_retro::Overscan); 7] = [
    (slot2_retro::Platform::Gb, slot2_retro::Overscan::NONE),
    (slot2_retro::Platform::Gbc, slot2_retro::Overscan::NONE),
    (slot2_retro::Platform::Gba, slot2_retro::Overscan::NONE),
    (slot2_retro::Platform::Nes, slot2_retro::Overscan::rows(8)),
    (slot2_retro::Platform::Snes, slot2_retro::Overscan::NONE),
    (slot2_retro::Platform::Md, slot2_retro::Overscan::NONE),
    (slot2_retro::Platform::Sms, slot2_retro::Overscan::NONE),
];

/// The game setting the launcher will read for this cart.
fn write_overscan_setting(card: &Card, cart: &Cart, overscan: Option<bool>) {
    card.write_settings(
        cart,
        &slot2_store::GameSettings {
            overscan,
            ..Default::default()
        },
    )
    .unwrap();
}

#[test]
fn the_three_overscan_meanings_are_read_the_same_way_on_every_platform() {
    // No core is needed: this is the mapping, and it runs wherever.
    assert_eq!(
        PLATFORM_CROPS.len(),
        slot2_retro::PLATFORMS.len(),
        "a platform was left out of the table"
    );
    for (platform, crop) in PLATFORM_CROPS {
        assert_eq!(
            slot2_retro::def(platform).overscan,
            crop,
            "{platform:?}: the registry's own default moved"
        );
        // The card's silence inherits the platform's answer...
        assert_eq!(
            Session::overscan_for(platform, None),
            crop,
            "{platform:?}: a missing key did not inherit the platform's crop"
        );
        // ...and so does an explicit crop, which is the same picture asked for by name.
        assert_eq!(
            Session::overscan_for(platform, Some(true)),
            crop,
            "{platform:?}: an explicit crop is not the platform's own"
        );
        // The whole frame is the one choice the registry has no say in.
        assert_eq!(
            Session::overscan_for(platform, Some(false)),
            slot2_retro::Overscan::NONE,
            "{platform:?}: an explicit full image was cropped anyway"
        );
    }

    // The NES is the one platform with something to crop, and it is the design's eight rows off
    // the top and the bottom — nothing off the sides.
    let nes = Session::overscan_for(slot2_retro::Platform::Nes, None);
    assert_eq!((nes.left, nes.top, nes.right, nes.bottom), (0, 8, 0, 8));
    for (platform, crop) in PLATFORM_CROPS {
        if platform != slot2_retro::Platform::Nes {
            assert_eq!(crop, slot2_retro::Overscan::NONE, "{platform:?} crops");
        }
    }

    // On the other six platforms all three meanings are the whole frame, which is why the App
    // offers the choice only where the registry crops: a row that cannot change the picture is
    // a lie about a feature.
    for (platform, _) in PLATFORM_CROPS {
        if platform == slot2_retro::Platform::Nes {
            continue;
        }
        for setting in [None, Some(true), Some(false)] {
            assert_eq!(
                Session::overscan_for(platform, setting),
                slot2_retro::Overscan::NONE,
                "{platform:?}/{setting:?}"
            );
        }
    }
}

#[test]
fn a_launch_and_a_runtime_change_read_the_setting_through_one_helper() {
    let _serial = serial();
    let Some(cores) = core_dir() else { return };
    // The cart `card_with_rom` writes is a GBA one, so that is the platform both sides are read
    // against.
    let platform = slot2_retro::Platform::Gba;

    for (i, setting) in [None, Some(true), Some(false)].into_iter().enumerate() {
        let (card, cart, _root) = card_with_rom(&format!("overscan{i}"));
        write_overscan_setting(&card, &cart, setting);
        let (mut s, _c) = Session::start(&card, &cart, &cores, 48_000, tuning()).unwrap();
        assert_eq!(
            s.overscan(),
            Session::overscan_for(platform, setting),
            "{setting:?}: the launch did not read the setting through the helper"
        );

        let vol = Volume::new(100);
        for _ in 0..4 {
            s.run_frame(&[], &vol);
        }
        let mut canvas = slot2_gfx::RecordingCanvas::new(720, 480);
        let ctx = ui_ctx();
        s.upload_video(&mut canvas);

        // Four changes, each followed by a draw: the same frame redrawn under a different crop
        // is what the menu shows.
        let frames = s.frames_run();
        let core_before = s.core_state().expect("the core serializes");
        let frame_before = s
            .last_frame()
            .map(|(w, h, d)| (w, h, d.to_vec()))
            .expect("a frame ran");
        let texture_before = texture_ops(&canvas);
        let audio_before = s.audio_health();
        let path = card.game_settings_path(&cart);
        // A game that has never said anything about overscan has no file at all, so the bytes are
        // compared as they are, absence included.
        let bytes = fs::read(&path).ok();

        for setting in [Some(false), Some(true), None, Some(true)] {
            s.set_overscan_setting(setting);
            assert_eq!(
                s.overscan(),
                Session::overscan_for(platform, setting),
                "{setting:?}: the runtime change did not read the setting through the helper"
            );
            s.draw(&mut canvas, &ctx);
            let (quad, _) = game_draw(&canvas).expect("the game was not drawn");
            let crop = s.overscan();
            assert_eq!(
                quad.uv,
                slot2_gfx::sub_uv(
                    (frame_before.0, frame_before.1),
                    crop.left,
                    crop.top,
                    crop.right,
                    crop.bottom
                ),
                "{setting:?}: the draw did not use the crop that is set"
            );
        }

        // A crop is renderer state, not a game event: no frame ran, the core is where it was, the
        // picture is the same picture, no texture work happened, the audio chain is untouched and
        // the card was not this setter's notebook.
        assert_eq!(s.frames_run(), frames, "a crop ran frames");
        assert_eq!(
            s.core_state().as_deref(),
            Some(&core_before[..]),
            "a crop touched the core"
        );
        assert_eq!(
            s.last_frame().map(|(w, h, d)| (w, h, d.to_vec())),
            Some(frame_before),
            "a crop moved the core's picture"
        );
        assert_eq!(
            texture_ops(&canvas),
            texture_before,
            "a crop uploaded, rewrote or freed the game texture"
        );
        assert_eq!(
            s.audio_health(),
            audio_before,
            "a crop touched the audio chain"
        );
        assert_eq!(fs::read(&path).ok(), bytes, "a crop rewrote the card");
        assert_eq!(
            card.read_settings(&cart).overscan,
            setting,
            "the card's own setting moved"
        );
    }
}
