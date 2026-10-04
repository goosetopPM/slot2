//! M0: identify the device, bring up a surface, draw the splash, idle. On the device the
//! first boot also dumps a hardware survey (`diag`) for the design doc's V-list.

use slot2::app;
#[cfg(feature = "device")]
mod device_app;
mod diag;
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
    /// The language `SLOT2_LANG` asked for, or `None` when the process has no such variable.
    ///
    /// `None` is not "English": it is "ask the card", and the two are told apart all the way to
    /// the startup that reads the stored language instead.
    pub lang_override: Option<String>,
}

pub fn boot() -> Boot {
    let detected = slot2_platform::detect();
    let p = detected.profile;
    let backend = if cfg!(feature = "device") {
        "device"
    } else {
        "host"
    };
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
    let lang_override = std::env::var_os("SLOT2_LANG").map(|value| match value.into_string() {
        Ok(code) => code,
        Err(_) => {
            // A value this process cannot decode is still an explicit choice rather than an
            // absence: an empty request fails to load like any other unusable code, and the
            // built-in English is what runs. Quietly using the card's language instead would
            // ignore whoever set the variable.
            eprintln!("slot2: SLOT2_LANG is not valid Unicode; asking for nothing");
            String::new()
        }
    });
    let utc = slot2_platform::clock::utc_now();
    // The card has not been read yet — that happens once the backends have the profile, below —
    // so this line says what was asked for and not what will run. The code is escaped: a language
    // code is never printed raw, and the environment is never dumped.
    let lang_log = match &lang_override {
        Some(code) => format!("{code:?}"),
        None => "none".to_string(),
    };
    eprintln!(
        "slot2: {} backend={backend} target={} panel={} safe_area_at={:?} lid={} sticks={} source={} root={} lang_override={lang_log} utc_offset_min={} utc_now={utc}",
        env!("CARGO_PKG_VERSION"),
        p.target,
        p.geometry,
        p.geometry.safe_area_offset(),
        p.has_lid,
        p.has_sticks,
        detected.source,
        root.display(),
        slot2_platform::clock::utc_offset_min(),
    );
    Boot {
        detected,
        root,
        lang_override,
    }
}

/// One line saying which language was asked for and which one is actually running.
///
/// Both backends build their context the same way, and both print this once, before the loop: a
/// card whose pack is missing or broken still boots, and this is where that is visible instead of
/// silent.
///
/// Both codes are escaped — including the effective one. A code is a file stem and nothing here
/// is promised to be clean: `SLOT2_LANG` may carry control characters, and a pack whose name has
/// them can load, so the code the context reports can too. A newline in either value would split
/// this into lines that read as something the frontend never said.
pub fn announce_language(requested: &str, effective: &str) {
    eprintln!("slot2: language requested={requested:?} effective={effective:?}");
}

/// Where the libretro cores live on a card.
pub fn core_dir(root: &std::path::Path) -> PathBuf {
    root.join("System").join("cores")
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
