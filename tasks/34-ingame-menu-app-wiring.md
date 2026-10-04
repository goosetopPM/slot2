# Task 34 - Wire the in-game menu into the running app

Work directly in the current checkout. This task connects the completed `InGameMenu` UI to
the app state machine and both runtime loops. Keep the task limited to opening, navigating,
closing, drawing, pausing audio, and the already-supported eject path.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\34-ingame-menu-app-wiring.md
- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\src\host_app.rs
- C:\SLOT2\crates\slot2\src\device_app.rs
- C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs
- C:\SLOT2\crates\slot2-ui\tests\in_game_menu.rs

Read directly related app tests or input/audio interfaces only when needed. Do not reread the
full handoff, milestones, design history, old task reports, or repository history. The binding
contracts from D-23 are copied below.

## Existing behavior

- `Screen::Playing` owns an active `Session`.
- `App::run_frame` advances the core only while the screen is exactly `Screen::Playing`.
- A MENU hold while playing stops the session and enters the existing eject animation.
- The runtime loops already pause their audio sink while `Screen::Power(_)` is visible.
- `Session::draw` draws the last uploaded game frame.
- `InGameMenu` already draws a dimmed overlay without clearing the canvas.

## Contract

1. Add an app screen state containing `InGameMenu`.
2. A MENU tap while `Screen::Playing` opens a fresh default in-game menu. The existing MENU
   hold eject gesture must keep working.
3. While the in-game menu is open, the core does not advance. Preserve the existing
   `run_frame` guard rather than adding a fake pause API to the core.
4. Both host and device audio sinks pause while the in-game menu is open and resume after it
   closes. Keep the existing power-menu pause behavior. Put the screen policy in one app method
   if that avoids duplicating screen matching in both loops.
5. Drawing the in-game menu must first draw the session's last frame, then draw the menu
   overlay. Do not draw the shelf wallpaper, HUD, or clear the game frame underneath it.
6. In the menu, Up and Down wrap through the existing `InGameMenu` API.
7. B or a MENU tap closes the menu and returns to `Screen::Playing` with the same session.
8. A on Continue also returns to `Screen::Playing` with the same session.
9. A on Eject uses the existing session stop, sink close, and eject animation path. Reset the
   animation and SFX flags exactly as the existing MENU-hold eject path does.
10. A on Save State, Cheats, Display, Core, or Device stays in the menu. Those features do not
    exist yet; do not add placeholder screens, fake success, settings files, or toasts.
11. Power and volume actions retain their existing global behavior.
12. Update the state-machine module comment so it describes the new transitions accurately.

## Tests

Add focused app tests that prove at least:

- MENU tap from Playing opens the default in-game menu, while MENU hold still ejects.
- Up and Down change and wrap the menu choice.
- B, MENU tap, and A on Continue return to Playing without dropping the session contract.
- A on an unimplemented choice stays in the menu.
- A on Eject enters Ejecting and requests the existing session/sink shutdown path where a
  session fixture is practical.
- the app reports audio paused for both power and in-game menus, and not for Playing.
- drawing the in-game menu emits the menu overlay over the game path without drawing the
  wallpaper or clearing between the game frame and overlay. Test behavior, not a brittle total
  operation count.

Use existing test helpers and the smallest practical fixture. Tests in `app.rs` may exercise
private state transitions directly when building a real core session would add unrelated cost.
Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\src\host_app.rs
- C:\SLOT2\crates\slot2\src\device_app.rs
- C:\SLOT2\crates\slot2\tests\ingame_menu_app.rs (new, only if integration coverage is useful)
- C:\SLOT2\tasks\34-ingame-menu-app-wiring.worker-result.md

If a minimal test-only change to an existing `crates/slot2/tests/*.rs` helper is required, list
it as a contract concern before changing it. Do not modify `slot2-ui`, `slot2-input`,
`slot2-audio`, `Session`, language assets, manifests, or documentation in this task.

## Out of scope and forbidden

- No save-state switcher, cheat loader, display settings, core picker, device settings, or
  game-specific ini work.
- No new session pause/resume abstraction and no audio ring redesign.
- No screenshot goldens or visual redesign of `InGameMenu`.
- No full workspace test or device distribution build.
- No hardware, adb, Samba, or SD-card access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.
- Preserve unrelated uncommitted changes. Do not clean or revert the working tree.

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

Write `C:\SLOT2\tasks\34-ingame-menu-app-wiring.worker-result.md` containing:

- Success, failure, or partial completion.
- Changed files and why each changed.
- The final screen transitions and audio-pause rule.
- Each validation command, exit code, and concise result.
- Whether any code changed after final validation.
- Remaining issues or contract concerns.
- Approximate elapsed time. Include model token/cost data only if the tool exposes it; do not
  spend time trying to obtain it.

Do not paste source code or full logs into the report. If blocked, leave the code internally
consistent, report the blocker, and stop. Do not change models or configuration. This task has
at most two total attempts.
