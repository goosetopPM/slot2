//! The noises the slot makes.
//!
//! Cut from a recording of the real thing and played as recorded: no stretching, no envelope,
//! nothing joined. See `assets/sfx/PROVENANCE.md`. Everything the sound needs to do it already
//! does, and keeping it with the picture is a question of *when the clip starts* rather than
//! of what is done to it.

const INSERT: &[u8] = include_bytes!("../../../assets/sfx/insert.pcm");
const EJECT: &[u8] = include_bytes!("../../../assets/sfx/eject.pcm");

/// What the clips were cut at.
pub const ASSET_HZ: f32 = 48_000.0;

/// A noise the frontend makes itself, as opposed to anything coming out of a core.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    /// The whole of a cart going in: the shell down the rails, then the contacts.
    Insert,
    /// The whole of one coming out: the contacts letting go, then the shell back up.
    Eject,
}

impl Sfx {
    /// The clip, as it is on disk: mono, signed 16-bit little endian, 48 kHz.
    fn pcm(self) -> &'static [u8] {
        match self {
            Sfx::Insert => INSERT,
            Sfx::Eject => EJECT,
        }
    }

    /// How far into the clip the contacts are.
    ///
    /// The caller starts the clip this long before the cart reaches them, which is the whole
    /// of how the two are kept together. Everything before it is the shell on its way, and a
    /// clip started when the cart arrives has already missed the part that matters.
    pub fn lead(self) -> f32 {
        match self {
            Sfx::Insert => 0.097,
            Sfx::Eject => 0.021,
        }
    }

    /// The whole clip, in seconds.
    pub fn seconds(self) -> f32 {
        todo!()
    }

    /// What is left after the contacts: the shell settling on the way in, the shell still
    /// moving on the way out. Nothing should cut across it.
    pub fn tail(self) -> f32 {
        todo!()
    }

    /// Interleaved stereo at `rate`, from mono content.
    ///
    /// Stereo because the sink is, not because the sound is: the same sample goes to both
    /// channels. A slot is in the middle of the machine.
    pub fn render(self, rate: u32) -> Vec<i16> {
        todo!()
    }
}
