# Task 117 - Host UI acceptance audit (worker result)

## Verdict

**SUCCESS** — cumulative attempt count: **1/2**.

All three prescribed real-GL commands ran sequentially to completion on the Windows host, and all 17 expected
PNG outputs were rewritten during this invocation, decoded, and matched to their exact dimensions. This is an
evidence-generation audit only: no product code, test, asset, documentation, or workflow changed, and no
visual-correctness or physical-device claim is made.

## Repository identity and status

| Item | Value |
| --- | --- |
| branch | `main` |
| full HEAD | `8662c6ec1fe5120cf9508c9fb4ff6df1786c3b6e` |
| baseline `git status --short --branch` | `## main...origin/main` and `?? tasks/117-host-ui-acceptance-audit.md` |
| final `git status --short --branch` | `## main...origin/main` and `?? tasks/117-host-ui-acceptance-audit.md`, plus this report — identical otherwise |
| tracked changes / staged entries | none (0) |

No tracked product file changed. Only untracked task records appeared; the PNGs and the worker log directory
are ignored by `.gitignore:2:/target` and do not appear in status.

## Phase 1 - baseline

All 17 expected PNGs already existed as historical evidence with timestamps between 2026-09-22 and
2026-09-28 (e.g. `target/gfx-screenshot.png` 10854 bytes, `target/splash-screenshot.png` 47631 bytes,
`target/shelf-720x720.png` 53574 bytes). They were treated as historical only.

**Phase 2 start timestamp: `2026-10-05T04:54:28Z` (local 13:54:28 +09:00).** No unrelated file under `target`
was inspected or listed.

## Phase 2 - sequential real-GL checks

Run from `C:\SLOT2` in one PowerShell process with `$env:SLOT2_GFX_TEST = '1'`, sequentially, with
`Remove-Item Env:SLOT2_GFX_TEST` executed after the loop. Full output went only to the worker-owned ignored
directory `target/ui-acceptance-task117/`. Nothing overlapped; the first command's exit code was 0, so no
command was skipped.

| # | Command | Exit | Elapsed | Test result |
| --- | --- | --- | --- | --- |
| 1 | `cargo test -p slot2-gfx --test screenshot -- --nocapture --test-threads=1` | 0 | 3.3 s | `1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — wrote `target/gfx-screenshot.png` |
| 2 | `cargo test -p slot2-ui --test splash splash_renders_for_real -- --nocapture --test-threads=1` | 0 | 12.9 s | `1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out` (the prescribed test-name filter) — wrote `target/splash-screenshot.png` |
| 3 | `cargo test -p slot2 --test shelf_shot -- --nocapture --test-threads=1` | 0 | 10.7 s | `1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — wrote all three shelves and twelve inserts |

