# Task 109 - Offline release-candidate and commit-set audit: cumulative worker result

**SUCCESS** - cumulative attempt **2 / 2**. Every hygiene rule, every attempt-1 gate and every focused
attempt-2 check passed. Attempt 1 found one real local/hosted artifact parity defect and reported
`FAILURE` on it; attempt 2 corrected it in the three allowed build files.

No release-readiness claim yet, and nothing in this tree is eligible for tagging or publication: the tree
is still dirty, so `dist-device/System/VERSION.txt` identifies **validation output**, never the artifact of
a commit. Dirty/non-releasable status is a **report-only** statement - the stamp itself is exactly its
three public semantic lines, with no dirty-tree marker, and nothing may add one.

Direct execution, no delegation. No commit, stage, push, publish, tag creation or move, remote
configuration, network access, hardware or card access, or shared-configuration change.

## 1. The defect attempt 1 found, and what attempt 2 changed

| | Attempt 1 | Attempt 2 |
| --- | --- | --- |
| `System/VERSION.txt` from the local build | UTF-8 **with** a byte-order mark, and the third line ended with CRLF (216 bytes) | UTF-8 **without** a mark, three LF-terminated lines, final byte LF (212 bytes) |
| `.github/workflows/device-artifact.yml` (hosted producer and gate) | wrote normalized bytes and asserted no BOM, no CR, three lines | **unchanged** - the local producer was brought to it, not the other way round |
| `build/package-release.ps1::Assert-VersionStamp` | accepted both encodings on purpose, so the mismatch could ship | requires one byte contract: valid strict UTF-8, no BOM, no CR anywhere, exactly three LF-terminated lines including the final LF, plus the semantic lines, version and 7-40 hex commit prefix |
| Regression coverage | none for the stamp bytes | the input tree must carry the normalized stamp, and four byte-level tree refusals prove the production validator rejects a mark, CRLF, a missing final LF and invalid UTF-8 |

Implementation choices, per the contract:

- `build/dist-device.ps1` builds the stamp as three lines joined with `"`n"` plus one final `"`n"` and writes
  them with `[System.IO.File]::WriteAllText` and an explicit `New-Object System.Text.UTF8Encoding($false)`
  instance. No shell text writer is involved anywhere in the file (the structural check below proves no
  `Out-File` or `Set-Content` command survives in it).
- The packager's validator is byte-level first: it names a mark, then every CR byte with the offsets it
  found, then a missing final LF, then a strict UTF-8 decode failure, and only then parses the three
  semantic lines. The old lenient path (accepting an optional mark, trimming CR/LF, splitting on `\r?\n`)
  is gone; the check proves `TrimEnd`, `\r?\n` and `[char]0xFEFF` no longer appear in the function.
- No archive, sidecar, publication, cleanup, identity, licence, source or Rust-notice behavior changed:
  the byte-level reverse-application below restores the starting `package-release.ps1` exactly from the
  new one, and the only changed lines are inside `Assert-VersionStamp` and its comment.

## 2. Starting baselines and final hashes (all three baselines matched before editing)

| File | Required starting state | On disk before editing | Final state |
| --- | --- | --- | --- |
| `build/dist-device.ps1` | 8672 bytes, `946a429d1954e89da6972449e5c62dc303813ca7011d551d8ca5f8137482de45` | matched | **9229 bytes**, `7d73ad18d287ec0a5d8b78047b9cf8e0322c3f80f0bdcc294344fbaf043834cd`, no BOM, 0 CR, 175 LF |
| `build/package-release.ps1` | 30719 bytes, `068ac42a55b298e48bcc4effc86011faaf06cb589a8518cc2e53314edaf442ed` | matched | **31957 bytes**, `ff4b54d510a5f55f5b9d1844a720cd5876db157d176caa780a7f058f5117bb43`, no BOM, 0 CR, 593 LF |
| `build/test-package-release.ps1` | 33835 bytes, `102121b7b7b8dc5e3866b4d1ab8a60ce43546c2ebfb2a3a1dfb851d51edf7d3a` | matched | **41710 bytes**, `9eb29189d2f844f3b4d541ffb97ad12b2674e67b80a02534d35eca1ff47da987`, no BOM, 0 CR, 654 LF |

