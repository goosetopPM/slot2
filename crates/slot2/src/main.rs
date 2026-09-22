//! M0 step 1: prove the workspace builds for host and device and that the device knows what
//! it is. Prints the profile and exits. BaseOS respawns the frontend on exit, so on a real
//! card this would loop — which is fine for a first boot log and gets replaced by the splash
//! screen in step 2.

fn main() {
    let d = slot2_platform::detect();
    let p = d.profile;
    eprintln!(
        "slot2: {} target={} panel={} safe_area_at={:?} lid={} sticks={} source={}",
        env!("CARGO_PKG_VERSION"),
        p.target,
        p.geometry,
        p.geometry.safe_area_offset(),
        p.has_lid,
        p.has_sticks,
        d.source
    );
    let backend = if cfg!(feature = "device") { "device" } else { "host" };
    eprintln!("slot2: backend={backend} cwd={}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default());
}
