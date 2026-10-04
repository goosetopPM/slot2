# Task 109 - Normalize VERSION stamp and close the audit (cumulative attempt 2/2)

## Goal

Fix the local/hosted `System/VERSION.txt` byte mismatch found by Task 109 attempt 1, enforce the normalized
stamp in the release packager, add focused regression tests, and update the audit report. This is the final
allowed Task 109 attempt.

Read only:

- `C:\SLOT2\tasks\109-offline-release-candidate-audit.result.md`
- `C:\SLOT2\tasks\109-offline-release-candidate-audit.worker-result.md`, sections 5-8 and 12
- `C:\SLOT2\build\dist-device.ps1`, VERSION write and final assembly only
- `C:\SLOT2\build\package-release.ps1`, `Assert-VersionStamp` only
- `C:\SLOT2\build\test-package-release.ps1`, setup, identity/tree refusals, and final summary only
- `C:\SLOT2\.github\workflows\device-artifact.yml`, VERSION write and `version stamp` check only

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
stage, push, publish, create or move a tag, configure a remote, use the network, access hardware or a card,
or change shared configuration. This is cumulative attempt 2/2; do not retry beyond it.

## Exact allowed files and baselines

- Modify `build/dist-device.ps1`.
- Modify `build/package-release.ps1`.
- Modify `build/test-package-release.ps1`.
- Update `tasks/109-offline-release-candidate-audit.worker-result.md` as cumulative 2/2.
- Normal ignored outputs under `target/`, `target-device/`, `dist-device/`, and worker-owned temporary
  scratch may change.

Do not modify any other file. Require these exact starting hashes or stop and report concurrent change:

- `build/dist-device.ps1`: 8672 bytes, SHA-256
  `946a429d1954e89da6972449e5c62dc303813ca7011d551d8ca5f8137482de45`
- `build/package-release.ps1`: 30719 bytes, SHA-256
  `068ac42a55b298e48bcc4effc86011faaf06cb589a8518cc2e53314edaf442ed`
- `build/test-package-release.ps1`: 33835 bytes, SHA-256
  `102121b7b7b8dc5e3866b4d1ab8a60ce43546c2ebfb2a3a1dfb851d51edf7d3a`

Also confirm before and after that `$env` remains absent, staged entries remain zero, and the two README
hashes remain `615e3ecd5f9ae011e14ff7a3c4b4374d9335bfbbb58bd2f84144691c3eb7a498`
and `631f69600ea8da6ac8de6b44f51f6428c1633a3d4c432a775f16d60f45ccf648`.

## Required implementation

### 1. Normalize the local producer

Replace the `Out-File -Encoding utf8` VERSION write in `build/dist-device.ps1` with an explicit .NET UTF-8
write that is compatible with Windows PowerShell 5.1 and writes **no BOM**. Construct exactly these three
semantic lines, each terminated by one LF byte (`0A`), with no CR byte:

1. `SLOT2 <workspace-version> (<short-head>)`
2. `Copy the contents of this folder to the root of the card BaseOS boots a frontend from.`
3. `BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.`

Use an explicit `System.Text.UTF8Encoding($false)` instance and `System.IO.File.WriteAllText`; do not rely
on shell defaults, `Out-File`, or `Set-Content`. Keep the file at exactly three lines and do not add a dirty
tree marker. Dirty/non-releasable status belongs in the audit report, not the public card stamp.

### 2. Enforce one stamp byte contract

Change `Assert-VersionStamp` in `build/package-release.ps1` so the release gate requires:

- valid UTF-8 decoded strictly, with invalid byte sequences rejected;
- no UTF-8 BOM;
- no CR byte anywhere;
- exactly three LF-terminated lines, including the final LF;
- the existing exact semantic lines, version, and 7-40 hex commit-prefix rules.

Give focused errors for BOM, CR/non-LF line endings, invalid UTF-8, missing final LF/wrong line count, and
the existing semantic mismatches. Update the function comment to describe the single normalized contract
and remove the obsolete statement that local BOM/CRLF and hosted LF are both accepted.

Do not change archive, sidecar, publication, cleanup, identity, license, source, or Rust-notice behavior.

### 3. Add focused regression coverage

Extend `build/test-package-release.ps1` so its normal input card tree is first required to carry the exact
normalized stamp. Add independent negative tree cases that prove the production validator rejects at
least:

- the same otherwise-valid stamp with a UTF-8 BOM;
- the same otherwise-valid stamp with CRLF;
- the same otherwise-valid three lines without the final LF;
- invalid UTF-8 bytes.

Each refusal must use the production `Assert-VersionStamp` path, expect the focused diagnostic, keep its
output parent fingerprint-identical, and leave no staging residue. Write mutated bytes explicitly; do not
let PowerShell encoding defaults construct the cases. Retain every existing 49 positive/negative check.

Add a structural check that the local producer uses BOM-free explicit UTF-8 plus LF construction and no
longer writes `System/VERSION.txt` through `Out-File` or `Set-Content`. Verify the reusable workflow still
writes and checks the identical three-line byte contract; do not modify the workflow.

## Focused verification

Do not repeat Cargo fmt, workspace tests, clippy, or core tests: attempt 1 already passed them and no Rust,
Cargo, core, or workflow input changes in this correction.

Run sequentially, with no network:

1. Parse all three changed PowerShell files with
   `[scriptblock]::Create((Get-Content -Raw <path>))` under Windows PowerShell 5.1.
2. `powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild`
3. Independently read `dist-device/System/VERSION.txt` as bytes and require: no BOM, zero CR, exactly three
   LF bytes, final byte LF, strict UTF-8, exact three semantic lines, current version and short HEAD.
4. `powershell -NoProfile -ExecutionPolicy Bypass -File build/test-package-release.ps1 -CardTree dist-device -ScratchRoot <fresh unique worker-owned temp path>`
5. Run a focused source scan proving the three mutated stamp cases use explicit byte writes, the validator
   is strict, the local and hosted producers carry identical text, and unrelated publication logic is
   unchanged from the starting `package-release.ps1` except inside `Assert-VersionStamp` and its comment.
6. Run `git diff --check`.

The dist command must end `==> done`; the expanded package suite must end `==> all checks passed`, with no
`[FAIL]` or `[SKIP]`. Do not create or retain a real release output pair. Confirm no worker-created staging,
candidate, or backup residue remains in the workspace.

## Cumulative report

Update `tasks/109-offline-release-candidate-audit.worker-result.md` as cumulative attempt 2/2. Retain the
attempt-1 hygiene and eight-gate evidence, then add:

- the corrected defect and explicit clarification that dirty status is report-only;
- final hashes/sizes and exact diff scope for the three build files;
- the normalized VERSION byte evidence;
- every focused command, exit code, elapsed time, expanded pass count, four refusal diagnostics, and
  structural/source checks;
- confirmation that README hashes stayed fixed, `$env` stayed absent, staged entries stayed zero, and only
  the three allowed build files plus this cumulative report changed outside ignored outputs;
- the remaining blockers: user commit-set decision and authorization, clean committed rebuild, remote/
  push/tag authorization, hosted acceptance, fresh-card acceptance, and broader hardware acceptance.

Stop after the report. Do not commit or continue to hosted acceptance.
