# Task 118 - Square-panel shelf hint visibility

## Goal

Fix the host-visible shelf affordance defect found by Task117: on the 720x720 GB shelf, the tall selected
cart covers the `A play` hint in the stable frame, and the hint reappears late during insertion after the
cart moves below it. Keep the stable hint readable on every supported geometry and platform, and make its
insertion behavior intentional and regression-tested.

This task is cumulative maximum 2 attempts. A failed or interrupted invocation counts as an attempt.

## Read only

Read only the minimum needed:

- `C:\SLOT2\AGENTS.md`
- the top current-state section of `C:\SLOT2\docs\HANDOFF-CODEX.md`
- this task in full
- `C:\SLOT2\tasks\117-host-ui-acceptance-audit.result.md`
- `C:\SLOT2\crates\slot2-ui\src\shelf_view.rs`
- the relevant hint/drawing tests in:
  - `C:\SLOT2\crates\slot2-ui\tests\shelf_resume.rs`
  - `C:\SLOT2\crates\slot2-ui\tests\insert.rs`
- `C:\SLOT2\crates\slot2\tests\shelf_shot.rs`, only its geometry/platform matrix and output paths
- only if needed for placement math: `slot2-ui/src/{layout,shelf,insert,skin}.rs` and the exact skin table
  entries used by the failing platform

Do not read old worker logs or unrelated task history. If a prescribed command fails, inspect only the
exact error and directly relevant code.

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
stage, push, publish, create or move a tag, use the network, access Raspberry Pi or handheld hardware,
access an SD card or Samba drive, change shared configuration, install software, or inspect local ROMs.

## Required behavior

### 1. Stable shelf

On a nonempty stable shelf, the complete play or resume/new-game hint line must:

- be visibly unobscured by the selected cart, adjacent carts, machine face, mouth, or port trim;
- remain entirely inside the 640x480 safe area;
- keep the existing horizontal centering and English/Korean message content;
- work for every `Geometry::{W640H480,W720H480,W720H720}` and every supported platform skin, including the
  tall GB/GBC cart on 720x720;
- remain absent on an empty shelf.

Choose the vertical position from actual layout/cart geometry rather than a platform-name exception or a
hard-coded `W720H720 + GB` branch. A shorter cart may keep the existing visual position if it already meets
the contract. Do not resize carts, labels, safe areas, mouth/port artwork, fonts, or message text to make the
test pass.

### 2. Insert and eject transition

At `seat == 0.0`, `ShelfView::draw_insert` must still be visually identical to the stable shelf for the
row, cart, hint, machine face, and trim. There must be no first-frame jump.

Once travel starts (`seat > 0.0`), do not draw the shelf action hint during either insert or eject. The hint
describes the action before it is accepted; hiding it during travel prevents the tall cart from covering it
and prevents it from flashing back when the cart moves below its former y position. It returns only when the
normal stable shelf frame is drawn again.

Do not change insertion/ejection timing, travel curves, cart/row placement, draw order between the moving
cart and machine face/port, audio timing, App state transitions, or core launch behavior.

### 3. Regression tests

Add focused structural tests, using `RecordingCanvas` and existing helpers where possible:

1. For all three geometries, all supported platform skins, English and Korean, and both play and
   resume/new-game variants, the stable hint exists, stays in the safe area, and its marks do not intersect
   the selected cart rectangle or machine face/port band.
2. An empty shelf still draws no hint.
3. At `seat == 0.0`, stable draw and insert draw have the same hint marks and positions in addition to the
   existing cart-image equality.
4. For representative positive travel samples including an early value, `0.5`, `0.8`, and `1.0`, neither
   insert nor eject draws any hint marks on any geometry. Include the 720x720 GB case that exposed the bug.
5. Preserve the existing port/cart occlusion, seated-cart visibility, upload-cache, title, and resume-hint
   tests unchanged unless an assertion is demonstrably stale under this contract. Do not weaken a test or
   replace a semantic assertion with a snapshot hash.

Tests must identify the hint by its established tint/cap operations and the selected cart through existing
placement/skin geometry. Do not add public production hooks only for tests.

## Allowed files

- `C:\SLOT2\crates\slot2-ui\src\shelf_view.rs`
- `C:\SLOT2\crates\slot2-ui\tests\shelf_resume.rs`
- `C:\SLOT2\crates\slot2-ui\tests\insert.rs`
- `C:\SLOT2\tasks\118-square-panel-shelf-hint-visibility.worker-result.md`
- ignored outputs/logs under `C:\SLOT2\target`

Do not modify `shelf_shot.rs`; it already produces the required before/after visual evidence. Do not modify
Task117 records, public documentation, other source/tests, assets, manifests, lockfiles, workflows, or
configuration. Preserve all unrelated current working-tree changes.

## Verification

Run sequentially from `C:\SLOT2`. Do not overlap Cargo commands or builds against the same target.

```text
cargo fmt --all -- --check
cargo test -p slot2-ui --test shelf_resume
cargo test -p slot2-ui --test insert
cargo test -p slot2-ui --test shelf_draw
```

Then regenerate the real-GL evidence in one PowerShell process and always remove the process-local variable:

```powershell
$env:SLOT2_GFX_TEST = '1'
cargo test -p slot2 --test shelf_shot -- --nocapture --test-threads=1
Remove-Item Env:SLOT2_GFX_TEST -ErrorAction SilentlyContinue
```

After the focused checks pass, run the required final gates:

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1
git diff --check
```

- Every command must exit 0.
- `cargo test --workspace` must have zero failed tests and no regression below the last 973-pass baseline;
  report every test-result line count and the summed passed/failed/ignored totals.
- `build/dist-device.ps1` must end with `==> done`.
- Confirm the 17 Task117 PNG paths were freshly rewritten by the GL command. Report their dimensions and
  byte lengths, but do not treat those structural facts as visual acceptance.
- Specifically report the fresh `shelf-720x720.png` and `insert-720x720-{00,05,08,10}.png` paths for Codex
  visual review.
- If source or tests change after a command, rerun every affected command. Do not repeat already successful
  unaffected checks.

## Worker report

Write `C:\SLOT2\tasks\118-square-panel-shelf-hint-visibility.worker-result.md` with:

- `SUCCESS` or `FAILURE` and cumulative attempt count;
- concise root cause and final stable/transition behavior;
- exact production and test changes, including why placement is geometry-derived rather than a special
  case;
- regression matrix results for geometry, platform, language, hint variant, seat samples, and both motions;
- every verification command, exit code, elapsed time, focused results, workspace summed totals, clippy
  warning count, and the final device-build line;
- fresh PNG count plus dimensions/bytes for the five 720x720 shelf/insert evidence files;
- final `git status --short --branch`, changed-file list, and confirmation that only allowed paths changed;
- explicit statement that Codex/user visual review, uncovered menu/dialog host evidence, and all physical
  device acceptance remain pending;
- confirmation of no delegation, commit, stage, push, tag, publish, network, hardware/card access, shared
  configuration change, software installation, or ROM inspection;
- any contract concern and elapsed time.

Stop after writing the report. Do not create Task119 or continue into menu screenshots, device testing, or
release work.
