# Task 105 worker result - reusable device artifact and guarded draft tag release (cumulative 2/2)

## 1. Verdict

**Success on cumulative attempt 2/2.** Attempt 1 delivered the reusable device workflow and the guarded
draft tag release; Codex then found one publication defect, and this attempt fixes exactly that defect and
re-proves it. All attempt-1 behavior is kept.

- Attempt 2 changed only `build/package-release.ps1` and `build/test-package-release.ps1`. The three
  workflows and both documents are byte-identical to what attempt 1 left (section 4).
- The defect: the staged zip and sidecar were promoted with two sequential **file** moves, so a failure
  between them could leave a new zip without its sidecar in the output.
- The fix: the output path must be **absent**, and the validated pair is published with **one same-volume
  directory rename**, with promotion ownership so cleanup can never touch the published directory.
- The published pair is byte-identical to attempt 1 for the same input tree: zip 47 432 444 bytes
  sha256 `ab4acd21070a6305d0e1d04b99fc5cd0a78899148179838cc465674100295861`, sidecar 83 bytes
  sha256 `41429b468d36534106742803e34af255ec07bc292ed95f380536d8681802e4a6`.

No tag was created, nothing was pushed, no workflow ran, no release was published, no network or hardware
was used, and nothing was committed.

## 2. The prior two-file promotion defect (attempt 1)

Codex's finding, quoted from `tasks/105-tag-release-workflow.result.md`:

> `build/package-release.ps1` promotes the staged zip and sidecar with two sequential file moves. If the zip
> move succeeds and the sidecar move fails, the final output contains a new zip without its sidecar. The
> `finally` block removes only the staging directory and does not roll the zip back. Therefore a failed run
> can change the output and expose a partial release pair.
>
> The worker's preservation scenarios all fail before these moves ... They do not inject or simulate failure
> between the first and second promotion, so the report's claim that any failure leaves output untouched is
> not established.

That is correct, and it also explains why attempt 1's preservation table was weaker than it looked: every
scenario there stopped before publication (identity, tree, or "the destination files already exist"), so the
publication step itself was never exercised. The two-file promotion was:

```powershell
        if (-not (Test-Path -LiteralPath $out)) { New-Item -ItemType Directory $out | Out-Null }
        [System.IO.File]::Move($stagedZip, $finalZip)
        [System.IO.File]::Move($stagedSidecar, $finalSidecar)
```

with a `finally` that removed only the staging directory, so a second-move failure left `$finalZip` in place
and no sidecar - a partial release pair in the output. Attempt 1's "an output file that already exists is
never overwritten" was true but insufficient: the failure mode was not overwriting, it was half-publication.

## 3. The final state machine: absent output, one directory rename

`build/package-release.ps1` now runs this sequence, in this order:

```text
identity       Tag == 'v' + [workspace.package] version (semver, no build metadata); Commit 40 hex;
               System/VERSION.txt stamps that version and a 7-40 hex prefix of that commit
path safety    neither path a filesystem root; the output may not be the repository or contain it, may not
               be the card tree or contain it or stand inside it; no wildcard/quote/control character
card root      the card root exists and is a real directory (a file or any reparse point is refused)
OutputDir      must be ABSENT as a name (directory, file or link) - refused BEFORE anything is written,
               and left byte-for-byte as it was; the nearest existing ancestor must be a real directory
card tree      one System root, frontend, exact fonts, Assert-CoreTree, Assert-LicensesTree (licenses,
               sources, archives, recipes, patches and every hash), Assert-RustNoticeBundle (the closure
               Cargo resolves now), the three-line version stamp, no reparse point anywhere
inventory      the file list the zip must hold (241 entries here)
stage          a dot-named directory beside the output, created by this run: write the zip (CreateNew) ->
               reopen and validate every entry -> write the sidecar -> read it back as bytes -> the staging
               directory must hold exactly '<tag>.zip' and '<tag>.zip.sha256' and nothing else
promote        $promoted = Publish-ReleasePair: re-read that the output name is still absent, then ONE
               [System.IO.Directory]::Move(staging -> OutputDir) - the only publication step in the file
publish check  the output directory holds exactly the pair, and the sidecar is revalidated against the
               final zip; a failure here names the output path and leaves it for manual inspection
cleanup        finally: only when $promoted is falsy is Remove-TreeBestEffort called, and only ever with the
               staging path this run created; a promoted output can never be removed by this script
```

