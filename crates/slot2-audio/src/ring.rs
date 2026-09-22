//! A lock-free single-producer single-consumer ring of i16 samples.
//!
//! Implementation notes for task 08 (see tasks/08-audio.md):
//!
//! - `Ring::new(frames)` allocates `frames * CHANNELS` samples, rounded up to a power of
//!   two so the index masks. `split()` hands out a `Producer` and a `Consumer`, each owning
//!   an `Arc` of the shared buffer; the buffer itself is `UnsafeCell<Box<[i16]>>` with
//!   `head`/`tail` as `AtomicUsize` (free-running counters, masked on use).
//!   `Producer: Send`, `Consumer: Send`, neither `Sync`.
//! - `write(&mut self, samples: &[i16]) -> usize`: copy as many as fit, return how many
//!   were taken. A full ring drops the rest (an overrun means the consumer stalled; the
//!   frame loop keeps going rather than blocking).
//! - `read(&mut self, out: &mut [i16]) -> usize`: fill as much as available, return how
//!   many were written. **An underrun is silence, not a repeat**: the caller zero-fills the
//!   tail itself, or uses `read_or_silence` which does it.
//! - `available()` (consumer side) and `space()` (producer side) report sample counts, not
//!   frames. `Ring::capacity()` is the sample capacity; one slot is left unused so full and
//!   empty are distinguishable, or use the counter difference — either is fine as long as
//!   the tests hold.
//! - Ordering: producer publishes with `Release` after copying; consumer reads the other
//!   index with `Acquire`. No `Mutex`, no allocation after `new`.

use crate::CHANNELS;

pub struct Ring {
    _todo: (),
}

pub struct Producer {
    _todo: (),
}

pub struct Consumer {
    _todo: (),
}

impl Ring {
    /// Room for `frames` stereo frames (rounded up to a power of two of samples).
    pub fn new(frames: usize) -> Ring {
        let _ = frames;
        todo!("task 08")
    }

    /// Sample capacity (frames * CHANNELS, rounded).
    pub fn capacity(&self) -> usize {
        todo!("task 08")
    }

    pub fn split(self) -> (Producer, Consumer) {
        todo!("task 08")
    }
}

impl Producer {
    /// Samples that fit right now.
    pub fn space(&self) -> usize {
        todo!("task 08")
    }

    /// Write what fits; returns how many samples were taken (may be fewer than `samples`).
    pub fn write(&mut self, samples: &[i16]) -> usize {
        let _ = samples;
        todo!("task 08")
    }
}

impl Consumer {
    /// Samples ready to read.
    pub fn available(&self) -> usize {
        todo!("task 08")
    }

    /// Read what is there; returns how many samples were written into `out`.
    pub fn read(&mut self, out: &mut [i16]) -> usize {
        let _ = out;
        todo!("task 08")
    }

    /// Read, then zero the rest of `out`. What an audio callback wants: an underrun is a
    /// moment of silence, never a repeated buffer.
    pub fn read_or_silence(&mut self, out: &mut [i16]) -> usize {
        let n = self.read(out);
        out[n..].fill(0);
        n
    }

    /// Stereo frames ready.
    pub fn frames(&self) -> usize {
        self.available() / CHANNELS
    }
}
