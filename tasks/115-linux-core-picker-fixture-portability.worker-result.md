# Task 115 - Linux core-picker fixture portability (worker result)

## Verdict

**SUCCESS** — cumulative attempt count: **1/2**.

The two `core_picker_app` failures exposed by the second hosted run are fixed entirely inside the test file.
The fixture now derives the external core's filename from the platform's own DLL extension instead of a
Windows-only literal, so Linux resolves the same path the product does. No product code, workflow, manifest,
lockfile, unrelated test, or handoff file changed.

## Hosted failures being corrected

- Workflow run `37256229710` (`https://github.com/gyuhangcho/slot2/actions/runs/37256229710`) on pushed
  commit `2941b077e93e9fee03240a489573f5fd73ec4236` — the current local `main` HEAD.
- `host audio prerequisites`, `clippy (host)`, and `clippy (device)` **passed**: the Task 114 ALSA
  correction is accepted at its failure boundary.
- The workspace `test` step failed only in `crates/slot2/tests/core_picker_app.rs`: **12 passed, 2 failed**.
  The downstream `device` job was skipped (`needs: check`) and did not fail independently.
- `an_empty_picker_says_so_and_changes_nothing` — failed at line 230: the cart stayed on `Screen::List`.
  The log reported no core at the Linux path `.../cores/mgba_libretro.so` once the named external core was
  not found.
- `a_recovery_that_cannot_open_the_old_core_takes_the_cart_out` — failed at line 1112: expected
  `Some(SinkRequest::Close)`, got `Some(SinkRequest::Open)`. Recovery found no Linux external core, so it
  fell back to mGBA successfully instead of trying the intended invalid external library.
- **Shared filename diagnosis**: both fixtures wrote or copied `mystery_libretro.dll`. Product
  `session::core_file_name("mystery")` — and equally `Core::file_name` — picks `dll` on Windows, `dylib` on
  macOS, and `so` elsewhere, so on Linux the fixtures placed bytes where the product never looks. One
  test-fixture portability cause, two failures.

## Helper contract and call sites

Added one helper (`crates/slot2/tests/core_picker_app.rs:165`):

```rust
/// The external core filename the fixtures write: what product resolution means on this host.
///
/// The setting names the core `mystery`, and `session::core_file_name` resolves that to the
/// `mystery_libretro` base name plus this platform's DLL extension: `dll` on Windows, `dylib`
/// on macOS, `so` on Linux and the other Unix targets. The extension comes from the standard
/// library's own DLL extension constant, so the fixture cannot pin a Windows-only suffix that a
/// Linux runner would then fail to find. No `lib` prefix: SLOT2 core filenames have none.
fn mystery_external_core() -> String {
    format!("mystery_libretro.{}", std::env::consts::DLL_EXTENSION)
}
```

The value came from the standard library rather than a duplicated literal:
`std::env::consts::DLL_EXTENSION` is `dll` on Windows, `dylib` on macOS, and `so` on Linux and the other
Unix targets — the same three arms as both product mappings:

```rust
    let ext = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
```

(`session::core_file_name` and `Core::file_name` are byte-identical here.) No `lib` prefix is added.

The helper is used at exactly the two fixture sites:

1. `external_fixture` (line 178) — after removing both official GBA core filenames, the real mGBA library is
   copied to `cores.join(mystery_external_core())` instead of `cores.join("mystery_libretro.dll")`. The test
   keeps a real external session on every platform.
2. `a_recovery_that_cannot_open_the_old_core_takes_the_cart_out` (line 1114) — the invalid bytes are written
   to `cores.join(mystery_external_core())` instead of the literal, so the intended invalid external library
   is found and the recovery fails as designed rather than falling back to mGBA.

No `StateNamespace::new("mystery_libretro")` call was touched: it names a state namespace, not a library
file. Neither assertion was weakened, skipped, conditionally ignored, or serialized away, and no sink
semantics, core fallback, picker behavior, or workflow change was needed.

## Validation results

| # | Check | Command | Result |
| --- | --- | --- | --- |
| 1 | no hard-coded `mystery_libretro.dll` path; both sites use the helper | `grep` over the test file | 0 occurrences of the literal; helper defined at line 165, the only two call sites are lines 178 and 1114 |
| 2 | helper yields the host's expected suffix and matches product mapping | `rustc` probe of the std constant + mapping comparison | `host DLL_EXTENSION=dll`; helper body is base name + `DLL_EXTENSION`; product `session::core_file_name` and `Core::file_name` both use the identical `windows`/`macos`/`else` = `dll`/`dylib`/`so` arms |
| 3 | `cargo fmt --all -- --check` | same | exit 0, no output |
| 4 | `cargo test -p slot2 --test core_picker_app --features slot2-input/host` | same | exit 0 in 55 s → **14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out** |
| 5 | `cargo clippy -p slot2 --test core_picker_app --features slot2-input/host -- -D warnings` | same | exit 0, `Finished dev profile`, 0 warnings |
| 6 | `git diff --check` | same | exit 0 |
| 7 | `git status --short` | same | the expected four paths (below) |

Validation 4 detail: all 14 test cases are listed as `... ok`, including both previously failing tests
(`an_empty_picker_says_so_and_changes_nothing`, `a_recovery_that_cannot_open_the_old_core_takes_the_cart_out`).
The run emitted no `skipping (run build/cores.ps1)` line, so the real mGBA/gpSP/FCEUmm libraries were present
in `vendor/` and the fixtures exercised real external sessions rather than returning early — the host-side
proof that the helper resolves the same path the product loads.

Validation 7 output:

```text
 M crates/slot2/tests/core_picker_app.rs
 M docs/HANDOFF-CODEX.md
?? tasks/115-linux-core-picker-fixture-portability.md
```

- ` M crates/slot2/tests/core_picker_app.rs` is the only allowed implementation file: +13/−2 lines, exactly
  one new helper and the two call sites.
- `?? tasks/115-linux-core-picker-fixture-portability.md` is this control file.
- ` M docs/HANDOFF-CODEX.md` is the **pre-existing handoff entry** (Codex's own current-state notes). It was
  accounted for and **not edited** by this worker.
- This report, `tasks/115-linux-core-picker-fixture-portability.worker-result.md`, is the fourth expected
  path and is written after these checks.

The full workspace suite, device build, core build, distribution build, and hosted workflow were not run:
Linux resolution can only be accepted by the next hosted run.

## Modified files

- `crates/slot2/tests/core_picker_app.rs` — one platform-native filename helper plus its two call sites
  (test-only; +13/−2)
- `tasks/115-linux-core-picker-fixture-portability.worker-result.md` — this report only

## Confirmation of no prohibited action

No product code, workflow, dependency, manifest, `Cargo.lock`, unrelated test, or handoff file changed. No
staging, commit, push, tag creation or movement, workflow rerun/cancel, publication, network access,
hardware/card access, or shared-configuration change occurred, and no credential was requested or printed.
Local HEAD remains `2941b077e93e9fee03240a489573f5fd73ec4236` on `main` with an empty index. The
worker-owned `%TEMP%` scratch used for the build probe and command logs was deleted after this report was
written.

## Remaining boundary

1. Codex review of this report and of the single test-file diff.
2. User-authorized **commit and push** of exactly that test change.
3. The next hosted CI run must pass **both** `check` and `device`.
4. A later rerun of that successful revision must accept the core-cache **hit** path and the uploaded device
   artifact — a single run cannot exercise both the miss and the hit path.

Stopping here — no commit, push, hosted rerun, tag, draft release, or publication was performed.
