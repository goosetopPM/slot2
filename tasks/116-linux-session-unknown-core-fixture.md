# Task 116 - Linux session unknown-core fixture portability

## Goal

Fix the one remaining hosted Linux test failure after Task 115 without changing product behavior. Commit
`5552aab52d0f07870285226b9c808641395213c9` passed the Task 114 ALSA prerequisite, host/device clippy,
and the two `core_picker_app` cases fixed by Task 115. Run `37257904339` then failed in the workspace test
step because one `session` fixture still copies `mgba_libretro.dll` to `mystery_libretro.dll` on every OS.
Linux has `mgba_libretro.so`, so the source copy failed with `No such file or directory`.

Read only:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state entries only
- this task in full
- `C:\SLOT2\AGENTS.md`, required rules only
- `C:\SLOT2\crates\slot2\tests\session.rs`, especially `core_dir` and lines 811-865
- `C:\SLOT2\crates\slot2\src\session.rs`, only `core_file_name` and `resolve_core`
- `C:\SLOT2\crates\slot2-retro\src`, only the existing `CoreId::file_name` implementation if needed
- `C:\SLOT2\tasks\115-linux-core-picker-fixture-portability.result.md`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, stage,
commit, push, publish, create or move a tag, rerun or cancel a hosted workflow, use the network, access
hardware or a card, or change shared configuration. Maximum two cumulative attempts. Wait up to 5 minutes
for the first response and 45 minutes total.

## Confirmed hosted evidence

- Run: `https://github.com/goosetopPM/slot2/actions/runs/37257904339`
- The audio prerequisite, host clippy, device clippy, and Task 115 `core_picker_app` failures all passed.
- The workspace test step failed only in `crates/slot2/tests/session.rs`: 42 passed, 1 failed.
- Failing test: `a_core_that_cannot_run_the_console_falls_back_to_the_platforms_own`.
- The test successfully proved the unsupported Gambatte setting falls back to mGBA, then entered its second
  assertion: copying the real mGBA binary under an unknown external-core name.
- The copy at the old lines 843-847 used source `cores.join("mgba_libretro.dll")` and destination
  `alt.join("mystery_libretro.dll")`. On Linux the source is `mgba_libretro.so`, so `fs::copy(...).unwrap()`
  failed with OS error 2 before the unknown-core behavior was tested.
- The downstream `device` job was skipped because `check` failed; it did not fail independently.

## Allowed changes

Edit only:

- `crates/slot2/tests/session.rs`
- `tasks/116-linux-session-unknown-core-fixture.worker-result.md`

Do not edit product code, workflows, dependencies, manifests, `Cargo.lock`, other tests, documentation,
Task 114/115 records, or repository metadata. If the failure cannot be fixed entirely in this fixture, stop
and report the conflict instead of changing production behavior.

## Required implementation

Change only the unknown-core copy fixture inside
`a_core_that_cannot_run_the_console_falls_back_to_the_platforms_own`:

1. Resolve the real source core with the existing registry contract (`CoreId::Mgba.file_name()`), rather
   than a hard-coded Windows filename.
2. Build the unknown destination as `mystery_libretro` plus the standard library's platform DLL extension:
   `dll` on Windows, `dylib` on macOS, and `so` on Linux/other Unix targets.
3. Do not add a `lib` prefix; SLOT2 core files have none.

A tiny local helper is acceptable if it makes the filename contract clearer, but do not refactor unrelated
fixtures. Preserve the test's two distinct behaviors: first reject Gambatte for a GBA cart and fall back to
mGBA; then load the same real mGBA bytes under an unknown filename and report `core_id() == None`.

Do not weaken, skip, ignore, split, or serialize away assertions. Do not change fallback behavior, external
core support, state namespaces, or runtime loading.

## Validation

Run only the focused checks justified by this test-only change:

1. prove the unknown-core copy source uses `CoreId::Mgba.file_name()` and its destination uses the native
   DLL extension rather than either hard-coded `.dll` path;
2. prove the edited test still asserts both mGBA fallback and unknown-core `None` identity;
3. `cargo fmt --all -- --check`;
4. `cargo test -p slot2 --test session --features slot2-input/host` — all 43 tests must pass with 0 ignored
   and 0 filtered out, including the previously failing case;
5. `cargo clippy -p slot2 --test session --features slot2-input/host -- -D warnings`;
6. `git diff --check`;
7. `git status --short` must contain only the allowed session test, this task, its worker report, and the
   pre-existing handoff entry. Do not edit the handoff.

Do not run the full workspace, device build, core build, distribution build, or hosted workflow. Linux and
device acceptance occurs only after later user-authorized commit and push.

## Worker report

Write `tasks/116-linux-session-unknown-core-fixture.worker-result.md` with:

- `SUCCESS` only if every focused validation passes;
- cumulative attempt count;
- the hosted run, failing test, exact OS error, and shared filename diagnosis;
- the platform-native source and destination construction used;
- proof both semantic halves of the test remain;
- focused test count and clippy/fmt/diff results;
- modified files;
- confirmation that no product code, workflow, manifest, lockfile, unrelated test, handoff, staging, commit,
  push, tag, hosted rerun, network, hardware/card, or shared configuration changed; and
- the remaining boundary: Codex review, then user-authorized commit and push. The next hosted run must pass
  both `check` and `device`; after that, rerun the same revision to accept the cache-hit path and inspect the
  device artifact before the repository is made public or a release tag is created.

Stop after the report. Do not continue into commit, push, hosted rerun, visibility change, tag, draft
release, or publication.
