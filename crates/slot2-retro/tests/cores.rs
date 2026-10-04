//! Every shelf's default core, against a ROM its console would accept.
//!
//! `mgba.rs` covers the GBA in depth; this is the breadth pass that M2 needs: for each of
//! the seven platforms, the core the registry names is a real file, it loads, it runs, and
//! what it reports about itself matches what it does. Cores are fetched or built by
//! `build/cores.ps1`; a platform whose core is missing skips rather than fails, so a
//! checkout without `vendor/` still goes green.

mod support;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use slot2_retro::{Core, Env, Platform};

/// One core instance per library at a time, and Genesis Plus GX answers for two shelves.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// A short name for messages. `PlatformDef` carries no display name of its own — the card
/// folder lives in slot2-store, which this crate deliberately does not depend on.
fn name(p: Platform) -> String {
    format!("{p:?}")
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-cores-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// File extensions each console's cartridges come as. The card's own extension table lives
/// in slot2-store, which this crate does not depend on, so the few this suite needs are
/// spelled out here.
fn exts(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Gb => &["gb"],
        Platform::Gbc => &["gbc", "gb"],
        Platform::Gba => &["gba"],
        Platform::Nes => &["nes"],
        Platform::Snes => &["sfc", "smc"],
        Platform::Md => &["md", "gen", "bin"],
        Platform::Sms => &["sms"],
    }
}

/// Where to look for real ROMs, in order: `$SLOT2_TEST_ROMS`, then `assets/test/local`.
///
/// Both are outside version control on purpose. Commercial ROMs cannot be committed to a
/// repository that is going out under MIT, but they answer questions the hand-built images
/// cannot — a real cartridge switches video modes, uses a mapper, fills its save RAM and
/// takes a state worth restoring. Drop files in either place and these tests pick them up
/// by extension; leave them out and the suite still passes on what it can build itself.
fn local_rom_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = std::env::var_os("SLOT2_TEST_ROMS") {
        dirs.push(PathBuf::from(d));
    }
    dirs.push(repo().join("assets/test/local"));
    dirs
}

/// The first file in those directories whose extension the platform claims.
fn local_rom(platform: Platform) -> Option<(Vec<u8>, &'static str)> {
    let exts = exts(platform);
    for dir in local_rom_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut found: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| {
                // ".md" is a Mega Drive cartridge *and* Markdown, and Genesis Plus GX will
                // happily boot a README — the 68000 executes whatever it is given.
                !p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.eq_ignore_ascii_case("README.md"))
            })
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| exts.iter().any(|x| x.eq_ignore_ascii_case(e)))
            })
            .collect();
        found.sort(); // same ROM every run, whatever the filesystem's order
        if let Some(path) = found.first() {
            let ext = exts
                .iter()
                .find(|x| {
                    path.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| x.eq_ignore_ascii_case(e))
                })
                .copied()
                .unwrap_or(exts[0]);
            eprintln!("{}: using {}", name(platform), path.display());
            return Some((std::fs::read(path).unwrap(), ext));
        }
    }
    None
}

/// The ROM each platform gets tested with, and the extension the core will see.
///
/// A real ROM wins when one is there; otherwise the hand-built image, which exists for
/// every platform whose loader will accept a file this suite is allowed to produce.
fn rom_for(platform: Platform) -> Option<(Vec<u8>, &'static str)> {
    if let Some(found) = local_rom(platform) {
        return Some(found);
    }
    match platform {
        Platform::Gba => {
            let p = repo().join("assets/test/arm.gba");
            p.is_file().then(|| (std::fs::read(&p).unwrap(), "gba"))
        }

        // mGBA's `GBIsROM` compares the 48 bytes at $0104 against the Nintendo logo and
        // refuses anything else — the same check the real boot ROM does, and the reason
        // unlicensed Game Boy cartridges could be sued over a trademark rather than a
        // patent. So there is no Game Boy image this suite may build: the only file mGBA
        // will accept carries Nintendo's artwork. Put a cartridge in assets/test/local to
        // cover these two shelves here; otherwise the device run covers them.
        Platform::Gb | Platform::Gbc => None,

        Platform::Nes => Some((support::nes(), "nes")),
        Platform::Snes => Some((support::snes(), "sfc")),
        Platform::Md => Some((support::megadrive(), "md")),
        Platform::Sms => Some((support::sms(), "sms")),
    }
}

