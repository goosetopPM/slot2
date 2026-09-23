# Task 11 — the shelf row and the artwork cache

Second piece of M3. Two new modules in `crates/slot2-ui`: `src/shelf.rs` and `src/art.rs`.

`crates/slot2-ui/tests/shelf.rs` is the contract. **Do not edit it.** Iterate with
`cargo test -p slot2-ui --test shelf`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`.

Do not change any existing file except `crates/slot2-ui/src/lib.rs`, to declare and
re-export the two modules. Task 10's `skin` and `svg` modules are already there and are what
this builds on.

---

## Where the numbers come from

This is a **port**, not a design. The original frontend's carousel is in
`C:\Users\gyuha\slot-2\crates\slot-ui\src\shelf.rs`, and its comments record decisions that
were argued out against a real device. Read that file before writing this one. Carry the
reasoning across in your own comments — a constant without its reason is a constant the next
person deletes.

The three that matter most, all of which the contract checks:

- **Carts share a centre line, not a floor.** A Game Boy pak is 253 units tall and a GBA
  cart 135. Measured from a shared floor the pak sat 59px higher and crowded the top of the
  screen. `rest_y(h) = (panel_h - h) / 2`.
- **A neighbour keeps its foot on the selection's foot line.** A side cart is scaled down,
  and it shrinks *upward* from where the selection stands rather than about its own middle,
  or the row reads as carts floating rather than objects on a shelf.
- **A row of two repeats; a row of one does not.** With two carts, one is drawn twice so
  every slot is filled and the row can scroll. With one, it stands alone — the selection
  never changes, so nothing would move, and three identical faces standing still read as a
  fault. This is odd and it is deliberate; say so in the comment.

## `src/shelf.rs`

```rust
/// How far apart carts sit along the row, in artwork units.
pub const PITCH: f32 = 240.0;
/// How much smaller a cart beside the selection is drawn.
pub const SIDE_SCALE: f32 = 0.78;
pub const SIDE_ALPHA: f32 = 0.55;
/// Slots considered either side of the selection.
pub const SLOTS: i32 = 3;

/// Where one cartridge sits on the row this frame, in panel pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// Which cart in the caller's list this is.
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// 1.0 for the selection, `SIDE_ALPHA` for its neighbours.
    pub alpha: f32,
}

pub struct Shelf { /* len, index, scroll, vel */ }

impl Shelf {
    pub fn new(len: usize) -> Shelf;
    /// After a rescan. Clamps the selection onto the new row.
    pub fn set_len(&mut self, len: usize);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn selected(&self) -> usize;
    pub fn select(&mut self, i: usize);
    pub fn left(&mut self);
    pub fn right(&mut self);
    /// Where the row is sliding to, in pitches. Takes the short way round the ring.
    pub fn scroll_target(&self) -> f32;
    pub fn scroll(&self) -> f32;
    /// Advance the spring. `dt` in seconds.
    pub fn update(&mut self, dt: f32);
    /// True while the row is still moving.
    pub fn settling(&self) -> bool;
    /// The cart `off` slots right of the selection, or `None` when that slot is empty.
    pub fn cart_at_offset(&self, off: i32) -> Option<usize>;
    /// Every cart to draw this frame, **selection last** so it lands on top. Anything
    /// entirely off the panel is dropped.
    pub fn placements(&self, safe: &SafeArea, cart: (f32, f32)) -> Vec<Placement>;
}
```

`placements` lays out against `safe.panel_w` / `safe.panel_h`, not the 640×480 safe area:
DESIGN §5 gives the extension either side of the safe area to the background and the shelf,
so a 720-wide panel shows more of the neighbours rather than moving the selection off centre.
Text and HUD stay in the safe area; that is somebody else's problem.

The spring: `accel = -2 * OMEGA * vel - OMEGA^2 * (scroll - target)` with `OMEGA = 16.0`,
integrated with `dt`. Critically damped, so a flick lands on a cart instead of bouncing past
and returning — the contract checks the overshoot.

`scroll_target` takes the short way round a ring:
`from + (index - from + n/2).rem_euclid(n) - n/2`, where `from` is the scroll position
rounded to the nearest whole pitch at the time of the move. Wrapping from the first cart to
the last must slide one pitch, not thirty.

## `src/art.rs`

```rust
/// Rasterised artwork, kept as long as it is being drawn at that size.
#[derive(Default)]
pub struct ArtCache { /* ... */ }

impl ArtCache {
    /// The texture for this drawing at this size, rasterising and uploading on first ask.
    /// `None` when the source will not parse — remembered, so a broken file is parsed once
    /// rather than every frame.
    pub fn mask(&mut self, canvas: &mut dyn Canvas, src: &str, w: u32, h: u32) -> Option<TexId>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    /// Whether a source is known not to parse at this size. For the contract; it is also
    /// the honest way to ask "did we already give up on this".
    pub fn remembers_failure(&self, src: &str, w: u32, h: u32) -> bool;
    /// Drop every texture. Call before the canvas goes away.
    pub fn clear(&mut self, canvas: &mut dyn Canvas);
}
```

A cart is redrawn sixty times a second; rasterising an SVG or uploading a texture per frame
is not an option. Key on the source **and** the size — the art is vector, so a different size
is a different drawing rather than a scaled one.

Keying on a `&str`'s contents means hashing the whole SVG on every lookup, sixty times a
second. Key on the pointer and length instead (`src.as_ptr() as usize, src.len()`), which is
stable for the `include_str!` constants every caller passes; the size is part of the key
either way. Say in a comment that this is why, and that a caller passing a freshly built
`String` each frame would defeat it.

Upload with `Canvas::upload_alpha8` — `svg::rasterize` already returns coverage.

## What is *not* in this task

No drawing. `placements` says where the carts go and `ArtCache` hands out their textures;
what actually emits the canvas calls, the slot under the row, the labels and the platform
switch are the next task.

## House style

Comments explain *why*, never *what*. No `unwrap()` on anything that depends on a file's
contents or on a caller's arithmetic. No `#[allow(...)]` to silence a lint — fix the cause.
`cargo fmt --all` before you finish.
