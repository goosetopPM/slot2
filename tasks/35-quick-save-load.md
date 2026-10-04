# Task 35 - Wire quick save and latest-state load gestures

Work directly in the current checkout. Add the already-decided SELECT chord behavior while a
game is running: SELECT+R1 saves a new numbered state and SELECT+L1 loads the newest numbered
state. Reuse the existing Card and Session state APIs. This task does not build the visual
state switcher or its 30-second undo system.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\35-quick-save-load.md
- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\src\session.rs
- C:\SLOT2\crates\slot2-input\src\gestures.rs
- C:\SLOT2\crates\slot2-store\src\card.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2\tests\session.rs
- C:\SLOT2\crates\slot2\tests\ingame_menu_app.rs

Read directly related tests or message-loading code only when needed. Do not read the full
handoff, milestone history, old task reports, or repository history. The relevant fixed
contract is copied below.

## Existing behavior

- `Gestures` emits `Action::Chord(Button::R1)` and `Action::Chord(Button::L1)` when SELECT is
  held and the shoulder is pressed.
- `Session::save_state(card, StateKind)` writes core bytes and a thumbnail.
- `Session::load_state(card, StateKind)` loads core bytes and resets the audio resampler.
- `Card::next_state_number(cart)` returns the next numbered slot.
- `Card::list_states(cart)` includes the special Resume entry plus numbered entries.
- `state-saved` already exists in English and Korean and takes `title` and `n` arguments.

DESIGN section 7 fixes the gestures as SELECT+R1 for immediate save and SELECT+L1 for loading
the most recent numbered state. Resume state is a separate boot/exit mechanism and is not a
quick-load candidate.

## Contract

1. Handle both chords only while the screen is exactly `Screen::Playing` with a live session.
   They do nothing in shelf, animation, power-menu, and in-game-menu screens.
2. SELECT+R1 obtains the active session's cart and `Card::next_state_number`, then saves to
   `StateKind::Numbered(n)` through `Session::save_state`.
3. A successful save keeps the same session and screen, makes no sink request, and shows the
   existing `state-saved` toast with the cart title and slot number.
4. SELECT+L1 chooses the numbered state with the greatest numeric slot. Ignore Resume even if
   it is the only state or has the newest mtime. Load through `Session::load_state`.
5. A successful load keeps the same session and screen, makes no sink request, and shows a
   localized success toast naming the slot number. Add the smallest natural English/Korean
   Fluent message needed for this.
6. If there is no numbered state, keep playing and show a localized short message. Do not
   silently load Resume and do not create a state.
7. Save or load errors must not panic, stop the session, change the screen, or close audio.
   Log one concise error and show a localized save-failed or load-failed message.
8. Loading a state must reset both the audio resampler and rewind history. A rewind chain from
   the pre-load future must not survive a state jump. Keep this policy in `Session::load_state`.
9. Do not change chord recognition, modifier timing, state file naming, thumbnail encoding,
   state numbering, or Resume behavior.
10. Do not implement the state-switcher UI, delete, undo, state-ring limits, MENU double tap,
    or new button mappings in this task.
11. Update the app state-machine/module comment where it documents in-game actions.

If borrowing the active Cart requires an accessor on Session, expose only a read-only
`cart(&self) -> &Cart` method. Do not duplicate Cart ownership in App.

## Tests

Add focused tests proving at least:

- SELECT+R1 produces numbered states 1 then 2 with thumbnails, keeps Playing/session/audio,
  and reports the success toast.
- SELECT+L1 loads the greatest numbered slot, not Resume and not merely the newest mtime.
- quick load restores actual core state in a real-session test when the existing fixture is
  available.
- no numbered states yields the localized no-state feedback and leaves the session running.
- save/load failures leave the app internally consistent and expose the correct failure key.
- both chords are ignored outside Playing, especially while `Screen::InGame` is open.
- `Session::load_state` clears rewind history as well as resetting audio state. Test this by
  behavior or a narrow test-visible query; do not expose mutable internals.

Do not weaken or rewrite existing tests. If a platform cannot run the real-core fixture, use
the established explicit skip pattern while keeping pure transition and storage tests active.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\src\session.rs
- C:\SLOT2\crates\slot2\tests\quick_state_app.rs (new, preferred integration test)
- C:\SLOT2\crates\slot2\tests\session.rs only for a focused Session regression test
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs only if needed for message coverage
- C:\SLOT2\tasks\35-quick-save-load.worker-result.md

Do not modify slot2-input, slot2-store, slot2-ui, manifests, documentation, or runtime loops.
Preserve unrelated uncommitted changes. Do not clean or revert the working tree.

## Out of scope and forbidden

- No visual state switcher, state deletion, 30-second undo, or ring-size policy.
- No changes to Resume state creation or loading.
- No new generic command/action framework.
- No full workspace test or device distribution build.
- No hardware, adb, Samba, or SD-card access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-i18n
cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\35-quick-save-load.worker-result.md` containing:

- Success, failure, or partial completion.
- Changed files and why each changed.
- Exact save/load selection and feedback behavior.
- Each validation command, exit code, and concise result.
- Whether any code changed after final validation.
- Remaining issues or contract concerns.
- Approximate elapsed time. Include model token/cost data only if the tool exposes it.

Do not paste source code or full logs into the report. If blocked, leave the code internally
consistent, report the blocker, and stop. Do not change models or configuration. This task has
at most two total attempts.
