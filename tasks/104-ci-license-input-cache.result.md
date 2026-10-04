# Task 104 - Codex final review

## Verdict

**Passed (cumulative attempt 1/2).** The checked-in device workflow now prepares both classes of input
that the Task 101-103 license packagers require on a fresh or cache-hit runner.

## Confirmed changes

- The device job installs the repository Rust toolchain and aarch64 target on the runner host, restores the
  existing Rust cache type, and runs locked host Cargo fetches before the unchanged offline Rust notice
  packager.
- The cross build remains in the Bullseye Docker image with the existing device profile, target, package,
  feature, and target directory.
- The device-core cache now contains both `vendor` and `target-device/cores`. Its `v2` key cannot accept an
  immutable vendor-only cache created under the old key.
- The manifest-driven post-cache gate checks every binary, stamp, unique Git checkout, tracked commit object,
  and checkout HEAD before core source packaging.
- Existing card assembly, six-core exact-set, source/license/archive/hash, Rust package/SBOM/manifest, and
  artifact gates remain present.

## Verification reviewed

- All workflow shell bodies passed `bash -n`; the available local workflow-subset parser confirmed step
  order, cache paths, key replacement, and Docker cross-build retention.
- The post-cache body passed against all six local pinned checkouts. Missing checkout, wrong HEAD,
  vendor-only, missing commit object, and ambiguous checkout simulations all failed with useful diagnostics.
- The unchanged offline Rust packager reproduced the Task 103 result: 66 packages and 186 files.
- `git diff --check` passed.

## Remaining acceptance

GitHub Actions was not run because this task forbade push and network use. A user-controlled hosted run is
still required to observe the first `v2` cache miss, a later cache hit, and the uploaded artifact. This is an
external acceptance check, not a Task 104 implementation failure; M7 artifact parity must remain open until
that evidence exists.
