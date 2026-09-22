# Task 03 — implement `slot2-gfx` (GL canvas, host window, device framebuffer)

Repository: `C:\SLOT2` (Rust workspace, Windows host). Reference code to port from:
`C:\Users\gyuha\slot-2\crates\slot-gfx\src\` (MIT licensed, same author lineage — copying
with a one-line attribution comment is expected, not just allowed).

## Goal
All of these must succeed, in this order:

```
cargo test -p slot2-gfx                                   # pure tests, GL test skipped
$env:SLOT2_GFX_TEST="1"; cargo test -p slot2-gfx -- --nocapture   # GL test runs for real, writes target/gfx-screenshot.png
cargo clippy -p slot2-gfx --all-targets -- -D warnings
cargo clippy -p slot2-gfx --no-default-features --features device -- -D warnings
```

The second command opens a real window briefly; that is expected.

## Rules
- **Do not edit** `crates/slot2-gfx/src/lib.rs`, `canvas.rs`, `fit.rs`, `Cargo.toml`,
  `tests/screenshot.rs`, or any other crate. They are the contract.
- Edit only these four files, replacing every `todo!()` and the `_todo: ()` placeholder
  fields. Read the module doc at the top of each one first — it is the spec:
  - `crates/slot2-gfx/src/glfn.rs` — GL loader
  - `crates/slot2-gfx/src/gl_canvas.rs` — offscreen panel framebuffer, batched quads, present, read_back
  - `crates/slot2-gfx/src/host.rs` — winit/glutin window, caller-driven `pump()`
  - `crates/slot2-gfx/src/fbdev.rs` — port of the original device EGL surface
- Dependencies are fixed: `gl 0.14`, `libloading 0.8`, `winit 0.30`, `glutin 0.32`,
  `glutin-winit 0.5`, `raw-window-handle 0.6`. Do not add any. If you believe `libc` is
  unavoidable for the fbdev ioctl, declare the two or three `extern "C"` functions you need
  yourself instead (the original does this).
- `unsafe` is allowed only for GL/EGL/ioctl calls, each in as small a block as practical.
  No `unwrap()`/`expect()` on anything that can fail at runtime (context creation, shader
  compile, file open) — return `GfxError`.
- Keep GLSL to ES 1.00 (`attribute`, `varying`, `gl_FragColor`, `texture2D`), no
  `#version` line, precision behind `#ifdef GL_ES`.
- The `Canvas` trait and `RecordingCanvas` in `canvas.rs` define exact semantics (alpha8
  = tint.rgb at coverage*tint.a; uv top-left origin; y down). `fit::integer_fit_rect` is
  what `present` must use for the blit rectangle.

## Verification tips
- The GL test asserts exact pixels at known coordinates. If it fails, print the pixel it
  found (the assertion messages already do) and look at `target/gfx-screenshot.png`.
- A wrong y direction shows up as "below red rect" failing; a wrong alpha8 expansion as
  "mask top"; a missing row flip in `read_back` as most assertions failing at once.
- If desktop GL refuses an ES context on this machine, the fallback GL context must still
  accept the ES 1.00 shader (Windows drivers do). If not, report the driver log.

## Definition of done
Paste the last lines of all four commands into your final answer and confirm that
`target/gfx-screenshot.png` exists. If a contract file must change, stop and explain
which one and why instead of editing it.
