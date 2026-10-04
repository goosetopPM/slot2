# Task 105 - Codex final review

## Verdict

**Passed (cumulative attempt 2/2).** Attempt 2 closes the partial-publication defect without changing the
reusable workflows, archive contents, card validators, or documentation completed in attempt 1.

## Confirmed correction

- `OutputDir` must be absent before staging; an existing directory, file, or link is refused unchanged.
- The zip and sidecar are fully written and validated in one same-parent staging directory.
- The staging directory is checked to contain exactly the expected pair and is published with one
  `System.IO.Directory.Move`.
- Production code contains no final file moves, copy fallback, overwrite, backup replacement, or retry.
- Cleanup is guarded by promotion ownership and can name only an unpromoted staging path. A promoted output
  is retained, including when its post-promotion validation reports a failure.

## Verification reviewed

- Two fresh outputs each contained exactly `slot2-v0.1.0.zip` and its `.sha256`; their 241 entries and
  extracted file hashes matched the source card tree and the attempt-1 output.
- Existing directory, file, and junction destinations remained fingerprint-identical after refusal.
- A late destination containing a marker was preserved; the real publication helper rejected it, removed
  only its owned staging directory, and placed neither release file at the destination.
- Structural checks found exactly one directory move, no file move/copy/rename publication, one helper call,
  and only staging-targeted cleanup.
- Identity, VERSION, missing-core, extra-root, Rust-notice mutation, unsafe archive-entry, sidecar, workflow
  structure, shell syntax, PowerShell syntax, and `git diff --check` validations passed.

## Remaining external acceptance

No hosted workflow, tag, draft release, or publication was executed. Task 104's first cache miss and later
cache hit, uploaded artifact inspection, the tag-triggered reusable build, PowerShell 7 packaging, draft
asset/checksum inspection, and manual publication remain user-controlled M7 acceptance work. README,
issue templates, and migration documentation also remain open.
