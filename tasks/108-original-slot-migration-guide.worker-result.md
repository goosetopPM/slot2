# Task 108 - Original slot card migration guide: cumulative worker result

**SUCCESS** - cumulative attempt **2 / 2** (the final allowed attempt).
Attempt 1 added the bilingual migration guides and the three README references. Attempt 2
(`tasks/108-original-slot-migration-guide-attempt2.md`) corrected the three unsupported runtime/recovery
statements that Codex raised in `tasks/108-original-slot-migration-guide.result.md`, in both guides only.

Documentation only. No behavior, code, packaging, workflow or release artifact was changed in either
attempt. No commit, push, tag, release, issue, pull request, network request, hardware access, or shared
configuration change.

## 1. Attempt 2: the three corrected defects

| # | Defect in attempt 1 | Correction now in both guides |
| --- | --- | --- |
| 1 | The diagnostics bullet said a failed state move is recorded in `System/slot2-diag.txt` | A state-adoption failure is written to the frontend's standard error output (**stderr**), which BaseOS keeps in the volatile `/tmp/frontend.log`; the reader is told to copy the lines before the next boot. `System/slot2-diag.txt` is described as what it is - the boot survey the frontend writes when it starts, readable in a card reader - and the guide states explicitly that runtime state-adoption errors are **not** appended to it |
| 2 | The collision bullet told the user to "load the state you want on the device to confirm" | A refusal now says SLOT2 keeps listing and using the state already in the namespace of the core that resolves for that game, and that the refused flat state stays outside SLOT2's state list and cannot be selected in the frontend. The reader is told to stop, keep the card and its backup, copy only the smallest relevant redacted excerpt from `/tmp/frontend.log` if available, and report the conflict; the pre-migration backup and the section-6 rollback are the safe route back, and the files must not be merged or moved by hand |
| 3 | A mid-move rename failure was described as "rolled back ... the flat files stay", and pulling the card mid-write as "the one thing" that could damage a recoverable migration | The guide now separates the safe preflight refusal (destination exists before the first rename: the whole game is refused and the flat originals are untouched) from an I/O failure after moves have begun: SLOT2 attempts a **best-effort reverse rename** and reports the original error, a reverse rename can fail as well, so it does **not** guarantee that every flat file is back where it started. The reader is told not to keep playing, not to tidy either directory by hand, to shut down, inspect and preserve both locations, and to restore from the verified pre-migration backup. The exclusivity claim about mid-write removal is gone; the shutdown-before-removal instruction stays |

Nothing else was reworded. The off-card backup, the complete `System/` replacement, the no-merge rule, the
core-specific state warning, the first-check-before-play rule and the post-start `States/` rollback are all
unchanged.

## 2. Baselines (all four verified before the attempt-2 edit)

| File | Required baseline (attempt-2 contract) | Read from disk | Result |
| --- | --- | --- | --- |
| `docs/MIGRATION.md` | 16094 bytes, `968c1f9aa5a7c9c13470dd8c6bc1d5c5c077654f701db6f1045559d20740aeea` | 16094 bytes, `968c1f9a...40aeea` | matched |
| `docs/MIGRATION.ko.md` | 19191 bytes, `58c4e78b7a67331e99fe342b25021bb0b8adf63df2fe66df7ae9bc6873ed2fc8` | 19191 bytes, `58c4e78b...ed2fc8` | matched |
| `README.md` | `615e3ecd5f9ae011e14ff7a3c4b4374d9335bfbbb58bd2f84144691c3eb7a498` | `615e3ecd...b7a498` | matched |
| `README.ko.md` | `631f69600ea8da6ac8de6b44f51f6428c1633a3d4c432a775f16d60f45ccf648` | `631f6960...5ccf648` | matched |

Both READMEs still carry exactly those hashes **after** the guide edits: `README.md` `615e3ecd...b7a498`
(17117 bytes) and `README.ko.md` `631f6960...5ccf648` (19975 bytes) - attempt 2 did not touch either of them.

## 3. Files

| Path | Action | Bytes (attempt 1 -> attempt 2) | Lines | SHA-256 (attempt 2) | mtime (local, UTC+9) |
| --- | --- | --- | --- | --- | --- |
| `docs/MIGRATION.md` | four paragraphs corrected | 16094 -> **17127** (+1033) | 219 | `0641ebd69005124b4e7bb3dd78ac602e32e8ddec45f44dfda606d6545f3ca10d` | 2026-10-04 08:39:32.162885900 |
| `docs/MIGRATION.ko.md` | the same four paragraphs corrected | 19191 -> **20384** (+1193) | 210 | `1aabffdeefd5e39412568f828e01aa995942281ff13df4910e21b238e1b7f4f6` | 2026-10-04 08:39:47.430989500 |
| `README.md` | attempt 1 only: three references | 16899 -> 17117 | 322 | `615e3ecd5f9ae011e14ff7a3c4b4374d9335bfbbb58bd2f84144691c3eb7a498` | 2026-10-04 00:24:19.126943500 |
| `README.ko.md` | attempt 1 only: three references | 19806 -> 19975 | 307 | `631f69600ea8da6ac8de6b44f51f6428c1633a3d4c432a775f16d60f45ccf648` | 2026-10-04 00:24:20.124179900 |
| `tasks/108-original-slot-migration-guide.worker-result.md` | this cumulative report | - | - | - (self-referential) | 2026-10-04 |

