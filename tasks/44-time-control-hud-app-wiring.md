# Task 44 - Wire the time-control HUD badge into App

Work directly in the current checkout. Connect Task 42's effective L2/R2 behavior to Task 43's
`slot2_ui::hud::TimeControl` badge. The badge must describe the exact control that `run_frame`
applies, and appear only over a live Playing game.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\44-time-control-hud-app-wiring.md
- C:\SLOT2\crates\slot2\src\app.rs, only module comment, `run_frame`, `draw`, time-control fields,
  `stop_session`, and Task 42 tests
- C:\SLOT2\crates\slot2\tests\time_controls_app.rs
- C:\SLOT2\crates\slot2-ui\src\hud.rs, only `TimeControl` and `draw_time_control`
- C:\SLOT2\crates\slot2-gfx\src recording-canvas definitions only if needed by the test

Read a directly related App drawing-test helper only when needed. Do not read Session internals,
input implementation, handoff, milestone history, old task reports, logs, or repository history.
The complete contract follows.

## Existing behavior

- Task 42 implemented a private App `ff_latch`. Physical R2 or the latch selects
  `FAST_FORWARD = 4`; L2 rewind takes priority; menus pause and preserve the latch; Session stop
  clears it.
- `run_frame` currently computes rewind and fast-forward directly from held buttons plus the
  latch. No visual status is drawn.
- Task 43 added `slot2_ui::hud::TimeControl::{Rewind, FastForward { speed }}` and
  `Hud::draw_time_control(..., Option<TimeControl>)`. It draws one localized panel-top-centre
  badge and draws nothing for `None`.
- App draws the Session frame first for Playing. InGame and Switcher draw that last frame under
  their overlay and deliberately hide game HUD. Shelf-side clock/battery HUD is separate.

## Contract

1. Add one small private App query for the **effective** time control. It returns `None` unless
   the screen is exactly `Screen::Playing` and a live Session exists.
2. In Playing, priority is:
   - physical L2 held → `TimeControl::Rewind`;
   - otherwise physical R2 held or `ff_latch` → `TimeControl::FastForward { speed:
     FAST_FORWARD }`;
   - otherwise `None`.
   L2 must therefore be the badge and execution mode when L2 and R2/latch overlap.
3. Use that same query in both `run_frame` and `draw`; do not leave two independent copies of the
   held/latch priority rules that can drift. `run_frame` still performs rewind without advancing
   the core, or sets Session speed and runs normally/fast as appropriate.
4. Draw Task 43's badge after the Session game frame, so it is visible over the game. Draw it
   before transient overlays/toasts that App owns. Do not clear, redraw, or upload the game frame
   just for the badge.
5. Show no time-control badge on List, Splash, Inserting, Ejecting, Power, InGame, or Switcher,
   and none on a Playing screen with no Session. A latched fast-forward remains logically set
   under InGame/Switcher but its badge is hidden until returning to Playing.
6. A physical R2 press shows FastForward immediately and release hides it unless latched. An R2
   double tap keeps it visible after release; the unlatching double tap hides it after its final
   release. Physical L2 temporarily replaces that badge with Rewind and release restores a live
   fast-forward latch.
7. `draw` must not advance frames, consume rewind history, toggle/clear the latch, alter Session
   speed, request a sink, or change screen state. It only maps current state to the UI method.
8. Preserve the Task 42 speed factor, audio behavior, Session-stop reset, input gestures, Resume,
   save states, menu pause behavior, corner clock/battery HUD, toast order, and Task 43 UI API.
9. Update the App module drawing comment so it states that the time-control badge is drawn over
   Playing only and hidden under menus/switcher.

Do not expose `ff_latch`, add a public App status API, or make `slot2-ui` depend on App/input.

## Tests

Extend `time_controls_app.rs` and add a small private App test only if needed. Prove at least:

- normal Playing draws the game frame with no time badge;
- physical R2 draws the FastForward badge immediately, and release removes it;
- latched R2 keeps the FastForward badge after release and unlatching removes it;
- L2 replaces a live FastForward badge with Rewind, and release restores FastForward;
- InGame and Switcher hide the badge while preserving the latch; returning to Playing shows it
  again;
- the badge operations occur after the game frame operations, remain at Task 43's top-centre
  geometry, and repeated draw does not create new glyph uploads after warm-up;
- drawing any of these states leaves frame count, screen, latch, Session speed, sink request, and
  rewind history unchanged;
- existing Task 42 momentary/latch/rewind/reset tests continue to pass.

Use `RecordingCanvas`, explicit `Instant`s, and the established real-core fixture/skip pattern.
Keep a pure screen/no-Session test active without a core; real game-frame ordering can skip only
when the core fixture is unavailable. Identify the badge by its Task 43 geometry/text operations,
not a brittle texture id. Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\tests\time_controls_app.rs
- C:\SLOT2\tasks\44-time-control-hud-app-wiring.worker-result.md

Do not modify `slot2-ui`, language files, i18n tests, Session, `slot2-input`, `slot2-store`, gfx,
audio/runtime loops, manifests, or documentation. Preserve unrelated uncommitted changes from
earlier tasks. Do not clean, revert, or reformat unrelated files.

## Out of scope and forbidden

- No new badge design, message, animation, toast, sound, speed selector, setting, or persistence.
- No change to fast-forward factor/audio, rewind encoding/budget, latch gestures/reset, button
  mapping, MENU/SELECT behavior, Resume, or numbered states.
- No generic status framework or public App inspection API.
- No workspace-wide test, device distribution build, Raspberry Pi test, or RG SP access.
- No adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC/BAI/OpenCodex/Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-ui
cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\44-time-control-hud-app-wiring.worker-result.md` with at most about 30
lines:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Changed files and the final shared priority query, draw visibility/order, and side-effect rules.
- Each validation command, exit code, and concise test result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste source code or logs. A path/option error or tool timeout counts as an invocation.
Stop at forty-five minutes and write a partial report; do not make a third invocation. Do not
change models or configuration.
