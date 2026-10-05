# Task 114 - Codex final review

## Verdict

**Passed (cumulative attempt 1/2).** The implementation matches the hosted failure and stays within the
allowed workflow and encoding scope.

## Confirmed changes

- `.github/workflows/ci.yml` now installs `libasound2-dev` in an unconditional `check` job step before
  `clippy (host)`. The step has no `if:` key, so it runs on both core-cache misses and hits.
- The existing conditional core-build step remains responsible for `cmake` and `build-essential` only.
- `crates/slot2-i18n/Cargo.toml` lost exactly its leading UTF-8 BOM. Its final size is 370 bytes, begins
  with `[packa`, and every byte after the original three-byte BOM is preserved.
- All 11 tracked workspace `Cargo.toml` files are now BOM-free.
- No Rust source, dependency, lockfile, reusable workflow, action version, test, or release content changed.

## Verification

- Workflow structure: audio prerequisite step at line 28, host clippy at line 68; exact install command and
  `apt-get update` present; no condition on the audio step.
- `cargo metadata --offline --no-deps --format-version 1`: passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- The index remains empty. No hosted workflow was rerun and no commit, push, tag, publication, hardware/card,
  or shared-configuration action occurred during the worker task or review.

## Remaining boundary

This is a local correction only. After explicit user authorization, commit and push the Task 114 change set.
The resulting hosted CI run must pass both `check` and `device`. Then rerun that successful revision once to
exercise the core-cache hit path and inspect the uploaded device artifact before any `v0.1.0` tag is created.
