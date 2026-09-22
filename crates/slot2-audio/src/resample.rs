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
//! - `from == to` is a pass-through: copy, no interpolation, no accumulated drift.
//! - `set_rates(from, to)` changes the ratio and resets `pos` but keeps the last frame.
//! - `reset()` clears everything (used when a core is swapped or a state is loaded).
//! - `expected_output_frames(input_frames)` is the count `process` will produce, ±1; the
//!   frame loop uses it to size buffers.

pub struct Resampler {
    _todo: (),
}

impl Resampler {
    pub fn new(from: u32, to: u32) -> Resampler {
        let _ = (from, to);
        todo!("task 08")
    }

    pub fn rates(&self) -> (u32, u32) {
        todo!("task 08")
    }

    pub fn set_rates(&mut self, from: u32, to: u32) {
        let _ = (from, to);
        todo!("task 08")
    }

    pub fn reset(&mut self) {
        todo!("task 08")
    }

    /// Resample interleaved stereo `input`, appending to `out`. Returns frames written.
    pub fn process(&mut self, input: &[i16], out: &mut Vec<i16>) -> usize {
        let _ = (input, out);
        todo!("task 08")
    }

    /// Roughly how many frames `input_frames` will become.
    pub fn expected_output_frames(&self, input_frames: usize) -> usize {
        let _ = input_frames;
        todo!("task 08")
    }
}
