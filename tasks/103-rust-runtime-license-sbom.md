# Task 103 - Rust runtime license notices and deterministic SBOM

## Goal

Complete the remaining Rust dependency notice work for the device release. Generate a deterministic,
offline bundle for every third-party Rust package in the normal runtime dependency closure of:

```text
slot2 --target aarch64-unknown-linux-gnu --no-default-features --features device
```

Bundle each package's declared license metadata and original license/notice files, generate a documented
SLOT2-specific SBOM, and include the complete bundle in local `dist-device`, release zip input, and the CI
device artifact.

This task does not create `release.yml`, publish anything, change Cargo dependencies, or cover system
libraries and emulator cores. Core, font, artwork, sound, and upstream SLOT2 notices already exist.

Read only the required parts of:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state section
- `C:\SLOT2\docs\MILESTONES.md`, M7
- `C:\SLOT2\docs\DESIGN.md`, core licensing and card layout sections
- `C:\SLOT2\CORE-NOTICES.md`, especially the final section
- `C:\SLOT2\build\dist-device.ps1`
- `C:\SLOT2\build\core-manifest.ps1`, licensing validators
- `C:\SLOT2\.github\workflows\ci.yml`, device build and assembly steps
- workspace `Cargo.toml`, `crates/slot2/Cargo.toml`, and `Cargo.lock`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate,
commit, push, publish, use the network, access hardware, or change shared configuration. Maximum two
attempts. Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed files

- Add `build/package-rust-notices.ps1`.
- Add or update focused license validation helpers under `build/` when needed.
- Modify `build/dist-device.ps1`.
- Modify `.github/workflows/ci.yml` only in the device notice/package/assembly validation path.
- Modify `CORE-NOTICES.md`, `docs/DESIGN.md`, and `docs/MILESTONES.md` only for the completed Rust notice
  bundle and its exact card layout.
- Create/update `tasks/103-rust-runtime-license-sbom.worker-result.md`.

Do not modify Rust source, Cargo manifests, `Cargo.lock`, core manifests/pins/patches, existing license
texts, assets, README, release workflow, Task 101/102 files, or unrelated documentation.

## Exact dependency scope

The package set is the transitive **normal dependency** closure of package `slot2` for the device target,
with default features disabled and feature `device` enabled.

- Include registry and git packages that are actually in that closure.
- Exclude all workspace/path packages from the third-party package list; SLOT2's root MIT license already
  covers them.
- Exclude dev dependencies, build dependencies that are not linked into the runtime binary, host-only
  dependencies, and packages present in `Cargo.lock` but absent from this closure.
- Discover the set from Cargo data. Do not hard-code crate names or counts.
- Run Cargo in offline mode. A missing local package source is a clear failure naming package/version and
  the command needed to populate the normal Cargo cache; do not fetch automatically.
- Compare the final unique third-party set with an independently normalized
  `cargo tree --edges normal --target aarch64-unknown-linux-gnu --no-default-features --features device`
  result. Any missing or extra package is a failure.

Handle duplicate names and versions without collisions. The stable package key and directory name must
distinguish source when required, contain only safe path characters, and be deterministic. Reject path
traversal, absolute output paths inside generated metadata, duplicate output paths, and overwrites.

## Packager

Add:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-rust-notices.ps1 -OutputDir <dir>
```

The script must derive the repository root from `$PSScriptRoot`, resolve a relative `OutputDir` from the
caller's cwd, reject repository/filesystem roots, and use a staging directory. It must not use the network,
install Cargo tools, mutate Cargo cache sources, or include machine-specific paths or timestamps.

Output layout:

```text
<OutputDir>/
|- THIRD-PARTY-RUST.md
|- RUST-SBOM.json
|- RUST-MANIFEST.txt
`- packages/
   `- <stable-package-key>/
      |- PACKAGE.txt
      `- <original license, copying, copyright, or notice files>
```

### Package evidence

For each third-party package:

- Record name, version, normalized source, Cargo.lock checksum when present, declared Cargo license
  expression, declared `license-file` when present, repository/homepage when present, and copied notice
  paths.
- Copy declared `license-file` plus all nonempty packaged source files whose top-level names match
  case-insensitive `LICENSE*`, `LICENCE*`, `COPYING*`, `COPYRIGHT*`, or `NOTICE*`. Deduplicate identical
  paths; preserve bytes exactly; never rewrite line endings or text.
- At least one original nonempty license/notice file is required for every third-party package. If a package
  has metadata but no packaged text, fail and report the exact package instead of inventing or downloading
  a license.
