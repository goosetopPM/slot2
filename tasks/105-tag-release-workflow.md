# Task 105 - Reusable device artifact and guarded tag release

## Goal

Add the M7 tag-release path without creating a second device assembly implementation. Extract the current
CI `device` job into one local reusable workflow, call it from normal CI and from a new tag-triggered release
workflow, then package the verified card tree as one installable zip plus a SHA-256 sidecar.

The tag workflow must create a **draft** GitHub Release. A user reviews and publishes it later, after the
Task 104 hosted CI cache-miss/cache-hit acceptance and artifact inspection. Do not create or push a tag,
run GitHub Actions, publish a release, or claim hosted success in this task.

Read only the required parts of:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state section
- `C:\SLOT2\docs\MILESTONES.md`, M7
- `C:\SLOT2\docs\DESIGN.md`, build/deploy and card layout sections
- `C:\SLOT2\.github\workflows\ci.yml`, complete workflow
- `C:\SLOT2\build\dist-device.ps1`, assembly validation, VERSION, and Zip sections
- `C:\SLOT2\build\core-manifest.ps1`, public validators
- `C:\SLOT2\build\rust-notices.ps1` and `build\verify-rust-notices.ps1`, public validators
- root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `cores\required.txt`, and `CORE-NOTICES.md`
- `C:\SLOT2\tasks\104-ci-license-input-cache.result.md`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
push, publish, create a tag, use the network, access hardware, or change shared configuration. Maximum two
attempts. Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed files

- Add `.github/workflows/device-artifact.yml`.
- Add `.github/workflows/release.yml`.
- Modify `.github/workflows/ci.yml` only to replace its current `device` job with a call to the reusable
  workflow while preserving `needs: check`.
- Add `build/package-release.ps1` and, only if needed, a focused offline test helper under `build/`.
- Modify `docs/DESIGN.md` and `docs/MILESTONES.md` only for the implemented but hosted-unverified draft tag
  release path.
- Create or update `tasks/105-tag-release-workflow.worker-result.md`.

Do not modify Rust code, Cargo files, core manifests/pins/patches, existing packagers/validators, assets,
license texts, README, issue templates, translation guide, migration guide, or Task 104 files.

## Reusable device workflow

Create `.github/workflows/device-artifact.yml` with `workflow_call` only. It accepts a required artifact name
and performs the device job currently in `ci.yml`:

- checkout;
- host Rust toolchain, aarch64 target, Rust source cache, and locked Cargo fetches from Task 104;
- Bullseye cross image and unchanged Docker device build;
- versioned cache of `vendor` plus `target-device/cores`;
- cache-miss core build and unconditional binary/stamp/checkout/pin/HEAD validation;
- core source and Rust notice packaging;
- card assembly and all existing six-core, source/license/archive/hash, Rust package/SBOM/manifest gates;
- artifact upload under the caller-supplied name.

Move this behavior; do not weaken, skip, or maintain a duplicate copy in `ci.yml`. The normal CI `device`
job must remain dependent on `check`, call the local reusable workflow, and keep an artifact name based on
`github.sha`. No reusable workflow may publish or request write permissions.

The reusable assembly must write `System/VERSION.txt` with the same semantic fields as
`build/dist-device.ps1`: Cargo package version, checked-out short commit, card-copy instruction, frontend
path, and diagnostic paths. Normalize the text for cross-platform use and validate it before upload.

## Offline release packager

Add:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-release.ps1 \
  -CardRoot <downloaded-card-tree> -Tag <v-version> -Commit <40-hex-commit> -OutputDir <dir>
