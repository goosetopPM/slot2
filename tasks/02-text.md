# Task 02 — implement `slot2-text`

Repository: `C:\SLOT2` (Rust workspace). Work only inside `crates/slot2-text/src/imp.rs`.

## Goal
Make this pass with zero failures and zero warnings:

```
cargo test -p slot2-text
cargo clippy -p slot2-text --all-targets -- -D warnings
```

## Rules
- **Do not edit** `crates/slot2-text/tests/text.rs`, `crates/slot2-text/src/lib.rs`,
  `crates/slot2-text/Cargo.toml`, `assets/fonts/*`, or any other crate. They are the contract.
- Edit only `crates/slot2-text/src/imp.rs`: replace `Inner`'s fields and every `todo!()`.
  You may add private structs/enums/functions in that file. Delete the `_unused` helper.
- Only dependency: `fontdue 0.9` (already listed) + std. No new crates.
- No `unsafe`. No `unwrap()`/`expect()` on file I/O or font parsing; return `Error`.
- Read the module doc at the top of `imp.rs` first: it specifies the data layout, lazy
  loading, line metrics from the first font, tofu handling, glyph placement math, and the
  cache key. Read `lib.rs` for the public contract each method must satisfy.

## fontdue 0.9 hints
- `fontdue::Font::from_bytes(bytes: Vec<u8>, fontdue::FontSettings::default()) -> Result<Font, &'static str>`
- `font.lookup_glyph_index(ch) -> u16` (0 = no glyph)
- `font.metrics(ch, px) -> fontdue::Metrics { xmin, ymin, width, height, advance_width, .. }`
- `font.rasterize(ch, px) -> (fontdue::Metrics, Vec<u8>)` (coverage, `width*height`)
- `font.horizontal_line_metrics(px) -> Option<fontdue::LineMetrics { ascent, descent, line_gap, new_line_size }>`
  (`descent` is negative in fontdue)
- Cache key: quantise `px` as `(px * 100.0).round() as u32`.

## Placement (repeat of the doc, because it is the part that usually goes wrong)
For a glyph with metrics `m` at pen position `pen` (f32) on a bitmap whose baseline row is
`baseline` (u32):
```
x0 = (pen.round() as i32) + m.xmin
y0 = baseline as i32 - m.height as i32 - m.ymin
```
Copy `m.width × m.height` coverage to `(x0, y0)`, clipping anything outside `[0,width)×[0,height)`,
compositing with `dst = max(dst, src)`. Then `pen += m.advance_width`.

## Definition of done
Paste the last lines of both commands' output into your final answer, including the
`font: lazy-loaded Noto Sans KR in N ms` line that one test prints (run with `-- --nocapture`
once to see it). If a test cannot pass without changing a contract file, stop and explain
which one and why instead of editing it.
