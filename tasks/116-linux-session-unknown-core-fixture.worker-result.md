# Task 116 - Linux session unknown-core fixture portability (worker result)

## Verdict

**SUCCESS** — cumulative attempt count: **1/2**.

The last hosted Linux failure is fixed entirely inside the `session` integration fixture. The unknown-core
copy now takes its source from the core registry and its destination from the platform's DLL extension
instead of two hard-coded Windows filenames. No product code, workflow, manifest, lockfile, unrelated test,
or handoff file changed.

## Hosted failure being corrected

- Workflow run `37257904339` (`https://github.com/goosetopPM/slot2/actions/runs/37257904339`) on pushed
  commit `5552aab52d0f07870285226b9c808641395213c9` — the current local `main` HEAD. (The account was
  renamed from `gyuhangcho` to `goosetopPM` between Task 115 and this run; the event is the same one the
  contract names.)
- The audio prerequisite, `clippy (host)`, `clippy (device)`, and both Task 115 `core_picker_app` cases
  passed. The workspace test step failed only in `crates/slot2/tests/session.rs`: **42 passed, 1 failed**.
- Failing test: `a_core_that_cannot_run_the_console_falls_back_to_the_platforms_own`.
- It first proved the unsupported Gambatte setting falls back to mGBA successfully, then reached its second
  half: copying the real mGBA library under an unknown external-core name.
- That copy used source `cores.join("mgba_libretro.dll")` and destination
  `alt.join("mystery_libretro.dll")`. Linux's `vendor/` holds `mgba_libretro.so`, so
  `fs::copy(...).unwrap()` panicked with **OS error 2 (`No such file or directory`)** before the
  unknown-core behavior could be exercised. The `device` job was skipped (`needs: check`) and did not fail
  independently.
- **Shared filename diagnosis**: both names in that one fixture were Windows literals, while product
  `session::core_file_name` / `Core::file_name` resolve `mgba` and `mystery` to `dll`/`dylib`/`so` per host.
  One fixture-portability cause, one failure — the same class Task 115 closed in `core_picker_app.rs`.

## Platform-native construction

Changed inside `a_core_that_cannot_run_the_console_falls_back_to_the_platforms_own` only
(`crates/slot2/tests/session.rs`, +5/−2 lines):

```rust
    let unknown = format!("mystery_libretro.{}", std::env::consts::DLL_EXTENSION);
    fs::copy(
        cores.join(slot2_retro::CoreId::Mgba.file_name()),
        alt.join(&unknown),
    )
    .unwrap();
```

- **Source** uses the existing registry contract, `CoreId::Mgba.file_name()` (`mgba_libretro.dll` /
  `.dylib` / `.so`), rather than a Windows literal.
- **Destination** is the `mystery_libretro` base name plus `std::env::consts::DLL_EXTENSION` (`dll` on
  Windows, `dylib` on macOS, `so` on Linux and the other Unix targets) — the same three arms as product
  resolution, so the unknown name is exactly what the product looks for on each host.
- No `lib` prefix: SLOT2 core filenames have none.
- The filename is built inline; no helper was added, and no unrelated fixture was touched. In particular
  `core_dir()` was left as it was: its existing three-arm `cfg!` already resolves all three OSes correctly
  and is outside this fix's scope.

## Proof both semantic halves remain

The test body (57 lines) still performs both behaviors, with no assertion weakened, skipped, ignored, split,
or serialized away:

- **Half 1 — fallback**: settings name the unsupported `gambatte_libretro` for a GBA cart;
  `s.core_id()` must equal `Some(slot2_retro::CoreId::Mgba)` with the message
  `"a core that cannot run this console was opened anyway"`, followed by a frame and
  `assert!(s.last_frame().is_some())`.
- **Half 2 — unknown core**: settings name `mystery`, the real mGBA bytes are copied to the native unknown
  filename, `Session::start` is given that directory, and `s.core_id()` must equal `None` with the message
  `"an unknown library was given an identity"`, followed by
  `assert!(s.last_frame().is_some(), "the unknown core did not run")` and `s.stop(&card)`.

