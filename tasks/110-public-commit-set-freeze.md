# Task 110 - Public commit-set freeze

## Goal

Freeze one exact, reviewable path set for the first clean public development snapshot after Task 109.
Validate the recommended policy of committing the complete current development tranche, including the
repository's existing task/operations history, without staging or committing anything. Produce a compact
human report and a deterministic path manifest that Codex can review before asking the user for explicit
commit authorization.

Read only:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state entry only
- this task in full
- `C:\SLOT2\AGENTS.md`, required rules only
- `C:\SLOT2\tasks\109-offline-release-candidate-audit.result.md`
- `C:\SLOT2\tasks\109-offline-release-candidate-audit.worker-result.md`, sections 5-8 only
- `C:\SLOT2\.gitignore`
- tracked-path history only where needed to prove whether `tasks/`, `.claude/resume.md`, `AGENTS.md`, and
  `docs/HANDOFF-CODEX.md` already have repository precedent

Use status, path counts, object metadata, and focused excerpts. Do not dump file contents, the whole diff,
or old logs. Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not
delegate, edit product/source/test/public-document/workflow files, stage, commit, reset, restore, checkout,
stash, clean, push, publish, create or move a tag, configure a remote, use the network, access hardware or
a card, or change shared configuration. Maximum two attempts. Wait up to 5 minutes for the first response
and 45 minutes total.

## Recommended policy to validate

The proposed public snapshot includes every current tracked modification and every current untracked
candidate that passed Task 109 hygiene. It also includes project task specifications/results and the
tracked operations/handoff documents. This is recommended because those categories already exist in HEAD,
they carry implementation provenance, Task 109 found no prohibited or secret content, and excluding only
their new revisions would leave an ambiguous dirty remainder after the product commit.

Do not silently accept the recommendation. Fail and report if repository history does not support the
precedent, if any candidate appeared after the Task 109 review that was not produced by the documented
Codex final-review/Task-110 preparation, or if any path is unsuitable for a public repository. Never repair
or remove a candidate in this task.

## Allowed changes

Create or update only:

- `tasks/110-public-commit-set-freeze.manifest.txt`
- `tasks/110-public-commit-set-freeze.worker-result.md`

Normal read-only Git commands may create no repository-visible files. Temporary files must stay outside the
workspace and be removed before the report.

## Phase 1 - reconcile the candidate set

Record branch, full HEAD, staged entry count, tracked-modified count, untracked file count, deletion count,
rename count, and a SHA-256 fingerprint of the sorted file-level candidate paths. Use a NUL-safe Git status
source and state the conversion method; fail if a path contains LF or cannot be represented unambiguously
as one manifest line.

Reconcile the current path set with the Task 109 cumulative report. New paths are allowed only when they
are exactly explained by:

- Task 109 Codex final records and handoff updates;
- this Task 110 specification;
- the two allowed Task 110 output paths;
- the reserved future Codex verdict path
  `tasks/110-public-commit-set-freeze.result.md`.

The future verdict may not exist yet. No other unexplained product, source, asset, build, workflow,
documentation, task, operation, log, archive, secret, ROM, link, or ignored-output candidate is allowed.
Require zero staged entries, deletions, and renames.

Repeat the safe candidate hygiene checks from Task 109 only for paths added after its final recorded set.
Do not repeat Cargo, clippy, core, device, or package tests: no implementation input is allowed to change in
this task and Task 109 already passed those gates.

## Phase 2 - prove repository precedent and policy consequences

Using `git ls-tree` and focused history metadata, report:

- how many task files exist in HEAD and representative tracked task paths;
- whether `.claude/resume.md`, `AGENTS.md`, and `docs/HANDOFF-CODEX.md` exist in HEAD;
- category counts for product source/tests, assets/provenance, build/workflows/licenses, public docs,
  operations/handoff, and task specifications/results;
- the number of files that would remain dirty if operations/handoff and task history were excluded.

Validate that the complete-tranche policy leaves no known candidate outside the proposed snapshot. State
the public tradeoff plainly: task and operations history becomes public repository content, but the
snapshot retains its existing provenance model and can reach a clean tree in one commit.

Inspect the 13 CRLF working-copy paths noted by Task 109. Prove using Git attributes or blob inspection that
the proposed committed blobs normalize to the repository's LF policy. Do not rewrite the working copies.
Retain the four already reported trailing-whitespace exceptions only if they are the same three verbatim
upstream licence texts plus the historical Task 78 report; list paths but do not edit them. Any new
exception is failure.

## Phase 3 - write the frozen manifest

Write `tasks/110-public-commit-set-freeze.manifest.txt` as UTF-8 without BOM, LF-only, one repository-relative
Git path per line, bytewise sorted, with one final LF. It must contain the union of:

1. every current file-level tracked modification and untracked candidate;
2. both Task 110 output paths; and
3. reserved `tasks/110-public-commit-set-freeze.result.md`.

The manifest must list its own path. Self-listing is intentional and does not require a content hash. It
must not contain ignored paths, directories as placeholders, duplicate lines, absolute paths, backslashes,
or paths outside the repository. Do not include deleted paths because deletion is forbidden in this set.

After writing the manifest and worker report, recompute status. Compare its file-level paths with the
manifest while treating the reserved future Codex verdict as the only allowed not-yet-existing path. The
set difference must otherwise be empty. Run `git diff --check` and report existing normalization warnings
separately from errors.

Do not run `git add`, including with a temporary index. Do not create a tree or commit object. The next
stage remains a separate Codex review followed by explicit user authorization.

## Worker report

Write `tasks/110-public-commit-set-freeze.worker-result.md` with:

- `SUCCESS` only if every reconciliation, precedent, policy, encoding, and exact-set check passed;
- cumulative attempt count;
- repository identity and all candidate counts/fingerprints;
- concise explanations for every path added since Task 109;
- category and exclusion-consequence counts;
- tracked-history precedent, CRLF normalization, trailing-whitespace, and focused hygiene results;
- manifest path, line count, byte count, SHA-256, encoding/newline checks, and exact final set comparison;
- `git diff --check` result;
- confirmation that only the two allowed outputs changed and nothing was staged, committed, removed,
  pushed, published, tagged, downloaded, or sent to hardware;
- the exact next decision: after Codex review, the user may approve or reject committing every manifest
  path in one Korean-message development snapshot. No commit command or proposed tag is needed here.

Stop after the report. Do not continue into staging, commit, rebuild, remote setup, or hosted acceptance.
