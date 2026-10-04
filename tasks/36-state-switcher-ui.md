# Task 36 - Build the state-switcher UI component

Work directly in the current checkout. Build the reusable visual state switcher in `slot2-ui`.
It presents numbered save states as polaroid-style thumbnail cards over the paused game frame.
This task is UI and navigation only: do not wire it into `App`, load or delete a state, or add
the 30-second undo system.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\36-state-switcher-ui.md
- C:\SLOT2\crates\slot2-ui\src\lib.rs
- C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs
- C:\SLOT2\crates\slot2-ui\src\image.rs
- C:\SLOT2\crates\slot2-ui\src\label.rs
- C:\SLOT2\crates\slot2-ui\tests\in_game_menu.rs
- C:\SLOT2\crates\slot2-store\src\card.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl

Read directly related Canvas or i18n code only when needed. Do not read the full handoff,
milestone history, old task reports, or repository history. The relevant fixed contract is
copied below.

## Existing contract

- DESIGN section 7 defines `Playing <-> InGameMenu -> StateSwitcher`.
- M4 requires a polaroid-style state switcher entered from the in-game menu.
- `Card::list_states` returns `StateSlot` values sorted Resume first and numbered ascending.
- Resume belongs to boot/exit recovery. It is not a user-visible numbered save slot.
- A state thumbnail is an optional PNG path. Missing or invalid thumbnails are normal and must
  have a deliberate placeholder.
- This is an overlay over the last game frame. It must dim but never clear the canvas.
- All visible geometry must fit the profile's 640x480 safe area.
- Stored and displayed state time follows D-25, but this component does not display mtime.

## Contract

1. Add a public `state_switcher` module and re-export its main `StateSwitcher` type from
   `slot2-ui`.
2. Construct it from a `Vec<StateSlot>`. Filter out `StateKind::Resume`, sort numbered slots by
   numeric value, and select the greatest number by default. Input order and mtime must not
   change that result.
3. Expose a read-only `selected_kind() -> Option<StateKind>`. It returns `None` when there are no
   numbered states. Do not expose a mutable index or filesystem operation.
4. Provide left/right navigation. It wraps at both ends and is a no-op for zero or one slot.
5. Draw as an overlay without `Canvas::clear`: first dim the whole physical panel, then draw a
   title and a horizontal polaroid-style strip inside the safe area. The selected card is
   visually dominant and centred; show adjacent cards when space permits.
6. Each card has a light frame, a 4:3 thumbnail area, and a localized numbered-slot label.
   Decode and upload a thumbnail lazily, cache it across frames, and preserve its aspect ratio
   within the 4:3 area. A missing, unreadable, or corrupt thumbnail draws a stable neutral
   placeholder and does not retry disk decoding every frame.
7. The empty case draws a localized message instead of a card and keeps
   `selected_kind() == None`.
8. Show the existing select/back hints. Add only the smallest natural English and Korean
   messages required for the title, numbered-slot label, and empty state.
9. The component owns no `Card`, `Cart`, `Session`, or app screen state. It must not read state
   files, mutate the filesystem, or infer behavior from mtimes.
10. Keep texture ownership bounded. Provide a way to release cached thumbnail textures before
    the canvas is destroyed, following existing UI cache conventions.

Choose exact layout constants in the module. Keep them public only when tests or callers need
them. Do not introduce a generic menu, carousel, or image-cache framework.

## Tests

Add focused `slot2-ui` tests proving at least:

- Resume is excluded, unordered numbered slots become numeric order, and the newest number is
  selected by default.
- Left/right wrap correctly and empty/singleton navigation is safe.
- Drawing never clears and every card, label, title, hint, and empty message stays inside the
  safe area on `rgsp`, `rg35xxsp`, and `rgcubexx` in English and Korean.
- A valid thumbnail is uploaded and drawn without stretching its aspect ratio.
- Missing and corrupt thumbnails use the placeholder, do not panic, and do not repeatedly
  attempt upload/decode across frames.
- English and Korean define every new message directly; Korean fallback must not hide a missing
  Korean key.

Use temporary test files only. Do not weaken existing tests or depend on a real core, ROM, GL
window, hardware, or wall clock.

## Allowed files

- C:\SLOT2\crates\slot2-ui\src\state_switcher.rs (new)
- C:\SLOT2\crates\slot2-ui\src\lib.rs
- C:\SLOT2\crates\slot2-ui\tests\state_switcher.rs (new)
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs only for direct message coverage
- C:\SLOT2\tasks\36-state-switcher-ui.worker-result.md

Do not modify `slot2`, `slot2-store`, `slot2-gfx`, input, manifests, documentation, or runtime
loops. Preserve unrelated uncommitted changes, including Task 35 changes in shared language and
i18n test files. Do not clean or revert the working tree.

## Out of scope and forbidden

- No App screen variant, in-game menu selection wiring, session load, save, delete, or undo.
- No state ring-size or retention policy, Resume behavior change, mtime display, or new gesture.
- No full workspace test or device distribution build.
- No hardware, adb, Samba, or SD-card access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\36-state-switcher-ui.worker-result.md` containing:

- Success, failure, or partial completion and cumulative attempt number out of two.
- Changed files and why each changed.
- Exact ordering, selection, navigation, thumbnail fallback, and texture-release behavior.
- Each validation command, exit code, and concise result.
- Whether any code changed after final validation.
- Remaining issues or contract concerns.
- Approximate elapsed time. Include token/cost data only if the tool exposes it.

Do not paste source code or full logs into the report. If blocked, leave the code internally
consistent, report the blocker, and stop. Do not change models or configuration. This task has
at most two total attempts.
