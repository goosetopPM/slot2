//! Volume, with a mute that remembers.
//!
//! Implementation notes for task 08: `level` is 0..=100 as the UI shows it; the gain applied
//! is **perceptual**, not linear — `gain = (level / 100)^2.0`, so the bottom of the range is
//! usable rather than all crammed into the last few steps. `apply` multiplies in f32 and
//! clamps to i16. Muting keeps the previous level so unmuting restores it. `step_up`/
//! `step_down` move by `STEP` and clamp; stepping up while muted unmutes first.

/// One press of VOL+ / VOL−.
pub const STEP: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Volume {
    level: u8,
    muted: bool,
}

impl Default for Volume {
    fn default() -> Self {
        Volume::new(70)
    }
}

impl Volume {
    pub fn new(level: u8) -> Volume {
        Volume {
            level: level.min(100),
            muted: false,
        }
    }

    /// 0..=100, as shown to the user. Unchanged by muting.
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn set_level(&mut self, level: u8) {
        self.level = level.min(100);
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Mute (keeping the level) or unmute back to it.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn step_up(&mut self) {
        self.muted = false;
        self.level = self.level.saturating_add(STEP).min(100);
    }

    pub fn step_down(&mut self) {
        self.level = self.level.saturating_sub(STEP);
    }

    /// The multiplier actually applied: 0.0 when muted, `(level/100)^2` otherwise.
    pub fn gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            let ratio = self.level as f32 / 100.0;
            ratio * ratio
        }
    }

    /// Scale a buffer in place.
    pub fn apply(&self, samples: &mut [i16]) {
        let g = self.gain();
        if g == 1.0 {
            return;
        }
        if g == 0.0 {
            samples.fill(0);
            return;
        }
        for s in samples {
            let val = *s as f32 * g;
            *s = val.clamp(i16::MIN as f32, i16::MAX as f32).round() as i16;
        }
    }
}