/// Load the platform's default core, or `None` when the core or the ROM is missing.
fn load(platform: Platform) -> Option<Core> {
    let def = slot2_retro::def(platform);
    let dylib = repo().join("vendor").join(def.default_core.file_name());
    if !dylib.is_file() {
        eprintln!(
            "{}: no {} — run build/cores.ps1",
            name(platform),
            dylib.display()
        );
        return None;
    }
    let Some((bytes, ext)) = rom_for(platform) else {
        eprintln!("{}: no test rom available, skipping", name(platform));
        return None;
    };
    let stem = name(platform);
    let rom = support::write_rom(&stem, ext, &bytes);
    let env = Env {
        system_dir: tmp(&format!("{stem}-sys")),
        save_dir: tmp(&format!("{stem}-save")),
        options: def
            .options
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        language: 0,
    };
    Some(
        Core::load(&dylib, &rom, env)
            .unwrap_or_else(|e| panic!("{stem} core refused its test rom: {e}")),
    )
}

/// Load one named core for a platform, or `None` with the reason printed when this machine
/// has no library or no ROM to try it with.
fn load_alternative(core: slot2_retro::CoreId, platform: Platform) -> Option<Core> {
    let dylib = repo().join("vendor").join(core.file_name());
    if !dylib.is_file() {
        eprintln!(
            "{} on {}: no {} — run build/cores.ps1",
            core.base_name(),
            name(platform),
            dylib.display()
        );
        return None;
    }
    let Some((bytes, ext)) = rom_for(platform) else {
        eprintln!(
            "{} on {}: no test rom available, skipping",
            core.base_name(),
            name(platform)
        );
        return None;
    };
    let stem = format!("{}-{}", name(platform), core.base_name());
    let rom = support::write_rom(&stem, ext, &bytes);
    let env = Env {
        system_dir: tmp(&format!("{stem}-sys")),
        save_dir: tmp(&format!("{stem}-save")),
        options: slot2_retro::options_for_core(core, platform, false, slot2_retro::Tuning::DESKTOP)
            .unwrap_or_default(),
        language: 0,
    };
    Some(
        Core::load(&dylib, &rom, env)
            .unwrap_or_else(|e| panic!("{} refused its test rom: {e}", core.base_name())),
    )
}

#[test]
fn the_alternative_cores_run_their_consoles() {
    let _serial = serial();
    // gpSP on GBA has the MIT test ROM in the repository and its host library is built by
    // `build/cores.ps1 -Core gpsp`, so that one is never skipped. Gambatte needs a ROM with the
    // Nintendo logo in it, which this suite may not build: `assets/test/local` is asked first,
    // and without a cartridge there the two Game Boy rows report their own skip.
    for (core, platform) in [
        (slot2_retro::CoreId::Gpsp, Platform::Gba),
        (slot2_retro::CoreId::Gambatte, Platform::Gb),
        (slot2_retro::CoreId::Gambatte, Platform::Gbc),
    ] {
        let case = format!("{} on {}", core.base_name(), name(platform));
        let Some(mut loaded) = load_alternative(core, platform) else {
            continue;
        };

        let av = loaded.av_info();
        assert!(av.fps > 1.0 && av.fps < 1000.0, "{case}: fps {}", av.fps);
        for _ in 0..10 {
            loaded.run();
        }
        let frame = loaded
            .frame()
            .unwrap_or_else(|| panic!("{case}: drew nothing in ten frames"));
        assert!(frame.width > 0 && frame.height > 0, "{case}: empty frame");
        assert_eq!(
            frame.to_rgba8().len(),
            (frame.width * frame.height * 4) as usize,
            "{case}: converted frame is the wrong size"
        );

        // Rewind and resume states are built on this: a core that cannot serialize cannot
        // keep either.
        let state = loaded
            .serialize()
            .unwrap_or_else(|e| panic!("{case}: serialize failed: {e}"));
        assert!(!state.is_empty(), "{case}: empty state");
        for _ in 0..5 {
            loaded.run();
        }
        loaded
            .unserialize(&state)
            .unwrap_or_else(|e| panic!("{case}: unserialize failed: {e}"));
        loaded.run();
        assert!(
            loaded.frame().is_some(),
            "{case}: no frame after restoring a state"
        );
    }
}

const ALL: [Platform; 7] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Nes,
    Platform::Snes,
    Platform::Md,
    Platform::Sms,
];

#[test]
fn every_platform_names_a_core_that_exists() {
    // Not a load, just the file: this is the check that says "the registry and vendor/ have
    // drifted apart" in one line instead of seven confusing skips.
    let mut missing = Vec::new();
    for p in ALL {
        let def = slot2_retro::def(p);
        let dylib = repo().join("vendor").join(def.default_core.file_name());
        if !dylib.is_file() {
            missing.push(format!("{} → {}", name(p), def.default_core.file_name()));
        }
    }
    if !missing.is_empty() {
        eprintln!("cores not built: {}", missing.join(", "));
    }
}

