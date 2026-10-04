# Task 108 - Codex final review

## Verdict

**Passed (cumulative attempt 2/2).** SLOT2 now has equivalent English and Korean guides for migrating an
original `slot` data card. They require a verified off-card backup, complete non-merged `System/`
replacement, careful first-run checks, and rollback that restores `States/` after a scan.

## Confirmed attempt-2 corrections

- Legacy-state adoption failures are correctly described as stderr output captured in volatile
  `/tmp/frontend.log`. `System/slot2-diag.txt` is identified separately as the startup boot survey and is
  not presented as a runtime error log.
- A destination collision is distinguished from a later I/O failure. Preflight collision leaves flat
  originals untouched; SLOT2 continues to expose only the already namespaced state, while the refused flat
  state remains outside the UI's scoped list.
- A partial rename failure is followed by a best-effort reverse rename, with no promise that every file
  returned. The guides direct the user to stop, preserve both locations, and restore the verified backup.
- The unsupported instruction to compare both colliding states in SLOT2 and the exclusive card-removal
  damage claim are gone in both languages.

## Delivered documentation

- `docs/MIGRATION.md` and `docs/MIGRATION.ko.md` cover card identification, checksum verification,
  independent full backup, staging, complete `System/` replacement, compatible root data, default-core
  state adoption, rollback, troubleshooting, and safe reporting.
- `README.md` and `README.ko.md` link the language-appropriate guide from the fresh-install warning and
  project-document section. Their attempt-2 hashes are unchanged from attempt 1.
- No public release, hosted rendering, automatic whole-card migration, cross-core state portability, or
  physical original-card acceptance is claimed.

## Verification reviewed

- Both corrected guide hashes and both unchanged README hashes match the cumulative worker report.
- Focused attempt-2 verification reports 154 passed, 0 failed, twice with byte-identical output.
- UTF-8/no-BOM, LF-only, final-newline, trailing-whitespace, Markdown fence, relative-link, bilingual
  parity, source-contract, stale-claim, focused-diff, README reconstruction, and `git diff --check` checks
  passed.
- Codex independently rechecked the corrected paragraphs against `app.rs`, `diag.rs`, and `card.rs`, and
  confirmed the four final file hashes and document hygiene.
- Cargo, clippy, distribution, network, hosted GitHub, and hardware checks were correctly omitted because
  this task changed documentation only.

## Remaining acceptance

The migration procedure has not been exercised on a physical original card. Hosted issue-form rendering,
hosted release/cache/tag/draft acceptance, fresh-card first-user acceptance, and broader physical-device
acceptance also remain open.