Exact diff scope, proved by reverse-application rather than by assertion: removing only the documented
additions from each new file reproduces its recorded starting digest byte for byte.

| File | Changed lines | Hunks (new-side line, count) | Reverse-application |
| --- | --- | --- | --- |
| `build/dist-device.ps1` | 11 | `[134,4]`, `[141,7]` - all inside the stamp block at lines 134-147 | reproduces `946a429d...82de45` |
| `build/package-release.ps1` | 30 | `[205,8]`, `[221,22]` - all inside `Assert-VersionStamp` and its comment, lines 200-267 | reproduces `068ac42a...af442ed` |
| `build/test-package-release.ps1` | 102 | `[23,4]`, `[152,38]`, `[288,8]`, `[395,27]`, `[434,1]`, `[575,24]` - the header bullet, the input-stamp helper, the input-stamp requirement, the four byte cases, the refusal-message format, and the stamp-contract checks | reproduces `102121b7...51edf7d3a` |

The `-U0` hunks for the first two files were also checked against the intended regions numerically
(`hunks_inside_region` true for both). The two structural line numbers the harness prints for
`package-release.ps1` moved from `540, 544 < 547` to `562, 566 < 569`, i.e. +22 net lines - exactly the
net effect of the 30 changed lines above (8 replaced within the comment, 3 lenient lines replaced by 25
strict ones), which is an independent confirmation of that file's scope.

## 3. Normalized VERSION byte evidence

`dist-device/System/VERSION.txt` after `build/dist-device.ps1 -NoBuild`: **212 bytes**, SHA-256
`ea4e650cf4d63d52a915136097bb5b79bdc345c9d8a225e56ff15b75ac48a19d`. Verified by reading the file as bytes,
independently of PowerShell and of the production validator (12 checks, 0 failed):

