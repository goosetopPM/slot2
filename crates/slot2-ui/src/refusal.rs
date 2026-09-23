//! Saying no.
//!
//! Ported from the original, whose whole error UI was one curve: no words, a flinch, and the
//! same answer for every action the frontend will not carry out. A shake says *no* in less
//! time than it takes to read *no*, and it cannot be mistranslated.
//!
//! SLOT2 needs a second half the original did not. It had one console and one core, so the
//! only refusals were "not now"; a card here can be missing the core for a whole shelf,
//! which is something the player can go and fix — if they are told. The flinch stays as the
//! answer to an action that cannot happen, and [`crate::toast`] carries the reason when
//! there is one.

/// Long enough to read as a movement, short enough to be over before it looks like something
/// the player has to dismiss.
pub const REFUSAL_S: f32 = 0.3;

/// How far off centre, at the start.
const SHAKE_PX: f32 = 6.0;
/// Fast enough to read as a rattle rather than a slide.
const SHAKE_HZ: f32 = 14.0;

/// A refusal in progress. Decays on its own clock, wherever it is being drawn.
#[derive(Clone, Copy, Debug, Default)]
pub struct Refusal {
    age: f32,
}

impl Refusal {
    /// A refusal starting now.
    pub fn new() -> Refusal {
        Refusal { age: 0.0 }
    }

    /// Advance it. `dt` in seconds.
    pub fn tick(&mut self, dt: f32) {
        self.age += dt;
    }

    /// True while it is still moving.
    pub fn active(&self) -> bool {
        self.age < REFUSAL_S
    }

    /// Horizontal pixels off centre, decaying to nothing.
    ///
    /// Cosine rather than sine, so the first frame is already at full throw: a flinch is a
    /// knock and everything after it is settling. Starting at zero would make the frame the
    /// action was refused on the one frame that did not move.
    pub fn offset(&self) -> f32 {
        if !self.active() {
            return 0.0;
        }
        let t = self.age / REFUSAL_S;
        let decay = 1.0 - t;
        let phase = self.age * SHAKE_HZ * 2.0 * std::f32::consts::PI;
        SHAKE_PX * phase.cos() * decay
    }
}
