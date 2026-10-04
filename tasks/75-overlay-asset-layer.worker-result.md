# Task 75 — Per-geometry overlay asset layer: worker result

- Status: success
- Cumulative calls used: 1 of 2 (this is the first call)

## Card path, built-in registration, override and fallback

- `slot2::overlay` is public (one `pub mod overlay;` line in `lib.rs`). `card_overlay_path(card,
  platform, geometry)` builds exactly `System/Overlays/<PLAT>/<geometry>.png` from the registry's
  own spellings — `Platform::folder()` (`GB`, `GBC`, `GBA`, `NES`, `SNES`, `MD`, `SMS`) and
  `Geometry`'s `Display` (`640x480`, `720x480`, `720x720`). All 21 pairs were asserted against
  written-out names.
- `BuiltInOverlay { platform, geometry, id, png: &'static [u8] }` is `Clone + Copy + Debug +
  PartialEq + Eq`; `BUILT_IN_OVERLAYS` is public and empty (D-11's default is no overlay and the
  artwork is a later task's), so that task only fills the table with `include_bytes!` entries.
- `resolve`/`resolve_with(table, ..)` do metadata and table lookup only: a regular file becomes the
  card source and the pair's built-in stays as the fallback; a missing path or a **directory** at
  the path is not an override (a folder named `720x480.png` cannot fail a launch); no file means the
  built-in alone; neither means empty sources. A pair registered twice resolves to the first entry —
  no panic, no nondeterminism.
- The layer never draws from the wrong pair, and the card's picture always wins when it decodes:
  the uploaded bytes were asserted to be the card's pixels, not the built-in's.

## Decode: size, colour, alpha, oversized headers

- `png = "0.17"` moved to `[dependencies]` and its now-redundant `[dev-dependencies]` entry
  removed; nothing else was added. One decoder serves a path and an in-memory slice
  (`decode_png_file` / `decode_png_bytes`), and a test asserts both give the same picture.
- The header is read first and `(w, h)` must equal the panel exactly; only then is the output
  buffer sized. A PNG claiming 40000×40000 is refused with a message naming both sizes before any
  allocation (that buffer would have been 6.4 GB), and a 720×480 picture is refused for a 640×480
  panel rather than stretched. `width * height` and `* 4` are `checked_mul`s; the raw buffer size is
  cross-checked against the expected pixel count, and the decoded frame's own size and depth are
  checked again after `next_frame`.
- `Transformations::normalize_to_color8()` (the png crate's own EXPAND + STRIP_16) plus one explicit
  match produce straight RGBA8 for RGB, RGBA, grayscale, grayscale+alpha, palette with and without
  tRNS, and 16-bit input; every case was compared byte for byte, and a fully transparent pixel kept
  `alpha == 0` in the RGBA, grayscale+alpha and palette+tRNS cases. Nothing premultiplies or
  invents an alpha.
- Malformed input (empty, signature only, truncated data, not a PNG, header cut short) returns an
  error string, never a panic, and the messages contain neither the card path nor the file bytes.

## Lazy upload, cache, free and geometry

- `OverlayLayer` holds the resolved sources, one texture, which source it came from, the last error
  and a "this set has had its one attempt" flag; `Default` is nothing at all. Setting the same
  value again is not a change: no free, no decode, no upload.
- A source change marks the set pending; the next canvas access frees the old texture **exactly
  once**, then attempts the new one lazily. Switching to empty sources frees it too, and one more
  frame then costs nothing. `release(canvas)` frees once and empties everything; calling it twice
  does not double-free, and after it `draw` does nothing. No `Drop` reaches for GL.
- `draw` orders things so the free waits for a canvas that owns the texture. A canvas whose size is
  not the sources' geometry records an error and performs no upload and no draw — and the attempt is
  not spent, so the panel's own canvas afterwards still uploads and clears the error.
- Success is exactly one plain `Canvas::image` per draw at `(0, 0, panel_w, panel_h)`, full UV,
  white tint — no clear, rect, effect, update or game placement. Three frames from one source set
  drew three images from exactly one upload, and the same set re-applied uploaded nothing.
- A failed card picture keeps its error and falls back to the built-in once; with both broken there
  is no upload and no image, both failures stay readable, and the next frame does not retry. Changing
  the card file on disk does **not** re-trigger a decode while the sources are unchanged; an empty
  set and back is a new attempt, which then uses the now-good card picture.
- Accessors: `used()` (`Card`/`BuiltIn`/`None`), `error()` (card fallback included, `None` when the
  sources drew or when there was nothing to try), `texture()`.

## Acceptance commands (run last, in this order, after the final code change)

| command | exit | last result line |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | (no output) |
| `cargo test -p slot2 --test overlay_layer` | 0 | `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s` |
| `cargo check -p slot2 --tests` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 8.40s` |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.33s` |
| `cargo clippy -p slot2 --test overlay_layer -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.33s` |

`cargo fmt --all` was applied (the new files) and clippy's `chunks_exact_to_as_chunks` findings in my
own decoder and test were fixed with `as_chunks`; the whole sequence was then run again in order, and
nothing was edited afterwards. Workspace tests, real GL tests and device deployment were not run.

## Files

- New: `crates/slot2/src/overlay.rs`, `crates/slot2/tests/overlay_layer.rs` (19 tests).
- Modified: `crates/slot2/src/lib.rs` (one `pub mod` line), `crates/slot2/Cargo.toml` (`png` moved
  from dev- to normal dependencies; the empty dev table removed).
- `Cargo.lock`: one added line, `slot2-retro` in the `slot2` package's dependency list. That entry
  is not new information — the manifest has had the dependency for several tasks and the lock was
  stale; regenerating the entry for `slot2` (which editing its manifest forced) wrote it down. `png`
  stayed at 0.17.16 with no version change. Nothing outside the allowed list was touched, and
  unrelated uncommitted changes were left alone.

## Out of scope, confirmed not implemented

- No Session/App fields, no game-draw ordering, no source swap on launch or eject, no
  `GameSettings::overlay` interpretation, no platform overlay default in the registry, no Display or
  Overlay menu, no translation, no instant preview or save toast, no shader/scale/overscan change,
  and no `slot2-gfx`, `slot2-ui`, `slot2-store`, `slot2-platform` or `slot2-retro` change. No real
  `assets/overlays/` sample PNG was created; every test fixture is generated in the test, and no
  build or distribution script was touched.

## Contract concern / remaining risk (1 line)

`BUILT_IN_OVERLAYS` is empty by design, so nothing in this build resolves to a picture yet: the
fallback path, the "first registration wins" rule and the whole texture lifecycle are only
exercised through `resolve_with` and test-local tables until the samples task fills the real one.

## Time

Start ~15:53, finish ~16:08 (Asia/Seoul) — about 15 minutes, one call.
