# Task 119 - Host UI visual gallery (worker result)

## Verdict

SUCCESS. Cumulative attempt 1/2 (this invocation is the first; no earlier attempt was started).

## Harness design

One new integration test, `crates/slot2-ui/tests/visual_gallery.rs`, one `#[test]`:

- Without `SLOT2_GFX_TEST=1` it prints `SLOT2_GFX_TEST not set; skipping GL visual gallery` and
  passes without touching the disk. This is the path `cargo test --workspace` exercises.
- With `SLOT2_GFX_TEST=1` it opens exactly one host window / event loop
  (`HostSurface::open("slot2 ui gallery", (720, 480), 1)`) and one offscreen `GlCanvas` at
  720x480, and renders all 30 frames sequentially through it. Every frame is
  `canvas.clear(deterministic dark background)` -> optional synthetic game frame -> the real
  component's own `draw` -> `canvas.read_back()` from the GL framebuffer.
- No process is spawned, no network is used, no game/core is loaded, and no App internals are
  touched. Fonts come from `assets/fonts` (`UiCtx::new` with the `rgsp` profile and the `en`/`ko`
  packs compiled into `slot2-i18n`); messages come from the normal localized keys.
- A fresh `UiCtx` is built per (scenario, locale) pair, so no face texture, font slot or
  selection can cross from one language or one screen into the next. Menu objects are constructed
  fresh per frame, so no selection state is carried between scenarios.
- Output directory: `target/ui-gallery-task119`, derived from `CARGO_MANIFEST_DIR` and refused
  unless its last two path components are exactly `target` and `ui-gallery-task119`. The test
  removes and recreates only that directory, so stale PNGs cannot survive and Task117/118 files
  under `target/` are untouched.
- The state switcher's one picture is a synthetic 96x72 gradient/checker PNG written to
  `target/ui-gallery-task119/fixtures/state-thumbnail.png`, the only documented non-gallery file.
- The synthetic game frame is flat colour bands and blocks (three fixed colours), drawn for the
  six overlay menus plus the state switcher and the toast (all of them overlay a game frame), so
  the dim layer has something to move. No screenshot and no copyrighted art is used.

Confirmation that one real GL surface rendered actual components: the run used one
`HostSurface`/`GlCanvas` pair and read back genuine pixels (`read_back` asserts 720x480 and each
frame has 345,600 pixels differing from the background, i.e. every pixel of the panel). I also
inspected four frames directly: `ingame-menu-ko.png` shows the seven Korean rows with the middle
row highlighted, the hint line `A 선택 / B 뒤로`, and the dimmed synthetic frame behind the box;
`shelf-menu-disabled-en.png` shows Language highlighted and Display defaults / Boot logo / Sync as
`Unavailable`; `state-switcher-en.png` shows three cards with Slot 3 selected, the synthetic
thumbnail on slots 2 and 3 and the neutral plate on slot 1, and the `Y undo` hint;
`toast-error-ko.png` shows the Korean `cheat-toggle-failed` sentence at full alpha over the
synthetic frame.

## Scenario x locale matrix (720x480, 15 x 2 = 30)

| # | slug | representative state drawn |
|---|------|----------------------------|
| 1 | ingame-menu | `InGameMenu`, row 3 of 7 (`Display`) selected; full seven-row overlay |
| 2 | cheat-menu | 8 synthetic cheats, mixed enabled/disabled, row 2 selected; rows-below bar visible |
| 3 | display-menu | `DisplayMenu::new(Some(AspectFit), true)`; 7-row cropping set, AspectFit highlighted |
| 4 | shader-menu | `ShaderMenu::new(Some(ZfastCrt))`; a non-default built-in preset highlighted |
| 5 | overscan-menu | `OverscanMenu::new(Some(false))`; explicit "whole frame" choice selected |
| 6 | overlay-menu | `OverlayMenu::new(Some(false))`; explicit "off" choice selected |
| 7 | core-picker | GBA, installed `[Mgba, Gpsp]`, running `Mgba`; highlight walked to `gpSP`, badge on `mGBA` |
| 8 | device-menu-disabled | `DeviceMenu::new(60, false, None, None)`; Volume usable, brightness/blue light unavailable |
| 9 | state-switcher | slots 1,2,3; thumbnails on 2 and 3; slot 3 (greatest) selected; `undo_available=true` |
| 10 | shelf-menu-disabled | language/time_zone/about available; display defaults/boot logo/sync unavailable and unreachable |
| 11 | language-picker | `en` + `ko` options, current `en`; highlight moved to the non-first row (`ko`) |
| 12 | timezone-menu | `TimezoneMenu::new(540)`; nonzero `+09:00` selected |
| 13 | about-sticker | `AboutInfo { version: "0.0.0-host-evidence", target: "synthetic-target" }` |
| 14 | power-menu | `PowerMenu`, destructive `PowerOff` row selected |
| 15 | toast-error | `Toast::new("cheat-toggle-failed", {title: "Sample Game"})`, ticked 1.0 s (full alpha) |