Ownership and failure behavior, line by line:

- `Publish-ReleasePair` re-reads the destination's absence immediately before the rename (the same
  `Test-PathTaken` probe), so a name that appeared while the tree was being validated is refused instead of
  replaced. On that refusal, and on any rename failure, it removes **the staging directory it was handed** -
  unpromoted by definition - and throws with the destination path in the message. It never touches the
  destination, copies, merges, replaces, backs up or retries, and there is no environment switch or
  test-only hook anywhere in it.
- The caller's `finally` covers failures *before* publication (identity, tree, zip write, zip validation,
  sidecar write, staging-directory check, or a failed promotion, where the helper has already removed the
  directory). `$promoted` is assigned from the helper's return value, so it is truthy only after the rename
  actually succeeded.
- After the rename the post-promotion check runs inside the same `try`; because `$promoted` is truthy the
  `finally` cannot remove anything, and the wrapped error names the retained output path
  (`the published release pair at <out> did not validate after the rename: ... - it was left in place for
  manual inspection`). That path was not exercised live (it would need a failure injection, which the
  contract forbids); it is established by reading the code plus the structural checks in section 11 that no
  cleanup call can name the output path.
- Platform note, written into the helper as a comment: the re-read is what makes the refusal hold
  everywhere, because POSIX `rename` replaces an *empty* destination directory while Windows `MoveFile`
  refuses any existing destination. A non-empty destination is refused by the rename itself on both. The
  only residual window is an empty directory created at the output name in the microseconds between the
  re-read and the rename, on POSIX, which loses nothing because it is empty.

The release workflow passes `-OutputDir "$RUNNER_TEMP/slot2-release"`, a fresh runner-temporary path, so the
stricter rule costs the release nothing and needed no workflow change (and none was allowed).

## 4. Created and modified files

Attempt 2 rewrote two files and updated this report. The rest is exactly as attempt 1 left it:

| File | Bytes | sha256 | State (UTC mtime) |
|---|---|---|---|
| `build/package-release.ps1` | 30719 | `068ac42a55b298e48bcc4effc86011faaf06cb589a8518cc2e53314edaf442ed` | rewritten in attempt 2 (2026-10-02T12:44:02Z) |
| `build/test-package-release.ps1` | 33835 | `102121b7b7b8dc5e3866b4d1ab8a60ce43546c2ebfb2a3a1dfb851d51edf7d3a` | rewritten in attempt 2 (2026-10-02T13:03:33Z) |
| `.github/workflows/device-artifact.yml` | 15253 | `b6a12bfe160d37e88b530d6f1d25e7c884b60073528e57923857711e411d910b` | unchanged from attempt 1 |
| `.github/workflows/release.yml` | 5410 | `b3ee7c77b26f67ea75a0f075e53b099f7594aef6bf0ec23673ded5f2f1b5647f` | unchanged from attempt 1 |
| `.github/workflows/ci.yml` | 3458 | `4381ed9b33d2c722044e2fc7bd80be3d0bca149e0a0ce65e00f8fbefbd47c9e3` | unchanged from attempt 1 |
| `docs/DESIGN.md` | 44082 | `3e9d23c38c482ae333b3521052af2b8b6d6672851aac867fb06d8bd36af56788` | unchanged from attempt 1 |
| `docs/MILESTONES.md` | 14094 | `5bcd6a0115e06cb31acd80f952d74cd5e6dc0fbb2cd9ed49b72d3c2e10844d2d` | unchanged from attempt 1 |

No workflow, documentation, Rust/Cargo file, shared packager/validator, card tree, license, asset, or Task
105 instruction/result file was touched. `build/package-release.ps1` and `build/test-package-release.ps1`
are new untracked files, like the rest of the uncommitted Task 100-104 work; all files here are LF-only with
a final newline and no trailing whitespace or tabs (`docs/DESIGN.md` keeps its CRLF working-tree endings).
`git diff --check` exits 0.

## 5. Workflow graph, triggers, concurrency, permissions, artifact names, release command (attempt 1, unchanged)

