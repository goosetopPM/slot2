# Task 119 - Host UI visual gallery

## Goal

Add one opt-in real-GL integration test that renders a reproducible 720x480 visual gallery for the major
menus and feedback surfaces not covered by Task117/118. Generate English and Korean PNGs from the real UI
components, plus a local static index, so Codex and the user can inspect typography, clipping, overlap,
selection, disabled rows, and error feedback before physical-device acceptance.

This is evidence infrastructure only. Do not change product behavior or production source. This task is
cumulative maximum 2 attempts; a failed or interrupted invocation counts as an attempt.

## Read only

Read only the minimum needed:

- `C:\SLOT2\AGENTS.md`
- the top current-state section of `C:\SLOT2\docs\HANDOFF-CODEX.md`
- this task in full
- `C:\SLOT2\tasks\118-square-panel-shelf-hint-visibility.result.md`
- `C:\SLOT2\crates\slot2\tests\shelf_shot.rs`, only for its one-window real-GL/read-back pattern
- the public constructors, navigation methods, draw signatures, and directly required data types in these
  `slot2-ui` modules:
  - `in_game_menu`, `cheat_menu`, `display_menu`, `shader_menu`, `overscan_menu`, `overlay_menu`
  - `core_picker`, `device_menu`, `state_switcher`
  - `shelf_menu`, `language_picker`, `timezone_menu`, `about_sticker`, `power_menu`, `toast`
- the top helpers/fixtures of the matching `crates/slot2-ui/tests/*.rs` files only when needed to construct a
  legal representative state
- `slot2-gfx` public host surface, canvas and image write APIs only as needed by the test

Do not read old worker logs or unrelated task history. If a command fails, inspect only the exact error and
directly relevant module/test/manifest lines.

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
stage, push, publish, create or move a tag, use the network, access Raspberry Pi or handheld hardware,
access an SD card or Samba drive, change shared configuration, install software, or inspect local ROMs.

## Implementation

Create exactly one new integration test file:

- `C:\SLOT2\crates\slot2-ui\tests\visual_gallery.rs`

It must follow the existing opt-in GL convention:

- without `SLOT2_GFX_TEST=1`, print a short skip message and pass without creating evidence;
- with `SLOT2_GFX_TEST=1`, open exactly one host window/event loop, render all scenarios sequentially through
  `GlCanvas`, read back actual pixels, and write the gallery;
- do not spawn processes, use the network, load a game/core, or depend on App internals;
- use the built-in fonts and translations through normal `UiCtx` construction;
- use only synthetic menu data, state metadata and thumbnails created inside the ignored output directory;
- do not read or name files under `assets/test/local`, `sdcard`, or any external card root.

The output directory is exactly:

- `C:\SLOT2\target\ui-gallery-task119`

Derive it from `CARGO_MANIFEST_DIR`; do not accept an arbitrary deletion path. The test may recreate only
that exact directory and must not delete or modify any other target output, including Task117/118 PNGs.

Each frame must start from a deterministic dark background. For overlay screens, draw a simple synthetic
colored game-frame pattern first so the dim layer is visible; do not use a ROM screenshot or copyrighted
art. Warm or create each `UiCtx` independently enough that one language/scenario cannot leak texture or
selection state into another.

## Exact gallery matrix

Render every scenario below once in English and once in Korean, all at 720x480 (`rgsp` profile), for exactly
30 PNG files:

1. `ingame-menu-{en,ko}.png` - a middle row selected, full seven-row overlay visible.
2. `cheat-menu-{en,ko}.png` - multiple synthetic cheats, mixed enabled/disabled, a non-first row selected.
3. `display-menu-{en,ko}.png` - representative scale and overscan availability.
4. `shader-menu-{en,ko}.png` - a non-default built-in preset selected.
5. `overscan-menu-{en,ko}.png` - explicit enabled/disabled choice with one selected.
6. `overlay-menu-{en,ko}.png` - explicit enabled/disabled choice with one selected.
7. `core-picker-{en,ko}.png` - GBA with mGBA and gpSP installed, current core and another row distinguishable.
8. `device-menu-disabled-{en,ko}.png` - usable volume plus unavailable brightness and blue-light rows.
9. `state-switcher-{en,ko}.png` - at least three numbered states, at least one synthetic thumbnail, a
   non-first selection, and undo available.
10. `shelf-menu-disabled-{en,ko}.png` - production-like availability: language, time zone and About usable;
    display defaults, boot logo and Sync visibly unavailable and unselectable.