| Check | Observed |
| --- | --- |
| no UTF-8 byte-order mark | first bytes `53 4c 4f` = `SLO` |
| zero CR bytes | 0 |
| exactly three LF bytes | 3 |
| final byte is LF | yes |
| strict UTF-8 decode | succeeds |
| exactly three LF-terminated lines | yes (the split's last element is empty) |
| line 1 | `SLOT2 0.1.0 (a8cb4af)` - the `[workspace.package]` version and the current short HEAD |
| line 2 | `Copy the contents of this folder to the root of the card BaseOS boots a frontend from.` |
| line 3 | `BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.` |
| stamped short head is a prefix of HEAD | `a8cb4af` vs `a8cb4af04a40...` |
| no dirty/non-releasable marker in the file | none (`dirty`, `unreleasable`, `WIP` absent) |

The artifact tree that now carries it: 241 files, 77315774 bytes (-4 bytes against attempt 1: the mark and
the carriage return are gone), exactly one top-level `System`, `System/frontend` 3061968 bytes, and exactly
the manifest's six cores (`fceumm`, `gambatte`, `genesis_plus_gx`, `gpsp`, `mgba`, `snes9x` `_libretro.so`)
with their six stamps.

## 4. Focused verification commands (attempt 2)

Run sequentially, offline, on the frozen bytes. Cargo fmt, workspace tests, clippy and the core test were
deliberately not repeated: attempt 1 passed them and this correction changes no Rust, Cargo, core or
workflow input.

| # | Command | Exit | Elapsed | Evidence |
| --- | --- | --- | --- | --- |
| 1 | `[scriptblock]::Create((Get-Content -Raw <path>))` for each of the three files, Windows PowerShell 5.1 | 0 | 2.9 s / 2.7 s / 2.2 s | `PARSE-OK` for all three, 0 failures |
| 2 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild` | 0 | 165.2 s | six cores reported current, Rust notices `66 third-party packages` / `186 files`, last line `==> done` |
| 3 | independent byte read of `dist-device/System/VERSION.txt` | 0 | instant | section 3: 12 checks, 0 failed |
| 4 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/test-package-release.ps1 -CardTree dist-device -ScratchRoot %TEMP%\slot2-t109\pkgsrc3` | 0 | 167.4 s | **65 `[OK]`, 0 `[FAIL]`, 0 `[SKIP]`**, last line `==> all checks passed` |
| 5 | focused source scan of the three files | 0 | <1 s | **25 scans, 0 failed**, plus the three reverse-applications of section 2 |
| 6 | `git diff --check` | 0 | 0.7 s | 13 CRLF-normalization notices, **0 whitespace errors** |

The package suite's expanded pass count is 49 + 16: the input-stamp acceptance plus the four byte refusals
(each also proving its parent directory was untouched) plus the seven stamp-contract checks. The 49
attempt-1 checks are all retained - comparing the check names of the attempt-1 and attempt-2 runs shows no
attempt-1 check missing; the single name that changed text is the publication-call message, whose line
numbers moved from `540, 544 < 547` to `562, 566 < 569` as explained in section 2.

The four refusal diagnostics, quoted from the run, each reached through the production
`Assert-VersionStamp` path with the mutated tree copied from the tree under test:

| Case | Diagnostic |
| --- | --- |
| UTF-8 byte-order mark | `System/VERSION.txt starts with a UTF-8 byte-order mark; the stamp is UTF-8 with no mark` |
| CRLF line endings | `System/VERSION.txt holds 3 carriage return byte(s), first at offset(s) 21, 109, 213; every line ends with one LF byte and nothing else` |
| no final LF | `System/VERSION.txt does not end with a line feed byte; the stamp is three LF-terminated lines including the last one` |
| invalid UTF-8 | `System/VERSION.txt is not valid UTF-8: Exception calling "GetString" with "1" argument(s): "Unable to translate bytes [FF] at index 210 from specified code page to Unicode."` |

Each case left its output parent fingerprint-identical (`case-...-fingerprints equal (1 line(s))`), no
staging directory appeared, and the scratch-wide check reported `no staging directory left behind anywhere
in the scratch`. The bytes for each case are written with `[System.IO.File]::WriteAllBytes` from the very
bytes the producer wrote (the mutated stamp starts as the file's own content): a mark prepended, `0x0A`
expanded to `0x0D,0x0A`, the last byte dropped, and `0xFF, 0xFE` inserted before the final `0x0A`. No
PowerShell encoding default constructs any of them, and the source scan confirms the four blocks contain
`WriteAllBytes`, their explicit byte tokens, and no `Set-Content`, `Out-File` or `GetBytes`.

The seven stamp-contract structural checks (all `[OK]` in the suite, and independently re-derived by the
source scan):

1. the local producer writes `VERSION.txt` with explicit BOM-free UTF-8
   (`[System.IO.File]::WriteAllText($stampPath` with `New-Object System.Text.UTF8Encoding($false)`);
2. it joins its lines with LF and ends the file with one LF;
3. it writes through no `Out-File` or `Set-Content` command at all;
4. it carries the same three stamp lines;
5. the reusable workflow writes the same three lines with `printf`;
6. that workflow's gate still checks the mark (`efbbbf`), the CR (`$'\r'`), the three lines and their text;
7. the validator names the same byte contract both producers write, with none of the lenient path left.

`.github/workflows/device-artifact.yml` was **not modified**: 15253 bytes,
`b6a12bfe160d37e88b530d6f1d25e7c884b60073528e57923857711e411d910b`, mtime 2026-10-02 10:50:52 local -
identical to the bytes recorded when Task 105 delivered it.

Package-suite facts: both runs wrote the pair from 241 files / 241 entries with root `System` only, the
extraction matched the tree on every size and hash, two runs produced identical entry names, identical
extracted hashes and identical container bytes, and the sidecar is `<sha256>  slot2-v0.1.0.zip` plus one LF
with no mark. The archive is 47432434 bytes, SHA-256
`50720632fe9e10ff8da1b06ec55b2c22ef9e7f04dbd43b8508349ea598c82015` - 10 bytes smaller than attempt 1's
package because the stamp lost the mark and the carriage return and deflate re-blocked around it. It is a
**validation-build digest** (the archive stores each entry's mtime and 201 entries are dated today by the
rebuild), so it identifies this run and not a commit. The packager was never invoked as a publication
command, and no release pair was retained in the workspace: the only pairs are the harness's own outputs
under `%TEMP%\slot2-t109\pkgsrc{,2,3}`, which the task allows as temporary scratch.

## 5. Confirmations

- **READMEs stayed fixed**: `README.md` `615e3ecd5f9ae011e14ff7a3c4b4374d9335bfbbb58bd2f84144691c3eb7a498`
  (17117 bytes) and `README.ko.md`
  `631f69600ea8da6ac8de6b44f51f6428c1633a3d4c432a775f16d60f45ccf648` (19975 bytes) - unchanged through both
  attempts.
- **`$env` stayed absent**: it was removed in attempt 1 after all four fail-closed checks, and no `$env`
  path exists or is reported now.
- **Staged entries stayed zero** in every snapshot.
- **Only the three allowed build files plus this cumulative report changed** outside ignored build outputs.
  Status reconciliation: attempt 1 ended at 431 default-form records (fingerprint `70fe04f5...815b38`); the
  current 433 differ by exactly the two files the review step added
  (`tasks/109-offline-release-candidate-audit.result.md`,
  `tasks/109-offline-release-candidate-audit-attempt2.md`) - removing exactly those two records reproduces
  `70fe04f5...815b38` in the default form and `d00ba8ca...481886` in the `-uall` form. Current snapshots:
  default 433 records, `4239d5cb78a76dd7cb9735f76f565a50824ee24485a59d9ab340dcce9f438324`; file-level
  `-uall` 445 records, `19283c626e6c80ee254cea59c8d5d257e6ae6f9029f962c442c322f5dd2f695b`.
- **No worker-created residue**: a workspace scan found no `.slot2-release-*` staging directory, no stray
  `.sha256` and no stray `.zip` outside the licence archives; `dist/` was never populated by this task.
- **Hygiene re-run on the current candidate set** (445 file-level candidates): 0 candidates under an ignored
  root, 0 ROM/disc images, 0 private-key-style names, 0 `.log` files, 0 raw HTTP logs, 0 private-key blocks,
  0 token-shaped values, 0 symlinks or reparse points; the 284 `.md` candidates all resolve as Markdown
  text; the same four pre-existing trailing-whitespace candidates and the one upstream CRLF licence remain,
  and no new file kind appeared - the set grew only by the two Markdown files the review step added.

## 6. Attempt-1 evidence retained (hygiene and the eight gates)

Candidate hygiene from attempt 1 (442 file-level candidates then), unchanged in kind: no staged entry, no
tracked deletion or rename, no ROM/disc image, no key-like name, no `.log`, no raw HTTP request log, no
private-key block, no token-shaped value, no symlink or reparse point, and no candidate under `target/`,
`target-device/`, `dist/`, `dist-device/`, `vendor/`, `sdcard/`, `.gjc/`, `.gjc-logs/` or
`assets/test/local/` (except its tracked `README.txt`). The one permitted deletion was the empty root
`$env`, after a regular-file check, zero length, `?? $env` in `git status --porcelain=v1 -- '$env'` and the
standard empty-file digest `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; no `git
clean`, reset, checkout, restore, add or stash was used, and no other file was removed.

The eight ordered gates of attempt 1 (2026-10-04 09:30:05 - 10:23:09 local, `CARGO_NET_OFFLINE=true`):

| # | Command | Exit | Elapsed | Evidence |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | 10 s | empty output |
| 2 | `cargo test --workspace` | 0 | 1647 s | 100 result lines, **973 passed / 0 failed / 0 ignored** (the Task 101 baseline, no regression) |
| 3 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 55 s | 0 warnings, 0 errors |
| 4 | `cargo clippy -p slot2 -p slot2-gfx -p slot2-ui -p slot2-input --no-default-features --features slot2/device -- -D warnings` | 0 | 12 s | 0 warnings, 0 errors |
| 5 | `cargo test -p slot2-retro --test cores -- --nocapture` | 0 | 4 s | 6 passed / 0 failed / 0 ignored; all seven platforms iterated, no `no test rom available, skipping` line, and no missing-core instruction in the log |
| 6 | `powershell ... build/dist-device.ps1` | 0 | 444 s | last line `==> done` |
| 7 | `powershell ... build/test-package-release.ps1 -CardTree dist-device -ScratchRoot <temp>` | 0 | 280 s | 49 `[OK]`, 0 `[FAIL]`, 0 `[SKIP]`, `==> all checks passed` |
| 8 | `git diff --check` | 0 | 1 s | 0 whitespace errors |

Attempt 1's core coverage: `Gb`, `Gbc`, `Nes`, `Snes` and `Md` loaded cartridges from the ignored
`assets/test/local/` root (names not listed or hashed), `Gba` used the in-repository MIT test image, `Sms`
used the suite's hand-built image, and the alternative-core case loaded gpSP on GBA and Gambatte on GB and
GBC - so all six manifest cores were exercised, and no platform was skipped for a missing core.

Two findings from the attempt-1 hygiene pass are unchanged and were left alone deliberately: trailing
whitespace in three verbatim upstream licence texts (plus one historical Task 78 report), and 13
tracked-modified files whose working copies carry CRLF under the repository's `* text=auto eol=lf` policy
(git reports it will normalize to LF, so the committed blob would be LF while the working copies differ).
Both belong to the user's commit-set decision.

## 7. Remaining release blockers

No artifact from this tree is eligible for tagging or publication until, at minimum:

1. Codex reviews this cumulative audit.
2. The user chooses the intended commit set and explicitly authorizes a commit - including the CRLF working
   copies and the task-history/provenance policy question above.
3. The committed, clean revision is rebuilt so `System/VERSION.txt` identifies its actual source rather than
   `a8cb4af` plus dirty content.
4. A remote exists and the user authorizes push and tag actions (`git remote -v` is still empty).
5. Hosted acceptance is inspected: the first cache miss and a later cache hit for the versioned device-core
   cache, the uploaded artifact, issue-form rendering, the tag-triggered reusable build, PowerShell 7
   packaging, and the draft release's assets and checksum.
6. Fresh-card first-user acceptance and broader hardware acceptance remain user-run checks.

## 8. Limits of this evidence

- Offline and from this machine only: Windows PowerShell 5.1 (not 7), git 2.55, cargo 1.98.1, Docker 29.6.1
  with a locally present `slot2-cross` image. No hosted run was observed, and the attempt-2 correction did
  not re-run Cargo gates by design, so the Rust-level evidence is attempt 1's 09:30-10:23 run - no Rust,
  Cargo, core or workflow input changed after it (the only files touched since are the three PowerShell
  files above).
- The stamp's byte contract is now enforced by both producers and by the packager, but the hosted
  `version stamp` step itself was not executed here; the parity claim rests on the byte-for-byte identical
  text and the workflow's own unchanged assertions, not on a hosted run.
- The package digest identifies this validation run only (section 4); it is not a release checksum.
- Attempt 1's package-restart timings (1647 s for the workspace suite, 444 s for the full device build) are
  this machine's; the pass/fail totals are the material results.
- No second retry of any gate was made with altered inputs.
