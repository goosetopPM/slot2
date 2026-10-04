# Task 106 - Public English and Korean README: cumulative worker result

Status: **SUCCESS, cumulative attempt 2/2**.
Attempt 1 wrote the full public English/Korean README pair (contract: `tasks/106-public-readme-en-ko.md`).
Attempt 2 (`tasks/106-public-readme-en-ko-attempt2.md`) fixed only the public data-safety overclaim Codex
raised in `tasks/106-public-readme-en-ko.result.md`, and made no other content change.

No commit, no push, no tag, no release, no network request, no hardware access, and no shared-configuration
change in either attempt. No product code, test, Cargo file, workflow, build script, asset, license file, or
other document was modified.

## 1. Attempt 2 correction (the only change in this attempt)

Codex finding: the README claimed that *every* write SLOT2 makes is atomic and that a card pulled mid-write
cannot leave a partial file. That is broader than the implementation - the card store's atomic path covers
save data, states, thumbnails and settings, while the diagnostic and input-probe reports are written
directly.

Removed wording, `README.md` (3 lines, byte for byte):

```text
Only `System/` comes from the release zip. Everything else is yours or is created on first use, and every
write SLOT2 makes is atomic - a save or a state is written to a temporary file and renamed into place, so a
card pulled mid-write does not leave half a file behind.
```

Removed wording, `README.ko.md` (3 lines, byte for byte):

```text
릴리스 zip에서 오는 것은 `System/`뿐입니다. 나머지는 사용자의 것이거나 처음 사용할 때 만들어지며, SLOT2의
모든 쓰기는 원자적입니다. 세이브나 상태는 옆에 임시 파일로 쓴 뒤 이름을 바꿔 넣기 때문에, 쓰는 도중 카드를
빼도 반쪽짜리 파일이 남지 않습니다.
```

Final wording, `README.md`:

```text
Only `System/` comes from the release zip. Everything else is yours or is created on first use. The save
data, states, state thumbnails and settings that `slot2-store` manages are written through its
temporary-file-and-rename path.
```

Final wording, `README.ko.md`:

```text
릴리스 zip에서 오는 것은 `System/`뿐입니다. 나머지는 사용자의 것이거나 처음 사용할 때 만들어집니다.
`slot2-store`가 관리하는 세이브 데이터, 상태, 상태 섬네일, 설정은 옆에 임시 파일로 쓴 뒤 이름을 바꿔 넣는
경로로 기록합니다.
```

Both keep the accurate statements (only `System/` comes from the release zip; the rest is the user's or is
created on first use), state only the narrower mechanism for the four classes the store manages, and make no
claim at all about diagnostic/probe files, arbitrary files, power loss, filesystem corruption, or safe card
removal. No new warning, FAQ item, implementation essay, or legal disclaimer was added.

## 2. Source evidence for the narrowed statement

| Named class | Public path | Same path in source | Written through `slot2-store` `atomic_write` |
| --- | --- | --- | --- |
| save data | `Saves/<PLATFORM>/<stem>.sav` | `card.rs` `save_path` (`join("Saves")` + `format!("{}.sav", cart.stem)`) | `card.rs::write_save`, line 490: `crate::atomic::atomic_write(&self.save_path(cart), bytes)` |
| states | `States/<PLATFORM>/<stem>/*.state` | `card.rs` `states_dir` (`join("States")`) | `card.rs::write_state_files`, line 214: `crate::atomic::atomic_write(path, data)?` |
| state thumbnails | picture beside a state (`*.png`) | `card.rs` `path.with_extension("png")` | `card.rs::write_state_files`, line 236: `crate::atomic::atomic_write(&path.with_extension("png"), &buf)?` |
| settings | `System/slot2.ini`, `System/games/<PLATFORM>/<stem>.ini` | `card.rs` `settings_path` (`join("slot2.ini")`), `game_settings_path` (`join("games")` + `format!("{}.ini", cart.stem)`) | `settings.rs::write_settings` / `write_global_settings` -> `ini.rs::save`, line 46: `crate::atomic::atomic_write(path, self.to_string().as_bytes())?` |

