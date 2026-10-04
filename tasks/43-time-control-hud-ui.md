# Task 43 - Build the rewind and fast-forward HUD badge UI

Work directly in the current checkout. Build the small visual badge that tells the player when
time control is active. This task is the `slot2-ui` component only; Task 44 will connect it to
the App's already-working L2/R2 state.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\43-time-control-hud-ui.md
- C:\SLOT2\crates\slot2-ui\src\hud.rs
- C:\SLOT2\crates\slot2-ui\tests\hud.rs
- C:\SLOT2\crates\slot2-ui\src\lib.rs, only HUD exports and shared text-size constants
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs, only message-definition tests

Read a directly related face/span helper only when needed. Do not read App, Session, input,
handoff, milestone history, old task reports, logs, or repository history. The complete UI
contract follows.

## Existing behavior

- `Hud` owns the existing shelf clock and battery drawing. They use a dark plate and light ink,
  sit at the panel's top corners, and cache unchanged text through the shared face cache.
- DESIGN 167 says HUD furniture belongs to the physical panel rather than the 640x480 layout.
  On a 720x720 panel, it stays at the panel edge instead of following the safe area down.
- Task 42 implemented the App behavior: L2 hold rewinds, R2 hold fast-forwards at 4x, and an R2
  double tap toggles a session-scoped 4x latch. No visual status exists yet.
- All user-visible strings must come from Fluent. English and Korean are embedded packs.

## Contract

1. Add the smallest public value type in `slot2-ui::hud` that represents the two things the
   badge can say: `Rewind` and `FastForward { speed: u32 }`. Derive Copy/Debug/Eq as appropriate.
   Do not expose App/input types or create a generic status framework.
2. Add a focused `Hud` method that draws an optional time-control badge. `None` draws nothing;
   Rewind and FastForward draw exactly one badge. Keep the existing clock/battery `draw` API and
   output unchanged so all existing callers and tests remain valid.
3. Place the badge at the physical panel's top centre, using panel coordinates: same top margin
   and band height as the corner HUD, horizontally centred in `safe.panel_w`. Do not use
   `safe.x`/`safe.y`; on the square panel the badge belongs in the extended top band.
4. Draw one compact dark plate behind localized light text. Measure the localized result before
   positioning it; keep horizontal padding and vertical centring consistent with the existing
   HUD. Every plate and text mark must stay inside all three panels.
5. Add direct English and Korean messages. Use concise natural labels:
   - rewind: `Rewind` / `되감기`
   - fast-forward: `{ $speed }x` semantics, rendered naturally as `4×` in English and `4배속`
     in Korean for speed 4. The numeric value must come from the enum, not be hard-coded in the
     drawing function.
6. Reuse the HUD's existing plate/ink visual language and shared face cache. Do not add an image,
   SVG, animation, timer, texture owner, filesystem access, or per-frame allocation beyond the
   existing localized/face lookup pattern.
7. Drawing the same badge repeatedly after a warm frame must upload no new glyph texture.
   Switching Rewind ↔ FastForward may upload the newly needed face once, then must also settle.
8. The method draws only the badge: no `clear`, origin change, clock, battery, toast, game frame,
   or other HUD element. App decides draw order and whether Playing/menu state permits it.
9. Update `hud.rs` module documentation so it describes corner furniture and the top-centre
   time-control badge without changing DESIGN 167's panel-coordinate rule.

Do not modify or add App wiring in this task. A standalone UI component with tests is the required
finished result, matching the established UI-then-App split used by Tasks 33/34 and 36/37.

## Tests

Extend the focused HUD tests and i18n tests to prove at least:

- `None` draws no operations;
- Rewind and FastForward each draw one plate plus the correct localized text and do not clear;
- FastForward uses the supplied speed value (test a value other than 4 as well as 4);
- for `rgsp`, `rg35xxsp`, and `rgcubexx`, in both English and Korean, the badge is horizontally
  centred on the physical panel, begins at `HUD_MARGIN`, fits within panel bounds, and on the
  square panel does not follow `safe.y` down;
- the existing clock/battery draw output and corner anchoring remain unchanged;
- repeated identical draws after warm-up add zero `UploadAlpha8`/`UploadRgba8` operations;
- English and Korean directly define both new keys, including exact speed-4 output, rather than
  receiving fallback text.

Use `RecordingCanvas` geometry and operation assertions. Avoid brittle texture ids or golden
screenshots. Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2-ui\src\hud.rs
- C:\SLOT2\crates\slot2-ui\src\lib.rs only if a re-export is genuinely needed
- C:\SLOT2\crates\slot2-ui\tests\hud.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs
- C:\SLOT2\tasks\43-time-control-hud-ui.worker-result.md

Do not modify `slot2`, Session, `slot2-input`, `slot2-store`, gfx Canvas APIs, runtime loops,
manifests, or documentation. Preserve unrelated uncommitted changes from earlier tasks. Do not
clean, revert, or reformat unrelated files.

## Out of scope and forbidden

- No App wiring, active-state inference, fast-forward latch changes, speed changes, rewind logic,
  input changes, toast, sound, setting, or persistent preference.
- No generic badge/status/notification framework and no new rendering primitive.
- No workspace-wide test, device distribution build, Raspberry Pi test, or RG SP access.
- No adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC/BAI/OpenCodex/Codex configuration changes.
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

Write `C:\SLOT2\tasks\43-time-control-hud-ui.worker-result.md` with at most about 30 lines:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Changed files and final enum/API, placement, localization, and caching behavior.
- Each validation command, exit code, and concise test result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste source code or logs. A path/option error or tool timeout counts as an invocation.
Stop at forty-five minutes and write a partial report; do not make a third invocation. Do not
change models or configuration.
