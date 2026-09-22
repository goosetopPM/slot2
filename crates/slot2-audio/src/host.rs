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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate};

use crate::{Consumer, Error, Sink};

pub struct HostSink {
    rate: u32,
    paused: Arc<AtomicBool>,
    _stream: cpal::Stream,
}

impl HostSink {
    pub fn open(rate: u32, _frames: usize, mut consumer: Consumer) -> Result<HostSink, Error> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| Error::Open("no output device found".into()))?;

        let supported_configs = device
            .supported_output_configs()
            .map_err(|e| Error::Open(format!("failed to query configs: {e}")))?;

        let mut chosen_config = None;
        for config in supported_configs {
            if config.channels() == 2
                && config.sample_format() == SampleFormat::F32
                && config.min_sample_rate().0 <= rate
                && config.max_sample_rate().0 >= rate
            {
                chosen_config = Some(config.with_sample_rate(SampleRate(rate)));
                break;
            }
        }

        let config: cpal::StreamConfig = match chosen_config {
            Some(c) => c.into(),
            None => device
                .default_output_config()
                .map_err(|e| Error::Open(format!("failed to get default config: {e}")))?
                .into(),
        };

        let actual_rate = config.sample_rate.0;
        let paused = Arc::new(AtomicBool::new(false));
        let s_paused = paused.clone();
        // Owned by the callback. Sized for a typical buffer; `data.len()` decides.
        let mut scratch: Vec<i16> = vec![0; 4096];

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    if s_paused.load(Ordering::Relaxed) {
                        data.fill(0.0);
                        return;
                    }
                    // Grown, never allocated per callback: allocating inside an audio
                    // callback can block on the allocator and underrun the stream. cpal
                    // may hand a longer buffer than the first one, so grow on demand and
                    // keep it.
                    if scratch.len() < data.len() {
                        scratch.resize(data.len(), 0);
                    }
                    let buf = &mut scratch[..data.len()];
                    consumer.read_or_silence(buf);
                    for (out, &s) in data.iter_mut().zip(buf.iter()) {
                        *out = s as f32 / 32768.0;
                    }
                },
                |err| eprintln!("slot2: audio: {err}"),
                None,
            )
            .map_err(|e| Error::Open(format!("failed to build stream: {e}")))?;

        stream
            .play()
            .map_err(|e| Error::Open(format!("failed to start stream: {e}")))?;

        Ok(HostSink {
            rate: actual_rate,
            paused,
            _stream: stream,
        })
    }
}

impl Sink for HostSink {
    fn rate(&self) -> u32 {
        self.rate
    }
    fn pause(&mut self) {
        self.paused.store(true, Ordering::Relaxed);
    }
    fn resume(&mut self) {
        self.paused.store(false, Ordering::Relaxed);
    }
}
