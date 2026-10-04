# Task 39 - Wire state deletion and 30-second undo into the switcher

Work directly in the current checkout. Connect Task 38's reversible numbered-state deletion
to the running state switcher. The player deletes the selected slot with X and may restore the
one most recently deleted slot with Y for thirty seconds.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\39-state-switcher-delete-undo.md
- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2-ui\src\state_switcher.rs
- C:\SLOT2\crates\slot2-store\src\card.rs, only the `StateBackup`, `take_state`, and
  `restore_state` definitions
- C:\SLOT2\crates\slot2\tests\state_switcher_app.rs
- C:\SLOT2\crates\slot2-ui\tests\state_switcher.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs

Read directly related App test helpers or UI hint code only when needed. Do not read the full
handoff, milestone history, old task reports, logs, or repository history. The complete
contract is below.

## Existing behavior

- `Screen::Switcher(menu)` pauses core advancement and audio, draws the last game frame and
  switcher overlay, uses Left/Right to navigate, A to load, and B/MENU to return.
- `StateSwitcher` owns a bounded thumbnail cache, filters Resume, sorts numbered slots, and
  exposes only its selected numbered kind.
- `Card::take_state` removes one numbered state and optional thumbnail into an opaque
  `StateBackup`; `Card::restore_state(&backup)` restores it without consuming the backup.
  Both protect Resume, card identity, conflicts, and partial writes.
- `App::tick(now)` already receives a monotonic `Instant`. Animation `dt` is clamped, so it
  must not be used to measure a thirty-second undo deadline.
- X and Y currently do nothing in the switcher. No input mapping change is needed.

## Contract

1. Add one private App-owned pending undo containing a `StateBackup` and an absolute monotonic
   expiry. Export a public `STATE_UNDO_S: u64 = 30` (or equivalent Duration constant) so the
   boundary can be tested. Do not put backup bytes in `Screen` or `StateSwitcher`.
2. In `Screen::Switcher`, X deletes the selected numbered state through `Card::take_state`.
   Empty selection is a no-op with the existing empty-state feedback; Resume is never passed.
3. A successful delete refreshes the switcher immediately, stays on `Screen::Switcher`, keeps
   the same session and sink, and shows a localized whole-sentence success toast naming the
   slot. Select the smallest remaining number greater than the deleted number; if none exists,
   select the greatest lower number; if none remains, select nothing.
4. A successful delete replaces the previous pending undo and starts a fresh thirty-second
   deadline from the `Instant` supplied to the tick that handles it. Replacing or expiring a
   pending backup makes that older deletion permanent simply by dropping the in-memory backup.
   There is no history, trash directory, or delayed filesystem operation.
5. A delete error or an unexpectedly missing selected state stays in the switcher, logs one
   concise error, shows localized failure feedback, leaves the on-screen list consistent with
   `Card::list_states`, and does not replace an already valid pending undo.
6. In the switcher, Y restores the active pending backup through `Card::restore_state`. Success
   clears the pending undo, refreshes the slots, selects the restored number, stays paused in
   the switcher, and shows localized whole-sentence success feedback.
7. Restore conflict or I/O failure stays in the switcher, logs one concise error, shows
   localized failure feedback, and retains the same backup until its original deadline so Y
   can retry. It must not extend the thirty seconds.
8. Pending undo is available while `now < expires_at` and expires before action handling when
   `now >= expires_at`. Expiry uses the absolute `Instant` passed to `App::tick`, continues
   while the player visits Playing or the in-game menu, and produces no toast by itself. Y with
   no active undo shows a localized “nothing to undo” response and changes no files.
9. Leaving and reopening the switcher in the same live session preserves a still-valid undo.
   Stopping/ejecting the session clears it. A successful state load may return to Playing
   without clearing it; the player may reopen the switcher and undo before expiry.
