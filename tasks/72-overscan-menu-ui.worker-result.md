# Task 72 — Overscan menu UI: worker result

- Status: success
- Cumulative calls used: 1 of 2 (this is the first call)

## Values, rows, constructor, query, navigation

- `ROWS: [Option<bool>; 3] = [None, Some(true), Some(false)]`.
- Keys in that order: `display-platform-default`, `overscan-crop`, `overscan-full`
  (`OverscanMenu::key` is an exhaustive match over `Option<bool>`; no bool formatting, no index).
- `OverscanMenu::new(v)` selects the row whose value equals `v` and `selected()` returns that
  exact `Option<bool>`. `None` and `Some(true)` land on different rows (asserted separately).
- `up`/`down` move one row and wrap at both ends; one full lap in either direction returns to the
  start. No input events, filesystem, platform lookup or App state in the type; nothing in the
  module inspects the platform.

## Draw

- Order: full-panel `DIM` (BLACK alpha 0.6) rect → safe-area-centred box (`BACKDROP`, BOX_W=360,
  BOX_H=208) → title `overscan-title` at PX_BODY/INK_DIM centred at box_y + PAD → three rows at
  `box_y + PAD + 28` each ROW_H=36 with labels PX_TITLE/INK centred → `hint-select`/`hint-back`
  at PX_HINT/INK_DIM on the box's bottom hint line. Exactly one row carries the
  `INK alpha 0.15` highlight, and no `Canvas::clear` is issued (overlay only).

## English / Korean wording

- en: `overscan-title` = `Overscan`, `overscan-crop` = `Crop edges`,
  `overscan-full` = `Show full image`.
- ko: `오버스캔`, `가장자리 자르기`, `전체 이미지 표시`.
- Both packs define them directly; the platform-default row reuses `display-platform-default`
  (`Platform default` / `플랫폼 기본값`). Asserted by value in both the new UI test and a new
  `slot2-i18n` test, plus "no two rows share words".

## Safe area and warm redraw

- For `rgsp`, `rg35xxsp`, `rgcubexx` × en/ko: box at `box_origin`, box and every op except the
  deliberate full-panel dim lie inside `SafeArea::contains`; the title sits at box top and the
  hints on `by + BOX_H - PAD - PX_HINT`.
- Each row's own label is drawn on its own row band (measured with `face::spans_width`).
- Warm redraw: first frame uploads; 8 further identical frames and one highlight move (Korean,
  CJK face included) upload 0 textures.

## Acceptance commands (run in order, after the last code change)

| command | exit | last result line |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | (no output) |
| `cargo test -p slot2-ui --test overscan_menu` | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.33s` |
| `cargo test -p slot2-ui -p slot2-i18n` | 0 | last crate: `Doc-tests slot2_ui` — `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`; summed over both crates: 239 passed / 0 failed / 0 ignored (Task 70's same pair was 231; +7 UI, +1 i18n) |
| `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 10.86s` |

The first `cargo fmt --all -- --check` run reported the two new `lib.rs` lines out of order; `cargo
fmt --all` was applied, then the check above passed. No code was edited after the last test or
clippy run. Workspace tests, GL tests and device deployment were not run.

## Files

- New: `crates/slot2-ui/src/overscan_menu.rs`
- New: `crates/slot2-ui/tests/overscan_menu.rs`
- Modified: `crates/slot2-ui/src/lib.rs` (module + re-export lines only)
- Modified: `assets/lang/en.ftl`, `assets/lang/ko.ftl` (three keys + one comment each)
- Modified: `crates/slot2-i18n/tests/i18n.rs` (one new test function only)
- Report: this file.
- Pre-existing uncommitted changes elsewhere in the checkout were left untouched.

## Out of scope, confirmed untouched

- NES-only conditional entry row from Display, App screen variant and input wiring: not touched.
- Saving the choice to the card and applying live crop to the running Session: not touched; the
  component holds stored meanings only and resolves nothing.
- No display/shader menu rows, layout, API, session, store, gfx, retro registry or core option
  changes.

## Contract concern / remaining risk (1 line)

`OverscanMenu::new` keeps `unwrap_or(0)` as the unreachable fallback (same pattern as the display
and shader menus), so a future fourth state would silently show as "Platform default" instead of
failing; today all three states exist and the round-trip test pins the behaviour.

## Time

Start ~11:57, finish ~12:09 (Asia/Seoul) — about 12 minutes.
