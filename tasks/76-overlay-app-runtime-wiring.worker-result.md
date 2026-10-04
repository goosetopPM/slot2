# Task 76 — Overlay App runtime wiring: worker result

- Status: success
- Cumulative calls used: 1 of 2 (this is the first call)

## Setting and geometry, and how often the launch resolves

- `overlay::overlay_enabled(Option<bool>)` is the one boundary: an exhaustive match with
  `None => false`, `Some(true) => true`, `Some(false) => false`, so the inheritance is written
  rather than hidden in `unwrap_or(false)`.
- `overlay::geometry_for_panel((u32, u32)) -> Option<Geometry>` matches the three supported panels
  as numbers and returns `None` for anything else — no string formatting, parsing or profile-name
  guessing. An unknown panel means no overlay and never a failed launch.
- `App` gained a private `OverlayLayer`, `Default` at construction. `resolve_overlay(cart)` runs
  once per successful launch: one `read_settings`, one `geometry_for_panel`, and either
  `set_sources(resolve(card, cart.platform, geometry))` or `clear_sources()`. A failed launch
  clears the sources instead. The test that deletes both the settings file and the PNG *after* the
  launch still draws the overlay from the same texture with no new upload, which is what pins the
  resolution to the launch boundary rather than to a frame.

## Draw order, and the screens that are excluded

- One draw boundary, `draw_game_and_overlay`: `session.upload_video` → `session.draw` →
  `overlay.draw`. It is used by `Playing`, every game-bearing menu (`InGame`, `Display`, `Shader`,
  `Overscan`, `Cheats`, `Switcher`, `Core`, `Device`) and by `Power` when a session is running. The
  time-control badge, menus, dims, hold bar and toast all still come after it.
- Asserted on a real GBA frame: game first only after the session's clear, overlay plain `Op::Image`
  (never an `ImageEffect`) next, then the in-game menu's panel, the Display menu's dim, and the
  Power menu's dim.
- `Inserting` draws no overlay even though the session is already up under the animation: the test
  catches that exact frame and asserts no panel-sized upload and no game effect, then asserts the
  overlay appears on the first real `Playing` frame.
- A draw with no session gives the canvas to the layer with empty sources — `draw_overlay_nothing`
  on `Splash`, `List`, `Ejecting`, `Inserting`-without-a-session and `Power` without a session — so
  a pending texture is freed there and no overlay is ever drawn on its own. A game-bearing screen
  constructed with no session draws no picture and makes no texture at all.

## Cache, core change and teardown

- Repeated frames from one session draw only the overlay image: the texture is uploaded once, and
  the warm-frame test asserts no further upload and no free across three frames.
- A real mGBA→gpSP change (both libraries copied to a temp core directory) keeps the same overlay
  texture: same tex id after the switch, no upload, no free. A switch whose target refuses to load
  falls back to mGBA and keeps the overlay the same way.
- A recovery that itself fails — reached with a cheat file the card will not hand over, so every
  later session start is refused — leaves no session, and the next frame frees the overlay texture
  exactly once and draws no overlay; the frame after that is not a second free. `stop_session`
  (MENU hold, InGame Eject, exit) clears the sources, and no `Drop` implementation tries to free GL
  without a canvas.
- Missing, corrupt and wrong-panel card pictures all leave the game frame intact with no overlay
  upload and no overlay image.

## Preserved behaviour

- The game's own draw is unchanged by the overlay: with and without one, the game op is byte-for-byte
  the same `ImageEffect(Lcd3x)` with `[0, 0, 1, 1]` UV, the frame counts are equal, the game texture
  is uploaded once, and no sink request appears. The overlay adds exactly one draw and one panel-sized
  upload.
- Existing app/session/display draw assertions were not touched and still pass with the overlay
  disabled by default (no settings file, no PNG).

## Acceptance commands (run last, in this order, after the final code change)

| command | exit | last result line |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | (no output) |
| `cargo test -p slot2 --test overlay_layer --test overlay_app` | 0 | `overlay_layer`: `19 passed; 0 failed; 0 ignored; finished in 1.21s`; `overlay_app`: `16 passed; 0 failed; 0 ignored; finished in 12.12s` |
| `cargo test -p slot2` | 0 | last crate `Doc-tests slot2`: `0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`; summed **287 passed / 0 failed / 0 ignored**; core-dependent skips **0** (mGBA, gpSP and FCEUmm are all in `vendor/`, and `overlay_app` fails rather than skips when mGBA is missing) |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 3.68s` |
| `cargo clippy -p slot2 --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 16.38s` |

`cargo fmt --all` was applied once (the new test file), then the whole sequence ran in order and
nothing was edited afterwards. Workspace tests, real GL tests and device deployment were not run.

## Files

- Modified: `crates/slot2/src/overlay.rs` (the two pure helpers),
  `crates/slot2/src/app.rs` (the field, `resolve_overlay`, `draw_game_and_overlay` /
  `draw_overlay_nothing`, the draw arms, `stop_session`, the launch and recovery-failure arms, and
  the module's draw-order comment).
- New: `crates/slot2/tests/overlay_app.rs` (16 tests).
- No existing test file was modified, so no existing assertion was relaxed or deleted. No manifest,
  lockfile or asset was touched this task, and unrelated uncommitted changes were left alone.

## Out of scope, confirmed not implemented

- No Display/Overlay menu row, no separate overlay menu, no translation, no settings write and no
  save-failure toast; no way to turn the overlay on or off at run time and no instant preview; no
  production `BUILT_IN_OVERLAYS` entry and no real `assets/overlays/` PNG; no registry default; no
  PNG decoder, resolver, canvas or session-renderer redesign; no shader, scale, overscan, core or
  cheat change.

## Contract concern / remaining risk (1 line)

The teardown of a session that vanished (`recover_from_switch` failure) is only reachable in a test
through a refused cheat file, because a core library cannot be taken away while it is loaded on
Windows; the production path was not rearranged for the test, so that branch is covered by one route
plus the ordinary stop path.

## Time

Start ~16:44, finish ~17:06 (Asia/Seoul) — about 22 minutes, one call.
