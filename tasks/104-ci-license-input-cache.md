# Task 104 - Repair CI license input preparation and cache-hit paths

## Goal

Repair the two CI-only blockers found in the final Task 103 review. The GitHub Actions `device` job must:

1. populate the runner host Cargo source cache before the offline Rust notice packager runs; and
2. restore the pinned core source checkouts as well as the built core binaries when the device-core cache hits.

Keep the Task 101-103 bundle contents and validators unchanged. This task fixes CI input preparation and
cache behavior only. It does not publish, create a release workflow, alter dependency resolution, or claim
that a GitHub-hosted run passed when no such run was observed.

Read only the required parts of:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state section
- `C:\SLOT2\tasks\103-rust-runtime-license-sbom.result.md`
- `C:\SLOT2\.github\workflows\ci.yml`, the complete `device` job and reusable Rust/cache patterns
- `C:\SLOT2\build\package-rust-notices.ps1`, preflight and Cargo invocation sections
- `C:\SLOT2\build\package-core-sources.ps1`, preflight section
- `C:\SLOT2\cores\required.txt` and the six `cores/*/commit` files
- root `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
push, publish, access hardware, or change shared configuration. Do not use the network while executing this
task. Maximum two attempts. Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed files

- Modify `.github/workflows/ci.yml` only in the `device` job's Rust source preparation, device-core cache,
  and adjacent explanatory comments or validation.
- Create or update `tasks/104-ci-license-input-cache.worker-result.md`.

Do not modify PowerShell packagers or validators, Rust code, Cargo files, core recipes/pins/patches, assets,
license content, release workflow, Task 103 files, or unrelated documentation.

## Host Cargo source preparation contract

The Rust notice packager runs in PowerShell on the Ubuntu runner host, outside the cross-build container.
Before `package rust notices`:

- install/select the repository Rust toolchain on the host using the existing pinned/toolchain-aware action
  style already used by this workflow;
- make the `aarch64-unknown-linux-gnu` target available on the host;
- restore/cache the host Cargo registry and git source data using the workflow's existing Rust cache style;
- run an explicit locked Cargo fetch for `aarch64-unknown-linux-gnu` on the host;
- keep the notice packager itself offline and unchanged;
- do not mount or copy an opaque Docker Cargo home into the host, and do not add a hard-coded crate list.

The ordering must make it mechanically clear that the host fetch completes before the offline packager.
The fetch must respect `Cargo.lock`; dependency drift is a failure. The cross build must remain in the
Bullseye Docker image and retain its current target, profile, package, features, and target directory.

Task execution itself remains offline. If the current machine cache cannot support an offline equivalent of
the fetch, record that limitation rather than using the network. The checked-in CI command is expected to
populate a fresh GitHub runner when CI later runs with its normal network access.

## Device-core cache contract

The device-core cache must restore all inputs required after a cache hit:

- `vendor/<core>_libretro.so` and matching `.meta` files; and
- all six pinned Git checkouts under `target-device/cores/<core>` needed by
  `package-core-sources.ps1`.

Update the cache paths accordingly. Bump/version the device-core cache key so an immutable cache created by
the old vendor-only definition cannot be accepted as a valid hit. Keep the key dependent on the tracked
core manifest, pins, recipes, and patches. Do not cache the complete `target-device` tree or the final source
bundle.

After either build or cache restore and before source packaging, validate for every name from
`cores/required.txt` that:

- its binary and `.meta` are nonempty;
- its checkout exists as a Git work tree;
- the checkout contains the tracked pin from `cores/<core>/commit`; and
- checkout `HEAD` equals that pin.

Use the manifest loop; do not duplicate six names in YAML. A stale, partial, vendor-only, or wrong-pin cache
must fail before `package-core-sources.ps1` with a useful core/path/pin diagnostic.

## Preserve existing gates

- Keep the existing six-core exact-set checks, source archive/license/recipe/hash checks, Rust SBOM/package
  set/manifest verification, card assembly, and artifact upload.
- Do not weaken cache misses into warnings or rebuild silently after accepting a cache hit.
- Do not change Task 103's 66-package result, declared-only policy, source-aware keys, bundle layout, or
  deterministic output.
- Do not change host `check` job behavior unless an unavoidable YAML-level correction is required; report
  that concern instead of expanding scope.

## Verification

Perform the following without network access:

1. Parse or otherwise validate `.github/workflows/ci.yml` with an available local YAML/action workflow
   checker. If no suitable checker is installed, use a safe local parser and report the limitation.
2. Prove from the final YAML, with step names and ordering, that host toolchain/cache/fetch precede the
   offline notice packager and that the cross build remains Docker-based.
3. Prove that the device cache paths include `vendor` and `target-device/cores`, and that its key is
   incompatible with the old `cores-aarch64-${{ hashFiles('cores/**') }}` cache.
4. Run the final post-cache core validation body locally against the current six device checkouts and
   binaries. Record all six names and pins. Do not rebuild, fetch, or alter them.
5. Exercise a safe temporary negative simulation showing that a missing checkout and a wrong `HEAD`/pin
   relation each fail. Do not modify real checkouts, pins, binaries, or cache data.
6. Run the existing Rust notice packager against a temporary output with the current local Cargo cache. It
   must stay offline and succeed with the Task 103 package set, or report the exact missing local input.
7. Run `git diff --check`.

Do not repeat workspace tests, clippy, core builds, cross builds, or `dist-device`; no product or packager
content changes are allowed. Do not run GitHub Actions, push, or claim actual hosted-CI success. A real CI
run remains a later user-controlled acceptance step.

After final verification, do not change files.

## Worker report

Write `C:\SLOT2\tasks\104-ci-license-input-cache.worker-result.md` even on failure. Include:

- success/failure and cumulative attempt count, maximum 2;
- exact YAML steps added or changed and their final order;
- host Rust action/cache/fetch command and why the offline packager can see those sources;
- device-core cache paths, old and new key forms, and cache-hit validation behavior;
- six core names, expected pins, observed checkout heads, and local validation result;
- missing-checkout and wrong-pin negative simulation results;
- Rust notice packager result and package/file counts, or the exact local-cache limitation;
- YAML validation method, `git diff --check`, exit codes, and last result lines;
- created/modified files, final verification time, and whether content changed afterward;
- an explicit statement that hosted CI was not executed, plus any remaining acceptance work.

Do not commit.
