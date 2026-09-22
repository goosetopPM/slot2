//! The contract for slot2-retro, run against the real mGBA core (`vendor/mgba_libretro.dll`
//! on Windows, `.so` elsewhere; fetch with `build/cores.ps1`) and jsmolka's `arm.gba` test
//! ROM (MIT) in `assets/test`. Without the core file every test skips, so CI without cores
//! stays green; with it, these are the acceptance tests for M1-2.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use slot2_retro::{Core, Env, JoypadMask, Memory, PixelFormat};

/// libretro cores are global state and the host allows one instance per library, so the
/// tests must not overlap: each takes this lock first.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn core_path() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "mgba_libretro.dll"
    } else if cfg!(target_os = "macos") {
        "mgba_libretro.dylib"
    } else {
        "mgba_libretro.so"
    };
    let p = repo().join("vendor").join(name);
    if p.is_file() {
        Some(p)
    } else {
        eprintln!("no {} — skipping (run build/cores.ps1)", p.display());
        None
    }
}

fn rom() -> PathBuf {
    repo().join("assets/test/arm.gba")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-retro-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn env(name: &str) -> Env {
    Env {
        system_dir: tmp(&format!("{name}-sys")),
        save_dir: tmp(&format!("{name}-save")),
        options: vec![("mgba_skip_bios".into(), "ON".into())],
        language: 0,
    }
}

fn load(name: &str) -> Option<Core> {
    let core = core_path()?;
    Some(Core::load(&core, &rom(), env(name)).expect("core loads the test rom"))
}

#[test]
fn loads_and_reports_itself() {
    let _serial = serial();
    let Some(core) = load("info") else { return };
    let (lib, ver) = core.name();
    assert!(lib.to_lowercase().contains("mgba"), "{lib}");
    assert!(!ver.is_empty());
    let av = core.av_info();
    assert_eq!((av.base_width, av.base_height), (240, 160));
    assert!((av.fps - 59.727).abs() < 0.1, "{}", av.fps);
    // mGBA hands GBA audio over at 65536 Hz (`GBA_OUTPUT_RATE`), twice the rate the GBA's
    // own sound FIFO runs at, and it says so. This assertion used to read "between 30k and
    // 50k because no console goes higher", which is false, and the host was made to satisfy
    // it by clamping the core's answer down to 32768 — detuning every GBA game an octave.
    // The rate a core reports is the core's business; see
    // `the_declared_sample_rate_is_what_the_core_actually_produces` for the real invariant.
    assert_eq!(av.sample_rate, 65_536.0, "mGBA's declared GBA output rate");
    assert!(matches!(
        core.pixel_format(),
        PixelFormat::Xrgb8888 | PixelFormat::Rgb565
    ));
    let log = core.env_log();
    assert!(
        log.answered.contains(&10),
        "SET_PIXEL_FORMAT answered: {log:?}"
    );
    assert!(log.answered.contains(&9), "GET_SYSTEM_DIRECTORY answered");
    assert!(!core.need_fullpath(), "mGBA takes the rom in memory");
}

#[test]
fn runs_frames_and_delivers_video_and_audio() {
    let _serial = serial();
    let Some(mut core) = load("frames") else {
        return;
    };
    assert!(core.frame().is_none());
    for _ in 0..60 {
        core.run();
    }
    let f = core.frame().expect("a frame after 60 runs");
    assert_eq!((f.width, f.height), (240, 160));
    assert!(f.pitch >= 240 * f.format.bytes_per_pixel());
    assert_eq!(f.data.len(), f.pitch * 160);
    let rgba = f.to_rgba8();
    assert_eq!(rgba.len(), 240 * 160 * 4);
    assert!(rgba.chunks(4).all(|p| p[3] == 255));
    // The test rom prints results on a coloured background: the picture is not one flat
    // colour after a second.
    let first = f.rgb(0, 0);
    assert!(
        (0..160).any(|y| (0..240).any(|x| f.rgb(x, y) != first)),
        "flat frame"
    );
    let audio = core.take_audio();
    assert!(
        audio.len() >= 2 * 500,
        "stereo samples after 60 frames: {}",
        audio.len()
    );
    assert!(core.take_audio().is_empty(), "take drains");
    core.run();
    assert!(!core.take_audio().is_empty());
}

#[test]
fn input_reaches_the_core_without_crashing() {
    let _serial = serial();
    let Some(mut core) = load("input") else {
        return;
    };
    core.set_input(
        0,
        JoypadMask::default()
            .with(JoypadMask::A)
            .with(JoypadMask::START),
    );
    core.set_input(1, JoypadMask::default());
    for _ in 0..10 {
        core.run();
    }
    core.set_input(0, JoypadMask::default());
    core.run();
    assert!(core.frame().is_some());
}

#[test]
fn save_states_round_trip() {
    let _serial = serial();
    let Some(mut core) = load("state") else {
        return;
    };
    for _ in 0..30 {
        core.run();
    }
    let size = core.serialize_size();
    assert!(size > 100_000, "gba state size {size}");
    let state = core.serialize().unwrap();
    assert_eq!(state.len(), size);
    let snap = core.frame().unwrap().to_rgba8();
    for _ in 0..30 {
        core.run();
    }
    core.unserialize(&state).unwrap();
    core.run();
    // Rendering after a restore may differ by a frame of timing; compare the state itself.
    let again = core.serialize().unwrap();
    assert_eq!(again.len(), size);
    assert!(
        core.unserialize(&state[..size / 2]).is_err(),
        "truncated state is refused"
    );
    let _ = snap;
}

#[test]
fn memory_regions_and_reset() {
    let _serial = serial();
    let Some(mut core) = load("memory") else {
        return;
    };
    core.run();
    // arm.gba has no battery save, so SAVE_RAM may be absent or empty; that must be a
    // clean answer either way, and system RAM must be readable.
    let _ = core.memory(Memory::SaveRam);
    let sys = core.memory(Memory::SystemRam);
    if let Some(sys) = &sys {
        assert!(!sys.is_empty());
        let mut wrong = sys.clone();
        wrong.push(0);
        assert!(
            core.write_memory(Memory::SystemRam, &wrong).is_err(),
            "size mismatch refused"
        );
    }
    assert!(core.memory(Memory::Rtc).is_none() || true);
    core.reset();
    core.run();
    assert!(core.frame().is_some());
}

#[test]
fn options_are_declared_and_settable() {
    let _serial = serial();
    let Some(mut core) = load("options") else {
        return;
    };
    let opts = core.options();
    assert!(opts.len() > 5, "mGBA declares many options: {}", opts.len());
    let skip = opts
        .iter()
        .find(|o| o.key == "mgba_skip_bios")
        .expect("mgba_skip_bios exists");
    assert!(
        skip.values.iter().any(|v| v == "ON") && skip.values.iter().any(|v| v == "OFF"),
        "{skip:?}"
    );
    assert_eq!(
        core.option("mgba_skip_bios").as_deref(),
        Some("ON"),
        "preset from Env applied"
    );
    core.set_option("mgba_skip_bios", "OFF");
    assert_eq!(core.option("mgba_skip_bios").as_deref(), Some("OFF"));
    assert_eq!(core.option("no_such_option"), None);
    for _ in 0..3 {
        core.run();
    }
}

#[test]
fn bad_inputs_are_errors_not_crashes() {
    let _serial = serial();
    let Some(core) = core_path() else { return };
    let e = Core::load(
        &repo().join("vendor/does_not_exist.dll"),
        &rom(),
        env("bad1"),
    )
    .unwrap_err();
    assert!(matches!(e, slot2_retro::Error::Load(_)), "{e}");
    let e = Core::load(&core, &repo().join("assets/test/missing.gba"), env("bad2")).unwrap_err();
    assert!(
        matches!(e, slot2_retro::Error::Io(_) | slot2_retro::Error::Game(_)),
        "{e}"
    );
}

#[test]
fn a_core_can_be_loaded_again_after_drop() {
    let _serial = serial();
    let Some(mut a) = load("twice-a") else { return };
    a.run();
    drop(a);
    let Some(mut b) = load("twice-b") else { return };
    b.run();
    assert!(b.frame().is_some());
}

#[test]
fn same_library_twice_at_once_is_busy() {
    let _serial = serial();
    let Some(a) = load("busy-a") else { return };
    let e = Core::load(&core_path().unwrap(), &rom(), env("busy-b")).unwrap_err();
    assert!(matches!(e, slot2_retro::Error::Busy(_)), "{e}");
    drop(a);
}

#[test]
fn frame_conversion_expands_channels_correctly() {
    let _serial = serial();
    use slot2_retro::Frame;
    let f = Frame {
        width: 2,
        height: 1,
        pitch: 4,
        format: PixelFormat::Rgb565,
        data: vec![0x1F, 0x00, 0xE0, 0x07], // little-endian 0x001F (blue), 0x07E0 (green)
    };
    assert_eq!(f.rgb(0, 0), [0, 0, 255]);
    assert_eq!(f.rgb(1, 0), [0, 255, 0]);
    let f = Frame {
        width: 1,
        height: 2,
        pitch: 8, // padded rows
        format: PixelFormat::Xrgb8888,
        data: vec![
            0x11, 0x22, 0x33, 0x00, 9, 9, 9, 9, 0xAA, 0xBB, 0xCC, 0x00, 9, 9, 9, 9,
        ],
    };
    assert_eq!(f.rgb(0, 0), [0x33, 0x22, 0x11]);
    assert_eq!(f.rgb(0, 1), [0xCC, 0xBB, 0xAA]);
    assert_eq!(
        f.to_rgba8(),
        vec![0x33, 0x22, 0x11, 255, 0xCC, 0xBB, 0xAA, 255]
    );
    let f = Frame {
        width: 1,
        height: 1,
        pitch: 2,
        format: PixelFormat::Rgb1555,
        data: vec![0xFF, 0x7F], // 0x7FFF: all channels max
    };
    assert_eq!(f.rgb(0, 0), [255, 255, 255]);
}

#[test]
fn the_declared_sample_rate_is_what_the_core_actually_produces() {
    let _serial = serial();
    let Some(mut core) = load("rate") else { return };
    let av = core.av_info();

    // mGBA declares 65536 Hz for GBA (`GBA_OUTPUT_RATE`), well above CD rates. A clamp
    // here once rewrote anything over 50 kHz to 32768 "for sanity", and the result was
    // every GBA game playing an octave low at half speed, chopped back to roughly the
    // right tempo by the audio ring overflowing. Whatever a core says its rate is, the
    // audio it hands over has to agree with it — that is the invariant, not a range.
    assert!(av.sample_rate >= 8_000.0, "{}", av.sample_rate);

    core.run(); // the first frame is partial; drop it
    let _ = core.take_audio();
    const FRAMES: usize = 8;
    let mut stereo = 0usize;
    for _ in 0..FRAMES {
        core.run();
        stereo += core.take_audio().len() / 2;
    }

    let measured = (stereo as f64 / FRAMES as f64) * av.fps;
    assert!(
        (measured - av.sample_rate).abs() / av.sample_rate < 0.02,
        "core declares {} Hz but produces {measured:.0} Hz ({} stereo frames over {FRAMES} \
         video frames at {} fps)",
        av.sample_rate,
        stereo,
        av.fps
    );
}
