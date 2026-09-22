# Task 07 — implement `slot2-store`

Repository: `C:\SLOT2` (Rust workspace, Windows host). Work only inside `crates/slot2-store/src/`.

## Goal
```
cargo test -p slot2-store
cargo clippy -p slot2-store --all-targets -- -D warnings
```
Both with zero failures / warnings.

## Rules
- **Do not edit** (contract): `crates/slot2-store/src/lib.rs`, `platform.rs`, `tests/card.rs`,
  `Cargo.toml`, other crates.
- Edit only, replacing every `todo!()`:
  - `src/atomic.rs` — `atomic_write`
  - `src/ini.rs` — `parse`, `load`, `save`
  - `src/card.rs` — `ensure_layout`, `scan`, `read_save`, `write_save`, `list_states`,
    `next_state_number`, `read_state`, `write_state`, `delete_state`
  You may add private helpers in those files.
- Read each file's module doc first; it is the spec (temp-file-then-rename, Windows rename
  quirk, skip rules for scanning, sort order, state naming, PNG encoding via the `png`
  crate that is already a dependency).
- No new dependencies. No `unsafe`. No `unwrap()`/`expect()` on filesystem results — map
  them to `Error::Io(path, e)`; `ensure_layout` and `scan` swallow errors (best effort) as
  documented.
- Sorting in `scan`: `sort_by(|a, b| a.title.cmp(&b.title).then(a.rom.extension().cmp(&b.rom.extension())))`
  — plain byte/code-point order, NOT case-insensitive (the test pins "Zelda" < "apotris").

## png 0.17 hint
```rust
let mut buf = Vec::new();
{
    let mut enc = png::Encoder::new(&mut buf, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header().map_err(..)?;
    wr.write_image_data(rgba).map_err(..)?;
}
atomic_write(&png_path, &buf)?;
```
Map `png::EncodingError` to `Error::Invalid(e.to_string())`.

## Definition of done
Paste the last lines of both commands. If a contract file must change, stop and explain.
