//! What the gauge says, and the one place that knows where to ask.
//!
//! Nothing here is a hardcoded path. The board that ships with the next kernel names its
//! supplies differently, and an absent gauge has to read as a device that does not have one
//! rather than as a boot failure. V-2 saw `axp2202-battery` and `axp2202-usb` on the RG SP:
//! two entries under the same class, one of which is a charger with no charge of its own.

use std::path::{Path, PathBuf};

/// Where the kernel publishes its supplies, under whatever sysfs root is being probed.
pub const CLASS: &str = "class/power_supply";

/// A percent and a state, for a host with no gauge. `<percent>` or `<percent>,<status>`,
/// the status in the kernel's own words, case ignored: `SLOT2_BATTERY=8,discharging`.
pub const BATTERY_ENV: &str = "SLOT2_BATTERY";

/// What the kernel's `status` attribute said, decoded. `Unknown` is a first-class answer
/// rather than an error: on this PMIC `current_now` already reads empty, so a standard
/// `power_supply` attribute being present is no promise that it is populated. Everything
/// downstream has to make `Unknown` behave the way the frontend did before it could read the
/// charge state at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Charge {
    Unknown,
    Discharging,
    Charging,
    Full,
}

/// One reading of the gauge. The two halves come from different files, so they are carried
/// together rather than fetched separately: a caller that asked for each in turn could draw
/// a percent and a charge state that never coexisted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Battery {
    pub percent: u8,
    pub charge: Charge,
}

/// Where to ask, decided once. Probing walks sysfs on every call otherwise, and the answer
/// does not change while the machine is on.
#[derive(Clone, Debug, Default)]
pub struct Gauge(Source);

#[derive(Clone, Debug, Default)]
enum Source {
    /// A `power_supply` directory with a `capacity` in it.
    Sysfs(PathBuf),
    /// A reading that does not come from a file. The host override, and tests.
    Fixed(Battery),
    /// No gauge on this machine.
    #[default]
    Absent,
}

impl Gauge {
    /// The real one: `/sys`, unless `SLOT2_BATTERY` names a reading instead. The override is
    /// for the host window, where there is no gauge and the corner would otherwise be empty
    /// on the one machine the HUD is developed on.
    pub fn detect() -> Gauge {
        match std::env::var(BATTERY_ENV)
            .ok()
            .as_deref()
            .and_then(parse_override)
        {
            Some(b) => Gauge::fixed(b),
            None => Gauge::probe(Path::new("/sys")),
        }
    }

    /// Against an arbitrary sysfs root, which is what makes the walk testable off device.
    ///
    /// Entries in name order, so a tree with two supplies picks the same one on every boot.
    /// A charger is a power supply too and has no charge of its own to report, so an entry
    /// counts only if its `type` is `Battery` *and* it has a `capacity` to read.
    pub fn probe(sysfs: &Path) -> Gauge {
        let class = sysfs.join(CLASS);
        let mut entries: Vec<PathBuf> = match std::fs::read_dir(&class) {
            Ok(rd) => rd
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_dir())
                .collect(),
            // A missing or unreadable class is a board without a gauge, not an error.
            Err(_) => return Gauge::none(),
        };
        // Name order, never read_dir order: read_dir order is the filesystem's, which a
        // reboot reshuffles, and two supplies must resolve the same way every boot.
        entries.sort();
        for p in entries {
            let is_battery = std::fs::read_to_string(p.join("type"))
                .map(|t| t.trim().eq_ignore_ascii_case("Battery"))
                .unwrap_or(false);
            if is_battery && p.join("capacity").is_file() {
                return Gauge(Source::Sysfs(p));
            }
        }
        Gauge::none()
    }

    /// A reading that never changes. What `SLOT2_BATTERY` builds, and what a test uses.
    pub fn fixed(battery: Battery) -> Gauge {
        Gauge(Source::Fixed(battery))
    }

    /// No gauge. What a device with no battery node has, and the default.
    ///
    /// The one constructor with a body already: `App` builds one in its constructor, so a
    /// `todo!()` here would take every app test down with it and hide the failures that
    /// matter.
    pub fn none() -> Gauge {
        Gauge(Source::Absent)
    }

    /// Both halves, read now. `None` where there is no gauge and where the capacity will not
    /// parse — zero would be a battery critical, which is a different screen entirely.
    ///
    /// Reads rather than remembers: the charge state changes the instant a cable moves, and
    /// a cached one is a gauge that lies for as long as the caller's poll interval.
    pub fn read(&self) -> Option<Battery> {
        match &self.0 {
            Source::Fixed(b) => Some(*b),
            Source::Sysfs(dir) => {
                // Both halves now, from the two files: read in turn and a cable move lands
                // between them, giving a percent and a charge that never coexisted.
                let percent = read_percent(dir)?;
                let status = std::fs::read_to_string(dir.join("status")).ok();
                Some(Battery {
                    percent,
                    charge: parse_charge(&status.unwrap_or_default()),
                })
            }
            Source::Absent => None,
        }
    }

    /// Which directory the walk settled on, for the diagnostics screen. `None` for a fixed
    /// reading and for no gauge, neither of which came from a file.
    pub fn path(&self) -> Option<&Path> {
        match &self.0 {
            Source::Sysfs(p) => Some(p),
            Source::Fixed(_) | Source::Absent => None,
        }
    }
}