The mechanism itself, `crates/slot2-store/src/atomic.rs`: `pub fn atomic_write(path: &Path, bytes: &[u8])`
writes to a temporary file beside the target (`tmp_os.push(".tmp")`) and then renames it into place
(`std::fs::rename(&tmp_path, path)?`) - which is why the README says "temporary-file-and-rename path".
`crates/slot2-store/src/lib.rs` documents the same scope for its own crate ("Every write goes through
`atomic_write`").

Excluded by the new wording, with the direct writers that are outside `slot2-store`:

- `crates/slot2/src/diag.rs` line 133: `let _ = std::fs::write(root.join("System").join("slot2-diag.txt"), &out);`
  - a direct `std::fs::write`, no temporary file, no rename;
- `crates/slot2/src/probe.rs`: `pub const LOG: &str = "System/input-probe.txt"` written with a direct
  `std::fs::File::create(&path)` (line 126), and `System/input-probe` removed with a direct
  `std::fs::remove_file` (line 37).

The diagnostic report is still documented where it belongs, in the troubleshooting section of both files
(`System/slot2-diag.txt` and the volatile `/tmp/frontend.log`), and the corrected paragraph in neither
language mentions diagnostics, probes, power loss, or filesystem corruption.

## 3. Attempt-2 locality proof (only that paragraph changed)

Mechanical, not by inspection: replacing the corrected paragraph in each current file with the exact
attempt-1 wording reproduces the attempt-1 bytes bit for bit.

| File | Attempt-1 SHA-256 | Reconstructed SHA-256 | Result |
| --- | --- | --- | --- |
| `README.md` | `90e29384dd9c220ce1a618b645a0d92c29788d0e77a9945b03962bd2774bdfa4` | same | match |
| `README.ko.md` | `b56de765565c3b5da2d1f1fe5cb3f220b15021eb858dbb49cbaef98bf1397c49` | same | match |

Corroborating evidence from the same check:

- `git diff --no-index` between the reconstructed attempt-1 copy and the current file: **1 hunk, 6 changed
  lines** (3 removed + 3 added) for each file - the paragraph only;
- measured byte delta equals the paragraph rewording exactly: `README.md` 16938 -> 16899 (**-39**),
  `README.ko.md` 19838 -> 19800 (**-38**);
- line counts unchanged: 320 (`README.md`) and 306 (`README.ko.md`);
- the new paragraph occurs exactly once and the removed wording occurs zero times in each file;
- `slot2-store` is named exactly once in each whole file, so no broader claim survives elsewhere.

## 4. Attempt-2 focused verification results

Focused suite `verify_a2.py` (outside the repository): **68 [OK], 0 [FAIL]**, exit 0.

1. **Equivalent facts in the two corrected paragraphs** - both keep the ownership/creation sentence, both
   name the same four classes (save data / 세이브 데이터, states / 상태, state thumbnails / 상태 섬네일,
   settings / 설정), both name the temporary-file-and-rename mechanism (`temporary-file-and-rename path` /
   `임시 파일` + `이름을 바꿔`), both scope it to `slot2-store`, and both paragraphs are three sentences on
   three lines.
2. **No broad phrase remains** - 14 pattern families checked in each file, all absent: `every write`,
   `all writes`, `모든 쓰기`, `atomic`, `원자적`, `half a file` / `half-written` / `partial file`, `반쪽`,
   `mid-write`, `쓰는 도중`, `(cannot|can't|never|does not) leave`, `남지 않`, card-removal-safety
   constructions in both languages, and `guarantee` / `보장`.
3. **Class check against `slot2-store`** - every named class was matched to the source path and the
   `atomic_write` call site in section 2, and the paragraph makes no claim about diagnostic/probe output
   (checked against `diag.rs` and `probe.rs`, which write directly), power loss, or corruption.
4. **Attempt-1 suite re-run on the changed bytes** - `verify.py`: **223 [OK], 0 [FAIL]**, exit 0. The nine
   sections, the five tables (`[12, 9, 8, 9, 13]`), the 12 relative links per file, the byte-identical tree
   and command blocks, the warning blocks, the unavailable items, the diagnostic paths, the license links,
   the 13-family forbidden-claim scan, and the UTF-8 / LF / no-trailing-whitespace / balanced-fence checks
   all still pass. External links were checked for syntax only and were not contacted.
5. **`git diff --check`** - exit 0, 0 non-warning lines.

## 5. Files at the end of attempt 2

| Path | Action | Bytes | Lines | SHA-256 | mtime (local, UTC+9) |
| --- | --- | --- | --- | --- | --- |
| `C:\SLOT2\README.md` | replaced in attempt 1, one paragraph corrected in attempt 2 | 16899 | 320 | `e10a30e64b506d03a07368d9a0cac7c451215c52aea58fea222cb677a4282081` | 2026-10-03 16:49:09.655440600 |
| `C:\SLOT2\README.ko.md` | added in attempt 1, one paragraph corrected in attempt 2 | 19800 | 306 | `9976df59f317ba301ba96f3f5a19fb8672b2a5b94fc6fd411dacb623d1c8fd6c` | 2026-10-03 16:49:09.942145500 |
| `C:\SLOT2\tasks\106-public-readme-en-ko.worker-result.md` | updated as this cumulative 2/2 report | - (self-referential) | - | - (self-referential) | 2026-10-03 |

This report's own size and digest are not recorded here, because writing them changes them. The artifacts
whose integrity matters are the two READMEs above.

`git diff --stat -- README.md` (against the stale committed README) -> `1 file changed, 313 insertions(+),
10 deletions(-)`, which is the attempt-1 rewrite plus the attempt-2 paragraph correction.
`git status --short -- README.md README.ko.md` -> ` M README.md` and `?? README.ko.md`.

**No other README paragraph and no other repository file was changed in attempt 2** - proven by the
reverse-edit hash comparison in section 3, and by the re-run parity suite in section 4. Files that were
already dirty before Task 106 keep their earlier mtimes (e.g. `.github/workflows/ci.yml` 2026-10-02
10:49:34, `.github/workflows/device-artifact.yml` 2026-10-02 10:50:52, `.github/workflows/release.yml`
2026-10-02 10:52:17, `docs/DESIGN.md` 2026-10-02 12:05:14, `docs/MILESTONES.md` 2026-10-02 12:05:14,
`build/package-release.ps1` 2026-10-02 21:44:02, `build/test-package-release.ps1` 2026-10-02 22:03:33,
`tasks/105-tag-release-workflow.worker-result.md` 2026-10-02 23:25:03).

## 6. Preserved attempt-1 evidence

Section list (required order, identical in both files): 1 overview and project status, 2 supported devices,
platforms, cores, 3 fresh installation, 4 ROM folders and card layout, 5 device controls, 6 PC preview and
host keyboard controls, 7 troubleshooting / FAQ, 8 translation and project documentation, 9 licenses,
upstream credit, legal boundary. Reciprocal language link on line 9 of each file.

Parity: five tables per file (`[12, 9, 8, 9, 13]` rows), the 10 device-profile ids in the same order with
identical panel / lid / sticks cells, byte-identical platform/core data rows, the required 6 shelf rows and
7 playing rows in the required order, 11 host-key rows, two warning blocks each, the same shell/PowerShell
commands (comments translated one for one), a byte-identical card-layout tree equal to the contract's tree,
the six unavailable items, both diagnostic paths, and the same license links. 12 relative link occurrences
per file, all resolving to the expected type; the only link-target difference is the reciprocal language
link.

Source locations used: `crates/slot2-platform/src/profile.rs` (10 profiles, unknown fallback, `/etc/baseos-release`),
`crates/slot2-store/src/platform.rs` (folders, extensions), `cores/required.txt` and
`crates/slot2-retro/src/registry.rs::supported_cores` (cores and alternates), `crates/slot2-store/src/card.rs`
`DIRS`/`ensure_layout` and `docs/DESIGN.md` (card layout), `crates/slot2/src/app.rs` (shelf/playing bindings,
`STATE_UNDO_S = 30`), `crates/slot2-ui/src/{in_game_menu,shelf_menu,device_menu,display_menu,shader_menu,overlay_menu}.rs`
(menu rows and unavailable entries), `crates/slot2-input/src/host.rs` (host key map, `Escape` unmapped),
`crates/slot2/src/main.rs` (`SLOT2_ROOT`, `./sdcard`) and `docs/DESIGN.md` (720x480 preview window),
`crates/slot2/src/diag.rs` and `docs/DESIGN.md` (diagnostics, volatile `/tmp/frontend.log`), and
`docs/DESIGN.md` (TF2 preferred, else the TF1 data partition).

Contract deviation carried forward from attempt 1 (unchanged and still recorded): the contract asks to link
`System/licenses/`, but that is a card path with no repository counterpart, so it is written as an inline
code path and the repository counterparts `CORE-NOTICES.md`, `licenses/`, `licenses/cores/`, and
`licenses/upstream-slot/LICENSE` are linked. A Markdown link there would not resolve and would fail
verification step 2.

## 7. Commands, exit codes, result lines

| Command (scratch directory unless noted) | Exit | Last result line |
| --- | --- | --- |
| `PYTHONIOENCODING=utf-8 python verify_a2.py` (focused, attempt 2) | 0 | `=== summary: 68 [OK], 0 [FAIL] ===` |
| `PYTHONIOENCODING=utf-8 python verify.py` (attempt-1 parity suite, re-run on the corrected bytes) | 0 | `=== summary: 223 [OK], 0 [FAIL] ===` |
| `git diff --check` (`C:\SLOT2`) | 0 | no non-warning output |
| `git diff --no-index prev-README*.md README*.md` | 1 (diff exists, expected) | 1 hunk, 6 changed lines per file |
| `git status --short -- README.md README.ko.md` (`C:\SLOT2`) | 0 | ` M README.md` / `?? README.ko.md` |
| `sha256sum` / `stat` / `wc -l` (`C:\SLOT2`) | 0 | the digests, sizes and line counts in section 5 |

Attempt-2 verification window: START `2026-10-03T08:05:28Z`, END `2026-10-03T08:05:29Z`
(17:05:28-17:05:29 local). Both suites were run after the last edit (16:49:09 local).

No Cargo test, clippy, build, packager, workflow, or hardware check was run, as the contract requires.

## 8. Freeze after verification

The two READMEs were not edited after the verification runs: their digests, sizes and mtimes re-read after
this report was written are identical to section 5 (`README.md` `e10a30e6...82081` 16899 bytes,
`README.ko.md` `9976df59...8fd6c` 19800 bytes, both mtime `2026-10-03 16:49:09 +0900`). Only this
worker-result file was written after the verification. Both READMEs and this report are UTF-8 without BOM,
LF-only, and `git diff --check` still exits 0.

## 9. Remaining work (not claimed)

Hosted tag-release execution, draft GitHub Release inspection and manual publication; fresh-card first-user
walkthrough; broad physical-device acceptance (RG SP remains the only physically tested device; the 640x480 /
720x720 panels, stick profiles, performance profile and HDMI items in M6 stay open); issue templates and the
migration guide (no filename created or promised). M7's README checkbox is the item this task closes.