Command 3 ink evidence (the test's own "the shelf did not fail silently" assertion, reported by the test):
`W640H480 Gba: 307199 pixels of ink`, `W720H480 Gba: 345599`, `W720H720 Gb: 518400`.

`SLOT2_GFX_TEST` was confirmed absent from the process environment after the loop. Phase 2 wall time:
**27.3 s**. No display, window-creation, GL-compile, or driver failure occurred, so the Phase 3 validation
was allowed to run.

## Phase 3 - generated-image validation

17 expected files, 17 present. Every file is nonempty, begins with the 8-byte PNG signature
(`89 50 4E 47 0D 0A 1A 0A`), decodes through the already-available .NET `System.Drawing.Bitmap` facility
(no decoder was installed), matches the geometry embedded in its name, and has a last-write time at or after
the Phase 2 start timestamp (`fresh = true` for all 17). Times below are UTC (`mtime`), measured after the
run.

| # | Path (repo-relative) | Dimensions | Bytes | SHA-256 | Fresh |
| --- | --- | --- | --- | --- | --- |
| 1 | `target/gfx-screenshot.png` | 720x480 | 10854 | `da97e1b27094e89361d9eccff0cc1c7b1099737d556bc93d2ac0477a49c2fcb5` | yes |
| 2 | `target/splash-screenshot.png` | 720x480 | 48123 | `5c1b79b15cd5541568e0c9d18e4c7be3aca794a2a801eb632620734a3f641510` | yes |
| 3 | `target/shelf-640x480.png` | 640x480 | 25165 | `d97d7ebcbb9528ff8ed56bdcaa76cdc3ae8a9cbe2bd64ef901bd147355a034a5` | yes |
| 4 | `target/shelf-720x480.png` | 720x480 | 28201 | `861aecf967d23cc82d81bffaa8a065d2b276d11dc5db42c62354566dff0c1ffa` | yes |
| 5 | `target/shelf-720x720.png` | 720x720 | 56167 | `d743ff90d4faf7ba0a9ac65ca60253f38e196e365405f8d0223bc5ea86946d41` | yes |
| 6 | `target/insert-640x480-00.png` | 640x480 | 23880 | `e00db80deedcb1aae455a0ae965ba3dfae90a9167d7fe271ba455223c3dc5bd1` | yes |
| 7 | `target/insert-640x480-05.png` | 640x480 | 21447 | `9bf809b7eff574936894e6ced416df471deeb935e606cd4659c68d88faa6adf5` | yes |
| 8 | `target/insert-640x480-08.png` | 640x480 | 18494 | `2d95ce38e9e49a9fecc7f1544a0403762484ae96be6d1209a77a3aa588bfa7f4` | yes |
| 9 | `target/insert-640x480-10.png` | 640x480 | 12872 | `7332b4231784d6d8d1d81e6d727caa1ccc73276442e002a53f9e06d4bb02690e` | yes |
| 10 | `target/insert-720x480-00.png` | 720x480 | 26931 | `a9b08eedb42252fce050e9d8b0b41f6dbf2fff7a968636b27be10044870a7137` | yes |
| 11 | `target/insert-720x480-05.png` | 720x480 | 22463 | `c782051f3be8a0cac6ffd0a7f6f90aff51be222be3b15abed963ec06b556ae50` | yes |
| 12 | `target/insert-720x480-08.png` | 720x480 | 19833 | `37d4ab6c0ea45086241d645fd0dc439d9e8ca1db66af76d7b35a1124edb36803` | yes |
| 13 | `target/insert-720x480-10.png` | 720x480 | 13873 | `f6c4fcc62166038911c31d51f130bdd34d45dd3e568e2fd726e4b0079713365c` | yes |
| 14 | `target/insert-720x720-00.png` | 720x720 | 54694 | `4123edb7b56cf7b9254072f11847e36778da18cb7f4cdd28188d4f71165ad85a` | yes |
| 15 | `target/insert-720x720-05.png` | 720x720 | 44962 | `0f769391eee251115fc06386459af2b3d350268b88d6deaeb5c99710662c2616` | yes |
| 16 | `target/insert-720x720-08.png` | 720x720 | 34267 | `0416fcf638fa0121cd6f51e8855a96d7f0e98fa276fc267131a012da36b1d486` | yes |
| 17 | `target/insert-720x720-10.png` | 720x720 | 20641 | `0b8df7d487d0354dabe501310d68781098499a3da49041f0b47e59345075a776` | yes |

Achieved dimensions are exact for all 17: `gfx-screenshot.png` and `splash-screenshot.png` at 720x480, and
each shelf/insert file at the width x height embedded in its name (`640x480`, `720x480`, `720x720`), with the
four insert seats `00`, `05`, `08`, `10` present at every geometry. These are local UI evidence hashes, not
release artifact hashes. The full table is also retained at
`target/ui-acceptance-task117/task117-image-validation.txt` (ignored).

**A decodable, nonblank image is not a claim of visual correctness.** Whether these frames are typographically
right, unclipped, well balanced, platform-identifiable, or well animated is left to the Codex reviewer and the
user, who will inspect the images after this report.

## Phase 4 - coverage map

**1. Machine-checked and rendered now** (17 PNGs, real GL offscreen read-back):

- GL primitives, presentation, and shader effects, plus the texture-rewrite path (`slot2-gfx` `screenshot`
  test);
- Korean splash screen, with the test's own wordmark-band and greeting-band ink assertions (`slot2-ui`
  `splash_renders_for_real`, `ko` locale);
- shelf at 640x480, 720x480, and 720x720, each asserted to have ink over the flat-background floor;
- four insertion positions (`seat` 0.0, 0.5, 0.8, 1.0) at each of the three geometries.

**2. Still requiring human host review** (structure is machine-checked, appearance is not): typography
(including the Korean CJK font), clipping, overlap, visual balance, platform identity (cart/port artwork and
labels), and animation appearance.

**3. Not covered by Task117 evidence — not rendered by these 17 files** (this is an evidence gap, not a
defect claim): the in-game menu and its child menus, the state switcher, shelf settings and dialogs,
English and Korean full-screen traversal beyond the splash frame, disabled-row appearance, and transient
toasts/errors.

**4. Physical-device only** (no host screenshot can speak to these; README section 6 states the same
limits): handheld input, audio output, Mali GPU behavior, frame pacing/performance, lid and power behavior,
display output, card mounting, and device-specific safe-area appearance.

No uncovered surface is inferred to be broken; each is marked `not covered by Task117 evidence`.

## Recommended next action

Phase 2 passed, so: **one focused host screenshot/walkthrough task** that drives the live `cargo run -p slot2`
preview (or adds equivalent offscreen captures) to cover the in-game menu and child menus, the state switcher,
shelf settings/dialogs, an English and Korean traversal, a disabled row, and a transient toast — at 720x480,
so a human can review those surfaces before any device run.

## Confirmations

- No delegation: all work was performed directly in this session.
- No commit, stage, push, publish, tag creation or movement, and no network access.
- No hardware, handheld, SD-card, or Raspberry Pi access.
- No shared-configuration change and no software installation (the PNG check used the already-present .NET
  `System.Drawing` facility).
- No local ROM filename or content was read, copied, listed, hashed, or reported; `assets/test/local/` was not
  inspected. The cart titles visible in the shelf captures are names the test invents (for example "Golden
  Sun", "Tetris") in a temporary card directory, not names taken from local ROMs.
- No product code, test, asset, documentation, workflow, manifest, or configuration file changed. Allowed
  writes were limited to this report, the ignored PNGs the prescribed tests rewrite, and worker-owned logs
  under `target/ui-acceptance-task117/`. No existing file was deleted.

**Elapsed time: ~2 min 3 s** (Phase 2 start `2026-10-05T04:54:28Z` to `2026-10-05T04:56:31Z`, local
13:54:28 to 13:56:31 +09:00), of which the three render commands took 27.3 s.