**`.github/workflows/device-artifact.yml`** - `on: workflow_call` only, one required input `artifact-name`;
`env: CARGO_TERM_COLOR=always`, `CARGO_INCREMENTAL=0` (a caller's `env` does not cross into a called
workflow); `permissions: contents: read`; one job `device` with the 14 Task 104 steps plus the new
`version stamp`, uploading `path: dist-device` under `name: ${{ inputs.artifact-name }}` with
`if-no-files-found: error`. No push/pull_request/workflow_dispatch trigger, no write permission, no publish.

**`ci.yml`** - `push` to `main` and `pull_request`; jobs `check` and `device`, where `device` keeps
`needs: check` and calls the reusable workflow with `artifact-name: slot2-device-${{ github.sha }}`, with no
`runs-on` and no `steps` of its own.

**`release.yml`** - `on: push: tags: ['v*']` only; `permissions: contents: read`; concurrency
`release-${{ github.ref }}` with `cancel-in-progress: false`; jobs `build` (calls the reusable workflow with
`artifact-name: slot2-release-${{ github.ref_name }}`) and `publish` (`needs: build`, `permissions:
contents: write` and nothing else) whose steps are: checkout `ref: ${{ github.ref }}`, `fetch-depth: 0`;
download that artifact by name into `card-tree`; `dtolnay/rust-toolchain@stable` (aarch64 target),
`Swatinem/rust-cache@v2` (`cache-targets: false`) and locked fetches; then

```
commit=$(git rev-parse HEAD)
pwsh -NoProfile -File build/package-release.ps1 -CardRoot "$GITHUB_WORKSPACE/card-tree" -Tag "$TAG" \
  -Commit "$commit" -OutputDir "$RUNNER_TEMP/slot2-release"
```

with the tag passed through `env:`, then an independent sidecar verification (`sha256sum` recomputed and
compared, plus `stat -c%s` equal to digest + two spaces + leaf name + one LF), then

```
gh release create "$TAG" --draft --verify-tag --generate-notes --title "SLOT2 $TAG" \
  "$out/slot2-$TAG.zip" "$out/slot2-$TAG.zip.sha256"
```

preceded by a hard failure when `gh release view "$TAG"` finds an existing release, with
`GH_TOKEN: ${{ github.token }}` and no personal token, secret or third-party release action. No `--clobber`,
no `workflow_dispatch`, no release-on-branch, no tag creation, no prerelease guessing, no automatic
publication. `--draft` means a human publishes.

## 6. Every Task 104 device step and gate in the reusable workflow, exactly once (attempt 1, unchanged)

Method: `reconstruct.py` rebuilds the pre-edit `ci.yml` from the untouched header plus the steps now in
`device-artifact.yml`, undoing exactly attempt 1's two edits; the reconstruction is validated against HEAD
(`git diff --no-index --numstat` -> `171  14`, the measurement taken before the edit, and 279 lines) and
against the table Task 104 recorded in `tasks/104-ci-license-input-cache.worker-result.md` section 2.

| # | Task 104 line | now | step |
|---|---|---|---|
| 1 | 84 | 31 | `actions/checkout@v4` |
| 2 | 92 | 39 | `dtolnay/rust-toolchain@stable` |
| 3 | 95 | 42 | `Swatinem/rust-cache@v2` |
| 4 | 100 | 47 | `device target on the host` |
| 5 | 103 | 50 | `build cross image` |
| 6 | 113 | 60 | `cache device cores` |
| 7 | 125 | 72 | `build device cores` |
| 8 | 143 | 90 | `core files and checkouts present` |
| 9 | 174 | 121 | `cross build (profile device)` |
| 10 | 183 | 130 | `package core sources` |
| 11 | 192 | 139 | `fetch Cargo sources for the notice packager` |
| 12 | 201 | 148 | `package rust notices` |
| 13 | 208 | 155 | `assemble card tree` |
| 14 | 275 | 257 | `actions/upload-artifact@v4` |

`device-artifact.yml` replaced ci.yml's 83-line head with a 30-line one, so steps 1-13 sit at exactly
`Task 104 line - 53`; step 14 is at 275 - 53 + 7 (the `VERSION.txt` write is 8 lines where the old `echo` was
1) + 28 (the inserted `version stamp` step). Steps 1-13 parse equal to the pre-edit structures; the only
differences in the moved job are the `assemble card tree` VERSION block, the upload step's `with.name`
(`${{ inputs.artifact-name }}`), and Task 104's own cache change (`path: vendor` + `target-device/cores`,
`key: cores-aarch64-v2-${{ hashFiles('cores/**') }}`, old unversioned key absent everywhere).

