# Task 109 - Offline release-candidate and commit-set audit

## Goal

Run the final local release-readiness gates against the current uncommitted SLOT2 tree, remove one known
empty accidental root file under an exact fail-closed condition, and produce a reviewable commit-set audit.
This task prepares the tree for a later user-authorized commit and hosted acceptance. It does not create a
release candidate identity: the current tree is dirty, so any `System/VERSION.txt` stamped with current
HEAD is validation output only.

Read only:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state section
- this task in full
- `C:\SLOT2\AGENTS.md`, required rules and final commands
- `C:\SLOT2\docs\MILESTONES.md`, M7 only
- `C:\SLOT2\.gitignore`
- `C:\SLOT2\.github\workflows\ci.yml`, check job only
- `C:\SLOT2\.github\workflows\device-artifact.yml`, local-equivalent gates only
- headers/parameters of `build/dist-device.ps1`, `build/test-package-release.ps1`, and
  `build/package-release.ps1`
- `tasks/104-ci-license-input-cache.result.md`, `tasks/105-tag-release-workflow.result.md`, and
  `tasks/108-original-slot-migration-guide.result.md`

Do not read old worker logs unless a failing gate requires one exact historical baseline. Use status,
counts, hashes, and focused excerpts; do not dump the whole diff or full command logs.

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
stage, push, publish, create or move a tag, configure a remote, use the network, access hardware, access a
card, change shared configuration, or change product/source/document/workflow files. Maximum two attempts.
Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed filesystem changes

