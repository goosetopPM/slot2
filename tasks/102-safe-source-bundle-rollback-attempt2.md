# Task 102 attempt 2 - Clean or report a partially copied candidate

This is cumulative attempt **2/2**, the final attempt for Task 102.

Read:

- `C:\SLOT2\tasks\102-safe-source-bundle-rollback.md`
- `C:\SLOT2\tasks\102-safe-source-bundle-rollback.worker-result.md`
- `C:\SLOT2\tasks\102-safe-source-bundle-rollback.result.md`
- `C:\SLOT2\build\package-core-sources.ps1`, lines 204 onward

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate,
commit, push, use the network, access hardware, or change shared configuration.

## Defect

The script sets `$candidateHolds = $true` only after this command returns successfully:

```powershell
Copy-Item -LiteralPath $stage -Destination $candidate -Recurse -Force
```

If the recursive copy creates `$candidate`, copies some files, and then fails, control enters `finally`
while `$candidateHolds` is still false. The partial candidate is neither removed nor reported. This
violates Task 102 requirement 8: residue is allowed only when cleanup actually fails, with its exact path
in a warning.

## Required fix

- Modify only `C:\SLOT2\build\package-core-sources.ps1` and the existing Task 102 worker report.
- Treat the GUID candidate path as script-owned from the moment it can exist. A failed or partial copy
  must trigger best-effort cleanup.
- Prefer checking actual path existence in state-aware cleanup, or set ownership state before the copy in
  a way that is correct for both pre-creation and partial-creation failures.
- If cleanup succeeds, no candidate remains. If an external lock prevents cleanup, preserve it and emit
  the existing exact-path warning without hiding the original copy error.
- Do not weaken the Task 102 rollback/backup preservation logic. In particular, `$preserve` must still
  prevent cleanup after rollback failure, and a promoted/validated output must not be mistaken for a
  candidate.
- Do not add a production test switch or environment variable.

## Verification

Use only the safe temporary staging repository. Add one failure scenario:

1. Force recursive candidate copy to fail after the candidate directory exists and at least one entry has
   been created.
2. Verify nonzero exit and that the real/old `OutputDir` tree and hashes are unchanged.
3. Without an external cleanup lock, verify the partial candidate is removed.
4. With the partial candidate locked, verify the original copy error remains visible and the warning names
   the exact candidate path. Release the handle and remove the temporary residue safely.

Then rerun the affected Task 102 final commands:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-core-sources.ps1 -OutputDir target/task102-core-sources-a
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-core-sources.ps1 -OutputDir target/task102-core-sources-b
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip
git diff --check
```

The two 28-file outputs must remain byte-identical to each other and to the Task 101 outputs. The dist
command must end with `==> done`. Rust code did not change, so do not rerun cargo tests or clippy.

Update `C:\SLOT2\tasks\102-safe-source-bundle-rollback.worker-result.md` as a cumulative **2/2** report.
Preserve the first-attempt evidence and append the fix, partial-copy failure evidence, four command results,
changed files, and whether code/content changed after final verification. If it still fails, report failure
and stop; there is no third attempt.
