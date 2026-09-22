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
    _todo: (),
}

impl Default for Volume {
    fn default() -> Self {
        Volume::new(70)
    }
}

impl Volume {
    pub fn new(level: u8) -> Volume {
        let _ = level;
        todo!("task 08")
    }

    /// 0..=100, as shown to the user. Unchanged by muting.
    pub fn level(&self) -> u8 {
        todo!("task 08")
    }

    pub fn set_level(&mut self, level: u8) {
        let _ = level;
        todo!("task 08")
    }

    pub fn is_muted(&self) -> bool {
        todo!("task 08")
    }

    /// Mute (keeping the level) or unmute back to it.
    pub fn set_muted(&mut self, muted: bool) {
        let _ = muted;
        todo!("task 08")
    }

    pub fn toggle_mute(&mut self) {
        let m = self.is_muted();
        self.set_muted(!m);
    }

    pub fn step_up(&mut self) {
        todo!("task 08")
    }

    pub fn step_down(&mut self) {
        todo!("task 08")
    }

    /// The multiplier actually applied: 0.0 when muted, `(level/100)^2` otherwise.
    pub fn gain(&self) -> f32 {
        todo!("task 08")
    }

    /// Scale a buffer in place.
    pub fn apply(&self, samples: &mut [i16]) {
        let _ = samples;
        todo!("task 08")
    }
}
