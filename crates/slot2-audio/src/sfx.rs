//! The noises the slot makes.
//!
//! Cut from a recording of the real thing. [`Sfx::render`] is that recording, played as it was
//! cut: no stretching, no envelope, nothing joined. See `assets/sfx/PROVENANCE.md`.
//!
//! Every shelf makes those same two noises, because there are only two recordings; what a
//! platform owns is how *its* cartridge sounds doing it — a light pak down a long rail against
//! a short cart biting a connector. That is [`Sfx::render_styled`]: the same clip read at a
//! different source rate and scaled, which is the resampler this crate already has rather than
//! a synthesised waveform. Keeping the noise with the picture stays a question of *when the
//! clip starts*, and the styled lead ([`Sfx::lead_at_speed`]) is measured at the same effective
//! speed as the styled render, so the contacts land on the same frame on every shelf.

const INSERT: &[u8] = include_bytes!("../../../assets/sfx/insert.pcm");
const EJECT: &[u8] = include_bytes!("../../../assets/sfx/eject.pcm");

/// What the clips were cut at.
pub const ASSET_HZ: f32 = 48_000.0;

/// The widest sink rate the styled path renders for. Nothing this frontend ships reports one,
/// and past it the output would be a document of a mistake rather than a noise.
const RATE_MAX: u32 = 384_000;

/// What a styled speed may ask for, in step with `slot2_ui::skin::SoundProfile`'s own bounds.
const SPEED_MIN: f32 = 0.80;
const SPEED_MAX: f32 = 1.20;

/// The loudest a styled render may be, which is the recording's own level.
const GAIN_MAX: f32 = 1.0;

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
    ///
    /// This is exactly [`Sfx::render_styled`] with nothing done to it, so the recording and its
    /// duration are what they always were.
    pub fn render(self, rate: u32) -> Vec<i16> {
        self.render_styled(rate, 1.0, 1.0)
    }

    /// The same clip, read at `speed` and scaled by `gain`, interleaved stereo at `rate`.
    ///
    /// `speed` is a source rate rather than a duration: 1 is the recording, above it is the
    /// clip read faster (shorter and higher — a light, quick cartridge), below it slower. So
    /// the ordinary resampler does the work, and nothing here invents a waveform, joins two
    /// clips or pads one out.
    ///
    /// A speed or gain off the table is refused with an empty clip rather than clamped: a value
    /// out of range is a table that has gone wrong, and a silent slot is a better answer than a
    /// loud one. So is a sink rate no device reports.
    pub fn render_styled(self, rate: u32, speed: f32, gain: f32) -> Vec<i16> {
        if rate == 0 || rate > RATE_MAX || !speed_ok(speed) || !gain_ok(gain) {
            return Vec::new();
        }
        let pcm = self.pcm();
        let mut stereo = Vec::with_capacity(pcm.len());
        for chunk in pcm.as_chunks::<2>().0 {
            let s = i16::from_le_bytes(*chunk);
            // Saturating on purpose: the table never asks for more than 1.0, and a caller that
            // did would otherwise hear a folded-over click rather than a clipped one. Clamping
            // in `f32` and then casting keeps this the same in debug and release.
            let s = ((s as f32) * gain)
                .round()
                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            stereo.push(s);
            stereo.push(s);
        }

        let mut resampler = crate::resample::Resampler::new(effective_source_rate(speed), rate);
        let mut out = Vec::with_capacity(resampler.expected_output_frames(stereo.len() / 2) * 2);
        resampler.process(&stereo, &mut out);
        out
    }

    /// How far into the styled clip the contacts are: the recording's lead, read at `speed`.
    ///
    /// This is the count the caller subtracts from the frame the cart seats, so a clip that is
    /// read faster starts later and still puts its contacts on the same frame.
    pub fn lead_at_speed(self, speed: f32) -> f32 {
        if !speed_ok(speed) {
            return 0.0;
        }
        self.lead() / effective_speed(speed)
    }

    /// The whole styled clip, in seconds.
    pub fn seconds_at_speed(self, speed: f32) -> f32 {
        if !speed_ok(speed) {
            return 0.0;
        }
        self.seconds() / effective_speed(speed)
    }

    /// What is left after the contacts, at `speed`. Nothing should cut across it.
    pub fn tail_at_speed(self, speed: f32) -> f32 {
        self.seconds_at_speed(speed) - self.lead_at_speed(speed)
    }
}

/// Whether a styled speed is one the table may ask for.
///
/// A NaN fails both comparisons and the infinities fail one each, so this range test is also
/// the finiteness test, and there is no separate one to get out of step.
fn speed_ok(speed: f32) -> bool {
    (SPEED_MIN..=SPEED_MAX).contains(&speed)
}

/// Whether a styled gain may be asked for: at most the recording's own level.
fn gain_ok(gain: f32) -> bool {
    (0.0..=GAIN_MAX).contains(&gain)
}

/// The source rate a styled render really reads the recording at, in whole Hz.
fn effective_source_rate(speed: f32) -> u32 {
    (ASSET_HZ * speed).round().max(1.0) as u32
}

/// [`effective_source_rate`] back as a playback speed, in recorded seconds per played second.
///
/// The render resamples from an integer source rate, so the timings have to be derived from the
/// rate that was actually used rather than from the float that asked for it. Otherwise the
/// cue and the samples would disagree by the rounding, and the contacts would drift off the
/// frame the cart seats on.
fn effective_speed(speed: f32) -> f32 {
    effective_source_rate(speed) as f32 / ASSET_HZ
}