- Delete the exact path `C:\SLOT2\$env` **only** if every condition below is true before deletion:
  - it is a regular file, not a directory, link, junction, or other reparse point;
  - its length is exactly zero bytes;
  - `git status --porcelain=v1 -- '$env'` reports exactly `?? $env`;
  - its SHA-256 is the standard empty-file digest
    `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- Create or update `tasks/109-offline-release-candidate-audit.worker-result.md`.
- Normal ignored build outputs under `target/`, `target-device/`, `dist-device/`, and temporary scratch
  output created by the prescribed commands may change.

If the exact `$env` checks do not all pass, do not delete it; mark the audit `FAILURE` and report which
condition differed. Delete no other file. Do not use `git clean`, reset, checkout, restore, add, or stash.

## Phase 1 - immutable repository snapshot and candidate hygiene

Before deletion or testing, record without network access:

- branch, full HEAD, HEAD subject/date, workspace Cargo version, local tags, remote names/URLs as printed by
  `git remote -v` (or explicitly none), and whether an upstream is configured;
- staged entry count, tracked-modified count, deleted count, renamed count, and untracked count;
- changed-path counts by top-level directory and by status, without pasting all 400+ paths;
- a SHA-256 fingerprint of the sorted NUL-safe `git status --porcelain=v1` records, with the method stated;
- the exact `$env` facts above, then remove it if and only if allowed.

The current HEAD is historically `a8cb4af04a403869671a4e829d5b97de45157cfe`; record what is actually on
disk rather than treating that value as a required baseline. A dirty build stamped with that commit must
be labeled a **validation build**, never the artifact of that commit.

Audit the commit candidates after the allowed deletion:

1. Require zero staged entries and zero tracked deletions/renames unless the existing status itself proves
   otherwise; any unexpected deletion/rename is a failure and must not be repaired here.
2. Confirm ignored build/local roots do not appear as candidates: `target/`, `target-device/`, `dist/`,
   `dist-device/`, `vendor/`, `sdcard/`, `.gjc/`, `.gjc-logs/`, and `assets/test/local/*` except its tracked
   README. Do not enumerate or hash local ROM filenames/content in the report.
3. Fail if any tracked/untracked candidate has a ROM/disc extension (`.gb`, `.gbc`, `.gba`, `.nes`, `.sfc`,
   `.smc`, `.md`, `.gen`, `.rom`, `.iso`, `.cue`, `.chd`, `.7z`), an obvious private-key file, raw HTTP
   request log, general `.log`, or a path under the ignored roots above.
4. Scan candidate file contents for private-key blocks and plausible live-token forms (`ghp_` or
   `github_pat_` followed by a token-length value, and `sk-` followed by at least 20 token characters)
   without printing matched text. Report only safe counts and paths. Plain documentation names such
   as `BAI_API_KEY` are not secrets. If a possible live value is found, fail and name only the path and
   pattern class.
5. Detect candidate symlinks/reparse points and report them. Fail unless the path is an intentional tracked
   repository contract already present at HEAD; do not follow a link while hashing or scanning.
6. For untracked text candidates, check UTF-8 decodability where applicable, trailing whitespace, and final
   newline. Exempt binary assets by type/signature and list only aggregate counts. `git diff --check` still
   covers tracked changes.
7. Group the candidate set into product source/tests, assets/provenance, build/workflows, licenses,
   public documentation, project operations/handoff, and task specifications/results. Give counts and a
   concise representative path list for review. Do not decide to exclude tracked-precedent operations or
   task history; flag the policy choice for the user if it materially affects a public commit.

## Phase 2 - sequential offline gates

Set Cargo offline for this process (`CARGO_NET_OFFLINE=true`). Do not run overlapping Cargo commands or
overlapping builds against the same target. Capture full output to worker-owned temporary logs, but put
only command, exit code, elapsed time, and the smallest useful final/error lines in the report. Do not add
logs to the repository.

Run in this order and stop after the first failure. Do not edit source to make a gate pass.

1. `cargo fmt --all -- --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo clippy -p slot2 -p slot2-gfx -p slot2-ui -p slot2-input --no-default-features --features slot2/device -- -D warnings`
5. `cargo test -p slot2-retro --test cores -- --nocapture`
6. `powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1`
7. `powershell -NoProfile -ExecutionPolicy Bypass -File build/test-package-release.ps1 -CardTree dist-device -ScratchRoot <unique worker-owned temp path>`
8. `git diff --check`

Network is forbidden. If a missing Cargo source, Docker image, core checkout, or other cache would require a
download, stop and report that external prerequisite. Do not retry with network access. `dist-device.ps1`
must finish with `==> done`; the package-release test must finish with `==> all checks passed`.

For the workspace test, count every test-result line and sum passed/failed/ignored. The last verified
source baseline was 973 passed, 0 failed, 0 ignored at Task 101; source behavior has not intentionally
changed since then. Treat a lower total, any failure, or any unexpected ignored test as a regression.

For the core test, distinguish a lawful missing test ROM from a missing core. The command must not contain
the missing-core instruction `run build/cores.ps1`; report which platform tests actually ran without naming
or hashing files under `assets/test/local/`.

After the device build and package test, verify and report without dumping the tree:

- `dist-device` contains exactly one top-level `System/`, a nonempty `System/frontend`, the exact six-core
  set and stamps required by `cores/required.txt`, and the validated license/source/Rust notice trees;
- `System/VERSION.txt` is UTF-8 without BOM, LF-only, exactly three lines, matches Cargo version and current
  short HEAD, and is explicitly marked non-releasable while the source tree is dirty;
- the release test's checked archive/file count, checksum/sidecar result, refusal tests, and final pass
  count;
- no staging/candidate/backup residue created by the release test remains in the workspace.

Do not run `build/package-release.ps1` as a real publication command and do not retain a release output
pair. The existing test harness is the required local proof.

## Phase 3 - final status and release blockers

Take a second status snapshot. It must differ from the post-cleanup snapshot only by the worker result file;
ignored build outputs do not appear. Require zero staged entries. Record the final sorted-status fingerprint
and explain the expected difference caused by this report.

State explicitly that no artifact is eligible for tagging or publication until, at minimum:

1. Codex reviews this audit;
2. the user chooses the intended commit set and explicitly authorizes a commit;
3. the committed clean revision is rebuilt so `VERSION.txt` identifies its actual source;
4. a remote exists and the user authorizes push/tag actions;
5. hosted CI cache-miss/cache-hit artifacts, issue-form rendering, tag workflow, draft assets, and checksum
   are inspected;
6. fresh-card and broader hardware acceptance remain user-run checks.

Do not create the next task, commit plan commands, a tag name beyond the current workspace version, or a
release announcement.

## Worker report

Write `tasks/109-offline-release-candidate-audit.worker-result.md` with:

- `SUCCESS` only if every hygiene rule and gate passed; otherwise `FAILURE`, the stopped phase, and no claim
  of release readiness;
- cumulative attempt count;
- before/after repository identity, counts, status fingerprints, staged state, and exact `$env` disposition;
- safe commit-candidate category table and prohibited-file/secret/link scan results;
- every gate command, exit code, elapsed time, concise evidence, total tests, core coverage, device-tree
  summary, and package-test summary;
- confirmation that only the allowed empty `$env` file was removed and only this report was added outside
  ignored build outputs;
- explicit confirmation of no delegation, commit, stage, push, tag, publish, remote change, network,
  hardware/card access, or shared-configuration change;
- the unresolved release blockers above.

Stop after the report. Do not continue into a commit or hosted acceptance.
