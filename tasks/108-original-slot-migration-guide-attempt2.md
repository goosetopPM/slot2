# Task 108 - Original slot migration guide correction (cumulative attempt 2/2)

## Goal

Correct three unsupported runtime/recovery statements identified in
`tasks/108-original-slot-migration-guide.result.md`. Preserve the completed migration procedure and make
no unrelated editorial changes. This is the final allowed Task 108 attempt.

Read:

- `C:\SLOT2\tasks\108-original-slot-migration-guide.result.md`
- `C:\SLOT2\docs\MIGRATION.md`, only sections 5-7
- `C:\SLOT2\docs\MIGRATION.ko.md`, only sections 5-7
- `C:\SLOT2\crates\slot2\src\app.rs`, only `rescan` adoption and scoped state listing calls
- `C:\SLOT2\crates\slot2\src\diag.rs`, only report output/write behavior
- `C:\SLOT2\crates\slot2-store\src\card.rs`, only `roll_back` and `adopt_legacy_states`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
push, publish, create a tag, open an issue or pull request, use the network, access hardware, or change
shared configuration. This is cumulative attempt 2/2; do not retry beyond it.

## Exact allowed scope

- Edit `docs/MIGRATION.md`.
- Edit `docs/MIGRATION.ko.md` equivalently.
- Update `tasks/108-original-slot-migration-guide.worker-result.md` as a cumulative 2/2 report.

Do not change either README or any other file. Require these current baselines before editing:

- `docs/MIGRATION.md`: 16094 bytes, SHA-256
  `968c1f9aa5a7c9c13470dd8c6bc1d5c5c077654f701db6f1045559d20740aeea`
- `docs/MIGRATION.ko.md`: 19191 bytes, SHA-256
  `58c4e78b7a67331e99fe342b25021bb0b8adf63df2fe66df7ae9bc6873ed2fc8`
- `README.md`: SHA-256
  `615e3ecd5f9ae011e14ff7a3c4b4374d9335bfbbb58bd2f84144691c3eb7a498`
- `README.ko.md`: SHA-256
  `631f69600ea8da6ac8de6b44f51f6428c1633a3d4c432a775f16d60f45ccf648`

Stop and report if any baseline differs.

## Required corrections

Make the smallest equivalent English/Korean changes needed for all three items.

1. **Correct the diagnostic destination.** State that legacy adoption failures are written to stderr and
   are available in BaseOS's volatile `/tmp/frontend.log`. Describe `System/slot2-diag.txt` only as the
   separate boot survey it actually is. Never say or imply that runtime state-adoption errors are appended
   to that card file.
2. **Correct destination-collision handling.** Explain that SLOT2 continues to list/use the already
   namespaced state for the resolved core, while the refused flat state stays outside SLOT2's scoped state
   list. Do not tell the user to compare or load both states in the SLOT2 UI. Tell the user to stop, retain
   the card and backup, capture only the smallest redacted relevant excerpt from `/tmp/frontend.log` if
   available, and report the conflict. The pre-migration backup/rollback path remains the safe way to
   recover the original flat layout. Do not prescribe manual merging or moving.
3. **Correct rollback guarantees.** State that after a rename fails partway, SLOT2 attempts a best-effort
   reverse rename and reports the original error. Because a reverse rename can also fail, do not guarantee
   that every flat file is back in its original location. Tell the user not to continue playing or edit the
   two locations manually; shut down, inspect/preserve both locations, and restore from the verified
   pre-migration backup as needed. Remove the claim that pulling a card mid-write is the only way a
   recoverable migration can become damaged. Keep the clear shutdown-before-removal instruction without
   an exclusivity claim.

Keep the collision-preflight fact: when a destination exists before the move begins, the whole game is
refused before the first rename and flat originals stay untouched. Distinguish that safe preflight refusal
from an I/O rename failure after moves have begun.

Do not weaken the full off-card backup, complete `System/` replacement, no-merge rule, core-specific state
warning, first-check-before-play rule, or post-start `States/` rollback.

## Verification and cumulative report

Do not run Cargo, clippy, distribution builds, or hardware checks. Run focused offline checks:

1. verify all four baselines before editing and prove both README hashes remain unchanged afterward;
2. source-check stderr/diag behavior, scoped listing behavior, collision preflight, and best-effort rollback;
3. assert both guides carry equivalent corrected claims and no stale unsupported sentence remains;
4. verify UTF-8 without BOM, LF-only, final newline, no trailing whitespace, balanced Markdown fences, and
   resolving relative links;
5. inspect the focused diff and require only the necessary paragraphs in guide sections 5-7 to change;
6. run `git diff --check`.

Update `tasks/108-original-slot-migration-guide.worker-result.md` as cumulative attempt 2/2. Retain the
attempt-1 evidence, add exact changed paragraphs/hashes and verification results, explicitly acknowledge
the three corrected defects, and retain the existing remaining limits. Stop after the report.
