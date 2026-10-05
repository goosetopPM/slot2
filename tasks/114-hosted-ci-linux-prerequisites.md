# Task 114 - Hosted CI Linux prerequisites and manifest encoding

## Goal

Fix the first hosted GitHub Actions failure on private repository `gyuhangcho/slot2` without changing
product behavior. The pushed `main` run `37254253979` at commit
`a9f57e2bf6db3d3561025b4757282ff55c4953e2` failed in job `check`, step `clippy (host)`, because Ubuntu
could not build `alsa-sys v0.3.1`: `pkg-config` could not find package `alsa` or `alsa.pc`. The dependency
path is `slot2 -> slot2-audio -> cpal -> alsa -> alsa-sys` for the Linux host feature.

The same run also emitted a non-fatal Rust cache warning because `crates/slot2-i18n/Cargo.toml` is the only
workspace manifest with a UTF-8 BOM. Remove that exact encoding defect while preserving its TOML text.

Read only:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state entries only
- this task in full
- `C:\SLOT2\AGENTS.md`, required rules only
- `C:\SLOT2\.github\workflows\ci.yml`
- `C:\SLOT2\crates\slot2-audio\Cargo.toml`
- `C:\SLOT2\crates\slot2\Cargo.toml`
- `C:\SLOT2\crates\slot2-i18n\Cargo.toml`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, stage,
commit, push, publish, create or move a tag, rerun or cancel a hosted workflow, use the network, access
hardware or a card, or change shared configuration. Maximum two cumulative attempts. Wait up to 5 minutes
for the first response and 45 minutes total.

## Confirmed evidence

- Workflow run: `https://github.com/gyuhangcho/slot2/actions/runs/37254253979`
- Run conclusion: `failure` for pushed commit `a9f57e2bf6db3d3561025b4757282ff55c4953e2`.
- `check` passed checkout, toolchain, Rust cache, host core cache lookup, six host core builds, core presence,
  and formatting.
- `clippy (host)` failed before SLOT2 linting completed.
- Final error: `The system library alsa required by crate alsa-sys was not found`; `pkg-config --libs
  --cflags alsa` exited 1 and reported `Package 'alsa' ... not found`.
- `device` was skipped because it depends on `check`; it did not independently fail.
- The core cache was a miss. The conditional `build cores` step currently installs only `cmake` and
  `build-essential`.
- `Swatinem/rust-cache` warned that `crates/slot2-i18n/Cargo.toml` could not be parsed because its first key
  starts with a BOM. Cargo itself continued by falling back to caching the file.
- The Node.js 20 deprecation warning and upstream C/C++ compiler warnings did not fail this run and are out
  of scope.

## Allowed changes

Edit only:

- `.github/workflows/ci.yml`
- `crates/slot2-i18n/Cargo.toml`
- `tasks/114-hosted-ci-linux-prerequisites.worker-result.md`

Do not change Rust source, dependencies, `Cargo.lock`, any other manifest, reusable device/release
workflows, action versions, core recipes, tests, documentation, or release contents.

## Required implementation

### 1. Install the Linux host audio prerequisite unconditionally

Add a clearly named step in the `check` job that installs the Ubuntu package providing `alsa.pc`:

- run `sudo apt-get update`;
- install `libasound2-dev` with `--no-install-recommends`;
- place the step before `clippy (host)`; and
- do not give it an `if:` condition tied to the core cache.

The step must run on both core-cache misses and hits. Do not put `libasound2-dev` only inside the existing
conditional `build cores` step: that step is skipped after a cache hit, which would reproduce the failure.
It is acceptable to keep `cmake` and `build-essential` in the conditional core-build step. Avoid installing
unrelated desktop packages or suppressing `pkg-config`/`alsa-sys` checks.

### 2. Remove only the manifest BOM

`crates/slot2-i18n/Cargo.toml` currently begins with bytes `EF BB BF`. Remove exactly those three bytes.
Preserve every following byte, including all TOML text and the existing newline convention. Do not reformat
or rewrite the manifest. Prove that the final bytes equal the original byte sequence starting at offset 3.

Confirm that no tracked workspace `Cargo.toml` begins with a UTF-8 BOM after the change.

## Validation

Run the smallest checks that prove this workflow-only correction:

1. show the final `check` job order and prove the Linux audio package step is unconditional and precedes
   `clippy (host)`;
2. prove `libasound2-dev` is named exactly and is not confined to `if: steps.cores.outputs.cache-hit !=
   'true'`;
3. prove the exact three-byte BOM removal and zero remaining BOM-prefixed tracked `Cargo.toml` files;
4. `cargo metadata --offline --no-deps --format-version 1` succeeds;
5. `cargo fmt --all -- --check` succeeds;
6. `git diff --check` succeeds;
7. `git status --short` contains only the two allowed implementation files, this task file, and the worker
   report. Account for any pre-existing handoff-only change without editing it.

Do not repeat workspace tests, clippy, core builds, device builds, or distribution builds: no product code
or dependency changed, and the Linux ALSA prerequisite can be accepted only by a later hosted run.

## Worker report

Write `tasks/114-hosted-ci-linux-prerequisites.worker-result.md` with:

- `SUCCESS` only if every local validation above passes;
- cumulative attempt count;
- the hosted run, job, step, exact missing package diagnosis, and dependency path;
- exact workflow step placement and why it also runs on a core-cache hit;
- the BOM before/after byte evidence and proof that the remaining manifest bytes did not change;
- validation commands and concise results;
- modified files;
- confirmation that no staging, commit, push, tag, workflow rerun, network, hardware/card, or shared
  configuration action occurred; and
- the remaining boundary: Codex review, then user-authorized commit and push, followed by a new hosted CI
  run. That later run must pass both `check` and `device`; a second run is still required to accept the
  core-cache hit path and artifact.

Stop after the report. Do not continue into commit, push, hosted rerun, tag, draft release, or publication.
