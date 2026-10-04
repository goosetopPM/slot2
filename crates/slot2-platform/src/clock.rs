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

use std::sync::atomic::{AtomicI32, Ordering};
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
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// The offset in force, an atomic read. Per frame without a lock or an environment walk:
/// the setter writes it and every later read is one relaxed load.
///
/// The sentinel stands for "still unset". Reads consult the environment only until initialization,
/// and publish that value only by replacing the sentinel; a setter that fired
/// first leaves its value standing. That is the whole reason the fallback is a
/// compare_exchange on the sentinel and not a store.
static OFFSET: AtomicI32 = AtomicI32::new(UNSET);

/// Outside OFFSET_MIN..=OFFSET_MAX, so a real offset can never equal it.
const UNSET: i32 = i32::MIN;

pub fn utc_offset_min() -> i32 {
    match OFFSET.load(Ordering::Relaxed) {
        UNSET => {
            let env = std::env::var(OFFSET_ENV)
                .ok()
                .and_then(|v| parse_offset_min(&v))
                .unwrap_or(0);
            match OFFSET.compare_exchange(UNSET, env, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => env,
                // A setter raced in first; its value is the clock's now.
                Err(set) => set,
            }
        }
        set => set,
    }
}

/// Change the offset at runtime. Rejects anything outside OFFSET_MIN..=OFFSET_MAX and
/// leaves the current value standing; Err carries the rejected input so the caller can say
/// why. UTC and the system clock are never touched.
pub fn set_utc_offset_min(mins: i32) -> Result<(), i32> {
    if (OFFSET_MIN..=OFFSET_MAX).contains(&mins) {
        OFFSET.store(mins, Ordering::Relaxed);
        Ok(())
    } else {
        Err(mins)
    }
}

/// What the HUD shows: seconds since the epoch, shifted into local time.
pub fn now_local() -> i64 {
    utc_now() + i64::from(utc_offset_min()) * 60
}

/// The one pure decision the HUD makes per frame: whether the clock is worth showing, judged
/// on the raw UTC sample, and only then the local rendering of that same sample. Judging
/// after the shift would let an offset move a sample across SET_AFTER and flicker the
/// corner; +14 or -12, the same UTC second gets the same answer.
pub fn hud_local(utc_secs: i64, offset_min: i32) -> Option<i64> {
    if !is_set(utc_secs) {
        return None;
    }
    utc_secs.checked_add(i64::from(offset_min) * 60)
}

/// The environment's grammar: whole minutes, within a day of UTC. `None` for anything else,
/// so a typo leaves the clock on UTC instead of somewhere arbitrary.
pub fn parse_offset_min(text: &str) -> Option<i32> {
    // i32, not i64: the bound check is the point, and "9.5"/"1e3"/"+" failing the parse
    // for free is the grammar. Truncation is not rounding; a fraction of a minute is
    // nothing a clock displays.
    let mins: i32 = text.trim().parse().ok()?;
    (OFFSET_MIN..=OFFSET_MAX).contains(&mins).then_some(mins)
}

/// Hours and minutes off a UTC-epoch second count. Never seconds: a clock showing them is a
/// clock being watched rather than glanced at.
///
/// Wraps rather than clamps, so a time before the epoch — which is what a board with no RTC
/// reads, and what a negative offset makes of it — is a time of day rather than a minus sign.
pub fn hhmm(secs: i64) -> String {
    // A day's remainder, never a clamp: negative secs come from a negative offset on a
    // clock that never started, and rem_euclid folds them into a time of day.
    let day = secs.rem_euclid(DAY);
    let h = (day / 3600) as u32;
    let m = ((day % 3600) / 60) as u32;
    format!("{h:02}:{m:02}")
}

/// Whether the clock is worth showing at all.
///
/// A board with no RTC boots at the epoch and counts up from there, so the HUD would show a
/// confident, wrong 09:00 within minutes. Anything still in 1970 is a clock that was never
/// set, and an empty corner says that better than a number does.
pub fn is_set(secs: i64) -> bool {
    secs >= SET_AFTER
}

/// The first second `is_set` accepts: 2020-01-01T00:00:00Z. Before the earliest plausible
/// build of this frontend, after every value a clock that never started can reach.
pub const SET_AFTER: i64 = 1_577_836_800;
