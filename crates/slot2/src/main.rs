//! M0: identify the device, bring up a surface, draw the splash, idle. On the device the
//! first boot also dumps a hardware survey (`diag`) for the design doc's V-list.

mod diag;
#[cfg(feature = "device")]
mod device_app;
#[cfg(feature = "host")]
mod host_app;

use std::path::PathBuf;

/// Where BaseOS mounts the card. The contract says the frontend is started with the card
/// root as cwd, so cwd wins; this is only the fallback.
pub const CARD: &str = "/mnt/sdcard";

/// Everything the two backends share before they diverge.
pub struct Boot {
    pub detected: slot2_platform::Detected,
    /// Card root on the device; `SLOT2_ROOT` or `./sdcard` on the host.
    pub root: PathBuf,
    /// Language code: `SLOT2_LANG`, else `en` (a settings file takes over in M4).
    pub lang: String,
}

pub fn boot() -> Boot {
    let detected = slot2_platform::detect();
    let p = detected.profile;
    let backend = if cfg!(feature = "device") { "device" } else { "host" };
    let root = if cfg!(feature = "device") {
        std::env::current_dir()
            .ok()
            .filter(|c| c.join("System").is_dir())
            .unwrap_or_else(|| PathBuf::from(CARD))
    } else {
        std::env::var_os("SLOT2_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("sdcard"))
    };
    let lang = std::env::var("SLOT2_LANG").unwrap_or_else(|_| "en".into());
    eprintln!(
        "slot2: {} backend={backend} target={} panel={} safe_area_at={:?} lid={} sticks={} source={} root={} lang={lang}",
        env!("CARGO_PKG_VERSION"),
        p.target,
        p.geometry,
        p.geometry.safe_area_offset(),
        p.has_lid,
        p.has_sticks,
        detected.source,
        root.display(),
    );
    Boot { detected, root, lang }
}

/// Font directories in search order: the card's `System/Fonts`, then the repo's assets
/// (host only, for `cargo run` from a checkout).
pub fn font_dirs(root: &std::path::Path) -> Vec<PathBuf> {
    let mut v = vec![root.join("System").join("Fonts")];
    if cfg!(feature = "host") {
        v.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts"));
    }
    v
}

fn main() {
    let boot = boot();
    #[cfg(feature = "device")]
    device_app::run(boot);
    #[cfg(feature = "host")]
    host_app::run(boot);
    #[cfg(not(any(feature = "device", feature = "host")))]
    {
        let _ = boot;
        eprintln!("slot2: built with neither `host` nor `device` feature");
    }
}
