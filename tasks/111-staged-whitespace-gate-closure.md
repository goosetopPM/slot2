# Task 111 - Staged whitespace gate closure and commit-set refreeze

## Goal

Close the whitespace defects exposed only after the Task 110 manifest was staged for the user-authorized
commit, preserve upstream licence bytes, and freeze a replacement exact commit manifest. The attempted
commit did not occur: Codex stopped when `git diff --cached --check` failed and restored the real index to
zero staged entries.

Read only:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state entries only
- this task in full
- `C:\SLOT2\AGENTS.md`, required rules only
- `C:\SLOT2\.gitattributes`
- `C:\SLOT2\tasks\110-public-commit-set-freeze.result.md`
- `C:\SLOT2\tasks\110-public-commit-set-freeze.worker-result.md`, sections 3-6 only
- the exact affected paths listed below, and no unrelated task contents

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, stage in
the real index, commit, reset, restore, checkout, stash, clean, push, publish, create or move a tag,
configure a remote, use the network, access hardware or a card, or change shared configuration. Maximum
two attempts. Wait up to 5 minutes for the first response and 45 minutes total.

## Why Task 110 needs this closure

Before staging, `git diff --check` could inspect tracked diffs but could not inspect the 390 newly added
files. After the exact 449-path manifest was staged, `git diff --cached --check` exposed 71 findings in 45
paths: 42 `new blank line at EOF` findings in task history, one trailing space in the historical Task 78
worker report, and 28 trailing-space findings in two byte-preserved upstream core licences. Codex did not
commit and returned the real index to zero staged entries.

This is a validation-gap correction. Do not rewrite product code or repeat Cargo/build tests.

## Allowed changes

Edit only:

- `.gitattributes`
- the 43 task-history files listed in the next section
- `tasks/111-staged-whitespace-gate-closure.manifest.txt`
- `tasks/111-staged-whitespace-gate-closure.worker-result.md`

Do not edit either upstream licence file. Temporary files must stay outside the workspace and be removed
before the report.

## Phase 1 - exact mechanical cleanup

For each path below, remove exactly one redundant blank line at EOF while preserving all other bytes and
leaving exactly one final LF:

- `tasks/101-core-license-source-bundle.worker-result.md`
- `tasks/39-state-switcher-delete-undo.result.md`
- `tasks/40-pi-delegation-smoke.result.md`
- `tasks/69-platform-shader-defaults-attempt2.md`
- `tasks/70-shader-menu-ui.md`
- `tasks/70-shader-menu-ui.result.md`
- `tasks/71-shader-menu-app-wiring.md`
- `tasks/71-shader-menu-app-wiring.result.md`
- `tasks/72-overscan-menu-ui.md`
- `tasks/72-overscan-menu-ui.result.md`
- `tasks/73-overscan-menu-app-wiring.md`
- `tasks/73-overscan-menu-app-wiring.result.md`
- `tasks/74-game-overlay-settings-store.md`
- `tasks/74-game-overlay-settings-store.result.md`
- `tasks/75-overlay-asset-layer.md`
- `tasks/75-overlay-asset-layer.result.md`
- `tasks/76-overlay-app-runtime-wiring.md`
- `tasks/76-overlay-app-runtime-wiring.result.md`
- `tasks/79-gb-cubexx-built-in-overlay.result.md`
- `tasks/80-global-timezone-settings-store.result.md`
- `tasks/81-timezone-startup-app-wiring.md`
- `tasks/81-timezone-startup-app-wiring.result.md`
- `tasks/82-timezone-menu-ui.md`
- `tasks/82-timezone-menu-ui.result.md`
- `tasks/83-shelf-menu-ui.result.md`
- `tasks/84-shelf-timezone-app-wiring.result.md`
- `tasks/85-about-sticker-ui.md`
- `tasks/85-about-sticker-ui.result.md`
- `tasks/86-about-sticker-app-wiring.md`
- `tasks/86-about-sticker-app-wiring.result.md`
- `tasks/87-global-language-settings-store.result.md`
- `tasks/88-language-startup-app-wiring-attempt2.md`
- `tasks/88-language-startup-app-wiring.result.md`
- `tasks/89-language-picker-ui.md`
- `tasks/89-language-picker-ui.result.md`
- `tasks/90-language-picker-app-wiring-recovery.md`
- `tasks/90-language-picker-app-wiring.md`
- `tasks/91-language-pack-preferred-font-attempt2.md`
- `tasks/91-language-pack-preferred-font.md`
- `tasks/91-language-pack-preferred-font.result.md`
- `tasks/92-translation-contract-and-guide.md`
- `tasks/92-translation-contract-and-guide.result.md`

