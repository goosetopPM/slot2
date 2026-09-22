//! Ring buffer, resampling, volume, and sinks.
//!
//! The chain, once per frame: the core hands over interleaved stereo i16 at *its* rate
//! (mGBA 32768 Hz, GPGX 44100, snes9x 32040…) → [`Resampler`] converts to the device rate
//! → [`Volume`] scales → [`Ring`] holds it until the audio callback takes it. Two threads
//! touch the ring: the frame loop writes, the sink's callback reads, so it is lock-free by
//! construction (one producer, one consumer, atomics only).
//!
//! Nothing here knows about a core or a screen; a sink is anything that can be started and
//! fed. [`Sink`] is implemented by `HostSink` (cpal, PC) and `AlsaSink` (libasound through
//! `dlopen`, device).
//!
//! Design: docs/DESIGN.md §6, decisions D-03.

pub mod alsa;
#[cfg(feature = "host")]
pub mod host;
pub mod resample;
pub mod ring;
pub mod volume;

pub use alsa::AlsaSink;
#[cfg(feature = "host")]
pub use host::HostSink;
pub use resample::Resampler;
pub use ring::{Consumer, Producer, Ring};
pub use volume::Volume;

use std::fmt;

/// What the device plays at. 48 kHz is what the H700's codec runs natively; everything is
/// resampled to it once, at the source.
pub const DEVICE_RATE: u32 = 48_000;

/// Stereo, always: every core this frontend runs produces two channels.
pub const CHANNELS: usize = 2;

#[derive(Debug)]
pub enum Error {
    /// The backend library, device or stream could not be opened. Human-readable detail.
    Open(String),
    /// The sink was asked for something it cannot do (unsupported rate or format).
    Unsupported(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Open(m) => write!(f, "audio: {m}"),
            Error::Unsupported(m) => write!(f, "audio: unsupported: {m}"),
        }
    }
}
impl std::error::Error for Error {}

/// An open audio output. Dropping it stops and closes the stream.
pub trait Sink {
    /// The rate the sink actually opened at, which may differ from what was asked for.
    fn rate(&self) -> u32;
    /// Pause output without closing the device (lid closed, menu open).
    fn pause(&mut self);
    fn resume(&mut self);
    /// How many stereo frames the sink has buffered but not yet played, if it can tell.
    /// Used by the frame loop to decide whether it is falling behind.
    fn queued_frames(&self) -> Option<usize> {
        None
    }
}
