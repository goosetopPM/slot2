# Task 103 - Codex final review

## Verdict

**Failed (cumulative attempt 2/2). Automatic retries stopped.** Local bundle, dist, and zip paths pass, but
the GitHub Actions device job cannot build the offline Rust notice bundle from a fresh runner as written.

## Confirmed local results

- The device normal runtime closure contains 66 third-party packages: 63 `packaged-text` packages with
  117 original files and three transparent `declared-only` packages.
- Source-aware identity no longer silently collapses duplicate name/version packages; ambiguity fails
  clearly and the synthetic registry/git fixture produces distinct stable keys.
- A/B bundles contain 186 byte-identical files. Local `dist-device -NoBuild -Zip` ended with `==> done` and
  retained six cores, core sources, and the Rust notice tree.
- Bundle mutation, package-set, unsafe path, empty cache, packager preservation, and dist preservation
  negative checks passed locally.

## Blocking CI finding

The `device` job runs `cargo build` inside the `slot2-cross` Docker container. It then runs
`build/package-rust-notices.ps1` on the GitHub runner host. The packager deliberately runs Cargo offline and
requires extracted crate sources in the host Cargo cache, but the workflow neither installs/configures Rust
for this job nor performs a host `cargo fetch`; the container's Cargo cache is not mounted or copied to the
host. Attempt 1 already proved that an empty Cargo home fails at `png`. Therefore the added CI step fails on
a fresh device runner before producing notices. The worker did not execute this CI path and reported it as
CI parity based only on local Windows postflight.

There is also a pre-existing related cache-hit defect: the device core cache stores only `vendor`, while
`package-core-sources.ps1` needs the pinned checkouts under `target-device/cores`. A cache hit skips the core
build that creates those checkouts, so core source packaging cannot run on that path.

## Required next decision

Task 103 has reached 2/2, so no third instruction is created. A separate follow-up must populate and cache
the host Cargo sources before the offline notice step and make the core checkout cache-hit path available.
The CI workflow then needs an actual run before M7 can claim artifact parity.
