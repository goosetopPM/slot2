# Task 45 - Build the per-game display scale menu UI

Work directly in the current checkout. Build the small `slot2-ui` submenu reached from the
in-game menu's Display row. This task is UI and navigation only. Task 46 will connect it to
`App`, apply the selected scale to the running Session, and persist the game override.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\45-display-scale-menu-ui.md
- C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs
- C:\SLOT2\crates\slot2-ui\tests\in_game_menu.rs
- C:\SLOT2\crates\slot2-ui\src\lib.rs, only module declarations and exports
- C:\SLOT2\crates\slot2-store\src\settings.rs, only `ScaleMode` and `GameSettings`
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs, only direct message-definition coverage

Read directly related shared drawing/span helpers only when needed. Do not read App, Session,
handoff or milestone history, old task reports, logs, or repository history. The complete UI
contract follows.

## Existing contract

- DESIGN section 7 says the in-game Display menu changes per-game display settings and reflects
  an override beside the platform default.
- `slot2-store::GameSettings::scale` is already `Option<ScaleMode>`. `None` means platform
  default; the three explicit values are `Integer`, `AspectFit`, and `Fill`.
- Session already understands the three corresponding render policies. This task must not touch
  or infer Session behavior.
- Shader and overlay selection are planned but have no runtime setting contract yet. Do not show
  unavailable controls merely to make the submenu look complete.
- The menu overlays the paused game's last frame and must never clear the canvas.
- All visible geometry must fit the profile's 640x480 safe area. All visible text comes from
  Fluent, with direct English and Korean definitions.

## Contract

1. Add a public `display_menu` module and re-export its main `DisplayMenu` type from `slot2-ui`.
2. Represent exactly four choices in this order: Platform default, Integer, Aspect fit, Fill.
   Use the existing `slot2_store::ScaleMode` for explicit values. The public selected value must
   be `Option<ScaleMode>`, where `None` is Platform default. Do not create another scale enum.
3. Construct the menu from the current `Option<ScaleMode>` and initially select that exact row.
   Expose only a read-only selected-value query and the navigation methods needed by App.
4. Up/down navigation wraps at both ends. No other input handling belongs in this component.
5. Draw as an overlay without `Canvas::clear`: dim the whole physical panel, then draw one
   centred panel inside the safe area with a localized title, the four rows, and the existing
   select/back hints. Highlight exactly one row.
6. Use concise natural messages:
   - title: `Display` / `화면`
   - platform default: `Platform default` / `플랫폼 기본값`
   - integer: `Integer scale` / `정수 배율`
   - aspect fit: `Fit aspect ratio` / `화면 비율 맞춤`
   - fill: `Fill screen` / `화면 채우기`
7. Reuse the in-game/power-menu visual language, text sizes, colors, safe-area rules, and shared
   face cache. Do not add a generic menu framework, image, animation, filesystem access, timer,
   texture owner, or setting writer.
8. Repeated drawing of the same menu after a warm frame must upload no new glyph texture.
9. Update only the minimum module documentation needed to state that this is the scale-only
   Display submenu. Do not claim shader or overlay controls exist.

## Tests

Add focused `slot2-ui` tests proving at least:

- each constructor value (`None` and all three `ScaleMode` values) selects and reports the exact
  corresponding row;
- up/down navigation follows the fixed order and wraps at both ends;
- drawing never clears, dims before the menu panel, and highlights exactly one row;
- for `rgsp`, `rg35xxsp`, and `rgcubexx`, in English and Korean, title, rows, highlight, and hints
  remain inside the 640x480 safe area;
- every choice draws the correct localized label and the current choice is visibly distinct;
- repeated identical draws after warm-up add zero `UploadAlpha8`/`UploadRgba8` operations;
- English and Korean directly define every new key; Korean fallback must not hide a missing key.

Use `RecordingCanvas` operation and geometry assertions. Avoid texture-id assertions and golden
screenshots. Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2-ui\src\display_menu.rs (new)
- C:\SLOT2\crates\slot2-ui\src\lib.rs
- C:\SLOT2\crates\slot2-ui\tests\display_menu.rs (new)
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs only for direct message coverage
- C:\SLOT2\tasks\45-display-scale-menu-ui.worker-result.md

Preserve all unrelated uncommitted changes from earlier tasks. Do not clean, revert, or reformat
unrelated files.

## Out of scope and forbidden

- No App screen variant or Display-row wiring, Session scale application, card read/write, game
  settings mutation, toast, sound, input gesture, shader, overlay, overscan, or rewind setting.
- No change to `slot2-store`, `slot2-gfx`, `slot2`, runtime loops, manifests, or documentation.
- No workspace-wide test, device distribution build, Raspberry Pi test, or RG SP access.
- No adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\45-display-scale-menu-ui.worker-result.md` with at most about 30 lines:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Changed files and the final constructor/query/navigation/drawing behavior.
- Each validation command, exit code, and concise test result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste source code or logs. A path/option error or tool timeout counts as an invocation.
Stop at forty-five minutes and write a partial report; do not make a third invocation. Do not
change models or configuration.
