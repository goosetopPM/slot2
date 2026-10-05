# Task 114 - Hosted CI Linux prerequisites and manifest encoding (worker result)

## Verdict

**SUCCESS** — cumulative attempt count: **1/2**.

The Ubuntu `alsa.pc` prerequisite is now installed by an unconditional step in the `check` job that
precedes `clippy (host)`, and the single UTF-8 BOM is gone from `crates/slot2-i18n/Cargo.toml` with every
remaining byte preserved. All seven local validations passed. No product code, dependency, lockfile, other
manifest, action version, reusable workflow, core recipe, test, or documentation was changed.

## Hosted failure being corrected

- Workflow run `37254253979` (`https://github.com/gyuhangcho/slot2/actions/runs/37254253979`), conclusion
  `failure`, for pushed commit `a9f57e2bf6db3d3561025b4757282ff55c4953e2` — the current local `main` HEAD.
- Failing job/step: job `check`, step **`clippy (host)`**:
  `cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`.
- Exact diagnosis: `The system library alsa required by crate alsa-sys was not found`; `pkg-config --libs
  --cflags alsa` exited 1 and reported `Package 'alsa' ... not found`. The runner image ships no
  `libasound2-dev`, so no `alsa.pc` was on `PKG_CONFIG_PATH`.
- Dependency path (Linux host feature): `slot2` → `slot2-audio` (`host = ["dep:cpal"]`) → `cpal 0.15` →
  `alsa` → `alsa-sys v0.3.1` → `pkg-config` lookup of `alsa`.
- Secondary, non-fatal: `Swatinem/rust-cache@v2` warned that it could not parse
  `crates/slot2-i18n/Cargo.toml` because the file began with a UTF-8 BOM; Cargo itself fell back to caching
  the file.
- `check` had already passed checkout, toolchain, rust cache, host core-cache lookup, six host core builds,
  core presence, and fmt. The core cache was a **miss**, and the conditional `build cores` step installed
  only `cmake` and `build-essential`. Job `device` was skipped (`needs: check`) rather than failing on its own.

## 1. Unconditional Linux audio prerequisite

Added to `.github/workflows/ci.yml`, in the `check` job, immediately after `Swatinem/rust-cache@v2`:

```yaml
      # cpal's ALSA backend compiles alsa-sys, which needs alsa.pc from libasound2-dev. This
      # step is unconditional on purpose: `build cores` is skipped once the core cache hits,
      # so a package installed there would be missing on exactly the run that hits.
      - name: host audio prerequisites (libasound2-dev)
        run: |
          sudo apt-get update
          sudo apt-get install -y --no-install-recommends libasound2-dev
```

Final `check` job step order (12 steps, line numbers from the edited file):

| # | Line | Step | `if:` |
| --- | --- | --- | --- |
| 0 | 18 | `uses: actions/checkout@v4` | none |
| 1 | 19 | `uses: dtolnay/rust-toolchain@stable` | none |
| 2 | 23 | `uses: Swatinem/rust-cache@v2` | none |
| 3 | 28 | **host audio prerequisites (libasound2-dev)** | **none** |
| 4 | 39 | cache cores | none |
| 5 | 45 | build cores | `steps.cores.outputs.cache-hit != 'true'` |
| 6 | 55 | cores present | none |
| 7 | 66 | fmt | none |
| 8 | 68 | **clippy (host)** | none |
| 9 | 70 | clippy (device) | none |
| 10 | 72 | test | none |
| 11 | 78 | no core was skipped | none |

**Why it also runs on a core-cache hit**: the step carries no `if:` key at all (structural check below), so
it is not tied to `steps.cores.outputs.cache-hit`. `build cores` — the only conditional step in the job,
line 46 — is skipped when the cache hits, which is exactly the run that previously would have had no
`libasound2-dev`. Placing the install before `clippy (host)` means both `clippy (host)` and the later
`test` step (which also builds the default `host` feature) get `alsa.pc` on a miss and on a hit alike.
`cmake` and `build-essential` were deliberately left in the conditional core-build step.

Nothing was installed in addition to `libasound2-dev` and no unrelated desktop package was added; the
`pkg-config`/`alsa-sys` check is untouched (the workflow runs the real `cargo clippy`).

## 2. BOM removed from the i18n manifest

Byte-level surgery on `crates/slot2-i18n/Cargo.toml` removed exactly the first three bytes and copied the
rest verbatim.

| Measurement | Before | After |
| --- | --- | --- |
| byte count | 373 | **370** (= before − 3) |
| SHA-256 | `68c32d34ef74738ec288ef233b253b47afeb76cf9d821fd70faa641f929bc6f4` | `630cc59fcb6f6c4882c5947f29f7ad9cfdc455de0f3ab49a23207cb8b5dac5ba` |
| first 6 bytes (hex) | `ef bb bf 5b 70 61` (`EF BB BF` + `[pa`) | `5b 70 61 63 6b 61` (`[packa`) |
| last 6 bytes (hex) | — | `73 22 5d 20 7d 0a` (`s"] }\n`) |
| BOM present | yes | **no** |
| CR / LF byte counts | — | CR 0 / LF 13 (newline convention preserved) |

