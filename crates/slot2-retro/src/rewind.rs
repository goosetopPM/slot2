//! Rewind: a ring of save states kept small enough to hold minutes of play.
//!
//! Storing whole states is hopeless. A Mega Drive state is a megabyte and one is taken every
//! six frames, so a plain ring would burn ten megabytes a second and hold three seconds of
//! play in a budget that would already be too big for this device.
//!
//! What makes it work is that almost none of a state changes in a tenth of a second.
//! Measured on real cartridges, six frames apart: 61 bytes of a 528 KB Game Boy Advance
//! state, 29 bytes of a 1 MB Mega Drive state, 4 KB of an 823 KB SNES state at its busiest.
//! So the ring holds one whole state and a chain of differences behind it.
//!
//! The differences are XOR, which is its own inverse, so one encoding serves both
//! directions and a delta applied to the newer state yields the older one. Runs of unchanged
//! bytes cost two varints, so a state where nothing moved costs almost nothing to remember.
//!
//! Rewinding *consumes* the ring, which is what makes the arithmetic honest: going back a
//! second and then playing forward again does not leave the second you rewound past lying
//! around, and the states captured on the way forward take its place.

use std::collections::VecDeque;

/// How often to take a state, and how much to spend remembering them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewindBudget {
    /// Frames between captures. Six is a tenth of a second, which is finer than anyone
    /// rewinds by hand and coarse enough that the cost per second stays small.
    pub interval_frames: u32,
    /// The cap on stored differences. Nothing is allocated up front — a game that changes
    /// little uses little — so this only decides how far back the ring reaches before the
    /// oldest end of it is dropped.
    pub ring_bytes: usize,
}

impl RewindBudget {
    /// What every platform gets until a measurement on the device says otherwise.
    ///
    /// Twenty-four megabytes on a box with just under a gigabyte, which buys minutes for a
    /// quiet game and around six seconds in the worst case anyone has produced: a scene
    /// change that rewrites a whole Mega Drive state every capture. The shape is per
    /// platform so one console can be tuned without disturbing the others; nothing measured
    /// so far justifies different numbers.
    pub const DEFAULT: RewindBudget = RewindBudget {
        interval_frames: 6,
        ring_bytes: 24 * 1024 * 1024,
    };
}

/// A chain of save states, newest first, stored as one state and the differences behind it.
#[derive(Debug)]
pub struct Rewind {
    budget: RewindBudget,
    /// The newest stored state, whole. Empty before anything is pushed.
    current: Vec<u8>,
    /// Differences, oldest at the front. Each turns the state in front of it into the one
    /// behind it.
    deltas: VecDeque<Vec<u8>>,
    bytes: usize,
    /// False until `current` itself has been handed out by `pop`.
    handed_out: bool,
    scratch: Vec<u8>,
}

impl Rewind {
    pub fn new(budget: RewindBudget) -> Rewind {
        Rewind {
            budget,
            current: Vec::new(),
            deltas: VecDeque::new(),
            bytes: 0,
            handed_out: false,
            scratch: Vec::new(),
        }
    }

    pub fn budget(&self) -> RewindBudget {
        self.budget
    }

    /// Whether a state should be taken on this frame.
    pub fn should_capture(&self, frames_run: u64) -> bool {
        self.budget.interval_frames > 0
            && frames_run.is_multiple_of(self.budget.interval_frames as u64)
    }

    /// How many states can still be stepped back to.
    pub fn depth(&self) -> usize {
        if self.current.is_empty() {
            0
        } else {
            self.deltas.len() + usize::from(!self.handed_out)
        }
    }

    /// What the differences are costing, not counting the one whole state.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn is_empty(&self) -> bool {
        self.depth() == 0
    }

    pub fn clear(&mut self) {
        self.current.clear();
        self.deltas.clear();
        self.bytes = 0;
        self.handed_out = false;
    }

    /// How often a state is being taken, which `Session` lowers when captures turn out to
    /// cost too much on the machine it is running on.
    pub fn interval(&self) -> u32 {
        self.budget.interval_frames
    }

    /// Take states less often. The chain already stored keeps its spacing; only what comes
    /// after is affected, so rewinding across the change is uneven but not wrong.
    pub fn set_interval(&mut self, frames: u32) {
        self.budget.interval_frames = frames;
    }

    /// Remember a state, taking it by value so the megabyte is moved rather than copied.
    ///
    /// A state of a different length than the last means the core has changed shape under
    /// us, which no delta can bridge, so the chain starts over from here rather than
    /// storing something that would decode into nonsense.
    pub fn push(&mut self, state: Vec<u8>) {
        self.handed_out = false;
        if state.is_empty() {
            return;
        }
        if self.current.len() != state.len() {
            self.deltas.clear();
            self.bytes = 0;
            self.current = state;
            return;
        }

        self.scratch.clear();
        encode_delta(&state, &self.current, &mut self.scratch);
        self.bytes += self.scratch.len();
        self.deltas.push_back(std::mem::take(&mut self.scratch));
        self.current = state;

        // Drop from the old end until it fits. The oldest differences are the furthest
        // back in time, so a full ring shortens how far rewind reaches rather than
        // corrupting what it holds.
        while self.bytes > self.budget.ring_bytes {
            match self.deltas.pop_front() {
                Some(d) => self.bytes -= d.len(),
                None => break,
            }
        }
    }

    /// Step back one capture and return the state there, or `None` at the end of the ring.
    ///
    /// The first call hands back the newest state; each one after that reconstructs the one
    /// before it. The slice is valid until the next call. States handed out are gone from
    /// the ring — rewinding spends it.
    pub fn pop(&mut self) -> Option<&[u8]> {
        if self.current.is_empty() {
            return None;
        }
        if !self.handed_out {
            self.handed_out = true;
            return Some(&self.current);
        }
        let delta = self.deltas.pop_back()?;
        self.bytes -= delta.len();
        apply_delta(&mut self.current, &delta);
        Some(&self.current)
    }
}

