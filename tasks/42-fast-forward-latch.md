# Task 42 - Wire the R2 double-tap fast-forward latch

Work directly in the current checkout. Finish the decided in-game time-control gestures by
wiring the `Action::DoubleTap(Button::R2)` that `slot2-input` already emits. Holding R2 already
fast-forwards only while held; a double tap must toggle a session-scoped fast-forward latch.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\42-fast-forward-latch.md
- C:\SLOT2\crates\slot2\src\app.rs, only App fields/construction, `feed`/`tick`, `run_frame`,
  `stop_session`, `act`, and nearby tests
- C:\SLOT2\crates\slot2\src\session.rs, only `run_frame`, `speed`, `set_speed`, and
  `rewind_step`
- C:\SLOT2\crates\slot2-input\src\gestures.rs
- C:\SLOT2\crates\slot2-input\tests\input.rs, only gesture tests
- C:\SLOT2\crates\slot2\tests\session.rs, only fast-forward/rewind tests and their fixture

Read a directly related App integration-test helper only when needed. Do not read handoff,
milestone history, old task reports, logs, or repository history. The complete contract follows.

## Existing behavior

- `GestureConfig::default()` enables double tap only for R2. MENU is deliberately absent, so a
  MENU tap is emitted immediately and D-23's old MENU-double-tap action is already gone.
- `Gestures` emits raw Down/Up for every button. A configured R2 first tap is delayed as a UI
  `Tap`, but the raw `State` still reflects the physical R2 press immediately.
- `App::run_frame` currently sets `Session` speed to `FAST_FORWARD` (4) while R2 is physically
  held and back to 1 after release. L2 rewind takes priority over forward execution.
- `Action::DoubleTap(Button::R2)` is currently ignored. `Session` already performs silent extra
  core frames and caps speed; do not change that implementation.

DESIGN section 7 fixes R2 hold as momentary fast-forward and R2 double tap as fast-forward lock.
D-23 keeps MENU tap/hold, SELECT chords, and L2/R2 gestures while discarding MENU double tap.

## Contract

1. Add the smallest private App boolean for a fast-forward latch. Keep it outside `Screen` and
   do not add a generic mode/action framework.
2. While exactly `Screen::Playing` with a live session, `DoubleTap(R2)` toggles the latch.
   Double taps on the shelf, during insert/eject, or under power/in-game/state-switcher screens
   do nothing and must not arm the next session.
3. Effective fast-forward is active when R2 is physically held **or** the latch is on. A normal
   press accelerates immediately; releasing a single tap returns to speed 1. Once a double tap
   turns the latch on, releasing R2 keeps speed at `FAST_FORWARD`.
4. A later R2 double tap turns the latch off. Its second press is still a physical hold, so it
   may remain fast until that press is released; after release the speed must be 1.
5. L2 rewind keeps priority. While L2 is held, step rewind instead of running forward even if
   R2 or the latch requests fast-forward. Releasing L2 resumes the still-latched fast-forward.
6. The latch belongs to one Session. Clear it whenever `stop_session` ends or attempts to end a
   session, covering MENU-hold eject, menu Eject, reboot/poweroff, and later callers. A newly
   launched game must start unlatched.
7. Opening and closing the in-game menu pauses execution as before and retains the latch for the
   same session. Quick save/load and state-switcher visits must not toggle or clear it.
8. Preserve `FAST_FORWARD = 4`, Session audio discard/capping, rewind storage, raw button state,
   sink behavior, Resume, save states, and all existing UI behavior.
9. Do not change `slot2-input`: its R2-only double-tap and immediate MENU semantics are already
   the required contract. Update the App state-machine/module comment to mention the R2 latch.

This task intentionally adds no visual badge. Task 43 will display the effective rewind and
fast-forward state after this behavior is stable.

## Tests

Add focused tests proving at least:

- one physical R2 hold makes a running Session speed 4 immediately and release restores 1;
- R2 double tap locks speed 4 after release, and another double tap plus release restores 1;
- a double tap outside Playing does not affect the next launched session;
- L2 rewind wins while held and the R2 latch resumes after L2 release;
- opening/closing the in-game menu retains the latch but runs no frames while open;
- every path through `stop_session` clears the latch, and the next Session starts at speed 1;
- MENU remains immediate tap/hold with no double-tap delay or action, and SELECT+L1/R1 behavior
  remains covered by existing tests.

Use explicit `Instant` values. Prefer small private App unit tests for pure transitions and one
real-core integration test for actual Session speed/frame behavior. Use the established explicit
real-core skip pattern if the fixture is unavailable; pure latch/screen/reset tests must remain
active without a core. Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\tests\time_controls_app.rs (new, preferred for real-core wiring)
- C:\SLOT2\tasks\42-fast-forward-latch.worker-result.md

Do not modify `slot2-input`, `Session`, `slot2-ui`, audio/runtime loops, language files,
manifests, or documentation. Preserve unrelated uncommitted changes from earlier tasks. Do not
clean, revert, or reformat unrelated files.

## Out of scope and forbidden

- No HUD badge, toast, sound, setting, speed selector, persistent preference, or new message.
- No changes to rewind encoding/budget, fast-forward factor/audio, button mapping, hold/double
  timing, MENU behavior, SELECT chords, Resume, or numbered states.
- No workspace-wide test, device distribution build, Raspberry Pi test, or RG SP access.
- No adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC/BAI/OpenCodex/Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-input
cargo clippy -p slot2 -p slot2-input --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\42-fast-forward-latch.worker-result.md` with at most about 30 lines:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Changed files and exact momentary/latch/rewind/reset behavior.
- Each validation command, exit code, and concise test result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste source code or logs. A path/option error or tool timeout counts as an invocation.
Stop at forty-five minutes and write a partial report; do not make a third invocation. Do not
change models or configuration.
