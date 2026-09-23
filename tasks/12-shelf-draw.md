# Task 12 — drawing the shelf

Third piece of M3. One new module, `crates/slot2-ui/src/shelf_view.rs`, which turns task 11's
placements into canvas calls.

`crates/slot2-ui/tests/shelf_draw.rs` is the contract. **Do not edit it.** Iterate with
`cargo test -p slot2-ui --test shelf_draw`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`, then
`cargo fmt --all`.

Change no existing file except `crates/slot2-ui/src/lib.rs`, to declare and re-export the
module. Everything this needs already exists:

- `skin::skin(platform)` — artwork, its natural size, the label rect, the shell colour
- `art::ArtCache::mask(canvas, src, w, h)` — a texture for a drawing at a size
- `shelf::Shelf::placements(safe, cart)` — where each cart goes, selection last
- `face::draw_text(canvas, ctx, text, px, x, y, colour)` — text; read `face.rs` first
- `svg::rasterize` — already used by the cache; you should not need it directly

---

## The module

```rust
#[derive(Default)]
pub struct ShelfView {
    pub shelf: crate::shelf::Shelf,
    cache: crate::art::ArtCache,
}

impl ShelfView {
    /// Draw the row and the slot it sits above.
    ///
    /// `titles` is one per cart, in the same order as the row. The caller owns the
    /// selection and the scroll; this only draws what they say.
    pub fn draw(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        safe: &SafeArea,
        platform: slot2_store::Platform,
        titles: &[&str],
    );
}
```

`Shelf` needs a `Default` for this to derive one; add `impl Default for Shelf` returning
`Shelf::new(0)` in `shelf.rs` if it is missing. That is the one exception to "change no
existing file".

## What to draw, in order

1. **The slot.** `skin.port` at the bottom of the panel, centred, scaled so its width is
   about a third of the panel. It is drawn first, and it is drawn even when the row is
   empty — an empty shelf is not a blank screen; the slot is what says something goes here.
2. **The carts**, in the order `placements` returns them, so the one nearest the middle
   lands on top. For each:
   - the shell: `skin.cart` rasterised at the placement's size, tinted `skin.shell.colour`
   - the moulding: `skin.cart_detail` at the same size and position, if it is not empty,
     tinted a little darker than the shell so the ribs read
   - the label plate and the title, inside `skin.label` mapped from artwork units into the
     placement's rectangle
3. **The title of the selection** only. A side cart is small and dim and its title would be
   illegible; print the one the player is on.

Map the label rect with the placement's own scale: `x + label.x * (w / cart_size.0)`, and the
same for y and the size. Do not assume the cart is at its natural size — it usually is not.

Text: measure it (`face::measure`) and shrink or clip rather than overflowing the plate.
DESIGN §7 is explicit that text width is always measured and there are no fixed-width slots.

`Finish::Translucent` — a clear shell. Draw it lighter toward the rim if you can do it
cheaply; if you cannot, draw it solid and leave a comment saying the finish is not honoured
yet. Do not invent an expensive effect to satisfy a field.

## The thing that will bite

**Nothing may rasterise or upload on a redraw.** A shelf is drawn sixty times a second, and
the contract fails the build if a second draw uploads anything. Go through `ArtCache` for
artwork, and for text go through whatever `face.rs` already does about caching — read it
rather than assuming. If `face::draw_text` uploads every call, use the `FaceCache` that
module has and say so in a comment.

## House style

Comments explain *why*, never *what*. No `unwrap()` on anything that depends on a file or on
a caller's arithmetic. No `#[allow(...)]` to silence a lint — fix the cause. Read a
neighbouring module before writing: `game_list.rs` is the closest in shape.