Gates asserted present exactly once in the reusable workflow and absent from the other two files (hits here
/ elsewhere; `1/0` unless noted): `docker build -t slot2-cross ...`; the `v2` cache key; both cache paths;
`steps.devcores.outputs.cache-hit`; `find "$base" -maxdepth 3 -name .git`; `cat-file -e "${pin}^{commit}"`;
`git -C "$found" rev-parse HEAD`; `rustup target list --installed | grep -qx aarch64-unknown-linux-gnu`;
`cargo fetch --locked --target aarch64-unknown-linux-gnu` (1 here / 1 in release.yml); `\ncargo fetch
--locked\n` (1 here / 1 in release.yml); `build/package-core-sources.ps1`; `build/package-rust-notices.ps1`;
`build/verify-rust-notices.ps1`; `System/cores is not the manifest`; `System/licenses is not the manifest`;
`diff -r licenses/cores`; `diff cores/common.sh`; `archive_sha256`; `[ "$patches" -eq 2 ]`;
`SLOT2-LICENSE` (2 here, the `cp` and the presence loop) / 0 elsewhere; one upload step. All 15 step names
are unique. `ci.yml` contains none of `dist-device`, `target-device`, `cores-aarch64`, `pwsh`, `docker`,
`System/licenses`, `upload-artifact`; `release.yml` contains no device step and no docker/cache/dist-device
marker.

## 7. Release identity, tree, path and output rules (attempt 1 plus the stricter output rule)

Identical to attempt 1 for the identity, the card tree and the path safety checks, with the output rule
replaced by the absent-directory requirement of section 3:

- the workspace version comes from the `[workspace.package]` section of root `Cargo.toml`
  (CRLF-normalized, section-scoped) and must match `^major.minor.patch(-prerelease)?$`; `Tag` must be
  exactly `v` + that version (`-cne`, so a slash, whitespace or separator is refused by the same
  comparison); `Commit` must match `^[0-9a-fA-F]{40}$`;
- `System/VERSION.txt` must be the three-line stamp `build/dist-device.ps1` writes, with the version equal
  to the workspace version and a 7-40 hex short hash that is a prefix of `Commit`; read as UTF-8 with an
  optional mark and either line ending, because the two producers differ (the current
  `dist-device/System/VERSION.txt` is 216 bytes, starts `efbbbf`, and mixes LF with one trailing CRLF, while
  the reusable workflow writes LF with no mark);
- path safety: no filesystem root as either path; the output may not be the repository, contain it, be the
  card tree, contain it, or stand inside it; no wildcard/quote/control character in a path argument; the
  card root, an existing output path, and the nearest existing ancestor of an absent output path must all be
  real directories (a file or a reparse point is refused);
- the card tree contracts are the repository's own: `Assert-CoreTree`, `Assert-LicensesTree` (licenses,
  sources with every archive/recipe/patch hash, the six meta stamps), `Assert-RustNoticeBundle` (the exact
  package set Cargo resolves now, every copied byte, every SBOM field, every manifest hash), the frontend,
  the exact font set, the version stamp, and a walk that refuses a reparse point anywhere;
- the output path must be absent, before staging, with the path left byte-for-byte as it was (section 3).

## 8. The published pair: file set, hashes, and byte-equivalence with attempt 1

`slot2-v0.1.0.zip` and `slot2-v0.1.0.zip.sha256` (tag `v0.1.0` = `v` + Cargo 0.1.0; commit
`a8cb4af04a403869671a4e829d5b97de45157cfe`, the actual HEAD and the commit `dist-device` was built at).

| Run | Output path | Exit | Pair |
|---|---|---|---|
| direct, `-OutputDir .../a2-smoke/out` (absent) | created by the rename | 0 | exactly the zip + sidecar; zip 47 432 444 B `ab4acd…5861`; sidecar 83 B `41429b…e4a6` |
| harness run A (`out-a`, absent) | created by the rename | 0 | exactly the zip + sidecar; same two hashes |
| harness run B (`out-b`, absent) | created by the rename | 0 | exactly the zip + sidecar; same two hashes |

