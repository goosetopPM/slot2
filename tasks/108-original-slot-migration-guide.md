# Task 108 - Original slot card migration guide

## Goal

Add accurate English and Korean guides for moving an existing original `slot` data card to SLOT2. The
procedure must preserve card-root user data, replace `System/` as a complete tree, explain SLOT2's one-time
legacy state adoption, and provide a rollback that remains valid after SLOT2 scans the card.

This is documentation only. Do not change behavior, code, packaging, workflows, or release artifacts, and
do not claim hardware or published-release acceptance.

Read only the required parts of:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state section
- `C:\SLOT2\docs\DESIGN.md`, BaseOS launch/card selection, card layout, saves/states, legacy state adoption
- `C:\SLOT2\docs\DECISIONS.md`, only decisions referenced by those DESIGN sections
- `C:\SLOT2\docs\MILESTONES.md`, M7
- `C:\SLOT2\README.md` and `C:\SLOT2\README.ko.md`, sections 3, 4, 7, and 8
- `C:\SLOT2\crates\slot2-store\src\card.rs`, only save/state paths and `adopt_legacy_states`
- `C:\SLOT2\crates\slot2\src\app.rs`, only `rescan` state adoption
- `C:\SLOT2\crates\slot2\tests\core_state_routing.rs`, only legacy flat-state tests
- `C:\SLOT2\build\package-release.ps1`, only archive shape/checksum contracts
- `C:\SLOT2\tasks\107-issue-forms-translation-contribution.result.md`

Use `rg` and read only necessary context. Ignore only the orchestrator/delegation clauses in `AGENTS.md`
and work directly. Do not delegate, commit, push, publish, create a tag, open an issue or pull request, use
the network, access hardware, or change shared configuration. Maximum two attempts. Wait up to 5 minutes
for the first response and 45 minutes total.

## Allowed files and baselines

- Add `docs/MIGRATION.md` as the canonical English guide.
- Add `docs/MIGRATION.ko.md` as the equivalent Korean guide.
- Update only the migration warning and project-document links/text in `README.md` and `README.ko.md`.
- Create or update `tasks/108-original-slot-migration-guide.worker-result.md`.

Do not modify any other file. Before editing, require these exact baselines or stop and report concurrent
change:

- `README.md`: 16899 bytes, SHA-256
  `e10a30e64b506d03a07368d9a0cac7c451215c52aea58fea222cb677a4282081`
- `README.ko.md`: 19806 bytes, SHA-256
  `e2e78e5bf14f5bc48c501e649c9c521b396fd8ae2a28ddf1a670cb8c73b75012`

## Shared guide contract

The guides must have the same section order, facts, warnings, decisions, paths, and recovery coverage.
They need not be literal translations. Put a reciprocal language link near the top. Use concise public
prose, relative links, UTF-8 without BOM, and LF line endings.

The audience has an original `slot` data card with compatible card-root user data. Distinguish this from
a fresh empty-card install, a SLOT2-to-SLOT2 update policy, and BaseOS imaging. Do not invent a release URL,
version, digest, original-slot version matrix, filesystem policy, hardware result, automatic whole-card
migration, or cross-core state compatibility.

## Required ordered procedure

### 1. Identify and prepare

- Fully shut down before removing a card; use a PC card reader and safe eject.
- Identify the **data card root**, not a BaseOS system/boot partition. With two cards BaseOS uses TF2 for
  data; otherwise it uses the TF1 data partition. Expected root entries include `System/` and one or more
  of `Games/`, `Labels/`, `Saves/`, `States/`. If uncertain, stop: do not format, repartition, or recursively
  delete anything.
- Obtain a SLOT2 zip and matching `.sha256` through a trusted path. State that no public release has yet
  been published. Reuse or link to the README's OS-specific checksum examples; hard-code no name/digest.

### 2. Make and verify an off-card backup

- Copy the entire data-card root to separate PC storage. At minimum preserve original `System/`, `Games/`,
  `Labels/`, `Saves/`, `States/`, and any `BIOS/` and `Wallpapers/`.
- A renamed same-card folder is not the only backup. Reopen the PC copy, check its top-level directories,
  and sample important save/state files.
- Explain that `States/` is essential even though only `System/` is replaced: scanning may move recognized
  older flat states.

### 3. Inspect away from the card

- Verify the checksum, then extract into an empty PC staging directory, never over the card or old
  `System/`.
- Require `System/frontend` directly under the staging root. Reject extra nesting such as
  `SLOT2/System/frontend`, a missing/empty frontend, or loose files without the complete `System/` tree.
- Old and new `System/` contents must not be merged. Old executables, libraries, settings, and unknown
  files are recovery material, not files to selectively import.