## 30-row evidence table

All paths are relative to `C:\SLOT2`. Dimensions are 720x480 for every row (asserted in-test and
re-decoded from disk). Bytes and SHA-256 are from the final evidence run; every file is fresh
(written 2026-10-05 18:46:43-18:48:23 KST / 09:46:43-09:48:23Z, i.e. during that run).

| # | relative path | dims | bytes | sha256 | fresh |
|---|---------------|------|-------|--------|-------|
| 1 | target/ui-gallery-task119/ingame-menu-en.png | 720x480 | 39523 | 73DD93A1351C138DB65F27A7A030EC60548FBF65B2358C6AE7E0524EF92823D1 | yes |
| 2 | target/ui-gallery-task119/ingame-menu-ko.png | 720x480 | 38050 | 451E66ACD8073E57D6FFBF37F28234288935913DB3578DC0A8A0C3522DF2818D | yes |
| 3 | target/ui-gallery-task119/cheat-menu-en.png | 720x480 | 48002 | 0D2A789A7C09B8325EE5528E467FC6F9C06C22D49DB679E1526B17C37CDCFEBB | yes |
| 4 | target/ui-gallery-task119/cheat-menu-ko.png | 720x480 | 50104 | 860AC067AF03779A6A511178F56980FF90A77838D9E08CFF06C18B6FB3FBC812 | yes |
| 5 | target/ui-gallery-task119/display-menu-en.png | 720x480 | 51474 | BE29A14B7454B0D232CE6791E8B11C963DC60FC2F4D97407672D4D1A71CDB193 | yes |
| 6 | target/ui-gallery-task119/display-menu-ko.png | 720x480 | 44634 | 931C77D6A26222FD44099DB83EC8DD1CEC6CDBBFBAAFBBE5CDEA36503287722E | yes |
| 7 | target/ui-gallery-task119/shader-menu-en.png | 720x480 | 43717 | A5210BE333FA49103BF4EEBC3699E53AF0DF3C7F9B4872EFE743DE4502F58235 | yes |
| 8 | target/ui-gallery-task119/shader-menu-ko.png | 720x480 | 40853 | F5D5B2065590D368391C938D3D3EB16CA9C4EA0A86776C6C4DC1C86CD4A57A19 | yes |
| 9 | target/ui-gallery-task119/overscan-menu-en.png | 720x480 | 37247 | CA1678F85A6F01B00C1746E4D6C065F31F108C5BF80523052DA03B7DC3BB2C24 | yes |
| 10 | target/ui-gallery-task119/overscan-menu-ko.png | 720x480 | 33957 | 157039A7A0141C5AEE067D711D0BAD98526F4B8AB07E812503505AD4C9E7DBAA | yes |
| 11 | target/ui-gallery-task119/overlay-menu-en.png | 720x480 | 28296 | C51ABBDC350824A604E35C9A7339B4275A9A99E9ECFDEA6FCEA63B2703732B7B | yes |
| 12 | target/ui-gallery-task119/overlay-menu-ko.png | 720x480 | 25211 | 07A2EDE654EF74C0E0BB3FBC20912549291C15D142C668B5E24351A6DE3C4865 | yes |
| 13 | target/ui-gallery-task119/core-picker-en.png | 720x480 | 28784 | D721B285E7BFC18D02580DAF56A2854271A9626352193282D7DD88AF6D4269DE | yes |
| 14 | target/ui-gallery-task119/core-picker-ko.png | 720x480 | 28237 | 7413A2DC2F45C58CFE4BF0AD4D24AF1C1F42A4A3C08643CC54DF6881037B6365 | yes |
| 15 | target/ui-gallery-task119/device-menu-disabled-en.png | 720x480 | 38097 | C0B250F6854E481368EFA1A39D15BFE4D06EEF62181192D3AEABEA879F0D2E6E | yes |
| 16 | target/ui-gallery-task119/device-menu-disabled-ko.png | 720x480 | 28363 | EA65D41DF4F63928597D54AB90084C5E4C082EF44BEE709C4C20A4630BBBC3C1 | yes |
| 17 | target/ui-gallery-task119/state-switcher-en.png | 720x480 | 82705 | 17AF8E5A17A8233B72D6A62316EF45369914C5A6DA078C8194925C43046E7364 | yes |
| 18 | target/ui-gallery-task119/state-switcher-ko.png | 720x480 | 85266 | 37B1A364ED9C30812B3E4DAA67EEAF9C1293A84CF0E842270614AA7D222AD8B6 | yes |
| 19 | target/ui-gallery-task119/shelf-menu-disabled-en.png | 720x480 | 50623 | 01D1563DD9BBC51DFD80A3E1BA06A058E259B4776F1C688DBED9734BD2177E7D | yes |
| 20 | target/ui-gallery-task119/shelf-menu-disabled-ko.png | 720x480 | 37465 | D5095CD379AB076AC5EDEDEB724185F2F5AB39840C6881E3BBA784DA9AC378F0 | yes |
| 21 | target/ui-gallery-task119/language-picker-en.png | 720x480 | 24680 | 816853A9E7DAD9EC46CB9FE40B54A0278F9A29C7727AF8759409BDBBAB6F7CEE | yes |
| 22 | target/ui-gallery-task119/language-picker-ko.png | 720x480 | 23012 | 7610B361FCE9EAB95E10D5CA63E0A078592099E719D0DEEA41441DE012E743B6 | yes |
| 23 | target/ui-gallery-task119/timezone-menu-en.png | 720x480 | 35569 | 8E3C880296B5668F0E5A74E99D21D48DAD7223C23B7A96E8DB9C94DC063593FD | yes |
| 24 | target/ui-gallery-task119/timezone-menu-ko.png | 720x480 | 34835 | 9CD3DE9EE487E97064326E9590A6FABB0A686E71ACC0547A4EC496AF971DDF96 | yes |
| 25 | target/ui-gallery-task119/about-sticker-en.png | 720x480 | 40951 | D6CC22C6030F8098816A2BE477125027F1EB1F1E1A92DC8DBD2B334E8FD643DA | yes |
| 26 | target/ui-gallery-task119/about-sticker-ko.png | 720x480 | 38178 | D7A5A5CDE3D3FE365F762A34AC428C7E145B980BBA88528EB338A60D66F6BF50 | yes |
| 27 | target/ui-gallery-task119/power-menu-en.png | 720x480 | 27387 | 05300121981B00842C3C583FDDC7D8B6544467CBDEDFB5FB5ED009315C02CE24 | yes |
| 28 | target/ui-gallery-task119/power-menu-ko.png | 720x480 | 25428 | CA984DFB84C45E24BB4246CD84E8D47AE134CC4B25D0010C3F53B0776AE2F763 | yes |
| 29 | target/ui-gallery-task119/toast-error-en.png | 720x480 | 22865 | 75552AF7E3524D1CFE21728302491C1BDFD5D42EE9FECCCDD2DB61A3CA32C18D | yes |
| 30 | target/ui-gallery-task119/toast-error-ko.png | 720x480 | 21785 | 39182D0EF91C07FDFEAF798674637401F41DA0D49CD6DAB862EDC0750B92305B | yes |