```

The script derives the repository root from `$PSScriptRoot`. Resolve relative input/output paths from the
caller's cwd. Reject filesystem roots, repository-containing output paths, files/reparse points where normal
directories are required, unsafe tag/path characters, and output/input overlap.

Release identity contract:

- `Tag` must be exactly `v` plus the workspace package version from root `Cargo.toml`; allow a valid Cargo
  prerelease version but no slash, whitespace, path separator, or build metadata outside that exact version.
- `Commit` must be exactly 40 lowercase or uppercase hexadecimal characters.
- `System/VERSION.txt` must name the same Cargo version and a short hash that is an unambiguous prefix of
  `Commit` with at least seven hexadecimal characters.

Before packaging, validate the downloaded card tree using the existing repository contracts:

- exactly the six manifest cores and matching six `.meta` files;
- complete root/upstream/font/core license and core source archive/recipe/patch tree with valid hashes;
- complete Rust notice bundle with exact package set, safe SBOM paths, and valid manifest hashes;
- nonempty `System/frontend`, required fonts, and the contracted `VERSION.txt`;
- no files or directories outside the single top-level `System` directory.

Create exactly:

```text
<OutputDir>/slot2-<Tag>.zip
<OutputDir>/slot2-<Tag>.zip.sha256
```

The zip root contains `System/...`, not an extra artifact/download directory. The sidecar is UTF-8 without
BOM, LF-only, and contains lowercase SHA-256, two spaces, and the zip leaf name. Build in same-parent staging
paths and publish both output files only after all validation succeeds. Refuse to overwrite existing output
files. On any failure, leave an existing output directory and its files unchanged and remove only staging
paths owned by this invocation; if cleanup is blocked, report the exact safe residual path.

After writing, reopen the zip and validate its entry names, root, file set, uncompressed byte lengths, and
SHA-256 against the source card tree. Reject absolute paths, backslashes, `.`/`..`, duplicate/case-colliding
entries, links, and unexpected files. Re-read and validate the sidecar.

## Tag release workflow

Create `.github/workflows/release.yml`:

- trigger only on pushed tags broadly matching `v*`; validate the exact tag in the packaging step;
- use concurrency per tag without cancelling an in-progress release;
- default permissions are `contents: read`;
- a build job calls `device-artifact.yml` with a release-specific artifact name;
- a publish job depends on build, has only `contents: write`, downloads that exact artifact by name, and
  checks out the exact tagged revision with full tag history;
- use the actual checked-out `HEAD` as the 40-hex commit passed to the packager;
- run `build/package-release.ps1`, then independently verify the sidecar before upload;
- use the runner's authenticated `gh` CLI to create a **draft** release for the existing tag with generated
  notes and attach only the zip and sidecar;
- require the tag to exist remotely (`gh release create --verify-tag` or an equivalent hard failure);
- fail if a release for the tag already exists; never overwrite, clobber, or silently replace assets;
- do not use personal tokens, secrets other than the workflow-provided GitHub token, or third-party release
  actions.

Do not add `workflow_dispatch`, release-on-branch, automatic tag creation, prerelease policy guessing, or
automatic publication. A Cargo prerelease tag may still produce a draft; the user decides publication.

## Verification

Run without network, tag creation, push, publication, Docker rebuild, core rebuild, or hardware access:

1. Parse all three workflow files with an available YAML/action checker. If only a local safe subset parser
   is available, state its limits. Run `bash -n` on every final shell body.
2. Prove structurally that normal CI has `check -> reusable device`, release has `reusable device -> publish`,
   and only publish has `contents: write`.
3. Prove that every Task 104 device step/gate exists exactly once in `device-artifact.yml` and no stale device
   implementation remains in `ci.yml` or `release.yml`.
4. Run `package-release.ps1` against the current last-known-good `dist-device` tree and a safe temporary
   output using its actual Cargo version and full Git commit. Do not rebuild the tree.
5. Extract the resulting zip safely and prove its only root is `System`, its relative file set, sizes, and
   hashes equal the source tree, and its sidecar is correct.
6. Repeat into a second temporary output and compare normalized archive entry names and extracted file
   hashes. Zip container bytes need not be identical because archive timestamps may differ.
7. Safe negative copies must fail without changing a marker in a previous output for at least: tag/version
   mismatch, malformed/traversal tag, non-40-hex commit, VERSION hash mismatch, missing core, extra top-level
   item, tampered Rust notice, pre-existing destination files, and zip entry validation mutation if it can be
   injected without weakening production code.
8. Run `git diff --check`.

Do not repeat workspace tests, clippy, cross builds, core builds, or full `dist-device`; product code is
unchanged. Do not execute `gh release`, GitHub Actions, or any command that contacts GitHub.

After final verification, do not change files.

## Documentation

- Update DESIGN to name the reusable artifact workflow and guarded draft tag-release flow.
- Keep the M7 `release.yml` checkbox open and record that implementation/local packaging passed but hosted
  CI, cache-hit artifact parity, tag run, draft inspection, and manual publication remain unverified.
- Do not mark README, issue templates, translation, migration, public release, or M7 acceptance complete.

## Worker report

Write `C:\SLOT2\tasks\105-tag-release-workflow.worker-result.md` even on failure. Include:

- success/failure and cumulative attempt count, maximum 2;
- workflow job graph, triggers, concurrency, permissions, artifact names, and draft-release command;
- proof that the reusable device implementation contains every former CI device step/gate exactly once;
- tag/version/commit/VERSION and release-tree validation rules;
- zip and sidecar names, entry/file/hash counts, roots, and two-run comparison;
- every negative scenario and previous-output preservation evidence;
- YAML validation method and limitations, all shell syntax results, and `git diff --check`;
- commands, exit codes, last result lines, created/modified files, final verification time, and whether content
  changed afterward;
- explicit confirmation that no tag, push, workflow, `gh release`, publication, network, or hardware action
  occurred;
- remaining Task 104/M7 hosted acceptance and documentation work.

Do not commit.
