# Task 109 - Codex final review

## Verdict

**Passed (cumulative attempt 2/2).** The current dirty tree passed the complete offline hygiene and local
release audit. Attempt 2 closes the only artifact parity defect: local and hosted producers now use one
strict `System/VERSION.txt` byte contract.

This is release-preparation evidence, not permission or eligibility to tag or publish. The validation tree
still contains dirty working-tree content while stamping historical HEAD `a8cb4af`.

## Confirmed correction

- `build/dist-device.ps1` writes the three public VERSION lines with explicit BOM-free UTF-8 and one LF per
  line, including the final LF. It no longer uses a shell text writer for the stamp.
- The assembled stamp is 212 bytes, strict UTF-8, contains zero CR bytes and exactly three LF bytes, and
  matches workspace version `0.1.0` plus short HEAD `a8cb4af`.
- `Assert-VersionStamp` now rejects a BOM, any CR, missing final LF, invalid UTF-8, wrong line count,
  semantic mismatch, version mismatch, and commit-prefix mismatch instead of normalizing producer drift.
- The package harness independently checks its input bytes and proves that BOM, CRLF, missing-final-LF, and
  invalid-UTF-8 mutations all fail through the production validator while preserving their destinations.
- `.github/workflows/device-artifact.yml` is unchanged and carries the same three normalized lines and
  hosted gate.

## Audit evidence retained

- Candidate hygiene found no staged entry, deletion, rename, ROM/disc candidate, ignored-output candidate,
  raw log, secret/token-shaped value, private key, symlink, or reparse point.
- The exact empty untracked root `$env` file was removed after all four fail-closed checks; it remains
  absent.
- Attempt 1 passed fmt, 973 workspace tests with zero failures/ignored, host and device clippy, six core
  tests, the full device build, the release package suite, and `git diff --check`.
- Attempt 2 passed PowerShell parsing, offline `dist-device.ps1 -NoBuild`, independent VERSION byte checks,
  the expanded package suite (65 passed, 0 failed/skipped), 25 focused source checks, and `git diff
  --check`.
- Final hashes of the three build files and both unchanged READMEs match the cumulative worker report.
- No commit, stage, push, tag, publication, remote change, network, hardware/card access, or shared
  configuration change occurred.

## Remaining release blockers

Before hosted acceptance or publication, the user must choose the intended commit set and explicitly
authorize a commit. The committed clean revision must then be rebuilt so VERSION identifies its actual
source. No remote is configured; push and tag actions require separate user authorization. Hosted cache
miss/hit artifacts, issue-form rendering, tag workflow, draft assets/checksum, fresh-card installation,
and broader hardware acceptance remain unverified.