All 30 files: PNG signature present, re-decoded successfully by the same `png` crate path the UI
uses, dimensions exactly 720x480, nonempty, and not modified after writing. Hashes are evidence
identity only; none is hard-coded into the test.

Extra files under the same directory (documented, and the only non-gallery entries allowed):

- `target/ui-gallery-task119/fixtures/state-thumbnail.png` (synthetic fixture, 96x72, 10159 bytes,
  SHA-256 C45119AB99D91CE67BB36D077BED4708E92D7805B416652C2885F38B2F0FD991)
- `target/ui-gallery-task119/index.html`

## index.html validation

Relative path: `target/ui-gallery-task119/index.html` (4511 bytes, SHA-256
5048734314D601F59E806340595C4DD16E88EBECC1AD6134021B5CE49A045D06).

- UTF-8: written from a Rust `String`; read back and decoded as UTF-8 without error (contains the
  caption `한국어`). No BOM.
- Title plus the warning that this is host evidence, not device acceptance.
- Exactly 30 `src="..."` references, one per PNG, each a bare relative file name; each of the 30
  names appears exactly once in the document (asserted in-test and re-checked by grep-style
  search: 30 `<img src=` lines, lines 21-78).
- One `<section>` per scenario with the English and Korean frame side by side.
- Inline CSS only; `img { width: 720px }` so the frames read at native width without scaling the
  source files.
