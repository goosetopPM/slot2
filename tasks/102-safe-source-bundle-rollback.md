# Task 102 - Preserve the last good source bundle on rollback failure

## Goal

Fix only the final directory replacement in `build/package-core-sources.ps1`.

Task 101 reached its 2/2 limit. This is a separate follow-up with a corrected Windows contract:
preserving the last known-good bundle is more important than deleting locked temporary directories.
A locked candidate or backup may remain if deleting it would risk the previous bundle. The script must
print its exact recovery path and fail clearly.

Read these files first:

- `C:\SLOT2\tasks\101-core-license-source-bundle.result.md`
- `C:\SLOT2\tasks\101-core-license-source-bundle.worker-result.md`, section 12 only
- `C:\SLOT2\build\package-core-sources.ps1`, especially lines 158 onward

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate,
commit, push, use the network, access hardware, or change shared configuration.

## Allowed files

- Modify: `C:\SLOT2\build\package-core-sources.ps1`
- Create/update report: `C:\SLOT2\tasks\102-safe-source-bundle-rollback.worker-result.md`

Do not modify Task 101 reports or any other code, workflow, license, documentation, source archive,
checkout, pin, patch, build output, or distribution content.

## Required replacement state machine

Keep the existing same-parent candidate and `[System.IO.Directory]::Move` approach. Replace the
unconditional cleanup with explicit state-aware handling.

1. Build and validate the candidate before touching `OutputDir`.
2. If moving the old `OutputDir` to backup fails, leave the old output unchanged. Candidate cleanup is
   best effort. If a lock prevents cleanup, report the candidate path; do not hide the original error.
3. If candidate promotion succeeds, validate the promoted `OutputDir` before considering replacement
   complete. Only after successful promotion and validation may the old backup be deleted.
4. If promotion or post-promotion validation fails, restore the old backup when one exists.
5. If restoration succeeds, the old output must be byte-identical to its pre-run state. Cleanup of the
   rejected candidate/new output is best effort.
6. If restoration fails, **never delete or overwrite the backup**. Leave every recoverable directory in
   place and throw an error that contains:
   - the original promotion/validation failure;
   - the rollback failure;
   - the exact backup path holding the last known-good bundle;
   - the exact candidate or rejected-output path when it still exists.
7. A `finally` block must not unconditionally delete backup. Cleanup must depend on confirmed state.
8. Locked candidate/backup residue is allowed only when cleanup actually fails. Emit a warning with its
   exact path. Do not claim that successful or failed runs always leave no residue.
9. On an ordinary successful replacement, no candidate or backup should remain.
10. Reject an existing `OutputDir` that is not a normal directory before creating candidate data. Keep
    the existing repository-root and filesystem-root safety guards.

Do not add a production test switch or environment variable. Do not weaken source, license, pin,
archive, recipe, hash, deterministic-output, or path safety validation.

## Safe failure verification

Use only a copied staging repository under the system temporary directory. Do not rename, delete, lock,
or modify the real `target/core-sources`, `dist-device`, tracked licenses, or core checkouts. Read-only
junctions to the existing core checkouts are acceptable if handled safely.

Verify and report these cases:

1. Normal replacement over an old marker tree: exit 0, valid 28-file bundle, no candidate/backup.
2. Move-aside failure: nonzero, old marker tree and hashes unchanged.
3. Promotion failure followed by successful rollback: nonzero, old marker tree and hashes unchanged.
4. Promotion failure followed by rollback failure: nonzero, backup remains byte-identical to the old
   marker tree, the error prints the exact backup recovery path, and no cleanup deletes it.
5. Locked residue cleanup failure: nonzero or the original operation result as appropriate, exact path
   warning, recoverable content preserved. Release the test handle and clean the temporary harness
   safely afterward.
6. Existing output path is a regular file: nonzero before candidate creation, file unchanged.

The harness may coordinate locks and filesystem races externally, but it must not add test hooks to the
production script.

## Final verification

The Rust code did not change, so do not repeat workspace tests or clippy that Task 101 already passed.
Run only the affected checks:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-core-sources.ps1 -OutputDir target/task102-core-sources-a
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-core-sources.ps1 -OutputDir target/task102-core-sources-b
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip
git diff --check
```

Compare the two bundle outputs by sorted relative path and SHA-256. They must be identical to each other,
contain 28 files, and retain Task 101's deterministic manifest and bundle digest unless the report gives
a concrete contract reason for a difference. `dist-device.ps1 -NoBuild -Zip` must end with `==> done` and
retain the six-core/license/source tree.

After final verification, do not change code or content.

## Report

Write `C:\SLOT2\tasks\102-safe-source-bundle-rollback.worker-result.md` even on failure. Include:

- success/failure and cumulative attempt count, maximum 2;
- the exact replacement states and cleanup rules implemented;
- all six safe failure scenarios, exit codes, preservation hashes, residue/recovery paths;
- the four final commands, exit codes, and last result lines;
- deterministic file count, manifest hash, bundle digest, final dist/zip counts;
- changed files and whether anything changed after final verification;
- any contract concern.

Wait up to 5 minutes for the first response and up to 45 minutes total. If this attempt fails, stop after
writing the report; do not silently retry or change models.
