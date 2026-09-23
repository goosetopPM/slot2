//! What rewind costs where it has to run.
//!
//! The device is a Cortex-A53 and so is the Raspberry Pi this is cross-built for, so the
//! numbers here are the ones that decide whether rewind can be on by default. A capture is
//! a `retro_serialize` plus a delta against the previous state, once every `interval_frames`
//! — so what matters is that cost divided by the interval, against a 16.7 ms frame.

#[allow(dead_code)]
mod support;

use std::path::PathBuf;
use std::time::Instant;

use slot2_retro::{Core, Env, Rewind, RewindBudget};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn core_file(name: &str) -> String {
    let ext = if cfg!(windows) { "dll" } else { "so" };
    format!("{name}_libretro.{ext}")
}

#[test]
fn a_capture_is_a_small_part_of_a_frame() {
    let repo = repo();
    let local = repo.join("assets/test/local");
    let pick = |ext: &str| -> Option<Vec<u8>> {
        std::fs::read_dir(&local)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().and_then(|e| e.to_str()) == Some(ext))
            .map(|p| std::fs::read(p).unwrap())
    };

    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        (
            "mgba",
            std::fs::read(repo.join("assets/test/arm.gba")).unwrap_or_default(),
            "gba",
        ),
        ("fceumm", pick("nes").unwrap_or_else(support::nes), "nes"),
        ("snes9x", pick("sfc").unwrap_or_else(support::snes), "sfc"),
        (
            "genesis_plus_gx",
            pick("md").unwrap_or_else(support::megadrive),
            "md",
        ),
    ];

    let budget = RewindBudget::DEFAULT;
    let frame_ms = 1000.0 / 60.0;
    let mut worst: f64 = 0.0;

    for (name, bytes, ext) in cases {
        if bytes.is_empty() {
            continue;
        }
        let dylib = repo.join("vendor").join(core_file(name));
        if !dylib.is_file() {
            eprintln!("no {}, skipping", dylib.display());
            continue;
        }
        let rom = support::write_rom("cost", ext, &bytes);
        let env = Env {
            system_dir: std::env::temp_dir(),
            save_dir: std::env::temp_dir(),
            options: vec![],
            language: 0,
        };
        let mut core = Core::load(&dylib, &rom, env).unwrap();
        let mut rewind = Rewind::new(budget);

        for _ in 0..60 {
            core.run();
        }

        const CAPTURES: usize = 30;
        let began = Instant::now();
        for _ in 0..CAPTURES {
            for _ in 0..budget.interval_frames {
                core.run();
            }
            let state = core.serialize().expect("a state");
            rewind.push(state);
        }
        let total = began.elapsed().as_secs_f64() * 1000.0;

        // Only the capture part is the cost of rewind; the core would have run anyway.
        let began = Instant::now();
        for _ in 0..CAPTURES * budget.interval_frames as usize {
            core.run();
        }
        let running = began.elapsed().as_secs_f64() * 1000.0;

        let capture_ms = ((total - running) / CAPTURES as f64).max(0.0);
        let per_frame = capture_ms / budget.interval_frames as f64;
        let share = 100.0 * per_frame / frame_ms;
        worst = worst.max(share);

        let depth = rewind.depth();
        let bytes = rewind.bytes();
        eprintln!(
            "{ext:5} capture {capture_ms:6.2} ms every {} frames = {per_frame:5.2} ms/frame \
             ({share:4.1}% of a frame); {depth} captures in {bytes} bytes \
             ({:.0} bytes each)",
            budget.interval_frames,
            bytes as f64 / depth.max(1) as f64
        );

        // No assertion on the share: what a capture costs is a property of the machine, and
        // `Session` answers a slow one by taking states less often rather than by
        // stuttering. This test exists to print the numbers that decision is made from —
        // run it on the Pi (build/pi-test.ps1) to see the device's architecture rather than
        // this desktop's.
        //
        // What must hold anywhere is that the deltas actually compress; a chain that stores
        // whole states would be megabytes here.
        assert!(
            bytes < depth * 256 * 1024,
            "{ext}: {bytes} bytes for {depth} captures is not compression"
        );
    }
    eprintln!("worst platform: {worst:.1}% of a frame");
}
