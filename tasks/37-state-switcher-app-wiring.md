# Task 37 - Wire the state switcher into the running app

Work directly in the current checkout. Connect the completed `StateSwitcher` to the in-game
menu and load the selected numbered state into the active session. Keep deletion and the
30-second undo system out of this task.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\37-state-switcher-app-wiring.md
- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\src\session.rs
- C:\SLOT2\crates\slot2-ui\src\state_switcher.rs
- C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs
- C:\SLOT2\crates\slot2\tests\ingame_menu_app.rs
- C:\SLOT2\crates\slot2\tests\quick_state_app.rs

Read directly related runtime or test helpers only when needed. Do not read the full handoff,
milestone history, old task reports, or repository history. The relevant contract is below.

## Existing behavior

- `Screen::InGame(InGameMenu)` pauses core advancement and audio while drawing the menu over
  the session's last frame.
- A on `InGameChoice::SaveState` currently stays in the in-game menu.
- `StateSwitcher::new(Vec<StateSlot>)` filters Resume, sorts numbered slots, selects the
  greatest number, navigates left/right, and draws a cached overlay without clearing.
- `Session::load_state` loads a numbered state and resets audio and rewind history.
- Quick load already uses `state-loaded`, `states-empty`, and `state-load-failed` feedback.
- `Screen` is Copy/Debug/Eq and existing tests rely on that lightweight state contract.

## Contract

1. Add a screen state for the active state switcher without putting the cache-owning
   `StateSwitcher` inside `Screen`. Preserve `Screen`'s existing Copy/Debug/Eq behavior. Store
   the one reusable `StateSwitcher` in `App`; the screen variant may carry the `InGameMenu`
   value needed to return to the same selected row.
2. Add the smallest `StateSwitcher` API needed to replace its source slots. It must apply the
   same Resume filtering and numeric sorting as construction, select the greatest numbered
   slot, and handle an empty list. Reuse one normalization path rather than duplicating it.
   Keep the bounded thumbnail cache alive across refreshes; do not replace a previously drawn
   switcher with a fresh instance that abandons live texture ids.
3. A on `InGameChoice::SaveState` obtains the active session cart, calls `Card::list_states`,
   refreshes the reusable switcher, and enters it. Enter even when the numbered list is empty
   so the existing empty view is visible.
4. While the switcher is open, the core stays stopped and host/device audio stays paused through
   the existing app policy. No sink open/close request is made.
5. Draw the session's last frame first and then the switcher overlay. Do not draw wallpaper,
   HUD, the in-game menu, or any clear between them.
6. Left and Right call the switcher's navigation. Other directional buttons do nothing.
7. B or a MENU tap returns to `Screen::InGame` with the same menu selection, session, and sink.
8. A with a selected numbered slot calls `Session::load_state`. On success, show the existing
   `state-loaded` toast with its number and return directly to `Screen::Playing` with the same
   session and sink.
9. A in the empty switcher stays there and shows `states-empty`. It must not load Resume,
   create a state, or resume the game.
10. A load error stays in the switcher, keeps the session and sink, logs one concise error, and
    shows `state-load-failed`.
11. Power and volume retain their global behavior. Quick-save/load chords remain Playing-only.
12. Update the app state-machine comments for the new transitions.

Do not add an App-owned duplicate Cart or state list. Borrow or clone only the active Session's
existing Cart as needed to satisfy Rust borrowing.

## Tests

Add focused tests proving at least:

- A on the in-game Save State row enters the switcher and defaults to the greatest numbered
  slot while excluding Resume and ignoring mtime.
- empty state lists still enter the switcher; A reports `states-empty` and stays paused.
- Left/Right wrap, and B/MENU return to the same Save State menu row.
- the switcher reports audio paused, stops core frame advancement, preserves the session, and
  makes no sink request.
- drawing emits the game frame before the switcher dim without wallpaper, HUD, in-game-menu
  overlay, or clear.
- A loads the selected slot into a real core session, returns to Playing, reports
  `state-loaded`, and leaves the existing sink open.
- invalid state bytes report `state-load-failed`, stay in the switcher, and leave the session
  usable.
- refreshing after a later quick save exposes the new greatest slot without reconstructing the
  cache-owning app field. Test the UI refresh API directly where texture lifetime is easier to
  observe.

Use the established explicit real-core skip pattern. Keep pure state and rendering tests active
without a core. Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2-ui\src\state_switcher.rs only for the minimal slot-refresh API
- C:\SLOT2\crates\slot2-ui\tests\state_switcher.rs only for that API's regression coverage
- C:\SLOT2\crates\slot2\tests\state_switcher_app.rs (new, preferred integration tests)
- C:\SLOT2\crates\slot2\tests\ingame_menu_app.rs only if an existing assertion must recognize
  the new screen
- C:\SLOT2\crates\slot2\tests\quick_state_app.rs only if refresh integration belongs there
- C:\SLOT2\tasks\37-state-switcher-app-wiring.worker-result.md

Do not modify Session, slot2-store, slot2-input, audio/runtime loops, language assets, manifests,
or documentation. Preserve unrelated uncommitted changes. Do not clean or revert the tree.

## Out of scope and forbidden

- No state deletion, undo timer, retention/ring-size policy, new save operation, or Resume change.
- No visual redesign, new messages, input gesture, generic command framework, or settings file.
- No full workspace test or device distribution build.
- No hardware, adb, Samba, or SD-card access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
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

Write `C:\SLOT2\tasks\37-state-switcher-app-wiring.worker-result.md` with at most about 30
lines unless a failure needs more evidence:

- Success, failure, or partial completion and cumulative attempt number out of two.
- Changed files and final screen/load behavior.
- Each validation command, exit code, and concise result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste code, logs, or repeat this specification. A correction attempt must update the
report with only the new delta, new regression evidence, and final validation; do not repeat
the complete first-attempt report. If blocked, leave the code consistent, report the blocker,
and stop. Do not change models or configuration. This task has at most two total attempts.
