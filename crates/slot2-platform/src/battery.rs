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
        todo!()
    }

    /// Against an arbitrary sysfs root, which is what makes the walk testable off device.
    ///
    /// Entries in name order, so a tree with two supplies picks the same one on every boot.
    /// A charger is a power supply too and has no charge of its own to report, so an entry
    /// counts only if its `type` is `Battery` *and* it has a `capacity` to read.
    pub fn probe(sysfs: &Path) -> Gauge {
        let _ = sysfs;
        todo!()
    }

    /// A reading that never changes. What `SLOT2_BATTERY` builds, and what a test uses.
    pub fn fixed(battery: Battery) -> Gauge {
        let _ = battery;
        todo!()
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
        todo!()
    }

    /// Which directory the walk settled on, for the diagnostics screen. `None` for a fixed
    /// reading and for no gauge, neither of which came from a file.
    pub fn path(&self) -> Option<&Path> {
        todo!()
    }
}

/// The kernel names five states and this cares about three. Anything else — "Not charging"
/// on a charger that is not filling, an empty file, a node that is not there — is the
/// reading that leaves every policy where it was.
pub fn parse_charge(text: &str) -> Charge {
    let _ = text;
    todo!()
}

/// `SLOT2_BATTERY`'s grammar. `None` for anything that is not a percent, so a typo in the
/// environment is an absent gauge rather than a gauge reading zero.
pub fn parse_override(text: &str) -> Option<Battery> {
    let _ = text;
    todo!()
}