- Fail when both Cargo `license` and `license-file` metadata are absent.
- Do not interpret an SPDX expression as legal advice or choose one side of `OR`. Preserve the declaration
  and all packaged original texts.

`PACKAGE.txt` and `THIRD-PARTY-RUST.md` are UTF-8 without BOM and LF-only. Sort packages by stable key and
files by ordinal relative path. State that the copied original text governs and this inventory is not legal
advice.

### SBOM contract

`RUST-SBOM.json` is a project-local deterministic inventory, not SPDX or CycloneDX. Declare its format as
`slot2-rust-sbom-v1`. It must contain:

- target triple, root package name/version, enabled root features, and SHA-256 of `Cargo.lock`;
- sorted package objects with stable key, name, version, normalized source, checksum, declared license,
  repository/homepage, and sorted bundle-relative notice paths;
- no timestamps, absolute paths, usernames, cache directories, or host-specific separators.

Serialize deterministically as UTF-8 without BOM with LF newlines. Parse it after writing and validate the
schema, sorted unique package keys, safe relative notice paths, file existence, and metadata equality with
the discovered Cargo package.

`RUST-MANIFEST.txt` records the SHA-256 and byte length of every other file in the bundle, sorted by
forward-slashed relative path. It must not hash itself. Validate every entry before replacing output.

Two runs from unchanged `Cargo.lock` and local package sources must have identical relative paths and file
SHA-256 values.

## Distribution and CI

In `build/dist-device.ps1`:

1. Build and validate the Rust notice bundle before deleting the previous `dist-device` tree.
2. Copy it to `System/licenses/rust/`.
3. Postflight-check `RUST-SBOM.json`, `THIRD-PARTY-RUST.md`, `RUST-MANIFEST.txt`, exact package set, each
   package's evidence files, and every manifest hash before zip or ADB.
4. `-NoBuild` still requires Cargo metadata and local cached package sources. Failure must preserve the
   previous good `dist-device` tree.

In the CI device job, run the same packager after Cargo has populated dependencies, assemble the same
`System/licenses/rust/` tree, and validate the package set and manifest hashes before artifact upload.
Do not duplicate a hard-coded crate list in YAML. Keep the existing six-core, license/source, and artifact
checks.

The release zip produced by `dist-device.ps1 -NoBuild -Zip` must contain the exact Rust notice tree.

## Negative and deterministic verification

Use safe temporary/staging copies. Do not modify or delete the real Cargo cache, `Cargo.lock`, final
`dist-device`, core checkouts, or tracked license files.

Verify at least:

1. one package notice file missing from a staged bundle fails postflight;
2. one byte changed in a staged notice file fails `RUST-MANIFEST.txt` hash validation;
3. an extra or missing package directory fails exact-set validation;
4. a malformed/unsafe SBOM notice path fails before distribution replacement;
5. packager preflight failure leaves a marker in a previous output unchanged;
6. dist preflight failure leaves a marker in a previous card tree unchanged.

## Completion commands

After the final code/content change, run:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-rust-notices.ps1 -OutputDir target/task103-rust-notices-a
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-rust-notices.ps1 -OutputDir target/task103-rust-notices-b
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip
git diff --check
```

Also execute the CI device assembly/licensing validation body locally against existing build outputs when
possible, without rebuilding cores or accessing the network. Record any exact CI-only step that cannot be
executed locally.

Rust code and Cargo inputs are unchanged, so do not repeat workspace tests, clippy, or a full device build
that Task 101 already passed. The final dist command must end with `==> done` and retain the exact six
cores, core source bundle, and all prior notices.

After final verification, do not change code or content.

## Documentation

- Update `CORE-NOTICES.md` to point to `System/licenses/rust/`, describe the exact device runtime scope,
  and remove the claim that Rust notices are still missing. Keep the not-legal-advice rule.
- Update the DESIGN card tree with the actual generated paths.
- Update M7 progress without marking tag release, README, issue templates, migration, or full M7 acceptance
  complete.

## Worker report

Write `C:\SLOT2\tasks\103-rust-runtime-license-sbom.worker-result.md` even on failure. Include:

- success/failure and cumulative attempt count, maximum 2;
- discovery commands and exact include/exclude rules;
- package count, stable keys, versions, sources, license expressions, and copied notice files;
- packages rejected for missing metadata/text, if any;
- SBOM schema, Cargo.lock hash, manifest path count/hash, deterministic A/B comparison;
- local dist/zip/CI parity and prior six-core/source gate status;
- every negative scenario and preservation evidence;
- completion commands, exit codes, and last result lines;
- created/modified files, final verification time, and whether content changed afterward;
- contract concerns and remaining M7 work.

Do not commit.