In `tasks/78-overlay-menu-app-wiring.worker-result.md`, remove only the trailing ASCII space on the line
ending with `wrap target row,`. Preserve the text and every other byte.

Before and after each edit, prove the allowed byte delta. A blank-EOF file must lose exactly one LF byte;
the Task 78 report must lose exactly one trailing space byte. Reject any other delta. All 43 final files
must be UTF-8 without BOM, LF-only, and end in exactly one LF with no trailing blank line.

## Phase 2 - preserve verbatim upstream licence bytes

Record the starting SHA-256 and byte count of:

- `licenses/cores/genesis_plus_gx/LICENSE.txt`
- `licenses/cores/mgba/LICENSE`

Add narrow path-specific `.gitattributes` entries that disable Git whitespace-error reporting for exactly
those two files. Include a short comment that these are byte-preserved upstream licence texts. Do not
weaken whitespace checks for a directory, extension, task file, or repository-wide pattern. Confirm with
`git check-attr whitespace -- <path>` that the attribute is explicitly unset for both files and remains
unspecified for `tasks/78-overlay-menu-app-wiring.worker-result.md`.

Recompute the licence hashes and byte counts; both must remain identical. Existing `text=auto eol=lf`
behavior may remain inherited. Do not normalize or otherwise rewrite the licence working copies.

## Phase 3 - validate the complete candidate contents

The real Git index must remain empty throughout. Validate:

1. `git diff --check` for tracked changes has no error;
2. every untracked text candidate is strict UTF-8 where applicable, has no trailing space/tab, and has
   exactly one final LF, except the two exact upstream licence paths exempted above;
3. no other `new blank line at EOF` or trailing-whitespace candidate remains;
4. binary candidates are identified by signature/type and not decoded as text;
5. staged entries, deletions, and renames remain zero;
6. the Task 109 prohibited-file, secret, ignored-root, and link checks remain clean for paths introduced
   after Task 110 only.

Do not use a temporary Git index and do not run `git add`, `git write-tree`, or another object-creating
command. Implement the untracked-file whitespace check directly over file bytes. Report safe counts and
paths only; do not dump file contents.

## Phase 4 - replacement frozen manifest

Write `tasks/111-staged-whitespace-gate-closure.manifest.txt` as UTF-8 without BOM, LF-only, one
repository-relative Git path per line, bytewise sorted, with one final LF. It must contain:

1. every current file-level tracked modification and untracked candidate;
2. both Task 111 output paths; and
3. reserved future verdict `tasks/111-staged-whitespace-gate-closure.result.md`.

It supersedes the Task 110 manifest. It must list itself, this Task 111 specification, its worker result,
and its reserved verdict. Require no ignored path, directory placeholder, duplicate, absolute path,
backslash, deleted path, or path outside the repository.

After writing both outputs, compare live file-level status with the replacement manifest. The reserved
future verdict must be the only manifest-only path, and there must be no status-only path. Record the line
count, byte count, SHA-256, encoding/newline facts, and both status/manifest fingerprints.

## Worker report

Write `tasks/111-staged-whitespace-gate-closure.worker-result.md` with:

- `SUCCESS` only if all exact byte deltas, licence preservation, attributes, whitespace checks, hygiene,
  and replacement-manifest checks pass;
- cumulative attempt count;
- the original 71-finding breakdown and why the pre-stage check missed untracked files;
- the 43 exact cleanup deltas and final encoding/newline state;
- before/after hashes and byte counts for both untouched licences;
- exact `.gitattributes` scope and `git check-attr` evidence;
- complete-candidate text/binary scan counts and remaining exemptions;
- replacement manifest facts and exact live-set comparison;
- confirmation that the real index stayed empty and no commit, push, tag, network, hardware/card, or
  shared-configuration action occurred;
- the next boundary: Codex review, then renewed user approval of the replacement complete manifest before
  staging and committing.

Stop after the report. Do not continue into staging, commit, rebuild, remote setup, or hosted acceptance.
