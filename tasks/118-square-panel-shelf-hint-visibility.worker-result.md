# Task 118 - Square-panel shelf hint visibility (worker result)

## Verdict

**SUCCESS** — cumulative attempt count: **1/2**.

The shelf action hint is now placed from the row's own geometry instead of a fixed line above the slot, so
the tall Game Boy cartridge on the square panel can no longer cover it; and the hint is drawn only on the
frame the action is still pending, disappearing for the whole of an insert or an eject. Every prescribed
local check and all four final gates passed.

## Root cause and final behavior

**Cause.** `ShelfView::draw_hint` pinned the line to `min(panel_h - MOUTH_H, safe bottom) - HINT_PAD - line_h`
and drew it before the carts. The row's carts stand on a line measured from the slot
(`centre = panel_h - MOUTH_H - CENTRE_ABOVE_SLOT`, feet at `centre + cart_h/2`). On a 720x720 panel the
selected 240x253 GB cartridge reaches from y 353.5 down to y 606.5 — below the safe area's own foot at 600 —
so the line at y ~574..592 sat inside the cartridge's body. On the 640x480 and 720x480 panels the same line
lands at y ~396..414 while the tallest cartridge's foot is at ~375, so nothing covered it there.

**Stable shelf, final behavior.** The line is placed at the existing position unless a cart that is
horizontally across it would cover it; in that case it moves to just above the highest such cart. All the
row's carts stand on the same foot line, so clearing the highest one that the line crosses clears the row.
On the square panel the GB hint now sits above the row (band bottom = cart top − `HINT_PAD`), inside the safe
area; on every 480-tall panel and on every short-cart shelf the line stays exactly where it was. The
horizontal centering, the message keys, the font, the padding and the empty-shelf behaviour are unchanged.

**Transition, final behavior.** `draw_insert` draws the hint only when `seat == 0.0`. At `seat == 0.0` the
parted row is identical to the stable row, so the first frame is pixel-for-pixel the shelf frame — no jump.
For any `seat > 0.0`, in both directions, no hint is drawn at all, so the travelling cart can neither cover
it nor flash it back as it passes below the old position, and the line returns only when the stable shelf
frame is drawn again. Insertion/ejection timing, travel curves, cart and row placement, the
cart/face/port draw order, audio timing, App state transitions and core launch are untouched.

## Production change (`crates/slot2-ui/src/shelf_view.rs`, +51/−24)

- `draw_hint` takes the frame's `placements: &[crate::shelf::Placement]` and computes the mark widths first,
  so it knows the line's own x-span. It then moves the line only if some placement overlaps it in x and its
  rectangle crosses the line's band, taking the highest such cart's top as the new anchor:
  `y = covered - HINT_PAD - line_h`. Nothing in it names a platform or a panel size.
- `draw` and `draw_insert` compute the row's placements before drawing anything (a pure computation — the
  canvas call order is unchanged: bay, hint, carts, travelling cart, face, trim) and pass them to
  `draw_hint`.
- `draw_insert` guards the hint with `if seat == 0.0`.

**Why geometry-derived rather than a special case.** The quantity that decides placement is the drawn row's
own rectangles, read from the same `Shelf::parted`/`placements` result the carts are drawn from. A
`W720H720 && Gb` branch would have been wrong twice over: the same occlusion exists for the NES (270) and SMS
(230) carts on the square panel, and it does not exist for a 135-tall GBA cart there. The rule asks "does a
cart stand across this line", which is true of every tall cart on a tall-panel layout and false of every
short one — the same code therefore leaves all five other shelves and both 480-tall panels untouched.

## Test changes

`crates/slot2-ui/tests/shelf_resume.rs` (+153):
- `hint_rects`/`hint_band`/`overlaps` helpers: the hint is identified by its established ink (`INK_DIM`) and
  cap tint (`INK_DIM` at 0.25), and the chosen cart from `Shelf::placements`/`skin::skin(...).cart_size`.
