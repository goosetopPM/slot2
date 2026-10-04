# Task 33 - Finish the partial M4 in-game menu UI

This is a user-authorized manual recovery after two failed OpenCodex transport attempts. Work directly in the current checkout. Do not delegate. Do not restart the task from scratch unless the partial code is unusable.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\33-ingame-menu-ui-recovery.md
- C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs
- C:\SLOT2\crates\slot2-ui\src\lib.rs
- C:\SLOT2\crates\slot2-ui\src\power_menu.rs
- C:\SLOT2\crates\slot2-ui\tests\power_menu.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl

Read directly imported UI helpers only when needed to compile. Do not read HANDOFF, MILESTONES, DESIGN, DECISIONS, CLAUDE, legacy operations notes, old task reports, or repository-wide history. This file contains the complete contract.

## Existing partial implementation

The failed worker already created `in_game_menu.rs` and added its module/re-exports to `lib.rs`.

What exists:

- InGameChoice with seven choices and dedicated Fluent keys.
- Fixed item order: Continue, Save State, Cheats, Display, Core, Device, Eject.
- InGameMenu with a private selected index, wrapping navigation, choice lookup, safe-area-centered overlay drawing, dim layer, panel, rows, highlight, and hints.
- The existing slot2-ui and slot2-i18n tests compile and pass with this partial code.

Known missing or failing items:

- `cargo fmt --all -- --check` fails because the module declaration order in lib.rs is not rustfmt order.
- The new Fluent keys are absent from en.ftl and ko.ftl.
- No Task33/Task32-specific tests exist.
- Clippy has not been run on the completed change.
- No worker result report exists.

Review the partial implementation for correctness, then make the smallest changes needed to complete it. Preserve useful existing code.

## Contract

1. Public API remains `slot2_ui::{InGameChoice, InGameMenu}`.
2. Item order remains Continue, Save State, Cheats, Display, Core, Device, Eject.
3. Each choice returns a stable dedicated Fluent key.
4. InGameMenu keeps its selected index private, defaults to Continue, wraps up/down, and exposes the current choice. Public callers cannot create an invalid index.
5. draw overlays the current frame and never clears the canvas.
6. It dims the whole panel, then draws an opaque panel centered in the 640x480 safe area with a title, seven rows, one selected highlight, and select/back hints.
7. All rows, title, and hints fit inside the 640x480 safe area. Larger device panels use UiCtx.safe positioning.
8. Reuse the existing PowerMenu style, splash palette, Canvas, UiCtx, text sizes, and draw_spans. Do not introduce a generic menu framework or new rendering abstraction.
9. Add natural English and Korean messages for the title and all seven dedicated keys. Keep both FTL files valid UTF-8. Existing hint-select and hint-back may be reused.
10. Add focused tests covering exact item order and keys, default choice, up/down wrapping, no clear operation, selected-row highlight, and every row rectangle fitting inside the 640x480 safe area. Do not assert the exact total number of draw operations.

## Allowed files

- C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs
- C:\SLOT2\crates\slot2-ui\src\lib.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-ui\tests\in_game_menu.rs (new, preferred test location)
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs only if a translation-specific assertion is necessary
- C:\SLOT2\tasks\33-ingame-menu-ui-recovery.result.md

Do not modify any other file. Preserve all unrelated uncommitted user changes. Do not clean or revert the working tree.

## Out of scope and forbidden

- No changes to crates/slot2/src/app.rs or the Screen state machine.
- No Session pause/resume, audio mute, input wiring, submenus, eject action, ini settings, or screenshot goldens.
- No full workspace test or device build.
- No hardware, adb, Samba, or SD-card access.
- No global Codex, GJC, OpenCodex, or BAI configuration changes.
- No agent delegation, recursive task creation, commit, or push.
- Do not add lint allowances, padding behavior, or assertions whose only purpose is satisfying a brittle test.

## Validation

Run in this exact order:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\33-ingame-menu-ui-recovery.result.md` containing:

- Success, failure, or partial completion.
- Changed files and why each changed.
- Each validation command, exit code, and concise result.
- Whether any code changed after the final validation.
- Remaining issues or contract concerns.
- Whether the existing partial implementation was preserved or replaced, and why.
- Approximate elapsed time and model token/cost information if the interactive tool exposes it; otherwise write `unavailable`.

Do not paste source code or full logs into the report. If blocked, leave the code in its best internally consistent state, report the blocker, and stop instead of changing models or configuration.