/// The kernel names five states and this cares about three. Anything else — "Not charging"
/// on a charger that is not filling, an empty file, a node that is not there — is the
/// reading that leaves every policy where it was.
pub fn parse_charge(text: &str) -> Charge {
    // The kernel's own words, exactly: the sysfs file is machine-written and consistent,
    // and only the override is hand-typed, so the override's case-insensitivity lives in
    // parse_override's normalization, not here. "charging" lowercase is a string the
    // kernel does not write and stays Unknown.
    match text.trim() {
        "Charging" => Charge::Charging,
        "Discharging" => Charge::Discharging,
        "Full" => Charge::Full,
        _ => Charge::Unknown,
    }
}

/// A gauge percent off a file's text or an override's own half: a whole number clamped to
/// 0..=100. Whitespace is the newline sysfs leaves and the spaces a hand-typed variable
/// carries. `None` for anything else — an unparsable capacity is no reading at all, never
/// zero, because zero is a battery critical and a different screen.
fn parse_percent(text: &str) -> Option<u8> {
    let n: u32 = text.trim().parse().ok()?;
    u8::try_from(n.min(100)).ok()
}

/// `capacity` off a supply directory. A driver advertising the file and leaving it empty —
/// which this PMIC does to `current_now` — is the same as the file not being there.
fn read_percent(dir: &Path) -> Option<u8> {
    parse_percent(&std::fs::read_to_string(dir.join("capacity")).ok()?)
}

/// `SLOT2_BATTERY`'s grammar. `None` for anything that is not a percent, so a typo in the
/// environment is an absent gauge rather than a gauge reading zero.
pub fn parse_override(text: &str) -> Option<Battery> {
    // One comma or none; a percent never contains one, so split at the first and take the
    // rest as the status, hand-typed and therefore matched case-insensitively.
    let (percent, status) = match text.split_once(',') {
        Some((p, s)) => (p, Some(s)),
        None => (text, None),
    };
    let percent = parse_percent(percent)?;
    // "50," and "50,sideways" are a typo, not an unknown charge: the state is optional by
    // absence only, never by being unparseable.
    let charge = match status {
        Some(s) => {
            // Hand-typed, so matched case-insensitively — not through parse_charge, whose
            // arms are the kernel's exact words. Lowercasing then matching those arms can
            // only ever produce Unknown. An empty state and "sideways" stay the typos
            // they are.
            match s.trim().to_ascii_lowercase().as_str() {
                "charging" => Charge::Charging,
                "discharging" => Charge::Discharging,
                "full" => Charge::Full,
                _ => return None,
            }
        }
        None => Charge::Unknown,
    };
    Some(Battery { percent, charge })
}
