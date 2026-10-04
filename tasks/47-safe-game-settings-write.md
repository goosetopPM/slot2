# Task 47 - Refuse to overwrite unreadable game settings

Work directly in the current checkout. Close the data-loss risk recorded after Task 46: shelf
scanning and launch may keep treating an unreadable game ini as default, but a settings write
must never replace a file whose existing contents could not be read.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\47-safe-game-settings-write.md
- C:\SLOT2\crates\slot2-store\src\settings.rs
- C:\SLOT2\crates\slot2-store\src\ini.rs
- C:\SLOT2\crates\slot2-store\tests\card.rs, only game-settings tests
- C:\SLOT2\crates\slot2\tests\display_menu_app.rs, only write-failure coverage

Read `Error` or atomic-write definitions only if needed to use the existing error contract. Do
not read App implementation, handoff/history, old reports, logs, or repository history unless a
failing focused test proves it necessary. The complete contract follows.

## Existing behavior and risk

- `Card::read_settings` deliberately returns `GameSettings::default()` when an ini is missing or
  unreadable. Shelf scanning and launch must remain usable even with a damaged settings file.
- `Ini::load` already distinguishes a missing path (successful empty ini) from an existing path
  that cannot be read or decoded (`Error::Io`).
- `Card::write_settings` currently calls `Ini::load(...).unwrap_or_else(empty)`. Therefore an
  unreadable existing file can be mistaken for an empty file and overwritten, losing known and
  unknown keys.
- Task 46's Display menu already treats any `write_settings` error correctly: it leaves the live
  Session unchanged, stays on the attempted row, and shows `display-save-failed`.

## Contract

1. Keep `Card::read_settings` exactly forgiving: missing, unreadable, directory, and invalid UTF-8
   inputs still return defaults rather than blocking card scan or launch.
2. Make `Card::write_settings` load the existing ini fallibly. A missing file remains a normal
   empty starting point because `Ini::load` already defines it that way.
3. If an existing settings path cannot be read for any reason, return the existing store error
   before changing or deleting that path. Do not create/truncate a temp file, rewrite the target,
   apply partial settings, or silently discard unknown keys.
4. Preserve all successful behavior: known fields update, unknown keys survive, defaults remove
   owned keys, an actually empty ini is removed, deletion errors propagate, and identical atomic
   writes retain their current no-op behavior.
5. Do not change the public shape or meaning of `GameSettings`, `ScaleMode`, `Ini`, `Error`, or
   `Card::read_settings`. The smallest intended production change is removing the lossy fallback
   from the mutation path.
6. Confirm Task 46 needs no App change: its existing failed-save path must handle this new error
   and leave the Session scale unchanged.

## Tests

Add focused regressions proving at least:

- an existing invalid UTF-8 game ini still makes `read_settings` return defaults;
- attempting to write a non-default setting over that invalid UTF-8 file returns `Err` and leaves
  its exact original bytes unchanged;
- attempting to clear settings at that unreadable path also returns `Err` and leaves it unchanged;
- a directory at the settings-file path returns `Err` for both a non-default write and a default
  clear, without removing the directory;
- a missing file still accepts a non-default write, and returning it to defaults removes it;
- existing unknown-key preservation, ordinary round trip, and deletion-error tests remain intact;
- through the Display-menu App integration path, an invalid UTF-8 ini may open on Platform default
  under the forgiving read policy, but pressing A on another scale reports `display-save-failed`,
  keeps the attempted row open, leaves the Session policy unchanged, and preserves the exact file
  bytes.

Use temporary card roots. Do not use filesystem permission bits, because those behave differently
on Windows; invalid UTF-8 and a directory at the file path are portable failure cases. The App
test may use the established explicit real-core skip pattern. Do not weaken existing tests.

## Allowed files

- C:\SLOT2\crates\slot2-store\src\settings.rs
- C:\SLOT2\crates\slot2-store\tests\card.rs
- C:\SLOT2\crates\slot2\tests\display_menu_app.rs
- C:\SLOT2\tasks\47-safe-game-settings-write.worker-result.md

Do not modify App, Session, UI, i18n, `Ini`, atomic-write code, manifests, or documentation unless
a focused regression proves the stated contract impossible; if so, stop and report rather than
expanding scope. Preserve unrelated uncommitted changes. Do not clean or revert the tree.

## Out of scope and forbidden

- No redesign of the forgiving launch/read policy and no new warning UI for merely opening an
  unreadable file.
- No new settings editor, menu item, migration, backup format, recovery file, or generic
  transaction API.
- No workspace-wide test, device distribution build, Raspberry Pi test, or RG SP access.
- No adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store
cargo test -p slot2 --test display_menu_app
cargo clippy -p slot2-store -p slot2 --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\47-safe-game-settings-write.worker-result.md` with at most about 30 lines:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Exact production change and unreadable-file behavior.
- Focused store/App regression results and each final validation command with exit code.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste source or logs. A path/option error or timeout counts as an invocation. Stop at
forty-five minutes and write a partial report; do not make a third invocation. Do not change
models or configuration.