- Expected-after hash computed independently from the original bytes at offset 3:
  `630cc59fcb6f6c4882c5947f29f7ad9cfdc455de0f3ab49a23207cb8b5dac5ba` — **equal to the written file's
  hash**.
- Byte-by-byte comparison of the final file against the original bytes from offset 3:
  `REMAINING_BYTES_IDENTICAL_WITH_ORIGINAL_OFFSET3=True`. No TOML text was reformatted or rewritten.
- `git diff` for the file is a single changed line: `-﻿[package]` → `+[package]` (1 insertion, 1 deletion,
  and total file size 373 → 370).
- BOM rescan of all 11 tracked `Cargo.toml` files after the change: no file begins with `EF BB BF`
  (`Cargo.toml` = `5b776f`, the nine other crate manifests = `5b7061`, `crates/slot2-i18n/Cargo.toml` =
  `5b7061`). Before the change `crates/slot2-i18n/Cargo.toml` was the only one reading `efbbbf`.

## Validation results

| # | Check | Command | Result |
| --- | --- | --- | --- |
| 1 | final `check` step order, audio step unconditional and before `clippy (host)` | structural scan of `.github/workflows/ci.yml` (`python`) | audio step index 3 at line 28; `clippy (host)` index 8 at line 68; `AUDIO_BEFORE_CLIPPY=True`; `AUDIO_STEP_HAS_IF=False`; `AUDIO_STEP_HAS_CACHE_HIT_EXPR=False`; `AUDIO_APT_UPDATE=True`; the job's only `if:` is line 46 on `build cores` |
| 2 | `libasound2-dev` named exactly, not confined to the cache-hit conditional | same scan + `git diff` | install line is exactly `sudo apt-get install -y --no-install-recommends libasound2-dev`; the cache-hit expression appears only on `build cores` |
| 3 | exact three-byte BOM removal, zero remaining BOM-prefixed tracked `Cargo.toml` | byte surgery + rescan of 11 tracked manifests | 373 → 370 bytes, hash match, remaining bytes identical, BOM scan clean |
| 4 | `cargo metadata --offline --no-deps --format-version 1` | same | exit 0, 31,397 bytes of JSON, empty stderr |
| 5 | `cargo fmt --all -- --check` | same | exit 0, no output |
| 6 | `git diff --check` | same | exit 0, no whitespace error |
| 7 | `git status --short` | same | exactly the expected set (below) |

Validation 7 output:

```text
 M .github/workflows/ci.yml
 M crates/slot2-i18n/Cargo.toml
 M docs/HANDOFF-CODEX.md
?? tasks/114-hosted-ci-linux-prerequisites.md
```

- ` M .github/workflows/ci.yml` and ` M crates/slot2-i18n/Cargo.toml` are the two allowed implementation
  files (1 insertion/1 deletion for the manifest; +8 lines for the workflow).
- `?? tasks/114-hosted-ci-linux-prerequisites.md` is this control file.
- ` M docs/HANDOFF-CODEX.md` is the **pre-existing handoff-only change** (+14 lines, Codex's 2026-10-05
  current-state entry). It was accounted for and **not edited** by this worker.
- This report, `tasks/114-hosted-ci-linux-prerequisites.worker-result.md`, is the fourth expected path and
  is written after these checks.

No workspace tests, clippy, core build, device build, or distribution build was run: no product code or
dependency changed, and the ALSA prerequisite can only be accepted by a later hosted run.

## Modified files

- `.github/workflows/ci.yml` (+8 lines, one new step)
- `crates/slot2-i18n/Cargo.toml` (−3 bytes: the leading UTF-8 BOM; +1/−1 diff line)
- `tasks/114-hosted-ci-linux-prerequisites.worker-result.md` (this report only)

## Confirmation of no prohibited action

No staging, commit, push, tag creation or movement, workflow rerun/cancel, publication, network access,
hardware/card access, remote configuration, or shared-configuration change occurred. No credential was
requested or printed. `docs/HANDOFF-CODEX.md` and every other tracked file were left untouched, and the
local HEAD remains `a9f57e2bf6db3d3561025b4757282ff55c4953e2` on `main`. The worker-owned `%TEMP%` scratch
files used for the byte surgery and structural checks were deleted after this report was written.

## Remaining boundary

1. Codex review of this report and of the two allowed diffs.
2. User-authorized **commit and push** of exactly the corrected workflow and manifest.
3. A **new** hosted CI run on the pushed commit — that run must pass **both** `check` and `device`.
4. A **second** hosted run is still required after that, to accept the core-cache **hit** path and to inspect
   the `device` artifact (a single run cannot exercise both the miss and the hit path).

Stopping here — no commit, push, hosted rerun, tag, draft release, or publication was performed.
