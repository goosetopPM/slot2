//! The input probe: a screen that shows every raw evdev event the hardware produces, and
//! writes them to the card, so a board's real key codes can be read off instead of guessed.
//!
//! It runs instead of the normal frontend when `System/input-probe` exists on the card, and
//! it removes that file as it starts, so a probe is always a one-boot thing — a card left
//! in probe mode by accident boots normally the next time.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use slot2_gfx::{Canvas, Color, GlCanvas, Surface};
use slot2_input::{EvdevSource, KeyMap, DEFAULT_H700_KEYMAP};
use slot2_ui::{face, UiCtx, PX_BODY, PX_HINT, PX_TITLE};

/// Present on the card = run the probe on the next boot.
pub const MARKER: &str = "System/input-probe";
/// Where the probe writes what it saw.
pub const LOG: &str = "System/input-probe.txt";

/// How many lines the screen shows at once.
const LINES: usize = 16;
/// The probe ends on its own after this long, so a device never sits in it forever.
const RUN_FOR: Duration = Duration::from_secs(300);

pub fn wanted(root: &Path) -> bool {
    root.join(MARKER).exists()
}

/// Read events until the time is up, drawing and logging everything.
pub fn run(root: &Path, canvas: &mut GlCanvas, surface: &mut dyn Surface, ctx: &mut UiCtx) {
    // One boot only: take the marker out before anything else can fail, and make the
    // removal reach the card — a probe that ends with the power key must not leave the
    // device probing forever.
    let _ = std::fs::remove_file(root.join(MARKER));
    sync_all_disks();

    let mut source = EvdevSource::open_all(KeyMap::from_pairs(DEFAULT_H700_KEYMAP));
    source.log_raw(true);
    let devices = source.device_count();
    eprintln!("slot2: input probe: {devices} evdev devices");

    let mut lines: Vec<String> = Vec::new();
    let mut file = String::new();
    let _ = writeln!(
        file,
        "slot2 input probe\ndevices: {devices}\nformat: event<dev> type code value  (type 1 = EV_KEY, 3 = EV_ABS, 5 = EV_SW)\n"
    );

    let began = Instant::now();
    let mut dirty = true;
    while began.elapsed() < RUN_FOR {
        let frame = Instant::now();
        let _ = source.poll(frame);
        for (dev, raw) in source.take_raw_log() {
            let line = format!(
                "event{dev}  type {:<2} code {:<5} (0x{:03x})  value {}",
                raw.type_, raw.code, raw.code, raw.value
            );
            let _ = writeln!(file, "{line}");
            lines.push(line);
            if lines.len() > LINES {
                lines.remove(0);
            }
            dirty = true;
        }

        if dirty {
            canvas.clear(Color::from_rgb8(0x0C, 0x0C, 0x10));
            face::draw_text(
                canvas,
                ctx,
                "INPUT PROBE",
                PX_TITLE,
                ctx.safe.px(24.0),
                ctx.safe.py(16.0),
                Color::from_rgb8(0xF2, 0xF2, 0xF0),
            );
            face::draw_text(
                canvas,
                ctx,
                "press every button once, then power off",
                PX_HINT,
                ctx.safe.px(24.0),
                ctx.safe.py(48.0),
                Color::from_rgb8(0x9A, 0x9A, 0xA0),
            );
            for (i, line) in lines.iter().enumerate() {
                face::draw_text(
                    canvas,
                    ctx,
                    line,
                    PX_BODY,
                    ctx.safe.px(24.0),
                    ctx.safe.py(80.0 + i as f32 * 22.0),
                    Color::from_rgb8(0xD0, 0xD8, 0xE0),
                );
            }
            dirty = false;
        }

        if let Err(e) = canvas.present(surface) {
            eprintln!("slot2: {e}");
            break;
        }
        // Flush as we go, all the way to the card: a probe usually ends with the power
        // key, and a vfat write that only reached the page cache is a write that never
        // happened.
        save(root, &file);

        if let Some(left) = Duration::from_millis(33).checked_sub(frame.elapsed()) {
            sleep(left);
        }
    }

    save(root, &file);
    eprintln!("slot2: input probe finished, wrote {LOG}");
}

/// Write the log and push it to the card. `fs::write` alone leaves the bytes in the page
/// cache, where a power cut loses them.
fn save(root: &Path, text: &str) {
    let path = root.join(LOG);
    if let Ok(mut f) = std::fs::File::create(&path) {
        let _ = f.write_all(text.as_bytes());
        let _ = f.sync_all();
    }
}
/// `sync(2)` through the shell: there is no std call for "flush every filesystem", and a
/// directory change (the marker's removal) is not covered by a file's own `sync_all`.
fn sync_all_disks() {
    let _ = std::process::Command::new("sync").status();
}
