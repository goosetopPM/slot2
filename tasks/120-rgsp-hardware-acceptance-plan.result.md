# Task 120 - Codex final review

## Verdict

**Passed (cumulative attempt 2/2).** The Korean RG SP hardware acceptance checklist is ready for later user-run
execution. No hardware result has been claimed, and no further worker call is allowed or needed for this task.

## Correction review

- Artifact identity is now field-correct: `VERSION.txt` supplies version plus short commit, About supplies version
  plus target, the short commit must prefix the full CI SHA, and the target must be `rgsp`.
- The committed gate now covers all accepted work through Task 120 and its final Codex review, followed by a clean
  exact-SHA hosted CI device artifact. It does not authorize a tag or release.
- `SAFE-02` is mandatory only when the optional legacy-card migration scenario is selected. A reasoned migration
  `SKIP` no longer prevents fresh-card acceptance.
- The normal lid round trip reopens after 30-60 seconds, below the three-minute threshold. The optional three-minute
  spare-card check separately expects the still-unverified M4 power-off target.
- Mute uses the confirmed in-game Device -> Volume -> A action; physical volume keys remain a separate test.
- The introduction distinguishes current documented behavior from unchecked M4/M6 acceptance targets and optional
  observations.

## Independent document verification

- 155 executable rows; 155 unique IDs; all rows have five data columns and all results remain `NOT RUN`.
- UTF-8 without BOM, zero CR bytes, final LF present, zero tabs/trailing whitespace.
- Scoped and repository `git diff --check` passed.
- No Task 120 lock file remains. Older unrelated lock directories were not touched.
- The worker changed only the two allowed Task 120 output files and performed no hardware, network, build, test,
  commit, push, tag, or release action.

## Acceptance boundary and next step

This passes the checklist-preparation task only. Physical acceptance is still `NOT RUN`. Before the user starts it,
the reviewed Task 117-120 tranche and this final record must be committed and pushed, the tree must be clean, and
hosted CI must produce a successful device artifact for that exact SHA. Commit and push still require an explicit
user instruction. Tagging and release remain prohibited until the user completes UI/hardware acceptance and later
authorizes them separately.
