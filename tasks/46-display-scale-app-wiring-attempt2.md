# Task 46 attempt 2/2 - Correct persistence failure handling and finish validation

This is the final allowed invocation for Task 46. Keep the existing implementation unless a
change below requires otherwise. Fix the three invalid tests from attempt 1, close the discovered
settings-file deletion error, and finish every validation command with exit code 0.

## Read first

Read only:

- C:\SLOT2\tasks\46-display-scale-app-wiring-attempt2.md
- C:\SLOT2\tasks\46-display-scale-app-wiring.md, only the Contract and Tests sections
- C:\SLOT2\tasks\46-display-scale-app-wiring.worker-result.md
- C:\SLOT2\crates\slot2\tests\display_menu_app.rs, only the three failing tests
- C:\SLOT2\crates\slot2-store\src\settings.rs, only `Card::write_settings`
- C:\SLOT2\crates\slot2-store\tests\card.rs, only settings tests

Read the Task 46 implementation only if a corrected test still exposes an implementation defect.
Do not read handoff/history/logs or unrelated source.

## Required corrections

1. Fix `navigation_wraps_and_closing_returns_to_the_same_row` to follow the actual fixed row
   order: Platform default, Integer, Aspect fit, Fill. Starting on Integer, one Up reaches
   Platform default; one more Up wraps to Fill. Keep both boundary wraps and B/MENU return to the
   same parent Display row covered. Navigation alone must not change card settings or live scale.
2. Fix `platform_default_removes_only_the_scale_override`:
   - Fill reaches Platform default with Down, not Up.
   - After App commits Platform default, directly assert scale is absent, other known settings
     and the unknown key remain, and live Session uses `Session::policy_for(None)`.
   - For the scale-only case, let App commit the already-selected Platform default and assert the
     ini file is gone immediately afterward. Remove the direct
     `card.write_settings(GameSettings::default())` cleanup that currently masks App behavior.
3. Replace the destructive setup in `a_write_that_fails_leaves_the_game_alone`:
   - write the original Integer setting and open Display while it is still readable, so the menu
     genuinely starts on Integer;
   - after opening, create a directory at the atomic temporary path `<game.ini>.tmp`; this makes
     the following write fail while leaving the original ini readable and intact;
   - move to Aspect fit and press A;
   - assert the failure toast, attempted row, unchanged live Integer policy, original card scale
     still Integer, open Display screen, existing Session, and no sink request;
   - remove the temporary blocking directory during cleanup.
4. Fix `Card::write_settings` so returning to an entirely empty settings file propagates a
   `remove_file` error instead of swallowing it and returning `Ok(())`. Missing files remain a
   successful no-op. Do not change unknown-key preservation or normal atomic writes.
5. Add a portable `slot2-store` regression test: put a directory at the game settings file path,
   call `write_settings(..., GameSettings::default())`, and require `Err` rather than false
   success. Keep the existing successful default-removal test.
6. Run the corrected focused display integration test first. If it exposes a real implementation
   defect, make only the smallest Task 46 fix and record it. Do not weaken assertions or replace
   App actions with direct store/session calls.

## Allowed files

- C:\SLOT2\crates\slot2\tests\display_menu_app.rs
- C:\SLOT2\crates\slot2-store\src\settings.rs
- C:\SLOT2\crates\slot2-store\tests\card.rs
- C:\SLOT2\crates\slot2\src\app.rs only if a corrected test proves an App defect
- C:\SLOT2\crates\slot2\src\session.rs only if a corrected test proves a mapping defect
- C:\SLOT2\tasks\46-display-scale-app-wiring.worker-result.md

Preserve every unrelated uncommitted change. Do not clean, revert, commit, or push.

## Validation

Run after the final code change, in this order:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store -p slot2 -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-store -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

All commands must exit 0. If code changes afterward, rerun affected commands. Do not run the full
workspace, dist build, Pi test, or hardware access.

## Result report

Update `C:\SLOT2\tasks\46-display-scale-app-wiring.worker-result.md` as the cumulative **2/2**
report. Put the attempt-2 delta first: exact test corrections, the deletion-error fix, focused
test result, all final validation exit codes, whether code changed afterward, remaining concern,
and elapsed time. Keep it concise and do not paste logs or source.

Stop by forty-five minutes even if incomplete. There is no third invocation. Do not delegate,
change models/configuration, access hardware/ADB/Samba/SD card, or modify shared configuration.