- `the_stable_hint_is_readable_on_every_panel_platform_and_language` — 3 panels x 7 platforms x 2 languages
  x 2 hint variants = **84 cases**: the hint exists, lies inside the safe area, does not intersect the
  selected cart's rectangle, and does not intersect the machine face/port band.
- `a_tall_cart_puts_the_hint_above_the_row` — the square panel with GB: the line's band ends at or above the
  tall cart's top.
- `an_empty_shelf_draws_no_hint_on_any_panel` — 3 panels x 2 languages x 7 platforms = 42 cases: no hint.
- One view is reused for the whole matrix; the artwork cache is keyed by drawing and size, so a fresh view
  per case would re-rasterise every cart and port without asking a new question (this took the suite from
  275 s to 20 s).

`crates/slot2-ui/tests/insert.rs` (+113):
- `ctx_lang` plus the same `hint_rects` helper.
- `the_resting_insert_frame_keeps_the_hint_where_the_shelf_had_it` — 3 geometries x 7 platforms x 2
  languages = **42 cases**: stable `draw` and `draw_insert(seat 0.0)` produce the same hint marks at the same
  positions, alongside the existing cart-image equality.
- `a_cart_in_travel_draws_no_hint_on_any_geometry` — 3 geometries x 7 platforms x 2 motions x 4 seats
  (`1/44` ≈ 0.023, `0.5`, `0.8`, `1.0`) = **168 cases**, including the 720x720 GB case that exposed the bug:
  neither insert nor eject draws any hint mark.

No existing assertion was weakened, replaced with a snapshot hash, or removed. The port/cart occlusion,
seated-cart visibility, upload-cache, title and resume-hint tests are unchanged. `shelf_shot.rs` was not
touched, and no public production hook was added for tests.

## Verification commands

| # | Command | Exit | Elapsed | Result |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | 5 s | clean |
| 2 | `cargo test -p slot2-ui --test shelf_resume` | 0 | 28 s | `6 passed; 0 failed; 0 ignored` (was 3) |
| 3 | `cargo test -p slot2-ui --test insert` | 0 | 25 s | `28 passed; 0 failed; 0 ignored` (was 26) |
| 4 | `cargo test -p slot2-ui --test shelf_draw` | 0 | 5 s | `13 passed; 0 failed; 0 ignored` |
| 5 | `$env:SLOT2_GFX_TEST='1'; cargo test -p slot2 --test shelf_shot -- --nocapture --test-threads=1; Remove-Item Env:SLOT2_GFX_TEST` | 0 | 4.9 s | `1 passed; 0 failed`; wrote all three shelves and twelve inserts; env var confirmed removed |
| 6 | `cargo test --workspace` | 0 | 1172 s | **100 `test result:` lines → 978 passed, 0 failed, 0 ignored** |
| 7 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 63 s | `Finished dev profile`; **0 warnings** |
| 8 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1` | 0 | 654 s | final line **`==> done`** |
| 9 | `git diff --check` | 0 | <1 s | clean |

Gate 6 is 978 passed against the 973-pass Task 109 baseline: **+5**, exactly the five new regression tests,
with no regression and no ignored test. (An earlier `cargo fmt` run caught one over-long line in the new
shelf_resume test; it was fixed and every affected command re-run.)

## Regenerated real-GL evidence

GL window `2026-10-05T05:25:36Z` to `05:25:41Z` (local 14:25:36 to 14:25:41). All 17 Task 117 PNG paths
exist and still have their exact expected dimensions; **15 were freshly rewritten by this run** — the three
shelf captures and the twelve insert captures, which are the only files `shelf_shot` writes. The remaining
two, `gfx-screenshot.png` (10854 bytes, 720x480) and `splash-screenshot.png` (48123 bytes, 720x480), are
produced by the other two real-GL test binaries; this change does not touch GL primitives, presentation,
shaders or the splash, so those unaffected commands were not re-run (see contract concerns).

The five 720x720 files for Codex visual review:

| Path (repo-relative) | Dimensions | Bytes | mtime (UTC) |
| --- | --- | --- | --- |
| `target/shelf-720x720.png` | 720x720 | 57571 | 2026-10-05T05:25:40 |
| `target/insert-720x720-00.png` | 720x720 | 56141 | 2026-10-05T05:25:40 |
| `target/insert-720x720-05.png` | 720x720 | 44962 | 2026-10-05T05:25:41 |
| `target/insert-720x720-08.png` | 720x720 | 34267 | 2026-10-05T05:25:41 |
| `target/insert-720x720-10.png` | 720x720 | 19153 | 2026-10-05T05:25:41 |

Against Task 117's files this is the fix showing up in bytes: `shelf-720x720.png` grew 56167 → 57571 (the
missing line is now drawn) and `insert-720x720-10.png` shrank 20641 → 19153 (the line is no longer drawn
during travel). Dimension and byte facts are structural only; they are **not** visual acceptance.

## Repository state

```text
## main...origin/main
 M crates/slot2-ui/src/shelf_view.rs
 M crates/slot2-ui/tests/insert.rs
 M crates/slot2-ui/tests/shelf_resume.rs
 M docs/HANDOFF-CODEX.md
