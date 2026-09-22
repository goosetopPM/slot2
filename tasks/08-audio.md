# Task 08 — implement `slot2-audio`

Repository: `C:\SLOT2` (Rust workspace, Windows host). ALSA reference to port from:
`C:\Users\gyuha\slot-2\crates\slot\src\audio\alsa.rs` (MIT, Brandon T. Kowalski).

## Goal
```
cargo test -p slot2-audio --features host
cargo clippy -p slot2-audio --all-targets --features host -- -D warnings
cargo clippy -p slot2-audio --no-default-features --features device -- -D warnings
```
All three clean. (The real-device test skips itself unless `SLOT2_AUDIO_TEST=1`.)

## Rules
- **Do not edit** (contract): `crates/slot2-audio/src/lib.rs`, `tests/audio.rs`,
  `Cargo.toml`, other crates.
- Edit only, replacing every `todo!()` and `_todo` field:
  - `src/ring.rs` — lock-free SPSC ring
  - `src/resample.rs` — linear stereo resampler
  - `src/volume.rs` — perceptual volume with sticky mute
  - `src/alsa.rs` — device sink (port; unix-only behind `#[cfg(unix)]`)
  - `src/host.rs` — cpal sink
- Each file's module doc is the spec: read it first. The tests pin the numbers
  (power-of-two capacity, drop-on-full, silence-on-underrun, `gain = (level/100)^2`,
  ±3 frames of drift over 100 resampler calls).
- Dependencies are fixed: `libloading 0.8`, `cpal 0.15` (host feature only). No `libc`,
  no `crossbeam`, no `rubato`.
- `unsafe` only for: the ring's shared buffer (`UnsafeCell` + atomics, with a comment
  naming the SPSC invariant) and the ALSA FFI. Nothing else.
- No `unwrap()`/`expect()` on device or library operations — return `Error`.

## Ring hints
```rust
struct Shared { buf: UnsafeCell<Box<[i16]>>, mask: usize, head: AtomicUsize, tail: AtomicUsize }
unsafe impl Send for Shared {} unsafe impl Sync for Shared {}   // SPSC: one writer, one reader
```
`head` = next write index, `tail` = next read index, both free-running; `len = head - tail`,
`space = capacity - len - 1`. Producer: copy, then `head.store(new, Release)`. Consumer:
`head.load(Acquire)`, copy, then `tail.store(new, Release)`. Handle the wrap with two
`copy_from_slice` halves.

## cpal 0.15 hints
- `cpal::traits::{DeviceTrait, HostTrait, StreamTrait}`
- `device.build_output_stream(&config.into(), move |data: &mut [f32], _| { … }, err_fn, None)?`
- `stream.play()?` after building; keep `stream` in the struct.

## Definition of done
Paste the last lines of all three commands. If a contract file must change, stop and
explain which one and why instead of editing it.
