# Task 38 - Add reversible numbered-state deletion to slot2-store

Work directly in the current checkout. Add the storage primitive needed for the state
switcher's 30-second undo: remove one numbered state while retaining an in-memory backup that
can restore the exact state and thumbnail bytes. This task is storage only; do not wire buttons,
timers, UI, or App behavior.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\38-reversible-state-delete-store.md
- C:\SLOT2\crates\slot2-store\src\lib.rs
- C:\SLOT2\crates\slot2-store\src\card.rs
- C:\SLOT2\crates\slot2-store\src\atomic.rs
- C:\SLOT2\crates\slot2-store\tests\card.rs

Read directly related store tests only when needed. Do not read app/UI code, the full handoff,
old task reports, or repository history. The complete contract is below.

## Existing behavior

- Numbered states are `States/<PLAT>/<stem>/<n>.state` with an optional sibling `<n>.png`.
- `Card::delete_state` permanently removes a state and best-effort removes its thumbnail.
- `atomic_write` writes a complete replacement through a sibling temporary file.
- Resume is a separate boot/exit mechanism and must never be exposed to user deletion.
- The future App layer will retain only one pending undo for 30 seconds. A single in-memory
  backup is therefore sufficient; no trash directory or persistent undo journal is required.

## Contract

1. Add and publicly export an opaque `StateBackup` type. It owns the original numbered
   `StateKind`, exact state bytes, optional raw PNG bytes, and enough private origin identity to
   prevent restoring into a different card root or cart. Expose only a read-only `kind()`.
2. Add `Card::take_state` (or an equally clear narrow name) for `StateKind::Numbered(n)`.
   It returns `Ok(None)` when the `.state` file is absent. It must reject `StateKind::Resume`
   with `Error::Invalid` without touching either file.
3. Before deleting anything, read the complete state and optional thumbnail bytes. Treat a
   missing thumbnail as normal. Preserve corrupt or non-PNG thumbnail bytes exactly; this layer
   must not decode them.
4. After the backup is complete, remove the state and its thumbnail. On success neither path is
   visible to `list_states`. A deletion error must make a best effort to roll back already
   removed parts so the state is not intentionally left half-deleted.
5. Add `Card::restore_state(&StateBackup)` or an equivalent non-consuming retryable API. Restore
   only to the backup's original card/cart paths and reject use through a different Card root.
6. Refuse restoration if either target state path or target thumbnail path already exists. Do
   not overwrite a newer state, attach an old thumbnail to it, or partially modify the target.
7. Restore the optional raw thumbnail atomically first and the state atomically last. The
   `.state` file is the visibility boundary: `list_states` must not expose a restored slot until
   every backed-up companion is ready. If the final state write fails, remove the thumbnail
   created by that restore attempt.
8. Keep `StateBackup` usable after a failed restore so the future App can retry or continue to
   offer undo. After a successful restore, a repeated restore must fail on the target conflict
   without changing either file.
9. Keep the existing permanent `delete_state`, state numbering, write format, Resume behavior,
   and public APIs compatible. Do not implement a persistent trash folder or cleanup scan.

Use existing `atomic_write` for restoration. Do not duplicate its temporary-file algorithm.

## Tests

Add focused `slot2-store` tests proving at least:

- taking a numbered state with a valid thumbnail removes it from disk/listing and restoring the
  backup reproduces the exact state bytes and exact raw PNG bytes;
- a state without a thumbnail round-trips without creating one;
- corrupt arbitrary thumbnail bytes round-trip byte-for-byte without decoding;
- a missing state returns `None` and does not remove an orphan thumbnail;
- Resume is rejected and both Resume files remain untouched;
- restore through a different Card root is rejected without writing there;
- an existing target state or thumbnail blocks restore without overwriting either target, and
  the same backup can still restore after the conflict is removed;
- a second restore after success is rejected and leaves the first restored bytes unchanged;
- existing permanent delete and state-list tests continue to pass.

Use temporary directories only. Do not weaken existing tests or depend on permissions, a real
ROM, a core, hardware, or wall-clock timing.

## Allowed files

- C:\SLOT2\crates\slot2-store\src\card.rs
- C:\SLOT2\crates\slot2-store\src\lib.rs
- C:\SLOT2\crates\slot2-store\tests\card.rs
- C:\SLOT2\crates\slot2-store\tests\state_undo.rs (new, preferred if clearer)
- C:\SLOT2\tasks\38-reversible-state-delete-store.worker-result.md

Do not modify `atomic.rs`, App, UI, input, language assets, manifests, documentation, or runtime
loops. Preserve unrelated uncommitted changes. Do not clean or revert the working tree.

## Out of scope and forbidden

- No 30-second timer, delete/undo button assignment, toast, overlay, or App screen change.
- No multiple-level undo history, persistent trash, startup recovery, or state retention policy.
- No change to Resume, state numbering, PNG encoding, save-state serialization, or permanent
  `delete_state` semantics.
- No full workspace test or device distribution build.
- No hardware, adb, Samba, or SD-card access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store
cargo clippy -p slot2-store --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\38-reversible-state-delete-store.worker-result.md` with at most about 30
lines unless a failure needs more evidence:

- Success, failure, or partial completion and cumulative attempt number out of two.
- Changed files and the final take/restore/failure behavior.
- Each validation command, exit code, and concise result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste code, logs, or repeat this specification. A correction attempt must report only
the new delta, regression evidence, and final validation. If blocked, leave the code consistent,
report the blocker, and stop. Do not change models or configuration. This task has at most two
total attempts.