?? tasks/117-host-ui-acceptance-audit.md
?? tasks/117-host-ui-acceptance-audit.result.md
?? tasks/117-host-ui-acceptance-audit.worker-result.md
?? tasks/118-square-panel-shelf-hint-visibility.md
```

Full HEAD `8662c6ec1fe5120cf9508c9fb4ff6df1786c3b6e` on `main`; staged entries 0. Only the three allowed
files changed (`shelf_view.rs` +51/−24, `insert.rs` +113, `shelf_resume.rs` +153). `docs/HANDOFF-CODEX.md`
is the pre-existing handoff entry written by Codex, carried along untouched, and the `tasks/117-*` records
are pre-existing untracked Task 117 files. This report,
`tasks/118-square-panel-shelf-hint-visibility.worker-result.md`, is the only path added by this task beyond
those. The PNGs and the `target/ui-acceptance-task118/` logs are ignored and do not appear in status.

## Still pending

- **Codex and user visual review** of the regenerated 720x720 (and other) stills — whether the relocated line
  reads well, whether its new position is typographically right, and whether the insertion frames look
  intentional. A decodable, correctly sized, non-blank image is not visual acceptance.
- **Uncovered host evidence**: the in-game menu and its child menus, the state switcher, shelf settings and
  dialogs, disabled-row appearance and transient toasts/errors are still `not covered by Task117/Task118
  evidence`.
- **All physical-device acceptance**: handheld input, audio, Mali GPU behavior, frame pacing and performance,
  lid/power, display output, card mounting, and device-specific safe-area appearance.

## Confirmations

No delegation, commit, stage, push, tag, publication, network access, hardware/card access, shared
configuration change, or software installation occurred. No local ROM was inspected (the GL test's cart
titles are invented strings written into a temporary card directory). No file outside the allowed list was
modified or deleted.

## Contract concerns

1. "Confirm the 17 Task117 PNG paths were freshly rewritten by the GL command" cannot be literally satisfied
   by the one prescribed GL command: `cargo test -p slot2 --test shelf_shot` writes 15 of the 17, and the
   other two belong to the `slot2-gfx` and `slot2-ui` real-GL tests. Those two were deliberately not re-run
   as unaffected checks. If a fully fresh 17 is required, the verification section needs all three GL
   commands.
2. The first GL invocation did not record a start timestamp before running, so freshness could not be proved
   against it. The affected GL command (the one whose output this change alters) was run once more with the
   start recorded, and that run is the evidence above. No unaffected check was repeated.

**Elapsed time: about 50 minutes** of wall clock (control file visible at local 14:08, report at 14:58), of
which the recorded verification window was `2026-10-05T05:22:56Z` to `05:57:40Z` and the final gates alone
took 32 min 54 s.
