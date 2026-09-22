# Task 04 — faces, spans, splash screen, and the two app loops

Repository: `C:\SLOT2` (Rust workspace, Windows host).

## Goal
All of these must succeed:

```
cargo test -p slot2-ui                                          # RecordingCanvas tests; GL test skipped
$env:SLOT2_GFX_TEST="1"; cargo test -p slot2-ui -- --nocapture  # GL splash test, writes target/splash-screenshot.png
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p slot2 -p slot2-gfx -p slot2-ui --no-default-features --features device -- -D warnings
cargo build -p slot2                                            # host binary builds
```

Then run the host binary once for ~2 seconds and confirm a window appears with the splash
(`cargo run -p slot2` and close it; or `$env:SLOT2_LANG="ko"; cargo run -p slot2`).

## Rules
- **Do not edit** (contract): `crates/slot2-ui/src/lib.rs`, `layout.rs`, `tests/splash.rs`,
  `Cargo.toml` files, `crates/slot2/src/main.rs`, `diag.rs`, anything in `slot2-gfx`,
  `slot2-text`, `slot2-i18n`, `slot2-platform`, `assets/`.
- Edit only, replacing every `todo!()`:
  - `crates/slot2-ui/src/face.rs` — FaceCache, `face`, `measure`, `draw_text`, `draw_spans`
    (you may add a `spans_width` helper and private items)
  - `crates/slot2-ui/src/splash.rs` — `Splash::draw`
  - `crates/slot2/src/host_app.rs` — window loop
  - `crates/slot2/src/device_app.rs` — framebuffer loop
- Read the module doc at the top of each file first; it is the spec, including the exact
  layout numbers and the button-cap formula the tests check.
- No new dependencies. No `unsafe`. No `unwrap()` on anything that can fail at runtime in
  the app loops (surface/canvas creation, present) — print `slot2: <error>` and return.
- Existing public APIs you will use (read their docs; do not modify them):
  - `slot2_text::FontChain::{measure, rasterize}` → `Metrics { width, ascent, descent, line_height }`, `Bitmap { width, height, baseline, data }`
  - `slot2_gfx::Canvas::{clear, rect, image, upload_alpha8}`, `Color::{with_alpha, from_rgb8}`
  - `slot2_i18n::{I18n::{t, t_args, spans}, Span, Button::label, Arg}`
  - `slot2_ui::{UiCtx, SafeArea::{px, py, centre_x, panel_centre_x}, PX_*}`
  - `slot2_gfx::{HostSurface::{open, pump}, HostEvent, KeyCode, GlCanvas::{new, present}, FbdevSurface::open, Surface::size}`
  - `slot2_platform::Geometry::{size, parse}` and `Profile`

## Verification tips
- `faces_upload_once_and_draw_many` fails if you upload on `measure` or re-upload a warm
  face. `spans_lay_out_text_and_button_caps_in_order` pins the cap geometry to the formula
  in face.rs's doc: rect at `pen + gap`, width `label_w + 2*pad`, label at `pen + gap + pad`.
- `splash_clears_then_draws_inside_the_safe_area_on_every_geometry` runs for 640x480,
  720x480 and 720x720; use `ctx.safe` for every coordinate, never literal offsets.
- `splash_debug_frame_outlines_the_safe_area` expects exactly the four 1-px rects listed
  in the test (top at y, bottom at y+479, left at x, right at x+639).
- The GL test looks at `target/splash-screenshot.png`; open it if pixel asserts fail.

## Definition of done
Paste the last lines of each command's output into your final answer, confirm the window
appeared with the wordmark and greeting, and confirm `target/splash-screenshot.png` exists.
If a contract file must change, stop and explain which one and why instead of editing it.