/// Write `new ^ old` as alternating runs: how many bytes match, how many differ, then the
/// XOR of the ones that differ. Lengths are LEB128, so a run costs one byte until it is
/// longer than 127.
///
/// `new` and `old` must be the same length; the caller checks.
fn encode_delta(new: &[u8], old: &[u8], out: &mut Vec<u8>) {
    const W: usize = std::mem::size_of::<usize>();
    let n = new.len().min(old.len());
    let mut i = 0;
    while i < n {
        // Almost all of a state is unchanged, so this loop is where the time goes: a
        // megabyte scanned a byte at a time costs 29% of a frame on the device's A53, and
        // a word at a time costs a fraction of that. Measured by tests/rewind_cost.rs.
        let same_start = i;
        while i + W <= n {
            let a = usize::from_ne_bytes(new[i..i + W].try_into().unwrap());
            let b = usize::from_ne_bytes(old[i..i + W].try_into().unwrap());
            if a != b {
                break;
            }
            i += W;
        }
        while i < n && new[i] == old[i] {
            i += 1;
        }
        let same = i - same_start;

        let diff_start = i;
        while i < n && new[i] != old[i] {
            i += 1;
        }
        let diff = i - diff_start;
        if diff == 0 {
            // Trailing run of matches: nothing more to record.
            break;
        }

        write_varint(same, out);
        write_varint(diff, out);
        for k in diff_start..i {
            out.push(new[k] ^ old[k]);
        }
    }
}

/// Undo `encode_delta` in place: XOR is its own inverse, so applying a difference to the
/// newer state produces the older one.
///
/// A truncated or malformed delta stops where it stops rather than panicking — this decodes
/// bytes that were in memory, but a bug here should cost a wrong picture, not the frontend.
fn apply_delta(buf: &mut [u8], delta: &[u8]) {
    let mut at = 0usize;
    let mut pos = 0usize;
    while pos < delta.len() {
        let Some((same, n)) = read_varint(&delta[pos..]) else {
            return;
        };
        pos += n;
        let Some((diff, n)) = read_varint(&delta[pos..]) else {
            return;
        };
        pos += n;

        at = match at.checked_add(same) {
            Some(a) if a <= buf.len() => a,
            _ => return,
        };
        if pos + diff > delta.len() || at + diff > buf.len() {
            return;
        }
        for k in 0..diff {
            buf[at + k] ^= delta[pos + k];
        }
        pos += diff;
        at += diff;
    }
}