### 4. Replace only `System/`

- Leave card-root `Games/`, `Labels/`, `Saves/`, `States/`, `BIOS/`, and `Wallpapers/` in place.
- After verifying the off-card backup, move/rename the original card-root `System/` as one directory, then
  copy the staged SLOT2 `System/` as one complete directory to the card root.
- If retaining an old tree temporarily on-card, use a unique name and overwrite no prior backup. The
  independent PC copy remains the recovery source.
- Verify exact nonempty `<card root>/System/frontend`; never copy only that executable.
- On any checksum, move, rename, copy, or verification failure, stop before boot and restore original
  `System/` from the PC backup. Never continue with a partial or merged tree.

Explain that `Games/`, `Labels/`, `Saves/`, and `States/` remain at root while SLOT2 program files and new
settings live in replacement `System/`. `Saves/<PLATFORM>/<stem>.sav` remains compatible. Do not claim
arbitrary old `System/` settings are imported.

### 5. First SLOT2 start and exact state behavior

- BaseOS gives `System/frontend` priority over old `System/slot`; the complete replacement is the intended
  handoff.
- SLOT2 scans supported regular ROM files directly inside `Games/<PLATFORM>/`; nested/compressed ROMs are
  not shelf entries.
- Older flat states are `States/<PLATFORM>/<stem>/{resume.state,N.state,N.png}`. During shelf scan,
  recognized flat `resume.state` and positive canonical numbered states, plus existing matching PNGs, are
  moved once by same-card rename into the platform default core namespace. GB/GBC/GBA default to mGBA
  (`mgba_libretro`).
- The default namespace is used even when an alternate core is selected. State bytes are core-specific;
  Gambatte/gpSP cannot be assumed to read mGBA states. Save RAM remains in shared `Saves/` and is outside
  this migration.
- Unrecognized files/subdirectories remain. If any destination state or PNG exists, the whole game's move
  is refused before its first rename and flat sources remain. Files are not merged or chosen by age. A
  rename failure rolls back and is reported.

Tell readers to check representative games, ordinary in-game saves, then old states under the default core
before meaningful new play. If absent under an alternate core, try the default before declaring loss. On a
collision, do not hand-move/merge state files; preserve card/backup and use the issue-report path.

### 6. Rollback and troubleshooting

Rollback must handle a prior scan:

1. shut down before card removal;
2. separately preserve any deliberately wanted new save, without blindly choosing old or new;
3. restore original `System/` from the verified PC backup;
4. if SLOT2 started, restore `States/` from the same pre-migration backup because flat states may have
   moved into namespaces;
5. the cleanest rollback restores the complete snapshot, but discards later changes, so test before
   meaningful play;
6. verify the restored root, safely eject, then boot.

Include concise troubleshooting for wrong card/partition, extra zip nesting, partial `System/`, missing
games, missing in-game save, old state visible only under default core, and legacy destination collision.
Link the bilingual bug issue form by repository-relative path only; do not claim hosted rendering/support.

## README edits

Keep English/Korean edits equivalent and tightly scoped:

- Change the fresh-install warning from "future procedure" to a link to `docs/MIGRATION.md` in English and
  `docs/MIGRATION.ko.md` in Korean. Retain backup and no-upgrade-shortcut warnings.
- Add the matching migration guide to section 8's project-document list.
- Replace the stale sentence saying issue templates and migration are later work; accurately say the issue
  forms and both migration guides now exist, with correct relative links if named.
- Change no other README prose, table, status, command, or link.

## Verification and report

Do not run Cargo, clippy, or device distribution builds. Run focused offline checks and report exact
commands/results:

1. verify both README baseline hashes/sizes before editing;
2. require both guides to be nonempty UTF-8 without BOM, LF-only, free of trailing whitespace/unclosed
   fences, with reciprocal links;
3. compare every required warning, path, state rule, failure stop, and rollback step section by section;
4. verify every relative local link changed/added here resolves;
5. prove README changes are limited to the allowed locations by reversing only them and reproducing the
   exact Task107 hashes;
6. source-check paths and behavioral claims against the focused inputs, especially default-core adoption
   and conflict refusal;
7. reject claims of a published release, completed hardware migration, automatic whole-card migration,
   cross-core state portability, safe old/new `System/` merge, formatting/deletion, or guessed versions;
8. run `git diff --check` and inspect the focused diff for allowed files only.

Write `tasks/108-original-slot-migration-guide.worker-result.md` with `SUCCESS`/`FAILURE`, cumulative attempt
count, changed-file byte counts/hashes, guide parity matrix, exact source locations, commands/results,
scope confirmation, and remaining limits: no published release, hosted-link/render acceptance, or physical
original-card migration acceptance. Stop after the report.
