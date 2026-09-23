# Task 10 — the platform skin table and the SVG rasteriser

First piece of M3. Two new modules in `crates/slot2-ui`, and nothing else changes.

`crates/slot2-ui/tests/skin.rs` is the contract. **Do not edit it.** Run it with
`cargo test -p slot2-ui --test skin` until it passes, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`.

`resvg` and `usvg` are already dependencies of the crate and are known to cross-compile for
aarch64. The artwork is already in `assets/skins/` — read `assets/skins/PROVENANCE.md` first;
it explains why every drawing is a white silhouette.

---

## `src/svg.rs`

```rust
/// A rasterised drawing, as coverage. One byte per pixel, row major, top row first.
pub struct Mask {
    pub w: u32,
    pub h: u32,
    pub a: Vec<u8>,
}

/// Draw `src` at exactly `w` by `h`. `None` for source that will not parse, or a zero size.
pub fn rasterize(src: &str, w: u32, h: u32) -> Option<Mask>;

/// Draw `src` as large as fits inside `box_w` by `box_h` while keeping `natural`'s aspect.
/// `natural` is the artwork's own size, which the caller has from the skin table.
pub fn rasterize_fit(src: &str, natural: (f32, f32), box_w: u32, box_h: u32) -> Option<Mask>;
```

Coverage, not colour. The drawings are white on transparency, so the alpha channel of the
rendered pixmap *is* the mask — take it and throw the colour away. The caller uploads it with
`Canvas::upload_alpha8` and draws it tinted, which is how one drawing serves a grey Game Pak,
a black one and a clear one.

Use `usvg::Tree::from_str` with `usvg::Options::default()`, a `tiny_skia::Pixmap` of the
target size, and `resvg::render` with a `tiny_skia::Transform` scaling the tree's own size to
the target. Do not load a font database: no drawing here has text in it, and pulling in the
system font stack would cost a second of start-up on the device for nothing.

Return `None` rather than panicking or substituting a blank, for: source that does not parse,
`w` or `h` of zero, a `natural` with a zero side, and a pixmap that fails to allocate.

## `src/skin.rs`

```rust
/// A rectangle in artwork units, which are fractional — `sticker.svg` is 205.762 wide.
/// `slot2_gfx::Rect` is whole screen pixels and is the wrong tool here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// How a cartridge shell is finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    Solid,
    /// Clear plastic: the colour lightens toward the rim where light catches the edge.
    Translucent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shell {
    pub colour: [u8; 3],
    pub finish: Finish,
}

/// Everything the shelf needs to draw one platform's cartridge and the slot it goes into.
pub struct PlatformSkin {
    pub platform: slot2_store::Platform,
    /// The shell outline. SVG source, white on transparency.
    pub cart: &'static str,
    /// Moulded ribs and recesses, drawn over the shell on the same grid. May be empty.
    pub cart_detail: &'static str,
    /// The artwork's own size, from its viewBox.
    pub cart_size: (f32, f32),
    /// Where a label sits on the shell, in artwork units.
    pub label: Rect,
    /// What colour to draw the shell when nothing is known about the game.
    pub shell: Shell,
    pub port: &'static str,
    pub port_size: (f32, f32),
    /// True when this platform has no drawing of its own and is using another's.
    pub borrowed: bool,
}

pub fn skin(platform: slot2_store::Platform) -> &'static PlatformSkin;
```

Include the SVG source with `include_str!("../../../assets/skins/<name>.svg")`.

The table:

| platform | cart | detail | shell colour | borrowed |
|---|---|---|---|---|
| Gb | `gb_cart.svg` | `gb_cart_detail.svg` | `0x9a 0x97 0x8f` solid — the grey Game Pak | no |
| Gbc | `gbc_cart.svg` | `gbc_cart_detail.svg` | `0x7c 0x7a 0x8a` translucent — smoke clear | no |
| Gba | `cart.svg` | `cart_detail.svg` | `0x35 0x35 0x3a` solid | no |
| Nes, Snes, Md, Sms | the GBA's | the GBA's | the GBA's | **yes** |

Every platform's port is `socket.svg`.

Sizes come from each file's `viewBox` — the test reads them back out of the source and will
catch a typo. Do not guess them; open the files.

The label rect: a sticker sits centred on the shell's face, above the connector. Take
`sticker.svg`'s own proportions (205.762 × 71.116) as the shape, place it centred
horizontally, and leave the lower fifth of the shell clear for the connector edge. It must
land inside the cart and cover well under nine tenths of it; the test checks both. Judgement
is yours within that — it will be tuned against a screenshot later.

## What is *not* in this task

No drawing, no shelf, no carousel, no caching, no texture upload. Those are the next task.
This one ends when a caller can ask for a platform's artwork and get coverage bytes back.

## House style

Read a neighbouring module first — `crates/slot2-ui/src/face.rs` is the closest in shape.
Comments explain *why*, never *what*; a comment restating the line below it is worse than no
comment. No `unwrap()` on anything that depends on a file's contents. No `#[allow(...)]` to
silence a lint — fix the cause.