A scan of files modified after 08:30 local lists, besides the two guides (08:39:32 and 08:39:47), only
`tasks/108-original-slot-migration-guide.result.md`, `tasks/108-original-slot-migration-guide-attempt2.md`,
`docs/HANDOFF-CODEX.md` and `.claude/resume.md`, all written at 08:35-08:36 by the review step of the
workflow *before* this attempt's edits; none of them was touched by this attempt. Only this report is
written afterwards.

## 4. Guide content (unchanged from attempt 1 except the four paragraphs)

Both guides keep the same eight sections in the same order, with the reciprocal language link in their first
lines and repository-relative links only: 1 Identify and prepare, 2 Make and verify an off-card backup,
3 Inspect the release away from the card, 4 Replace only `System/`, 5 First SLOT2 start and the state move,
6 Rollback, including after SLOT2 has already scanned the card, 7 Troubleshooting, 8 Reporting a problem
(Korean: 확인과 준비 / 카드 밖 백업을 만들고 검증하기 / 카드 밖에서 릴리스 검사하기 / `System/`만
교체하기 / SLOT2 첫 실행과 스테이트 이동 / 롤백 (SLOT2가 이미 스캔한 뒤에도) / 문제 해결 / 문제 보고).

The parity matrix still checks **100 required items** against both guides (warnings, paths, state rules,
failure stops, rollback steps, distinct-diagnostic and collision wording, and the intro's three
distinctions), including the corrected claims added in attempt 2:

- collision refusal is a safe preflight, distinct from an I/O failure after the move has begun;
- best-effort reverse rename with no guarantee that every flat file returned;
- stop playing, do not tidy either directory by hand, inspect and preserve both locations, restore from the
  pre-migration backup;
- the collision bullet keeps the namespaced state selectable and the flat state outside the list, and sends
  the reader to stop, keep the card and backup, take a redacted `/tmp/frontend.log` excerpt and report;
- the diagnostics bullet names stderr and the volatile `/tmp/frontend.log`, describes `System/slot2-diag.txt`
  as the boot survey, and says adoption errors are not appended to it.

## 5. Source locations behind the corrected claims

| Claim | Source read |
| --- | --- |
| an adoption failure goes to stderr (`eprintln!`), and the flat files stay | `crates/slot2/src/app.rs` `rescan`, around lines 671-676 (`eprintln!("slot2: cannot move {}'s older states into {}: {e}; leaving them where they are", ...)`) |
| stderr is what BaseOS keeps in the volatile `/tmp/frontend.log` | `docs/DESIGN.md` §2 (로그 row: `stdout/stderr → /tmp/frontend.log (휘발)`) |
| `System/slot2-diag.txt` is the first-boot hardware survey, written to stderr and to the card | `crates/slot2/src/diag.rs` module doc ("First-boot hardware survey ... Written to stderr (BaseOS keeps that in `/tmp/frontend.log`) and to `System/slot2-diag.txt` on the card") |
| that card file is one write of the survey, not an append log | `crates/slot2/src/diag.rs:123` (`let _ = std::fs::write(root.join("System").join("slot2-diag.txt"), &out);`) |
| the survey runs when the frontend starts | `crates/slot2/src/device_app.rs:45` (`eprint!("{}", crate::diag::report(&boot.root));`) |
| SLOT2 lists and uses only the resolved core's namespace, so the refused flat state is not selectable | `crates/slot2/src/app.rs:2225` (and 2288, 2297, 2335, 2523) `scoped_list_states(&cart, &namespace)`, `app.rs:681` / `2804` `scoped_state_path`, `crates/slot2/src/session.rs:1027` `scoped_read_state` |
| a destination collision is refused before the first rename, both state and PNG destinations checked | `crates/slot2-store/src/card.rs` `adopt_legacy_states` (`is already there`, `dest.with_extension("png")`), plus `crates/slot2/tests/core_state_routing.rs::a_flat_state_that_cannot_move_is_left_where_it_is_and_never_merged` |
| rollback is best effort and ignores a failed reverse rename | `crates/slot2-store/src/card.rs:267-271` `fn roll_back` (`let _ = std::fs::rename(to, from);`), with the doc comment ending "back is broken in a way this cannot repair" |