#[test]
fn every_core_loads_runs_and_draws() {
    let _serial = serial();
    for p in ALL {
        let Some(mut core) = load(p) else { continue };
        let av = core.av_info();

        assert!(
            av.fps > 1.0 && av.fps < 1000.0,
            "{}: fps {}",
            name(p),
            av.fps
        );
        assert!(
            av.base_width > 0 && av.base_height > 0,
            "{}: {}x{}",
            name(p),
            av.base_width,
            av.base_height
        );

        for _ in 0..10 {
            core.run();
        }
        let frame = core
            .frame()
            .unwrap_or_else(|| panic!("{} drew nothing in ten frames", name(p)));
        assert!(
            frame.width > 0 && frame.height > 0 && frame.width <= av.max_width,
            "{}: frame {}x{} against max {}x{}",
            name(p),
            frame.width,
            frame.height,
            av.max_width,
            av.max_height
        );
        assert_eq!(
            frame.to_rgba8().len(),
            (frame.width * frame.height * 4) as usize,
            "{}: converted frame is the wrong size",
            name(p)
        );
    }
}

#[test]
fn every_core_produces_audio_at_the_rate_it_declares() {
    let _serial = serial();
    // The regression this guards is a host that "corrects" a core's declared rate: mGBA
    // says 65536 Hz for the GBA and means it. Whatever a core claims, the samples it hands
    // over per frame have to agree, because that number is what the resampler is built on.
    for p in ALL {
        let Some(mut core) = load(p) else { continue };
        let av = core.av_info();
        assert!(av.sample_rate >= 8_000.0, "{}: {}", name(p), av.sample_rate);

        core.run(); // the first frame is partial
        let _ = core.take_audio();
        const FRAMES: usize = 10;
        let mut stereo = 0usize;
        for _ in 0..FRAMES {
            core.run();
            stereo += core.take_audio().len() / 2;
        }
        let measured = (stereo as f64 / FRAMES as f64) * av.fps;
        assert!(
            (measured - av.sample_rate).abs() / av.sample_rate < 0.05,
            "{}: declares {} Hz, produces {measured:.0} Hz",
            name(p),
            av.sample_rate
        );
    }
}

#[test]
fn every_core_round_trips_a_save_state() {
    let _serial = serial();
    for p in ALL {
        let Some(mut core) = load(p) else { continue };

        for _ in 0..10 {
            core.run();
        }
        let state = match core.serialize() {
            Ok(s) => s,
            Err(e) => panic!("{}: serialize failed: {e}", name(p)),
        };
        assert!(!state.is_empty(), "{}: empty state", name(p));

        for _ in 0..10 {
            core.run();
        }
        core.unserialize(&state)
            .unwrap_or_else(|e| panic!("{}: unserialize failed: {e}", name(p)));

        // Restoring must land somewhere a frame can still be produced from.
        core.run();
        assert!(
            core.frame().is_some(),
            "{}: no frame after restoring a state",
            name(p)
        );
    }
}

#[test]
fn every_mapped_button_reaches_every_core() {
    let _serial = serial();
    use slot2_retro::LogicalButton as B;
    const BUTTONS: [B; 14] = [
        B::A,
        B::B,
        B::X,
        B::Y,
        B::L1,
        B::R1,
        B::L2,
        B::R2,
        B::Select,
        B::Start,
        B::Up,
        B::Down,
        B::Left,
        B::Right,
    ];
    for p in ALL {
        let Some(mut core) = load(p) else { continue };

        // Every button at once, then each on its own. A core that mishandles an id it was
        // never going to receive crashes here rather than in someone's hands.
        let all = slot2_retro::mask_for(p, BUTTONS);
        core.set_input(0, all);
        for _ in 0..3 {
            core.run();
        }
        for b in BUTTONS {
            core.set_input(0, slot2_retro::mask_for(p, [b]));
            core.run();
        }
        assert!(core.frame().is_some(), "{}: stopped drawing", name(p));

        // And the mask really is the union of the parts, not the last one written.
        let mut expected = 0u16;
        for b in BUTTONS {
            if let Some(bit) = slot2_retro::joypad_bit(p, b) {
                expected |= bit;
            }
        }
        assert_eq!(all.0, expected, "{}: mask lost a button", name(p));
    }
}