10. Update switcher hints without a confirmation dialog: A load, X delete, B back, and Y undo
    only while an undo is active. Empty state plus active undo still shows Y undo and B back.
    Keep all English and Korean hint text inside every supported safe area.
11. Add the smallest StateSwitcher refresh/selection API needed for the post-delete and
    post-restore rules. Keep Resume filtering and numeric sorting in the existing single
    normalization path. Do not expose mutable indices or filesystem operations.
12. A deleted thumbnail path must not leave a stale image if the number is later reused. Evict
    cache entries for paths no longer present and release any texture on the next draw or
    `clear`, since refresh has no Canvas. Keep still-present cached thumbnails and preserve the
    existing cache bound; never abandon a live texture id.
13. Power/volume global behavior, A load, B/MENU return, quick-save/load chords, pause/draw
    order, Resume behavior, and permanent `delete_state` remain unchanged. Update the App
    state-machine comments for X/Y.

Use localized complete messages rather than joining fragments in Rust. Keep the success toast
short; the persistent Y hint, not a thirty-second toast, advertises the remaining opportunity.

## Tests

Add focused tests proving at least:

- X removes the selected state and exact thumbnail, remains paused in the switcher, reports
  success, preserves the session/sink, and chooses the specified next/previous selection;
- a second delete replaces the first pending backup, so Y restores only the second deletion;
- Y before thirty seconds restores exact state and raw thumbnail bytes, selects that slot,
  clears undo, and a second Y changes nothing;
- the pending backup survives switcher -> menu or Playing -> switcher while the same session
  lives, but session stop/eject clears it;
- at 29.999 seconds undo succeeds, while at exactly 30 seconds and after it Y cannot restore;
  use explicit `Instant` values, never sleeps or wall-clock timing;
- a restore conflict/failure retains the backup and deadline, and succeeds on retry after the
  obstruction is removed; a failed delete preserves a previous pending undo;
- deleting the last state shows the empty view with an active Y hint, and undo restores it;
- deleted/reused thumbnail paths cannot draw stale cached art and every evicted uploaded texture
  is eventually freed exactly once;
- English and Korean directly define all new messages and hints, and all hint combinations fit
  the `rgsp`, `rg35xxsp`, and `rgcubexx` safe areas;
- existing switcher navigation, A load, B/MENU return, drawing order, audio pause, and quick
  state behavior continue to pass.

Use temporary files and explicit test Instants. Use the established real-core skip pattern only
for behavior that truly needs a live session; keep timer, localization, cache, and pure UI tests
active without a core. Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\tests\state_switcher_app.rs
- C:\SLOT2\crates\slot2-ui\src\state_switcher.rs
- C:\SLOT2\crates\slot2-ui\tests\state_switcher.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs
- C:\SLOT2\tasks\39-state-switcher-delete-undo.worker-result.md

Do not modify `slot2-store`, Session, input mappings, audio/runtime loops, manifests, or
documentation. Preserve all unrelated uncommitted changes from Tasks 24-38. Do not clean,
revert, or reformat unrelated files.

## Out of scope and forbidden

- No confirmation dialog, multiple undo levels, persistent trash, startup recovery, retention
  policy, countdown display, new gesture, or change to quick-save numbering.
- No full workspace test or device distribution build.
- No hardware, adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
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

Write `C:\SLOT2\tasks\39-state-switcher-delete-undo.worker-result.md` with at most about 30
lines unless a failure needs more evidence:

- Success, failure, or partial completion and cumulative attempt number out of two.
- Changed files and final X-delete/Y-undo, deadline, selection, and cache behavior.
- Each validation command, exit code, and concise result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste code, logs, or repeat this specification. A correction attempt must report only
the new delta, regression evidence, and final validation. If blocked, leave the code consistent,
report the blocker, and stop. Do not change models or configuration. This task has at most two
total attempts. Wait up to five minutes for the first response and up to forty-five minutes
overall; do not infer failure from a quiet or empty buffered log.
