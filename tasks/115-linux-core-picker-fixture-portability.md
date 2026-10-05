# Task 115 - Linux core-picker fixture portability

## Goal

Fix the two `core_picker_app` test failures exposed by the second hosted GitHub Actions run without changing
product behavior. Commit `2941b077e93e9fee03240a489573f5fd73ec4236` passed the Task 114 Linux audio
prerequisite, host clippy, and device clippy. Run `37256229710` then failed in the workspace `test` step
because the integration fixture hard-codes an external core as `mystery_libretro.dll` while Linux product
code resolves the same setting to `mystery_libretro.so`.

Read only:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state entries only
- this task in full
- `C:\SLOT2\AGENTS.md`, required rules only
- `C:\SLOT2\crates\slot2\tests\core_picker_app.rs`
- `C:\SLOT2\crates\slot2\src\session.rs`, only `core_file_name` and `resolve_core`
- `C:\SLOT2\tasks\114-hosted-ci-linux-prerequisites.result.md`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, stage,
commit, push, publish, create or move a tag, rerun or cancel a hosted workflow, use the network, access
hardware or a card, or change shared configuration. Maximum two cumulative attempts. Wait up to 5 minutes
for the first response and 45 minutes total.

## Confirmed hosted evidence

- Run: `https://github.com/gyuhangcho/slot2/actions/runs/37256229710`
- `host audio prerequisites`, `clippy (host)`, and `clippy (device)` passed. The Task 114 ALSA correction is
  accepted at its failure boundary.
- The workspace test step failed only in `crates/slot2/tests/core_picker_app.rs`: 12 passed, 2 failed.
- `an_empty_picker_says_so_and_changes_nothing` failed at line 230 because the cart stayed on `Screen::List`.
  The log says no core existed at the Linux path `.../cores/mgba_libretro.so` after the named external core
  was not found.
- `a_recovery_that_cannot_open_the_old_core_takes_the_cart_out` failed at line 1112: expected
  `Some(SinkRequest::Close)`, got `Some(SinkRequest::Open)`. Recovery found no Linux external core and
  successfully fell back to mGBA instead of trying the intended invalid external library.
- Both fixtures write or copy `mystery_libretro.dll`. Product `session::core_file_name("mystery")` chooses
  `dll` on Windows, `dylib` on macOS, and `so` elsewhere. The two failures therefore have one test-fixture
  portability cause.
- The downstream `device` job was skipped because `check` failed; it did not fail independently.

## Allowed changes

Edit only:

- `crates/slot2/tests/core_picker_app.rs`
- `tasks/115-linux-core-picker-fixture-portability.worker-result.md`

Do not edit product code, workflows, dependencies, manifests, `Cargo.lock`, other tests, documentation, or
Task 114 records. If the two failures cannot be fixed entirely in the fixture, stop and report the contract
conflict instead of changing production behavior.

## Required implementation

Define one small test helper that returns the platform-native external core filename for the base name
`mystery`, with the same shape as product resolution:

- Windows: `mystery_libretro.dll`
- macOS: `mystery_libretro.dylib`
- Linux and other Unix targets used here: `mystery_libretro.so`

Prefer the standard-library platform DLL extension constant so the test does not duplicate a Windows-only
literal. Do not add a `lib` prefix: SLOT2 core filenames have none.

Use this one helper in both exact fixture sites:

1. `external_fixture` must copy the real mGBA library to the platform-native external filename after
   removing both official GBA core filenames. This preserves the test's real external-session behavior.
2. `a_recovery_that_cannot_open_the_old_core_takes_the_cart_out` must create the invalid bytes at the same
   platform-native external filename before choosing the other core. This preserves the intended recovery
   failure instead of allowing Linux to fall back to mGBA.

Do not weaken, skip, conditionally ignore, or serialize away either assertion. Do not change sink semantics,
core fallback, picker behavior, or the hosted workflow.

## Validation

Run only the focused checks justified by this test-only change:

1. prove `core_picker_app.rs` contains no hard-coded `mystery_libretro.dll` path and both fixture sites use
   the one helper;
2. prove the helper yields the current host's expected suffix and structurally covers Windows, macOS, and
   Linux/other Unix consistently with product `core_file_name`;
3. `cargo fmt --all -- --check`;
4. `cargo test -p slot2 --test core_picker_app --features slot2-input/host` — all 14 tests must pass and none
   may be ignored or filtered out;
5. `cargo clippy -p slot2 --test core_picker_app --features slot2-input/host -- -D warnings`;
6. `git diff --check`;
7. `git status --short` must contain only the allowed test file, this task, its worker report, and the
   pre-existing handoff entry. Do not edit the handoff.

Do not run the full workspace, device build, core build, distribution build, or hosted workflow. Linux
acceptance occurs only after later user-authorized commit and push.

## Worker report

Write `tasks/115-linux-core-picker-fixture-portability.worker-result.md` with:

- `SUCCESS` only if every focused validation passes;
- cumulative attempt count;
- the hosted run, two failing tests, and their shared filename diagnosis;
- the helper contract and the two exact call sites changed;
- focused test count and clippy/fmt/diff results;
- modified files;
- confirmation that no product code, workflow, manifest, lockfile, unrelated test, handoff, staging, commit,
  push, tag, hosted rerun, network, hardware/card, or shared configuration changed; and
- the remaining boundary: Codex review, then user-authorized commit and push. The next hosted run must pass
  both `check` and `device`; a later rerun must accept the core-cache hit path and device artifact.

Stop after the report. Do not continue into commit, push, hosted rerun, tag, draft release, or publication.