11. `language-picker-{en,ko}.png` - English and Korean options with the non-first option selected.
12. `timezone-menu-{en,ko}.png` - a nonzero representative UTC offset selected.
13. `about-sticker-{en,ko}.png` - deterministic synthetic version/commit/device information, with no claim
    that it came from the current release artifact.
14. `power-menu-{en,ko}.png` - a destructive row selected so selection contrast is visible.
15. `toast-error-{en,ko}.png` - a built-in error toast at full visible alpha over a deterministic base frame.

Use the actual component APIs and normal localized message keys. Do not reimplement a menu in test code,
draw replacement text by hand, alter localization files, or add screenshot-only production hooks.

## Evidence contract

For every PNG:

- dimensions must be exactly 720x480;
- it must be nonempty, begin with a PNG signature, and be decodable by the test's existing PNG path;
- require a meaningful minimum count of pixels different from the deterministic background, so an empty
  draw cannot pass;
- the filename must uniquely identify scenario and locale;
- no snapshot hash is a correctness assertion. Pixel hashes are evidence identity only and must not be
  hard-coded into the test.

After all 30 frames, write `target/ui-gallery-task119/index.html`:

- UTF-8 without external resources, JavaScript, remote URLs, or embedded image data;
- title and short warning that this is host evidence, not device acceptance;
- one section per scenario, with the English and Korean PNG side by side using relative paths;
- simple inline CSS only, readable locally at 720-pixel image width without scaling the source files;
- no repository, user, machine, ROM, token, or absolute-path data.

The test must assert that the final output contains exactly the 30 expected PNG names plus `index.html` and
only any explicitly documented synthetic thumbnail fixture kept in a clearly named subdirectory. Stale PNGs
must not survive from an earlier run.

## Allowed files

- `C:\SLOT2\crates\slot2-ui\tests\visual_gallery.rs`
- `C:\SLOT2\tasks\119-host-ui-visual-gallery.worker-result.md`
- ignored output/logs under `C:\SLOT2\target\ui-gallery-task119`

Do not modify production code, existing tests, assets, manifests, lockfiles, public documentation,
workflows, Task117/118 records, or `docs/HANDOFF-CODEX.md`. Preserve every unrelated current working-tree
change, including the uncommitted Task118 implementation.

## Verification

Run sequentially from `C:\SLOT2`; do not overlap Cargo commands or builds against the same target.

1. `cargo fmt --all -- --check`
2. In one PowerShell process:

```powershell
$env:SLOT2_GFX_TEST = '1'
cargo test -p slot2-ui --test visual_gallery -- --nocapture --test-threads=1
Remove-Item Env:SLOT2_GFX_TEST -ErrorAction SilentlyContinue
```

Always remove the process-local variable, including after failure. Stop after the first failure and do not
edit production code to make the evidence test pass.

After the GL test passes:

3. Validate and report all 30 PNG paths, dimensions, byte lengths, SHA-256 values, fresh timestamps, and
   successful decoding without modifying them.
4. Validate `index.html` is UTF-8, contains all 30 relative image references exactly once, has no remote URL,
   script, data URI, absolute path, token-like value, or ROM filename.
5. `cargo test --workspace`
6. `cargo clippy --workspace --all-targets -- -D warnings`
7. `git diff --check`

Every command must exit 0. The workspace test must have zero failures and no regression below Task118's 978
passed/0 failed/0 ignored; adding this integration test should increase the pass count when the opt-in flag
is absent. Report all test-result line counts and summed totals. Clippy must have zero warnings.

Do not rerun `build/dist-device.ps1`: Task118 already passed it, and this task adds one host-only opt-in test
without changing product/device source, assets, manifests or packaging inputs.

## Worker report

Write `C:\SLOT2\tasks\119-host-ui-visual-gallery.worker-result.md` with:

- `SUCCESS` or `FAILURE` and cumulative attempt count;
- concise harness design and confirmation that one real GL surface rendered actual components;
- the 15-scenario x 2-locale matrix and representative state used for each;
- a 30-row table with relative path, dimensions, bytes, SHA-256 and freshness;
- `index.html` validation results and its relative path;
- every verification command, exit code, elapsed time, real-GL result, workspace summed totals, clippy
  warning count and diff-check result;
- final `git status --short --branch`, changed-file list, and confirmation that only allowed paths changed;
- explicit statement that image existence/decoding is not visual acceptance, and that Codex/user review plus
  all physical-device checks remain pending;
- confirmation of no delegation, commit, stage, push, tag, publish, network, hardware/card access, shared
  configuration change, software installation, or ROM inspection;
- any contract/API concern and elapsed time.

Stop after writing the report. Do not create Task120, fix visual findings, open a browser, access hardware,
or continue into release work.