No fallback behavior, external-core support, state namespace, or runtime loading path was changed.

## Validation results

| # | Check | Command | Result |
| --- | --- | --- | --- |
| 1 | copy source uses `CoreId::Mgba.file_name()`, destination uses the native DLL extension, no hard-coded `.dll` path | extraction of the test body + `grep` | source call present once (line 36 of the body), destination `format!("mystery_libretro.{}", std::env::consts::DLL_EXTENSION)` present once, **0** `.dll` literals in the body |
| 2 | both semantic halves still asserted | same extraction | fallback assertion (`Some(CoreId::Mgba)` + message) and unknown-core assertion (`None` + `"an unknown library was given an identity"` + frame + stop) both present |
| 3 | `cargo fmt --all -- --check` | same | exit 0 in 14 s, no output |
| 4 | `cargo test -p slot2 --test session --features slot2-input/host` | same | exit 0 in 71 s → **43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out** |
| 5 | `cargo clippy -p slot2 --test session --features slot2-input/host -- -D warnings` | same | exit 0 in 15 s, `Finished dev profile`, 0 warnings |
| 6 | `git diff --check` | same | exit 0 |
| 7 | `git status --short` | same | the expected four paths (below) |

Validation 4 detail: `a_core_that_cannot_run_the_console_falls_back_to_the_platforms_own ... ok` appears in
the run, all 43 cases are `... ok`, and the log contains no `no core in ... — skipping` line, so the real
`vendor/mgba_libretro` library was present and the fixture performed a real copy and a real load rather than
returning early. `std::env::consts::DLL_EXTENSION` was separately probed on this host (`dll`), matching what
`CoreId::Mgba.file_name()` returns here, which is why the host-side copy resolves the same file the product
opens.

Validation 7 output:

```text
 M crates/slot2/tests/session.rs
 M docs/HANDOFF-CODEX.md
?? tasks/116-linux-session-unknown-core-fixture.md
```

- ` M crates/slot2/tests/session.rs` is the only allowed implementation file: +5/−2 lines, all inside the one
  test's unknown-core fixture.
- `?? tasks/116-linux-session-unknown-core-fixture.md` is this control file.
- ` M docs/HANDOFF-CODEX.md` is the **pre-existing handoff entry** (Codex's own current-state notes),
  accounted for and **not edited** by this worker.
- This report, `tasks/116-linux-session-unknown-core-fixture.worker-result.md`, is the fourth expected path
  and is written after these checks.

The full workspace suite, device build, core build, distribution build, and hosted workflow were not run:
Linux and device acceptance can only come from the next hosted run.

## Modified files

- `crates/slot2/tests/session.rs` — registry-native source and platform-native unknown destination in the one
  unknown-core copy fixture (test-only; +5/−2)
- `tasks/116-linux-session-unknown-core-fixture.worker-result.md` — this report only

## Confirmation of no prohibited action

No product code, workflow, dependency, manifest, `Cargo.lock`, unrelated test, documentation, or handoff file
changed. No staging, commit, push, publication, repository-visibility change, tag creation or movement,
workflow rerun/cancel, network access, hardware/card access, or shared-configuration change occurred, and no
credential was requested or printed. Local HEAD remains `5552aab52d0f07870285226b9c808641395213c9` on `main`
with an empty index. The worker-owned `%TEMP%` scratch used for the body extraction, probe, and command logs
was deleted after this report was written.

## Remaining boundary

1. Codex review of this report and of the single test-file diff.
2. User-authorized **commit and push** of exactly that test change.
3. The next hosted run must pass **both** `check` and `device`.
4. After that, rerun the same revision once to accept the core-cache **hit** path and inspect the uploaded
   device artifact — and only then consider making the repository public or creating a release tag.

Stopping here — no commit, push, hosted rerun, visibility change, tag, draft release, or publication was
performed.
