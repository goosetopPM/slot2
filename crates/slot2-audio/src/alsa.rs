//! ALSA through `dlopen`, exactly as the libretro core is loaded: linking `libasound` would
//! make the device build need ALSA headers on the build box, and the device already has the
//! library. Port from `C:\Users\gyuha\slot-2\crates\slot\src\audio\alsa.rs` (MIT, Brandon T.
//! Kowalski), adapted to this crate's `Sink`/`Consumer`.
//!
//! Implementation notes for task 08 (see tasks/08-audio.md):
//!
//! - Open `libasound.so.2`, falling back to `libasound.so`; resolve
//!   `snd_pcm_open`, `snd_pcm_set_params`, `snd_pcm_writei`, `snd_pcm_recover`,
//!   `snd_pcm_prepare`, `snd_pcm_pause`, `snd_pcm_avail_update`, `snd_pcm_drop`,
//!   `snd_pcm_close`, `snd_strerror`. Any missing symbol → `Error::Open`.
//! - `snd_pcm_open(&mut handle, c"default", SND_PCM_STREAM_PLAYBACK = 0, 0)`, then
//!   `snd_pcm_set_params(handle, FORMAT_S16_LE = 2, ACCESS_RW_INTERLEAVED = 3, channels = 2,
//!   rate, soft_resample = 1, latency_us)`. `latency_us` comes from `frames`:
//!   `frames * 1_000_000 / rate`.
//! - A writer thread owns the `Consumer`: loop { read up to `period` frames through
//!   `read_or_silence`, `snd_pcm_writei`; on a negative return call `snd_pcm_recover(err, 1)`
//!   and continue; sleep nothing — `writei` blocks, which is the pacing }. The thread exits
//!   when an `Arc<AtomicBool>` stop flag is set; `Drop` sets it, joins, then `snd_pcm_drop`
//!   + `snd_pcm_close`.
//! - `pause`/`resume` set an `AtomicBool` the writer checks: while paused it writes silence
//!   instead of draining the ring (so the game's audio resumes where it left off rather than
//!   fast-forwarding). Do not call `snd_pcm_pause`: not every driver supports it.
//! - `queued_frames` → `snd_pcm_avail_update` converted to "buffered" (buffer size minus
//!   available), or `None` if the call fails.
//! - Everything unix-only sits behind `#[cfg(unix)]`; on other targets `AlsaSink::open`
//!   returns `Error::Open("alsa is unix only")` so the crate still builds and tests run on
//!   Windows.

use crate::{Consumer, Error, Sink};

pub struct AlsaSink {
    _todo: (),
}

impl AlsaSink {
    /// Open `default` at `rate` with about `frames` of buffer, fed from `consumer`.
    pub fn open(rate: u32, frames: usize, consumer: Consumer) -> Result<AlsaSink, Error> {
        let _ = (rate, frames, consumer);
        todo!("task 08")
    }
}

impl Sink for AlsaSink {
    fn rate(&self) -> u32 {
        todo!("task 08")
    }
    fn pause(&mut self) {
        todo!("task 08")
    }
    fn resume(&mut self) {
        todo!("task 08")
    }
    fn queued_frames(&self) -> Option<usize> {
        todo!("task 08")
    }
}