- 241 files in the card tree -> 241 zip entries -> 241 files after extraction; exactly one root, `System`.
- The zip and the sidecar are byte-identical to attempt 1: `ab4acd21070a6305d0e1d04b99fc5cd0a78899148179838cc465674100295861`
  and `41429b468d36534106742803e34af255ec07bc292ed95f380536d8681802e4a6`, the exact hashes attempt 1
  recorded for the same input tree.
- The sidecar is 83 bytes of `sha256sum -c`-compatible text - 64 lowercase hex, two spaces, the 16-character
  leaf name, one LF, no mark, no CR - parsed by the harness's own byte reader, not by the packager's.
- Safe extraction: `ZipFile.ExtractToDirectory` produced a tree whose root set is `{System}` and whose
  directory fingerprint (relative paths, sizes, SHA-256) equals the source tree's; the extraction of run B
  equals run A's; the two runs' archive entry names are identical. The containers also came out
  byte-identical here because the tree was copied once, so the timestamps match.
- Independent third reading with Python's `zipfile`: 241 entries, roots `{System}`, no absolute/
  backslashed/relative-segment names, no links, 241/241 entry hashes equal to the tree, entry-name set equal
  to the tree's file set.

## 9. Refusals and preservation fingerprints

Fingerprints are the harness's own `Get-DirectoryFingerprint`: every entry's relative path, each file's size
and SHA-256, each link recorded as a link and never followed. Every case compares the fingerprint of the
**destination's parent** (which is where a staging directory would appear) before and after the failed run.

### 9.1 Identity refusals, pointed at a real previous output (`out-a` = zip + sidecar + marker)

| Scenario | Result | Observed |
|---|---|---|
| tag/version mismatch (`v9.9.9`) | refused | `the tag 'v9.9.9' is not 'v0.1.0' (v plus the workspace version in Cargo.toml); a release tag names the version it ships` |
| malformed/traversal tag (`v0.1.0/../../etc`) | refused | same message for `'v0.1.0/../../etc'` |
| non-40-hex commit (`abc123`) | refused | `the commit 'abc123' is not 40 hexadecimal characters` |

Each left `out-a` fingerprint-identical, and the final cumulative check
(`the previous output survived every failing run -- out-a fingerprints equal (3 line(s))`) covers every
failing run in this attempt, including the marker (`previous output marker`, still present with its SHA-256).

### 9.2 Tree refusals, each with its own absent output path

| Scenario | Observed |
|---|---|
| VERSION hash mismatch | `System/VERSION.txt stamps commit deadbee, which is not a prefix of the released commit a8cb4af04a403869671a4e829d5b97de45157cfe` |
| missing core (`System/cores/mgba_libretro.so` removed) | `the assembled card is not the manifest's six cores:` / `...System\cores\mgba_libretro.so is missing` |
| extra top-level item (`README.txt` beside `System`) | `a card tree's root is exactly one System directory; ... holds 2 item(s): System, README.txt` |
| tampered Rust notice | `THIRD-PARTY-RUST.md is not the length RUST-MANIFEST.txt records` |
| tampered Rust notice, same length (one byte flipped, attempt 1) | `THIRD-PARTY-RUST.md is not the hash RUST-MANIFEST.txt records` |

Each parent fingerprint stayed `<empty>` (1 line, unchanged): the absent output path stayed absent and no
staging directory appeared, which is the "fail before staging" evidence for the tree gate.

### 9.3 The output path must be absent

| Case | Observed | Fingerprint before == after |
|---|---|---|
| directory with a marker and a stray file | `refusing to publish: ...\out already exists as a directory, file or link; a release pair is published only into an absent directory, and that path was left untouched` | `dir  out \| file out/marker.txt 22 c9ec2319504b9b346b68ac7320a8131ac1f237cb89e7127e8ac8c8fbef542dc6 \| file out/stray.txt 18 6f04f4adcca7d25978819199b055d907abc3ac9c2ad41d3590aea93ae0609258` (3 lines) |
| otherwise empty directory plus a marker | same message | `dir  out \| file out/marker.txt 15 0f570181122588e39c493aa6b2818136ff896c615be38c262c577a51417ef082` (2 lines) |
| a file at the output path | same message | `file out 23 0ae96b3c3edef4d93180cf1ff5727270a60975435760ef4cce693af96899fa50` (1 line) |
| a directory junction at the output path | same message | `dir  link-target \| link out` (2 lines) |