fn write_varint(mut v: usize, out: &mut Vec<u8>) {
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// The value and how many bytes it took, or `None` if the buffer ends mid-number.
fn read_varint(b: &[u8]) -> Option<(usize, usize)> {
    let mut v = 0usize;
    let mut shift = 0u32;
    for (i, byte) in b.iter().enumerate() {
        if shift >= usize::BITS {
            return None;
        }
        v |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            return Some((v, i + 1));
        }
        shift += 7;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(new: &[u8], old: &[u8]) {
        let mut d = Vec::new();
        encode_delta(new, old, &mut d);
        let mut buf = new.to_vec();
        apply_delta(&mut buf, &d);
        assert_eq!(buf, old, "delta did not reconstruct the older state");
    }

    #[test]
    fn a_delta_reconstructs_the_older_state() {
        roundtrip(&[1, 2, 3, 4], &[1, 9, 3, 4]);
        roundtrip(&[0; 8], &[0; 8]);
        roundtrip(&[1, 2, 3], &[4, 5, 6]);
        roundtrip(&[], &[]);
        // A change at each end, which is where off-by-ones live.
        roundtrip(&[9, 0, 0, 0, 9], &[1, 0, 0, 0, 1]);
    }

    #[test]
    fn an_unchanged_state_costs_almost_nothing() {
        let a = vec![7u8; 1 << 20];
        let mut d = Vec::new();
        encode_delta(&a, &a, &mut d);
        assert!(d.is_empty(), "{} bytes for no change", d.len());
    }

    #[test]
    fn a_realistic_state_compresses_hard() {
        // A megabyte with thirty scattered bytes changed, which is what a Mega Drive
        // actually does over six frames.
        let old = vec![0xA5u8; 1 << 20];
        let mut new = old.clone();
        for i in 0..30 {
            new[i * 9973] ^= 0x5A;
        }
        let mut d = Vec::new();
        encode_delta(&new, &old, &mut d);
        assert!(d.len() < 200, "{} bytes for thirty changes", d.len());
        let mut buf = new.clone();
        apply_delta(&mut buf, &d);
        assert_eq!(buf, old);
    }

    #[test]
    fn a_malformed_delta_does_not_panic() {
        let mut buf = vec![1u8, 2, 3, 4];
        apply_delta(&mut buf, &[0xFF]); // varint that never ends
        apply_delta(&mut buf, &[0, 200]); // claims 200 changed bytes it does not have
        apply_delta(&mut buf, &[200, 1, 7]); // skips past the end
        assert_eq!(buf, vec![1, 2, 3, 4], "a bad delta changed the state");
    }

    #[test]
    fn varints_round_trip_at_the_boundaries() {
        for v in [0usize, 1, 127, 128, 255, 16383, 16384, usize::MAX >> 1] {
            let mut b = Vec::new();
            write_varint(v, &mut b);
            assert_eq!(read_varint(&b), Some((v, b.len())), "{v}");
        }
    }

    #[test]
    fn states_come_back_newest_first() {
        let mut r = Rewind::new(RewindBudget::DEFAULT);
        let states: Vec<Vec<u8>> = (0..5u8).map(|i| vec![i; 64]).collect();
        for s in &states {
            r.push(s.clone());
        }
        assert_eq!(r.depth(), 5);

        for want in states.iter().rev() {
            assert_eq!(r.pop().expect("a state"), &want[..]);
        }
        assert_eq!(r.pop(), None, "the ring should be spent");
        assert_eq!(r.depth(), 0);
    }

    #[test]
    fn rewinding_spends_the_ring_and_playing_forward_refills_it() {
        let mut r = Rewind::new(RewindBudget::DEFAULT);
        for i in 0..4u8 {
            r.push(vec![i; 32]);
        }
        assert_eq!(r.pop().unwrap()[0], 3);
        assert_eq!(r.pop().unwrap()[0], 2);
        assert_eq!(r.depth(), 2);

        // Playing on from here stores new states behind the ones still held. State 2 is
        // where the player resumed, so it stays on the chain: they really did pass through
        // it, and stepping back from the new state has to land there before going further.
        r.push(vec![20u8; 32]);
        assert_eq!(r.depth(), 4);
        assert_eq!(r.pop().unwrap()[0], 20);
        assert_eq!(r.pop().unwrap()[0], 2);
        assert_eq!(r.pop().unwrap()[0], 1);
        assert_eq!(r.pop().unwrap()[0], 0);
        assert_eq!(r.pop(), None);
    }

    #[test]
    fn the_budget_drops_the_far_end_not_the_near_one() {
        // Room for a handful of differences, each of which is a whole 64-byte state's worth
        // of change plus its varints.
        let mut r = Rewind::new(RewindBudget {
            interval_frames: 6,
            ring_bytes: 200,
        });
        for i in 0..40u8 {
            r.push(vec![i; 64]);
        }
        assert!(r.bytes() <= 200, "{} bytes over budget", r.bytes());
        assert!(r.depth() > 1, "the ring dropped everything");

        // What survives is the recent end: the first step back is still the state before
        // the last one.
        assert_eq!(r.pop().unwrap()[0], 39);
        assert_eq!(r.pop().unwrap()[0], 38);
    }

    #[test]
    fn a_state_that_changes_length_starts_the_chain_over() {
        let mut r = Rewind::new(RewindBudget::DEFAULT);
        r.push(vec![1u8; 64]);
        r.push(vec![2u8; 64]);
        r.push(vec![3u8; 128]); // the core changed shape
        assert_eq!(r.depth(), 1);
        assert_eq!(r.pop().unwrap(), &vec![3u8; 128][..]);
        assert_eq!(r.pop(), None);
    }

    #[test]
    fn capture_follows_the_interval() {
        let r = Rewind::new(RewindBudget {
            interval_frames: 6,
            ring_bytes: 1024,
        });
        assert!(r.should_capture(0));
        assert!(!r.should_capture(1));
        assert!(r.should_capture(6));
        assert!(r.should_capture(600));

        // An interval of zero would divide by zero; it means "never".
        let never = Rewind::new(RewindBudget {
            interval_frames: 0,
            ring_bytes: 1024,
        });
        assert!(!never.should_capture(0));
        assert!(!never.should_capture(99));
    }
}