- No remote URL (`http://`, `https://`), no `<script`, no `data:` URI, no `file://`, no absolute
  path (`:\`, `/home/`, `/Users/`), no token-like value, no ROM filename. Verified by the test's
  validator and by an independent content search (zero matches).
- No repository name, user name, machine name or path appears in the document.

## Verification commands

| step | command | exit | elapsed |
|------|---------|------|---------|
| 1 | `cargo fmt --all -- --check` | 0 | 18 s |
| 2 | one PowerShell process: `$env:SLOT2_GFX_TEST='1'; cargo test -p slot2-ui --test visual_gallery -- --nocapture --test-threads=1; Remove-Item Env:SLOT2_GFX_TEST` | 0 | 102.1 s (test binary 100.37 s) |
| 3 | file inventory / SHA-256 / decode (PowerShell `Get-FileHash` + in-test re-decode) | 0 | not separately timed |
| 4 | `index.html` structural validation (in-test + search) | 0 | not separately timed |
| 5 | `cargo test --workspace` | 0 | 1246 s |
| 6 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 4 s (incremental; the changed test target was re-checked) |
| 7 | `git diff --check` | 0 | not separately timed |

The opt-in variable was process-local and removed at the end of the PowerShell process,
including on the failure path.

Real-GL result: the GL test ran once to completion with the real components. One host window, one
`GlCanvas`, 30 frames read back from the GL framebuffer; `1 passed; 0 failed; 0 ignored`, exit 0,
`VISUAL_GALLERY_EXIT=0`.

Workspace summed totals (step 5): `RESULT_LINES=101 PASSED=979 FAILED=0 IGNORED=0`, exit 0. This
is Task118's 978/0/0 baseline plus exactly 1, the new opt-in test counting as passed while it
skips without the flag. No regression, no ignored tests. Clippy warning count: 0.

`git diff --check`: exit 0, no whitespace errors.

## Working tree

Final `git status --short --branch`:

```
## main...origin/main
 M crates/slot2-ui/src/shelf_view.rs
 M crates/slot2-ui/tests/insert.rs
 M crates/slot2-ui/tests/shelf_resume.rs
 M docs/HANDOFF-CODEX.md
?? crates/slot2-ui/tests/visual_gallery.rs
?? tasks/117-host-ui-acceptance-audit.md
?? tasks/117-host-ui-acceptance-audit.result.md
?? tasks/117-host-ui-acceptance-audit.worker-result.md
?? tasks/118-square-panel-shelf-hint-visibility.md
?? tasks/118-square-panel-shelf-hint-visibility.result.md
?? tasks/118-square-panel-shelf-hint-visibility.worker-result.md
?? tasks/119-host-ui-visual-gallery.md
```

Changed by this task:

- `crates/slot2-ui/tests/visual_gallery.rs` (new, untracked)
- `tasks/119-host-ui-visual-gallery.worker-result.md` (this file)
- ignored output under `target/ui-gallery-task119/`

Confirmation that only allowed paths changed: the four ` M` entries are the pre-existing
uncommitted Task118 implementation and handoff changes (mtimes 2026-10-05 14:12-14:22 KST and
18:34 KST, all before this invocation began at 18:37 KST); the `??` task records are Codex's own
files. I did not edit any production source, existing test, asset, manifest, lockfile,
documentation, workflow, or Task117/118 record, and I did not touch `docs/HANDOFF-CODEX.md`.
`cargo fmt` was never run in write mode; formatting was applied to the new file only.

## Not acceptance

Image existence and successful decoding are not visual acceptance. They only prove that the
frames are real, correctly sized PNGs produced by the components. Codex and user review of the 30
images, and every physical-device check, remain pending.

## Compliance statement

No delegation, no commit, no stage, no push, no tag, no publish, no network use, no hardware or SD
card access, no shared-configuration change, no software installation, and no ROM inspection
occurred. `build/dist-device.ps1` was not re-run.

## Contract / API concerns

1. The task asks the About sticker to show "version/commit/device information", but the component's
   public contract (`AboutInfo`) carries only `version` and `target` and the module documents that
   no git revision is compiled into the binary. The gallery therefore shows a synthetic version
   string and a synthetic target string, and a commit cannot be shown without changing production
   code, which this task forbids. Reported rather than worked around.
2. `ShelfMenu`'s example availability constant is `timezone_only()`; the required
   language + time zone + About combination is expressible with the public `ShelfAvailability`
   fields, so no production change was needed.
3. The synthetic game frame is drawn for the six overlay menus plus the state switcher and the
   toast (eight frames), because all of those overlay a game frame; the contract names only the
   overlay screens. This is a superset and is documented in the test.
4. `index.html` uses `alt="en"` / `alt="ko"` rather than the file names so that each of the 30
   filenames occurs exactly once, which keeps the "exactly once" check unambiguous.

Elapsed time for this invocation: about 64 minutes of wall clock, dominated by two full
`cargo test --workspace` runs (the first lost its summary to output truncation and was repeated
with shell aggregation).