All four fingerprints are unchanged after the failure, and none of the cases left a staging directory
behind. The junction case proves the "link" half of the rule rather than the "directory or file" halves; the
harness reports `SKIP` with the `mklink` error if a host cannot create a directory link (this host can, and
no case was skipped).

### 9.4 The publication step itself, with a collision that appears after staging

The production helper is called directly: `build/package-release.ps1` is dot-sourced (its dot-source guard
only skips the packaging run) and `Publish-ReleasePair` is given a real staged two-file pair produced by the
production writer (`slot2-v0.1.0.zip` + `.sha256`, sidecar validated first) and a destination that was
created **after** the staging directory was prepared and holds `marker.txt`.

| Claim | Result |
|---|---|
| the rename fails | `refusing to publish: ...\out already exists as a directory, file or link; it was left untouched and the staged pair at ...\owned-staging was removed` |
| the destination fingerprint is unchanged | `dir  out \| file out/marker.txt 11 ea3d607436bbde54952767fa4bdbbb08db4ef030a40657cf70c0589f0d281783` before == after (2 lines) |
| the owned staging path is cleaned | `Test-Path` on the staging path is false after the call - removed by production code, not by the harness |
| no zip or sidecar appears at the destination | both `Test-Path` probes false |

The staging pair was `[slot2-v0.1.0.zip, slot2-v0.1.0.zip.sha256]` before the call, and the destination held
only `marker.txt` after it.

### 9.5 Owned staging cleanup and promoted-output ownership

- After every run in the harness - the two successes and all nine failures - a recursive scan for
  `.slot2-release-*` finds **0** leftovers, in the scratch and in both final scratch trees kept for
  inspection.
