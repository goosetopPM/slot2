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
        self.pcm().len() as f32 / 2.0 / ASSET_HZ
    }

    /// What is left after the contacts: the shell settling on the way in, the shell still
    /// moving on the way out. Nothing should cut across it.
    pub fn tail(self) -> f32 {
        self.seconds() - self.lead()
    }

    /// Interleaved stereo at `rate`, from mono content.
    ///
    /// Stereo because the sink is, not because the sound is: the same sample goes to both
    /// channels. A slot is in the middle of the machine.
    pub fn render(self, rate: u32) -> Vec<i16> {
        if rate == 0 {
            return Vec::new();
        }
        let pcm = self.pcm();
        let mut stereo = Vec::with_capacity(pcm.len());
        for chunk in pcm.as_chunks::<2>().0 {
            let s = i16::from_le_bytes(*chunk);
            stereo.push(s);
            stereo.push(s);
        }

        let mut resampler = crate::resample::Resampler::new(ASSET_HZ as u32, rate);
        let mut out = Vec::with_capacity(resampler.expected_output_frames(stereo.len() / 2) * 2);
        resampler.process(&stereo, &mut out);
        out
    }
}
