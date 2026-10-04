# Task 32 - M4 in-game menu UI only (final attempt 2/2)

Implement a self-contained in-game menu component in slot2-ui. Do not read HANDOFF, MILESTONES, DESIGN, DECISIONS, CLAUDE, legacy operations documents, or unrelated task reports. This file contains the complete contract. Read only this file, the allowed source files, and directly imported UI code needed to compile.

Existing uncommitted user changes must be preserved. Do not clean, revert, commit, or push them.

Allowed files:

- crates/slot2-ui/src/in_game_menu.rs (new)
- crates/slot2-ui/src/lib.rs
- assets/lang/en.ftl
- assets/lang/ko.ftl
- Task32-only tests under crates/slot2-ui/tests or crates/slot2-i18n/tests if needed
- tasks/32-ingame-menu-ui.worker-result.md

Do not edit any other file.

Contract:

1. Add InGameChoice and a fixed seven-item array in this order: Continue, Save State, Cheats, Display, Core, Device, Eject. Each choice returns a stable Fluent message key.
2. Add InGameMenu with a private selected index, Default, wrapping up/down navigation, and current choice lookup. Callers must not construct an invalid index through the public API.
3. draw overlays the current game frame and must not clear the canvas.
4. Dim the whole panel with translucent black. Draw an opaque panel centered in the 640x480 safe area, with a title, seven rows, selected-row highlight, and select/back hints. Every element must fit inside the safe area at 640x480. Larger panels remain centered through UiCtx.safe.
5. Reuse the existing PowerMenu structure, splash palette, text sizes, draw_spans, and Canvas abstractions. Do not create a generic menu framework or new rendering abstraction.
6. Add natural English and Korean Fluent messages for the title and all seven items. Use dedicated in-game-menu keys where reuse would be ambiguous. Keep both FTL files valid UTF-8.
7. Re-export slot2_ui::{InGameChoice, InGameMenu} from lib.rs.
8. Tests must cover item order and keys, default choice, up/down wrap, selected-row highlight, and all row y bounds fitting the 640x480 safe area. Avoid assertions on the exact total draw-op count.

Out of scope: slot2 app Screen changes; Session pause/resume; audio mute; input wiring; submenu behavior; eject behavior; ini settings; screenshot goldens; workspace-wide test; device build; hardware, adb, Samba, or SD access; global Codex/OpenCodex/BAI configuration; delegation; commit; push.

Run these commands in order:

1. cargo fmt --all -- --check
2. cargo test -p slot2-ui -p slot2-i18n
3. cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings

All must exit 0. If code changes after a validation command, rerun affected validation. Do not run full workspace tests or the device build.

Write tasks/32-ingame-menu-ui.worker-result.md with: success/failure/partial; cumulative attempt 2/2; changed files and reasons; each command and exit code plus concise result; whether code changed after final validation; remaining issues or contract concerns; elapsed time and input/output/cached token figures when visible, otherwise "unavailable". Do not paste logs or source code into the report.

This is the second and final automatic implementation attempt. If blocked or the provider fails, stop and report what exists. Do not retry with another model or delegate.
