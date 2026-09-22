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

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::CHANNELS;

struct Shared {
    /// SPSC invariant: Only the Producer writes to the buffer elements indexed by [head, tail+mask].
    /// Only the Consumer reads from the buffer elements indexed by [tail, head].
    /// The indices are masked by `mask` for actual buffer access.
    buf: UnsafeCell<Box<[i16]>>,
    mask: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

pub struct Ring {
    shared: Arc<Shared>,
}

pub struct Producer {
    shared: Arc<Shared>,
}

pub struct Consumer {
    shared: Arc<Shared>,
}

impl Ring {
    /// Room for `frames` stereo frames (rounded up to a power of two of samples).
    pub fn new(frames: usize) -> Ring {
        let cap = (frames * CHANNELS).next_power_of_two();
        let buf = vec![0i16; cap].into_boxed_slice();
        Ring {
            shared: Arc::new(Shared {
                buf: UnsafeCell::new(buf),
                mask: cap - 1,
                head: AtomicUsize::new(0),
                tail: AtomicUsize::new(0),
            }),
        }
    }

    /// Sample capacity (frames * CHANNELS, rounded).
    pub fn capacity(&self) -> usize {
        self.shared.mask + 1
    }

    pub fn split(self) -> (Producer, Consumer) {
        (
            Producer {
                shared: self.shared.clone(),
            },
            Consumer {
                shared: self.shared,
            },
        )
    }
}

impl Producer {
    /// Samples that fit right now.
    pub fn space(&self) -> usize {
        let head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);
        let cap = self.shared.mask + 1;
        let len = head.wrapping_sub(tail);
        // Leaving one slot empty as per hint or standard practice to distinguish full/empty
        // although wrapping counters don't strictly need it if cap is power of two.
        // The hint says: "space = capacity - len - 1".
        cap.saturating_sub(len).saturating_sub(1)
    }

    /// Stereo frames written but not yet played. Dynamic rate control steers on this.
    pub fn queued_frames(&self) -> usize {
        let head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail) / CHANNELS
    }

    /// Stereo frames the ring holds when full.
    pub fn capacity_frames(&self) -> usize {
        (self.shared.mask + 1) / CHANNELS
    }

    /// Write what fits; returns how many samples were taken (may be fewer than `samples`).
    pub fn write(&mut self, samples: &[i16]) -> usize {
        let head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);
        let cap = self.shared.mask + 1;
        let mask = self.shared.mask;
        let len = head.wrapping_sub(tail);
        let space = cap.saturating_sub(len).saturating_sub(1);
        let n = samples.len().min(space);

        if n == 0 {
            return 0;
        }

        unsafe {
            let buf = &mut *self.shared.buf.get();
            let first_part = n.min(cap - (head & mask));
            buf[(head & mask)..(head & mask) + first_part].copy_from_slice(&samples[..first_part]);
            if n > first_part {
                buf[..n - first_part].copy_from_slice(&samples[first_part..n]);
            }
        }

        self.shared
            .head
            .store(head.wrapping_add(n), Ordering::Release);
        n
    }
}

unsafe impl Send for Producer {}

impl std::fmt::Debug for Producer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Producer")
            .field("space", &self.space())
            .finish()
    }
}

impl Consumer {
    /// Samples ready to read.
    pub fn available(&self) -> usize {
        let head = self.shared.head.load(Ordering::Acquire);
        let tail = self.shared.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    /// Read what is there; returns how many samples were written into `out`.
    pub fn read(&mut self, out: &mut [i16]) -> usize {
        let head = self.shared.head.load(Ordering::Acquire);
        let tail = self.shared.tail.load(Ordering::Relaxed);
        let cap = self.shared.mask + 1;
        let mask = self.shared.mask;
        let len = head.wrapping_sub(tail);
        let n = out.len().min(len);

        if n == 0 {
            return 0;
        }

        unsafe {
            let buf = &*self.shared.buf.get();
            let first_part = n.min(cap - (tail & mask));
            out[..first_part].copy_from_slice(&buf[(tail & mask)..(tail & mask) + first_part]);
            if n > first_part {
                out[first_part..n].copy_from_slice(&buf[..n - first_part]);
            }
        }

        self.shared
            .tail
            .store(tail.wrapping_add(n), Ordering::Release);
        n
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

unsafe impl Send for Consumer {}

impl std::fmt::Debug for Consumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Consumer")
            .field("available", &self.available())
            .finish()
    }
}
