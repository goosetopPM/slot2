# Task 117 - Host UI acceptance audit

## Goal

Regenerate the existing real-GL UI evidence on the Windows host, verify that every generated image is
fresh and structurally valid, and identify the exact UI surfaces that still require live preview or
physical-device review before release. This is an audit task. Do not change product code or tests.

This task is cumulative maximum 2 attempts. A failed or interrupted invocation counts as an attempt.

## Read only

Read only the minimum needed:

- `C:\SLOT2\AGENTS.md`
- the top current-state section of `C:\SLOT2\docs\HANDOFF-CODEX.md`
- this task in full
- `C:\SLOT2\README.ko.md`, section 6 only
- `C:\SLOT2\crates\slot2-gfx\tests\screenshot.rs`, test header and output paths
- `C:\SLOT2\crates\slot2-ui\tests\splash.rs`, `splash_renders_for_real` only
- `C:\SLOT2\crates\slot2\tests\shelf_shot.rs`

Do not read old worker logs or broad source trees. If a prescribed command fails, inspect only the exact
error and the directly relevant manifest or test lines.

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
stage, push, publish, create or move a tag, use the network, access Raspberry Pi or handheld hardware,
access an SD card or Samba drive, change shared configuration, or install software. Do not read, copy,
list, hash, or report local ROM filenames or contents.

## Allowed changes

- Create or update `C:\SLOT2\tasks\117-host-ui-acceptance-audit.worker-result.md`.
- The prescribed tests may overwrite their existing ignored PNG outputs under `C:\SLOT2\target`.
- Worker-owned temporary logs may be written under `C:\SLOT2\target\ui-acceptance-task117`.

No tracked source, test, asset, public documentation, workflow, manifest, or configuration file may change.
Do not delete existing files. Do not create a gallery or copy images into tracked paths.

## Phase 1 - baseline and display capability

Record:

- current branch and full HEAD;
- `git status --short --branch`;
- the names, byte lengths, and last-write times of the expected PNG outputs listed below, if already
  present. Do not inspect or mention unrelated files under `target`.

The expected outputs are exactly:

- `target/gfx-screenshot.png`
- `target/splash-screenshot.png`
- `target/shelf-640x480.png`
- `target/shelf-720x480.png`
- `target/shelf-720x720.png`
- `target/insert-640x480-{00,05,08,10}.png`
- `target/insert-720x480-{00,05,08,10}.png`
- `target/insert-720x720-{00,05,08,10}.png`

There are 17 expected PNG files. Existing files are historical evidence only; the commands below must
rewrite them during this invocation. Record one start timestamp immediately before Phase 2.

## Phase 2 - sequential real-GL checks

Run from `C:\SLOT2` in the same PowerShell process, sequentially and without another Cargo build in
parallel:

```powershell
$env:SLOT2_GFX_TEST = '1'
cargo test -p slot2-gfx --test screenshot -- --nocapture --test-threads=1
cargo test -p slot2-ui --test splash splash_renders_for_real -- --nocapture --test-threads=1
cargo test -p slot2 --test shelf_shot -- --nocapture --test-threads=1
Remove-Item Env:SLOT2_GFX_TEST -ErrorAction SilentlyContinue
```

Always remove the process-local environment variable before stopping, including after a failure. Capture
full command output only in the worker-owned ignored log directory. Put only exit codes, elapsed times,
test-result lines, and the smallest useful error excerpt in the report.

Stop after the first failed command. Do not edit code or retry with a different renderer, driver, Cargo
feature, or environment setting. A missing display, window creation failure, GL compile failure, or driver
failure is a real audit failure to report.

## Phase 3 - generated-image validation

Only if all three commands pass, validate the 17 expected PNG files without altering them:

1. Every file exists, is nonempty, and has a last-write time at or after the Phase 2 start timestamp.
2. Every file begins with the PNG signature and can be decoded by an already available repository or
   standard runtime facility. Do not install a decoder.
3. Dimensions are exact:
   - `gfx-screenshot.png`: 720x480
   - `splash-screenshot.png`: 720x480
   - each shelf/insert filename: the dimensions embedded in its filename
4. Record byte length and SHA-256 for each generated PNG. These are local UI evidence hashes, not release
   artifact hashes.
5. Require exactly 17 files in the expected set. Unrelated PNGs elsewhere under `target` are outside scope.

Do not claim that a decodable, nonblank image is visually correct. The Codex reviewer and user will inspect
the images after this worker report.

## Phase 4 - coverage map

Using only the three prescribed test files and README section already allowed, make a compact coverage map:

- machine-checked and rendered now: GL primitives/presentation/shaders, Korean splash, shelf at 640x480,
  720x480, and 720x720, plus four insertion positions at each geometry;
- still requiring human host review: typography, clipping, overlap, visual balance, platform identity, and
  animation appearance;
- not rendered by these 17 files: the in-game menu and its child menus, state switcher, shelf settings and
  dialogs, English/Korean full-screen traversal, disabled-row appearance, and transient toasts/errors;
- physical-device only: handheld input, audio, Mali GPU behavior, frame pacing/performance, lid/power,
  display output, card mounting, and device-specific safe-area appearance.

Do not infer that an uncovered surface is broken. Mark it as `not covered by Task117 evidence`. Recommend
one next action only: either a focused host screenshot/walkthrough task if Phase 2 passed, or repair of the
single blocking GL/test issue if it failed.

## Final repository check

Run `git status --short --branch` again. It must match the baseline exactly except for the untracked or
modified worker-result file. PNGs and logs are ignored and must not appear. Do not run the full workspace
suite, clippy, device build, Pi test, or any release command because this task changes no product code.

## Worker report

Write `C:\SLOT2\tasks\117-host-ui-acceptance-audit.worker-result.md` with:

- `SUCCESS` or `FAILURE` and cumulative attempt count;
- branch, full HEAD, baseline/final status, and confirmation that no tracked product file changed;
- the three exact commands, exit codes, elapsed times, and concise test results;
- the 17-file validation table with path relative to repository root, dimensions, bytes, SHA-256, and fresh
  timestamp result;
- the four-part coverage map from Phase 4;
- explicit statement that visual correctness and all physical-device behavior remain unclaimed;
- the single recommended next action;
- confirmation of no delegation, commit, stage, push, tag, publish, network, hardware/card access, shared
  configuration change, software installation, or ROM filename/content inspection;
- elapsed time.

Stop after writing the report. Do not create the next task or continue into live preview, fixes, hardware
testing, or release work.
