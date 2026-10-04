# Task 105 attempt 2 - Atomic release-pair publication

This is cumulative attempt 2/2. Fix only the publication defect identified by Codex in
`C:\SLOT2\tasks\105-tag-release-workflow.result.md`. Keep all successful attempt-1 behavior and update the
existing worker report as a cumulative 2/2 report.

Read only:

- `C:\SLOT2\tasks\105-tag-release-workflow.md`, publication and verification contracts
- `C:\SLOT2\tasks\105-tag-release-workflow.result.md`
- `C:\SLOT2\build\package-release.ps1`, staging/publication section and directly used helpers
- `C:\SLOT2\build\test-package-release.ps1`, output-preservation tests
- `C:\SLOT2\tasks\105-tag-release-workflow.worker-result.md`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
push, publish, create a tag, use the network, access hardware, or change shared configuration. This is the
last allowed attempt. Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed files

- Modify `build/package-release.ps1`.
- Modify `build/test-package-release.ps1`.
- Update `tasks/105-tag-release-workflow.worker-result.md` as the cumulative attempt 2/2 report.

Do not modify any workflow, documentation, Rust/Cargo file, existing shared packager/validator, card tree,
license, asset, or Task 105 instruction/result file.

## Exact correction

The staged zip and sidecar are one release unit. Do not move them into the final output one file at a time.

- Require `OutputDir` itself to be absent before staging begins. If it already exists as a directory, file,
  or link, fail before writing and leave it byte-for-byte unchanged. This deliberately stricter rule keeps
  the public interface safe; the release workflow already passes a fresh runner-temporary path.
- Continue to stage beside `OutputDir`, validate the complete zip and sidecar there, and ensure the staging
  directory contains exactly those two expected files.
- After all validation, publish with one same-volume directory rename from the owned staging directory to
  the absent `OutputDir`.
- Track whether the staging directory was promoted. Cleanup may remove only an unpromoted staging directory
  owned by this invocation. It must never delete or alter the promoted output.
- If the final directory name appears before the rename, or the rename otherwise fails, leave that
  destination untouched, clean only the owned staging directory, and fail with a useful destination path.
- Do not add a fallback copy, file-by-file promotion, overwrite, backup deletion, retry loop, or test-only
  environment switch to production code.
- After a successful rename, confirm the final directory holds exactly the expected zip and sidecar and
  revalidate the sidecar against the final zip. A post-promotion validation failure must name the retained
  output path for manual inspection; do not delete it automatically.

The resulting success output and archive contents must remain byte-equivalent to attempt 1 for the same
input tree, except that normal zip timestamp variance remains allowed.

## Focused verification

Run offline and do not rebuild the card tree:

1. Package the current last-known-good `dist-device` into two fresh output paths. Both must succeed and each
   output directory must contain exactly the zip and sidecar.
2. Re-run against an existing output directory containing a marker, including an otherwise empty directory
   plus marker. It must fail before staging/publication and preserve the full directory fingerprint.
3. Exercise the actual directory-promotion helper with a staged two-file pair and a destination containing a
   marker created after initial setup. The rename must fail, the marker/destination fingerprint must remain
   unchanged, the owned staging path must be cleaned, and no zip or sidecar may appear at the destination.
   Use a focused helper call or safe temporary harness; do not add a production failure-injection switch.
4. Prove structurally that production publication has exactly one directory move and no final zip/sidecar
   file moves.
5. Re-run the existing identity, missing-core, extra-root, tampered-notice, VERSION, archive-entry, extracted
   tree, two-run, and sidecar checks. Adjust only the pre-existing-output expectation to the stricter
   `OutputDir` rule.
6. Run `git diff --check`.

Do not run workflows, `gh`, Docker, Cargo tests, clippy, core builds, cross builds, `dist-device`, tag, push,
or publication. After final verification, do not change files.

## Cumulative worker report

Update `C:\SLOT2\tasks\105-tag-release-workflow.worker-result.md` and clearly state attempt 2/2. Preserve the
useful attempt-1 evidence and add:

- the prior two-file promotion defect;
- the final absent-output and single-directory-rename state machine;
- successful output file sets and hashes;
- pre-existing-output and late-destination-collision fingerprints;
- owned staging cleanup and promoted-output ownership evidence;
- structural move count, full focused verification results, commands, exit codes, final file list/time, and
  whether content changed afterward.

Do not commit.
