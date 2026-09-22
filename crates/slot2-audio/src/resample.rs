//! Rate conversion, stereo, linear interpolation.
//!
//! Implementation notes for task 08 (see tasks/08-audio.md):
//!
//! - `Resampler::new(from, to)` keeps the ratio as a fixed-point step
//!   `step = from as f64 / to as f64` and a fractional position `pos` carried across calls,
//!   plus the last input frame of the previous call so a frame boundary interpolates
//!   correctly instead of clicking.
//! - `process(&mut self, input: &[i16], out: &mut Vec<i16>)`: `input` is interleaved stereo
//!   (`len % 2 == 0`; an odd tail is dropped). For each output frame, take
//!   `i = pos.floor()`, `f = pos.fract()`, interpolate `a + (b - a) * f` per channel where
//!   `a` is frame `i` and `b` is frame `i+1` (the previous call's last frame stands in for
//!   `i == -1`), push both channels, `pos += step`. Stop when `i+1` would leave `input`;
//!   keep the leftover fractional position and the last frame for the next call.
//!   Interpolate in `f32` and round to i16 with clamping.
//! - `from == to` is a pass-through: copy, no interpolation, no accumulated drift — but
//!   only while no rate correction is applied (see `drc_trim`), since a trim makes the two
//!   rates unequal however the fields read.
//! - `set_rates(from, to)` changes the ratio and resets `pos` but keeps the last frame.
//! - `reset()` clears everything (used when a core is swapped or a state is loaded).
//! - `expected_output_frames(input_frames)` is the count `process` will produce, ±1; the
//!   frame loop uses it to size buffers.

/// The widest rate correction [`drc_trim`] will ask for. Half a percent is under what the
/// ear hears as a pitch change, and still an order of magnitude more than the drift it has
/// to cancel once the frame loop is paced by the core's own fps.
pub const DRC_MAX: f64 = 0.005;

/// Dynamic rate control: how much to stretch the output rate so the ring settles near half
/// full. `1.0` means no correction; above 1.0 produces more output samples per input frame
/// (the ring is draining), below 1.0 produces fewer (the ring is filling).
///
/// Without this the two clocks involved — the host clock the frame loop sleeps against and
/// the codec's own 48 kHz crystal — drift apart, and the ring eventually sits empty
/// (silence) or full (every frame's tail thrown away). A slow ±0.5 % nudge cancels the
/// drift inaudibly; the alternative, dropping or repeating samples, is what a click is.
pub fn drc_trim(queued_frames: usize, capacity_frames: usize) -> f64 {
    if capacity_frames == 0 {
        return 1.0;
    }
    let target = capacity_frames as f64 / 2.0;
    // +1 when the ring is empty, 0 at half, -1 when it is full.
    let err = ((target - queued_frames as f64) / target).clamp(-1.0, 1.0);
    1.0 + err * DRC_MAX
}

pub struct Resampler {
    from: u32,
    to: u32,
    trim: f64,
    step: f64,
    pos: f64,
    last: [i16; 2],
}

impl Resampler {
    pub fn new(from: u32, to: u32) -> Resampler {
        Resampler {
            from,
            to,
            trim: 1.0,
            step: from as f64 / to as f64,
            pos: 1.0,
            last: [0, 0],
        }
    }

    pub fn rates(&self) -> (u32, u32) {
        (self.from, self.to)
    }

    pub fn set_rates(&mut self, from: u32, to: u32) {
        self.from = from;
        self.to = to;
        self.step = self.compute_step();
        self.pos = 1.0;
    }

    /// The rate correction currently applied. `1.0` = none.
    pub fn output_trim(&self) -> f64 {
        self.trim
    }

    /// Stretch the output rate by `trim` (see [`drc_trim`]), clamped to ±[`DRC_MAX`].
    ///
    /// The phase is deliberately *not* reset: this is called every frame, and restarting
    /// the interpolator mid-stream is exactly the discontinuity it exists to avoid.
    pub fn set_output_trim(&mut self, trim: f64) {
        let trim = if trim.is_finite() { trim } else { 1.0 };
        self.trim = trim.clamp(1.0 - DRC_MAX, 1.0 + DRC_MAX);
        self.step = self.compute_step();
    }

    fn compute_step(&self) -> f64 {
        self.from as f64 / (self.to as f64 * self.trim)
    }

    /// True when input can be copied through untouched — same rate *and* no correction
    /// pending. With a trim applied the rates are no longer equal, whatever the fields say.
    fn is_pass_through(&self) -> bool {
        self.from == self.to && self.trim == 1.0
    }

    pub fn reset(&mut self) {
        self.pos = 1.0;
        self.last = [0, 0];
    }

    /// Resample interleaved stereo `input`, appending to `out`. Returns frames written.
    pub fn process(&mut self, input: &[i16], out: &mut Vec<i16>) -> usize {
        let input_len = (input.len() / 2) * 2;
        if input_len == 0 {
            return 0;
        }

        if self.is_pass_through() {
            out.extend_from_slice(&input[..input_len]);
            self.last = [input[input_len - 2], input[input_len - 1]];
            return input_len / 2;
        }

        let input_frames = input_len / 2;
        let start_len = out.len();

        // We can sample at `pos` where `pos` is index in sequence [last, input[0], input[1], ..., input[N-1]]
        // last is at 0, input[0] is at 1, ..., input[N-1] is at N.
        // We need floor(pos) and floor(pos)+1 to be within [0, N].
        // So floor(pos) <= N - 1.
        while self.pos <= input_frames as f64 + 1e-9 {
            let i = self.pos.floor() as usize;
            let f = (self.pos - i as f64) as f32;

            let (a, b) = if i == 0 {
                (self.last, [input[0], input[1]])
            } else if i < input_frames {
                (
                    [input[(i - 1) * 2], input[(i - 1) * 2 + 1]],
                    [input[i * 2], input[i * 2 + 1]],
                )
            } else {
                // i == input_frames. Valid only if f is very small.
                (
                    [input[(i - 1) * 2], input[(i - 1) * 2 + 1]],
                    [input[(i - 1) * 2], input[(i - 1) * 2 + 1]], // Just repeat the last one
                )
            };

            for c in 0..2 {
                let sample_a = a[c] as f32;
                let sample_b = b[c] as f32;
                let val = sample_a + (sample_b - sample_a) * f;
                out.push(val.clamp(i16::MIN as f32, i16::MAX as f32).round() as i16);
            }

            self.pos += self.step;
        }

        self.last = [input[input_len - 2], input[input_len - 1]];
        self.pos -= input_frames as f64;

        (out.len() - start_len) / 2
    }

    /// Roughly how many frames `input_frames` will become.
    pub fn expected_output_frames(&self, input_frames: usize) -> usize {
        if self.is_pass_through() {
            return input_frames;
        }
        // Current pos is relative to the start of the next input (which will be index 1 in the sequence).
        // So we have input_frames available at indices 1..input_frames+1.
        // Plus index 0 (last frame).
        // Total indices 0..input_frames+1.
        // We can sample until floor(pos) > input_frames.
        ((input_frames as f64 - self.pos + 1.0) / self.step)
            .ceil()
            .max(0.0) as usize
    }
}
