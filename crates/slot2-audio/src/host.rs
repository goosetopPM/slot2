//! cpal on the PC: the same `Sink`, so the frame loop is identical on both.
//!
//! Implementation notes for task 08 (see tasks/08-audio.md):
//!
//! - `cpal::default_host().default_output_device()` → `Error::Open` when there is none.
//! - Ask for a stereo f32 config at `rate` (`supported_output_configs`, pick one whose
//!   channel count is 2 and whose range covers `rate`; else fall back to the device's
//!   `default_output_config` and report ITS rate from `Sink::rate`, since the caller
//!   resamples to whatever we return).
//! - `build_output_stream` with a callback that pulls from the `Consumer` via
//!   `read_or_silence` into a scratch `Vec<i16>` and converts to f32 (`s as f32 / 32768.0`).
//!   While paused, write silence without draining.
//! - Keep the stream alive in the struct; `Drop` drops it (cpal stops on drop).
//! - cpal's error callback: `eprintln!("slot2: audio: {err}")`, nothing else.
//! - `queued_frames` → `None` (cpal does not report it).

use crate::{Consumer, Error, Sink};

pub struct HostSink {
    _todo: (),
}

impl HostSink {
    pub fn open(rate: u32, frames: usize, consumer: Consumer) -> Result<HostSink, Error> {
        let _ = (rate, frames, consumer);
        todo!("task 08")
    }
}

impl Sink for HostSink {
    fn rate(&self) -> u32 {
        todo!("task 08")
    }
    fn pause(&mut self) {
        todo!("task 08")
    }
    fn resume(&mut self) {
        todo!("task 08")
    }
}
