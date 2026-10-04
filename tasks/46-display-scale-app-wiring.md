# Task 46 - Wire per-game display scale into the running app

Work directly in the current checkout. Connect Task 45's `DisplayMenu` to the in-game menu,
apply a chosen scale to the paused Session immediately, and persist the game override through
the existing `GameSettings` file contract.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\46-display-scale-app-wiring.md
- C:\SLOT2\crates\slot2\src\app.rs, only `Screen`, `App`, `act`, draw/audio/run guards, and
  nearby in-game-menu/state-switcher helpers
- C:\SLOT2\crates\slot2\src\session.rs, only settings initialization and scale accessors
- C:\SLOT2\crates\slot2-ui\src\display_menu.rs
- C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs
- C:\SLOT2\crates\slot2-store\src\settings.rs
- C:\SLOT2\crates\slot2\tests\ingame_menu_app.rs
- C:\SLOT2\crates\slot2\tests\session.rs, only scale/settings tests and their helpers
- C:\SLOT2\assets\lang\en.ftl and C:\SLOT2\assets\lang\ko.ftl, only related menu/error keys
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs, only related direct-message tests

Read a directly related test helper only when needed. Do not read the full handoff, milestone
history, old task reports, logs, or repository history. The complete contract follows.

## Existing behavior

- `Screen::InGame(InGameMenu)` pauses core advancement and audio while drawing the menu over the
  Session's last frame. Its Display row currently does nothing.
- `DisplayMenu::new(Option<ScaleMode>)` selects Platform default (`None`), Integer, Aspect fit,
  or Fill; it owns no filesystem or App state.
- `Card::read_settings` and `write_settings` preserve unknown keys. A settings object with no
  known or unknown values removes the per-game file.
- Session launch maps a stored scale to `ScalePolicy`; `None` currently uses the same renderer
  default as Integer. `Session::set_scale` changes placement immediately without advancing a
  frame.
- Shader and overlay settings still have no runtime contract and remain out of scope.

## Contract

1. Add a lightweight `Screen` variant for the open Display submenu. Carry both the parent
   `InGameMenu` and Task 45's Copy `DisplayMenu`, so B/MENU can return to the same Display row.
   Preserve `Screen`'s Copy/Debug/Eq behavior; do not add an App-owned duplicate menu.
2. A on `InGameChoice::Display` obtains the active Session's cart, reads its current
   `GameSettings::scale`, constructs `DisplayMenu` from that exact override, and enters the new
   screen. With no active Session, stay on the in-game menu and perform no I/O.
3. While the Display submenu is open, Up/Down navigate it. B or MENU returns to the parent
   in-game menu with the Display row still selected. Other ordinary buttons do nothing.
4. A attempts to commit the highlighted `Option<ScaleMode>` for the active cart:
   - read the current full `GameSettings`;
   - replace only `scale`, preserving `core`, `overscan`, `rewind`, and unknown ini keys;
   - call `Card::write_settings`;
   - only after a successful write, apply the same choice to the active Session immediately.
5. Keep the Display submenu open after successful A so the changed game frame is visible behind
   it and the player can compare another choice. Do not show a success toast or advance the core.
6. Platform default (`None`) must remove only the scale override. When it was the file's last
   value, the existing store behavior removes the empty game ini. Apply the same renderer policy
   that Session launch uses for `None`; do not invent a platform lookup in App.
7. Keep launch-time and live scale mapping in one Session-owned implementation. Add the smallest
   helper/API needed so Session construction and App's live change cannot drift. Preserve the
   existing direct `set_scale(ScalePolicy)` API and tests unless removal is demonstrably required.
8. If settings persistence fails, leave the Session's current scale unchanged, stay in the
   Display submenu on the attempted row, log one concise error, and show a localized
   `display-save-failed` toast: `Could not save the display setting` / `화면 설정을 저장하지 못했습니다`.
9. The Display submenu has the same paused behavior as the in-game menu and state switcher:
   core frames do not advance, audio remains paused, and no sink open/close request occurs.
   Global Power and volume behavior remains unchanged.
10. Draw the Session's last frame with its current scale first, then `DisplayMenu`; draw transient
    hold progress/toast using the existing ordering afterward. Do not draw wallpaper, HUD time
    badge, parent in-game menu, state switcher, or a clear between the frame and submenu.
11. Update the app state-machine/module comments for the new transitions. Do not claim that
    shader or overlay controls exist.

Use the active Session's existing Cart as the identity. Clone it only as needed to satisfy Rust
borrowing; do not add another current-cart field.

## Tests

Add focused integration/regression tests proving at least:

- A on the in-game Display row opens the submenu with the exact stored override selected,
  including `None`;
- Up/Down wrap and B/MENU return to the same Display row without changing settings or scale;
- the submenu pauses core frames and audio, preserves the Session and sink, and makes no sink
  request;
- drawing emits the game frame before the Display dim/menu, with no wallpaper, time-control
  badge, parent menu, switcher, or clear between them;
- choosing Integer, Aspect fit, and Fill persists the exact `ScaleMode` and immediately changes
  the paused frame placement without running a core frame;
- choosing Platform default removes only the scale override, restores the same policy a fresh
  Session with `scale=None` uses, preserves every other known setting, and removes a scale-only
  ini file;
- an existing unknown ini key survives both setting and clearing the scale override;
- a deliberately induced write failure leaves the prior Session scale and card setting intact,
  stays on the attempted row, and reports the localized failure toast;
- closing and reopening the submenu reflects the last successfully persisted choice;
- English and Korean directly define the failure message rather than relying on fallback.

Use temporary card roots and existing App/real-core test patterns. Keep pure state, persistence,
and drawing tests active when a core is unavailable; use the established explicit skip pattern
only for behavior that truly requires the test core. Avoid timing, hardware, GL windows, or
golden screenshots. Do not weaken existing tests.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\src\session.rs
- C:\SLOT2\crates\slot2\tests\display_menu_app.rs (new, preferred)
- C:\SLOT2\crates\slot2\tests\ingame_menu_app.rs only if an existing assertion must recognize
  the new screen
- C:\SLOT2\crates\slot2\tests\session.rs only for shared scale-mapping regression coverage
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs only for the new failure key
- C:\SLOT2\tasks\46-display-scale-app-wiring.worker-result.md

Do not modify `slot2-ui`, `slot2-store`, `slot2-gfx`, input, audio/runtime loops, manifests, or
documentation. Preserve unrelated uncommitted changes. Do not clean, revert, or reformat
unrelated files.

## Out of scope and forbidden

- No shader, overlay, overscan, rewind, core picker, cheat, device settings, shelf settings, or
  generic settings/menu framework.
- No new gesture, sound, success toast, Session restart, core reload, save-state action, or
  settings migration.
- No workspace-wide test, device distribution build, Raspberry Pi test, or RG SP access.
- No adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-ui -p slot2-i18n
cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\46-display-scale-app-wiring.worker-result.md` with at most about 35 lines:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Changed files and final screen, input, persistence, live-application, and failure behavior.
- Each validation command, exit code, and concise test result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste source code or logs. A path/option error or tool timeout counts as an invocation.
Stop at forty-five minutes and write a partial report; do not make a third invocation. Do not
change models or configuration.
