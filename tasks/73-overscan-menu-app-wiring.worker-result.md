# Task 73 — Overscan menu App wiring: worker result

- Status: success
- Cumulative calls used: 1 of 2 (this is the first call)

## Registry-driven conditional Display rows and layout

- `DisplayChoice::Overscan` added. Two fixed arrays: `ROWS` (5: platform default, integer, aspect
  fit, fill, shader) and `CROPPING_ROWS` (the same five, then overscan). `DisplayMenu::new(scale,
  overscan_available)` keeps availability, `choices()` returns the row slice, and `choice`,
  `up`, `down` and `draw` all read that one slice — a row cannot be drawn that navigation refuses
  or reached without being drawn.
- `box_h()` is 280 without the row and 316 with it (`BOX_H + ROW_H`); `box_origin(ctx)`/`row_y(ctx, i)`
  are instance methods on the current row set; `BOX_W`, `PAD`, `ROW_H` and the `box_y + PAD + 28`
  row start are unchanged. The overscan row reuses `overscan-title`.
- No platform enum, NES name or registry type entered `slot2-ui`.
- App: `Screen::Overscan(InGameMenu, DisplayMenu, OverscanMenu)` (all three menus `Copy`, so `Screen`
  stays `Copy`/`Debug`/`Eq`). `open_display_menu` converts the cart's platform with the existing
  `retro_platform` and computes `def(platform).overscan != Overscan::NONE`. Display A routes scale
  → existing commit, shader → existing screen, overscan → `open_overscan_menu` (which returns
  without a session, before any card read). Overscan Up/Down wrap the three rows; B/MENU return to
  the same Display Overscan row; any other button does nothing. The screen is in `audio_paused`,
  the no-wallpaper guard, the frame/overlay draw arm and both exhaustive matches; `run_frame` only
  advances on `Playing`, so the core stays paused and no sink request is made.

## Settings: write first, then runtime

- `Session::overscan_for(platform, setting)` is the single mapping: `None | Some(true)` →
  `PlatformDef::overscan`, `Some(false)` → `Overscan::NONE`. Launch initialisation and the new
  `set_overscan_setting(Option<bool>)` both call it; the old `set_overscan(bool)` now delegates to it.
  `overscan()` reads the crop in force.
- `commit_overscan` writes the whole `GameSettings` with only `overscan` replaced (core, scale,
  rewind, shader and hand-written unknown keys preserved) and applies `set_overscan_setting` only
  after the write succeeds. Success stays on the Overscan screen and row; the next draw reworks
  placement, aspect and UV from the new crop over the same last frame — no success toast, frame,
  restart, texture upload/update/free or audio change. Failure logs and toasts
  `overscan-save-failed`, keeps the card bytes and the prior crop and stays on the attempted row.
- Real NES results: default and explicit crop draw UV `[0, 8/240, 1, 232/240]` at 256×224 visible;
  `Some(false)` draws `[0, 0, 1, 1]` at 256×240; placement matches `slot2_gfx::place` for each
  visible size and the two differ. `Platform default` removes only the overscan key (and an
  overscan-only ini with it); explicit choices leave canonical `overscan = on` / `overscan = off`.

## Preserved behaviour

- GBA Display still has exactly the five rows and the 280 box; walking two laps cannot reach an
  overscan row. All pre-existing scale/Shader selection, saving, failure and unreadable-file tests
  pass unchanged against the dynamic constructor.
- Overscan draw order is the session's last frame (through the current shader) then the overscan
  overlay only: no parent Display/InGame menu, no wallpaper, no switcher card, no HUD band, no
  clear; hold progress and toast still come after the menu.
- Pause/audio/sink/texture invariants are asserted after every overscan change (frames, core state,
  last-frame bytes, texture upload/update/free counts, audio health, sink request).
- English `overscan-save-failed = Could not save the overscan setting`, Korean
  `오버스캔 설정을 저장하지 못했습니다`, asserted directly in both packs.

## Acceptance commands (run last, in this order, after the final code change)

| command | exit | last result line |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | (no output) |
| `cargo test -p slot2-ui --test display_menu` | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 81.30s` |
| `cargo test -p slot2 --test session --test display_menu_app` | 0 | `display_menu_app`: `28 passed; 0 failed; 0 ignored`, then `session`: `43 passed; 0 failed; 0 ignored; finished in 54.47s` |
| `cargo test -p slot2 -p slot2-ui -p slot2-i18n` | 0 | last crate `Doc-tests slot2_ui`: `0 passed; 0 failed; 0 ignored`; summed 492 passed / 0 failed / 0 ignored (slot2 252, slot2-ui 210, slot2-i18n 30) |
| `cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.69s` |

Core-dependent skips: 0. `vendor/fceumm_libretro.dll` and `mgba_libretro.dll` are both present, so
every NES and GBA case ran; no test was ignored or weakened. Workspace tests, GL tests and device
deployment were not run.

Two clippy findings in my own test code (`type_complexity` on two case tables, `needless_update`)
were fixed with type aliases and a complete struct literal, after which the whole sequence above was
run again in order. No code was edited after the final clippy run.

## Files

- Modified: `crates/slot2-ui/src/display_menu.rs` (dynamic row set, `Overscan` choice, `box_h`),
  `crates/slot2-ui/tests/display_menu.rs`, `crates/slot2/src/session.rs`,
  `crates/slot2/src/app.rs`, `crates/slot2/tests/session.rs`,
  `crates/slot2/tests/display_menu_app.rs`, `assets/lang/en.ftl`, `assets/lang/ko.ftl`,
  `crates/slot2-i18n/tests/i18n.rs` (one assertion for the new failure key).
- No new file; no `lib.rs` change was needed (`OverscanMenu` was already exported). Nothing outside
  the allowed list was touched, and unrelated uncommitted changes were left alone.
- Report: this file.

## Out of scope, confirmed untouched

- Overlay selector (still no runtime or store contract, and not claimed as implemented anywhere).
- Real-hardware picture quality on the Pi/RG SP, adb, SD card and device deployment: not reached.

## Contract concern / remaining risk (1 line)

Row availability is exactly `def(platform).overscan != Overscan::NONE` read at open time, so the
Display row count and box height move with any future registry crop change (intended, but only GBA
and NES are pinned by tests); the NES UV expectations in `display_menu_app` also depend on FCEUmm
continuing to produce a 256×240 frame, which is asserted in the fixture rather than assumed silently.

## Time

Start ~12:20, finish ~13:18 (Asia/Seoul) — about 58 minutes, one call.