## 6. Verification: commands and results

`verify108.py` (54437 bytes, SHA-256
`79aefce42835e3c1fb2ae11fd4f9cca719c485527fb8d72f7406bfed6874cc93`, outside the repository), run twice with
byte-identical output:

| Check | Result |
| --- | --- |
| Full suite, run 1 and run 2 (attempt-2 suite) | `=== summary: 154 [OK], 0 [FAIL] ===`, exit 0 and exit 0; window `2026-10-03T23:44:21Z` -> `23:44:26Z` (2026-10-04 08:44:21 - 08:44:26 local) |
| `git diff --check` (inside the suite) | exit 0, 0 non-warning lines |

The suite covers the attempt-2 contract's six focused items:

1. **Baselines**: all four required values were checked on disk before editing, and the two README hashes are
   re-asserted after the guide edits (section 2 above).
2. **Source check** (34 assertions in total, section 5 above): stderr/`eprintln!` for adoption failures, the
   `diag.rs` module doc and its single `std::fs::write`, the survey call in `device_app.rs`, the scoped
   listing/path/read calls that make the flat source unselectable, the preflight destination check, and the
   best-effort `roll_back` that drops a failed reverse rename.
3. **Equivalent corrected claims, no stale sentence**: 12 matrix items for the corrected wording across both
   guides, plus 8 explicit absence checks - the English "the one thing that can", Korean "유일한 경우", the
   "load the state you want on the device" instruction and its Korean equivalent, the old "reported in the
   log; the flat files stay" sentence and its Korean equivalent, and the claims that a failed move is
   recorded in `slot2-diag.txt` (both languages) are all gone. Each guide mentions `slot2-diag.txt` exactly
   once, and that mention is the corrected boot-survey wording.
4. **File hygiene**: both guides are UTF-8, **no BOM**, **LF-only**, end with a final newline, have no
   trailing whitespace, keep two balanced `powershell`/`sh` fences, and every relative link resolves
   (`MIGRATION.ko.md`/`MIGRATION.md`, `DESIGN.md`, `../.github/ISSUE_TEMPLATE/bug-report.yml`); neither guide
   contains an external URL.
5. **Focused diff**: reversing only the four corrected paragraphs in each guide reproduces the attempt-1
   bytes exactly - `docs/MIGRATION.md` 16094 bytes / `968c1f9a...40aeea`, `docs/MIGRATION.ko.md` 19191 bytes /
   `58c4e78b...ed2fc8`. The `-U0` diff against that reconstruction is 3 hunks per guide (sections 5 and 6 are
   separate, while the two section-7 bullets are adjacent and share a hunk), and **every changed line lies
   inside sections 5-7** (English lines 108-211, Korean 104-202): no other section, heading, list item or
   fence moved.
6. **README locality retained**: the attempt-1 proof still passes - reversing the three README edits
   reproduces the Task 107 bytes (`README.md` 16899 / `e10a30e6...82081`, `README.ko.md` 19806 /
   `e2e78e5b...75012`), 3 hunks each, +8/-6 and +6/-5 lines, every changed line inside an allowed edit.

No Cargo test, clippy, device distribution build, packager, workflow or hardware check was run. Nothing ran
against the network.

## 7. Scope confirmation

- Attempt 2 changed exactly two files: the four corrected paragraphs in `docs/MIGRATION.md` and the
  equivalent four in `docs/MIGRATION.ko.md`, then this report. Both READMEs are byte-identical to the values
  attempt 1 left.
- No other guide text changed (`documents`-level proof: the reversal reproduces the attempt-1 hashes and the
  diff is confined to sections 5-7).
- `docs/MILESTONES.md`, `docs/DESIGN.md`, `docs/DECISIONS.md`, `docs/HANDOFF-CODEX.md`, the READMEs, the
  issue forms, the translation guides, code, tests, language packs, workflows, build scripts, assets and
  licence files were **not** modified in either attempt.

## 8. Remaining limits (unchanged, and still stated in the guides)

- **No published release**: no download page, asset name or digest to quote; the guides point at the README's
  checksum commands and describe the expected zip shape instead.
- **No hosted-link or render acceptance**: every link was resolved on disk only; how GitHub renders the
  guides and the issue form was not exercised (no network).
- **No physical original-card migration acceptance**: no original slot card has been migrated on hardware,
  and neither the described adoption nor the rollback has been exercised on a real card. The corrected
  statements describe the implementation's behavior and its own limits, not a device result.
- **Core-specific states stay core-specific**, and a partially failed adoption is described as what it is -
  a best-effort recovery with the backup as the reliable route - rather than a guaranteed restoration.
- Anything a future BaseOS or SLOT2 version changes in the launch, card-layout or state contracts would
  require these guides to be revisited.