- Ownership is structural, and asserted: cleanup of the staging directory appears exactly once and is
  guarded by `if (-not $promoted)`; all three `Remove-TreeBestEffort` calls in the file name a staging path
  (`$staging` or the helper's `$Staging`) and no other; and no cleanup call can name `$out`, `$OutputDir`,
  `$finalZip` or `$finalSidecar`.

## 10. Archive validation (attempt 1, re-run in attempt 2)

A control archive written by the production writer is accepted, and each mutation of a hand-built archive is
refused, with the same messages as attempt 1:

| Mutation | Message |
|---|---|
| absolute entry | `the zip entry '/System/a.txt' is an absolute path` |
| backslash entry | `the zip entry 'System\a.txt' is not forward-slashed` |
| `..` segment | `the zip entry 'System/../a.txt' has an empty or relative path segment` |
| `.` segment | `the zip entry 'System/./a.txt' has an empty or relative path segment` |
| duplicate entry | `the zip holds 'System/a.txt' twice, or twice over when case is ignored` |
| case collision | `the zip holds 'System/A.txt' twice, or twice over when case is ignored` |
| wrong root | `the zip entry 'Other/a.txt' is not under System/` |
| unexpected file | `the zip holds 'System/c.txt', which the card tree does not` |
| symbolic link | `the zip entry 'System/a.txt' is a symbolic link` |
| wrong uncompressed length | `the zip's 'System/a.txt' is 7 bytes and the tree's is 6` |
| wrong bytes (same length) | `the zip's 'System/a.txt' is not the bytes the tree holds (... against ...)` |
| missing entry | `the zip is missing what the tree holds: System/a.txt` |

## 11. Structural and syntax verification

Publication structure, asserted against the production source text by the harness (not by inspection):

| Claim | Result |
|---|---|
| exactly one directory move in production code | 1 (`[System.IO.Directory]::Move`, inside `Publish-ReleasePair`) |
| no `[System.IO.File]::Move` anywhere | 0 |
| no `Move-Item`, `Copy-Item` or `Rename-Item` anywhere | 0 (matched case-sensitively with word boundaries; a case-insensitive substring search would also match `Remove-Item`) |
| the move lives in the publication helper, not in the caller | move line between the `Publish-ReleasePair` and `Invoke-ReleasePackaging` definitions |
| the helper is called once, from the packaging run | one `$promoted = Publish-ReleasePair ...` line |
| cleanup of the staging directory is guarded by the promotion flag | one guarded `Remove-TreeBestEffort -Path $staging` |
| every cleanup call names a staging path and nothing else | 3 of 3 |
| no cleanup call can name the output path or a released file | none |
| the publication call comes after the zip, sidecar and staging-directory checks | lines 540, 544 < 547 |

YAML and shell (attempt 1, re-run unchanged in attempt 2): `workflow_check.py` exits 0 with
`all workflow checks passed (102 assertions)`, including the job graph, triggers, concurrency, permissions,
artifact names, the draft-release command, the Task 104 step/gate inventory, and `bash -n` on **all 21**
`run` bodies (7 in ci.yml, 10 in device-artifact.yml, 4 in release.yml), every one rc=0. The parser is a
local subset parser, because this host has no PyYAML/actionlint/yq/ruby/node: it handles block mappings,
block sequences, block scalars, comments, quoted and plain scalars and flow sequences of scalars, and does
**not** implement anchors, aliases, tags, multi-document streams, flow mappings, folded-scalar folding or
YAML 1.1 coercion (every scalar stays a string), and it validates nothing about the GitHub Actions schema.
Two self-tests (a bad indent is rejected; `on`/`false` are strings) keep "it parsed" from being vacuous.
`reconstruct.py` exits 0 with `numstat vs HEAD : ['171', '14']`.

PowerShell syntax: `[System.Management.Automation.Language.Parser]::ParseFile` -> `PARSE-OK` for both
`build/package-release.ps1` and `build/test-package-release.ps1`.

## 12. Commands, exit codes, final state, and whether anything changed afterwards

All from `C:\SLOT2`, all offline (`cargo` reads only the local cache, `--offline`):

| Command | Exit | Last result line |
|---|---|---|
| `powershell -NoProfile -ExecutionPolicy Bypass -File build/package-release.ps1 -CardRoot dist-device -Tag v0.1.0 -Commit a8cb4af04a403869671a4e829d5b97de45157cfe -OutputDir <temp>\a2-smoke\out` | 0 | `==> done` (`released: ...slot2-v0.1.0.zip (47432444 bytes, sha256 ab4acd...5861)`) |
| `powershell -NoProfile -ExecutionPolicy Bypass -File build/test-package-release.ps1 -CardTree dist-device -ScratchRoot <temp>\a2-final2` | 0 | `==> all checks passed` (49 `[OK]`, 0 `[FAIL]`, 0 `[SKIP]`) |
| `python .../workflow_check.py` | 0 | `all workflow checks passed (102 assertions)` |
| `python .../reconstruct.py` | 0 | `numstat vs HEAD   : ['171', '14'] (measured before the edit: 171 14)` |
| `git diff --check` | 0 | no whitespace diagnostics (only the pre-existing autocrlf CRLF warnings) |
| trailing-whitespace/tab/encoding scan of the touched files | 0 | `clean` for all of them |
| staging-leftover scan of both final scratch trees | 0 | `staging leftovers: 0` for each |

The harness ran three times in attempt 2 (`a2-scratch`, `a2-final`, `a2-final2`); the final one ran
2026-10-02T13:21:04Z - 13:21:51Z (47 s) with exit 0. The direct packager run took 27 s. Tool versions:
Windows PowerShell 5.1.26100.9444, cargo 1.98.1, git 2.55.0.windows.2, Python 3.12.10. No product test
suite, clippy, cross build, core build or `dist-device` rebuild was run; the card tree was used as-is.

### 12.1 Re-validation on the frozen tree

The whole focused verification was run once more against the frozen files, from the repository as it stands,
to confirm the recorded state is the state on disk (no build file differs from section 4, and the sha256 of
each of the five untouched files is still the attempt-1 value):

| Command | Exit | Evidence |
|---|---|---|
| `powershell -NoProfile -ExecutionPolicy Bypass -File build/test-package-release.ps1 -CardTree dist-device -ScratchRoot <temp>\recheck` | 0 | `==> all checks passed`; 49 `[OK]`, 0 `[FAIL]`, 0 `[SKIP]`; 2026-10-02T14:21:20Z - 14:22:39Z (79 s) |
| `powershell -NoProfile -ExecutionPolicy Bypass -File build/package-release.ps1 -CardRoot dist-device -Tag v0.1.0 -Commit a8cb4af04a403869671a4e829d5b97de45157cfe -OutputDir <temp>\recheck-out2` | 0 | `==> done`, 0 error lines, output holds exactly `slot2-v0.1.0.zip` + `.sha256` |
| `git diff --check` | 0 | no whitespace diagnostics |

The re-validation produced the same pair again - zip 47 432 444 bytes
`ab4acd21070a6305d0e1d04b99fc5cd0a78899148179838cc465674100295861`, sidecar 83 bytes
`41429b468d36534106742803e34af255ec07bc292ed95f380536d8681802e4a6`, content
`ab4acd...5861  slot2-v0.1.0.zip` + one LF - so every packaging run of attempt 2 and of this re-validation
produced those same two hashes, byte-identical to attempt 1 for the same input tree. Nothing in section 4
changed; only this report was written after the re-validation.

**After the final verification, no file changed**: the sha256/mtime table in section 4 was taken after the
last check, and only this report was written afterwards - the two build scripts and all five untouched files
still carry the hashes listed there.

## 13. No tag, push, workflow, release, network, or hardware action

Confirmed explicitly: no git tag was created or pushed, no commit was made, no push was made, no GitHub
Actions workflow was triggered or dispatched, no `gh release` was executed (the command exists only as the
text of a workflow step), no release or asset was created, overwritten or clobbered, no network request was
made (`cargo` ran `--offline` against the local cache; the only `git` uses were local
`rev-parse`/`cat-file`/`show`/`diff`), no Docker image was built, no cross or core build was run, and no
hardware, ADB, SD card or Raspberry Pi was accessed. Everything ran against local temporary copies under
`C:/Users/gyuha/AppData/Local/Temp/slot2-t105/`, outside the repository.

## 14. Remaining Task 104 / M7 hosted acceptance and documentation work

Unverified and still open:

1. **Hosted CI has never run this.** No `device` job execution, so the first `v2` cache miss, a later cache
   hit, the post-cache checkout gate, and the uploaded artifact are all unobserved; cache-hit artifact
   parity is therefore unverified.
2. **No tag run.** `release.yml` has never executed: the reusable-workflow call from a tag, the artifact
   download by name, `gh release create --draft --verify-tag`, the generated notes, and the "a release
   already exists" path are unexercised, and `gh` semantics are unverified.
3. **Draft inspection and manual publication** are the user's steps; the published checksum must be compared
   against the sidecar at that point.
4. **PowerShell 7 was not exercised** (only Windows PowerShell 5.1 exists here). The runner path uses the
   same `System.IO.Compression`/`ZipFile` and `System.IO.Directory::Move` code the existing core-source
   packager already uses on the runner, but that is an argument, not an observation. A POSIX-specific
   behavior worth knowing is documented in the helper: `rename` replaces an *empty* destination directory,
   which is why the destination's absence is re-read immediately before the rename.
5. **M7 remains open**: README (en/ko), issue templates, translation guide, migration guide and M7
   acceptance are untouched and unclaimed; the `release.yml` checkbox in `docs/MILESTONES.md` stays
   unchecked on purpose.
6. Not attempted (out of scope): packaging an artifact downloaded from a real hosted run.

## 15. Contract notes

1. The stricter `OutputDir` rule is a deliberate interface change: a release pair is now published only
   into an absent directory. It makes half-publication impossible by construction (one directory rename
   instead of two file moves) and it keeps every failure path from touching an existing path, at the cost
   that a caller can no longer point the packager at a directory that already exists. The release workflow
   already passes a fresh runner-temporary path, so nothing about the release changed; and if that path ever
   did exist, the run fails loudly with the path named instead of replacing anything.
2. `Publish-ReleasePair` is factored as the single publication step so the focused test can call the real
   helper with a real staged pair and a real late collision. There is no failure-injection switch, no
   environment variable, and no test-only branch in production code; the harness dot-sources the packager
   and calls the same function the release run calls.
3. The packager still re-derives the Rust notice closure from Cargo's own offline resolution, so the publish
   job carries a toolchain, the Rust cache and two locked fetches. That is the cost of a gate that can
   detect a bundle drifting from the tagged `Cargo.lock`, and it is unchanged from attempt 1.
4. `VERSION.txt` validation still tolerates a byte-order mark and CRLF because `build/dist-device.ps1`
   writes them while the reusable workflow writes LF without a mark; both forms are checked for the same
   three semantic lines, and the CI step additionally refuses a BOM or a CR in the artifact it uploads.
5. The post-promotion validation failure path (retained output named for manual inspection, never deleted)
   was not exercised live, because doing so would require a failure injection the contract forbids. It is
   established by the code path plus the structural checks that no cleanup call can name the output path.
