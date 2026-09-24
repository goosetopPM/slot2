//! What time it is where the player is.
//!
//! The card keeps UTC and the screen shows local, and nothing on this box knows which one
//! that is: there is no `tzdata` on the image and no network to ask. So the difference is a
//! number the player sets once — minutes to add to UTC — and until the clock screen exists
//! (M5) it comes from the environment, which is enough to develop the HUD against and
//! honest about being a stand-in.
//!
//! No calendar here. The HUD shows hours and minutes and nothing else, and hours and minutes
//! need only division; the year needs a civil-from-days conversion that belongs with the
//! screen that sets it.

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds in a day, and the modulus everything here wraps on.
pub const DAY: i64 = 86_400;

/// Minutes to add to UTC, until the clock screen owns it. `SLOT2_UTC_OFFSET_MIN=540` is KST.
pub const OFFSET_ENV: &str = "SLOT2_UTC_OFFSET_MIN";

/// The furthest any inhabited place is from UTC, in minutes: Kiritimati at +14, Baker Island
/// at -12. A bound rather than a wrap, because an offset outside it is a typo and a typo
/// that silently becomes a valid time is a clock nobody can debug.
pub const OFFSET_MAX: i32 = 14 * 60;
pub const OFFSET_MIN: i32 = -12 * 60;

/// Seconds since the epoch, UTC. Zero if the system clock is before it, which on a board
/// with a dead RTC is a real reading rather than an error.
pub fn utc_now() -> i64 {
    todo!()
}

/// The offset in force, read from the environment once. Once because it is read every frame
/// and the environment does not change under a running process, and because reading it per
/// frame is a syscall's worth of work for a number that is already known.
pub fn utc_offset_min() -> i32 {
    static OFFSET: OnceLock<i32> = OnceLock::new();
    let _ = &OFFSET;
    todo!()
}

/// What the HUD shows: seconds since the epoch, shifted into local time.
pub fn now_local() -> i64 {
    todo!()
}

/// The environment's grammar: whole minutes, within a day of UTC. `None` for anything else,
/// so a typo leaves the clock on UTC instead of somewhere arbitrary.
pub fn parse_offset_min(text: &str) -> Option<i32> {
    let _ = text;
    todo!()
}

/// Hours and minutes off a UTC-epoch second count. Never seconds: a clock showing them is a
/// clock being watched rather than glanced at.
///
/// Wraps rather than clamps, so a time before the epoch — which is what a board with no RTC
/// reads, and what a negative offset makes of it — is a time of day rather than a minus sign.
pub fn hhmm(secs: i64) -> String {
    let _ = secs;
    todo!()
}

/// Whether the clock is worth showing at all.
///
/// A board with no RTC boots at the epoch and counts up from there, so the HUD would show a
/// confident, wrong 09:00 within minutes. Anything still in 1970 is a clock that was never
/// set, and an empty corner says that better than a number does.
pub fn is_set(secs: i64) -> bool {
    let _ = secs;
    todo!()
}

/// The first second `is_set` accepts: 2020-01-01T00:00:00Z. Before the earliest plausible
/// build of this frontend, after every value a clock that never started can reach.
pub const SET_AFTER: i64 = 1_577_836_800;
