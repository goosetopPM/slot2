//! M0 step 2: the first thing to boot on a real card. Identify the device, dump a hardware
//! survey (see `diag`), then idle. BaseOS respawns the frontend the moment it exits, so a
//! quick exit would spin; sleeping keeps the box quiet until the splash screen replaces this.

mod diag;

use std::path::PathBuf;
use std::time::Duration;

/// Where BaseOS mounts the card. The contract says the frontend is started with the card
/// root as cwd, so cwd wins; this is only the fallback.
const CARD: &str = "/mnt/sdcard";

fn main() {
    let d = slot2_platform::detect();
    let p = d.profile;
    let backend = if cfg!(feature = "device") { "device" } else { "host" };
    eprintln!(
        "slot2: {} backend={backend} target={} panel={} safe_area_at={:?} lid={} sticks={} source={}",
        env!("CARGO_PKG_VERSION"),
        p.target,
        p.geometry,
        p.geometry.safe_area_offset(),
        p.has_lid,
        p.has_sticks,
        d.source
    );

    if cfg!(feature = "device") {
        let root = std::env::current_dir()
            .ok()
            .filter(|c| c.join("System").is_dir())
            .unwrap_or_else(|| PathBuf::from(CARD));
        eprintln!("slot2: root={}", root.display());
        eprint!("{}", diag::report(&root));
        eprintln!("slot2: diag written to System/slot2-diag.txt; idling 5 min then exiting (BaseOS respawns)");
        std::thread::sleep(Duration::from_secs(300));
    }
}
