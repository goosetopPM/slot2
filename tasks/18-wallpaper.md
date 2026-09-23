# Task 18 — what the shelf stands on

Ninth piece of M3, and it fixes a bug rather than adding decoration.

**Nothing paints the whole panel.** The slot covers its band, the carts cover themselves, and
neither `host_app` nor `device_app` clears — so every pixel the shelf does not draw is
whatever the last frame left there. On a double-buffered device that is two frames of history
alternating behind a row that moves. It has been invisible so far only because the screenshot
test clears before it draws and a still shelf smears into itself.

Two contracts, both **do not edit** — not their assertions, not their formatting, not their
lint attributes:

- `crates/slot2-ui/tests/wallpaper.rs`
- `crates/slot2/tests/wallpaper_app.rs`

Iterate with `cargo test -p slot2-ui --test wallpaper` and
`cargo test -p slot2 --test wallpaper_app`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings` exactly
as written, with no extra `-A` flags, then `cargo fmt --all`. The whole suite must stay
green: `cargo test --workspace --features slot2-input/host`.

Files you may change — and nothing else:

- `crates/slot2-gfx/src/fit.rs` — `cover_uv`, `todo!()`.
- `crates/slot2-ui/src/image.rs` — a new shared decoder, `todo!()`.
- `crates/slot2-ui/src/label.rs` — move its PNG decoding into `image.rs` and call that.
- `crates/slot2-ui/src/wallpaper.rs` — skeleton, `todo!()` bodies.
- `crates/slot2-store/src/card.rs` — `wallpaper`, `todo!()`.
- `crates/slot2/src/app.rs` — the wiring.

---

## 1. `cover_uv`

The panel comes in three shapes — 4:3, 3:2 and square — so no one picture matches all of
them and something has to give. The choices are the aspect (which stretches faces), the
coverage (which leaves bars) or the edges. **The edges of a background are the part nobody
composed**, so the picture is cropped to cover and centred on what is left.

The crop goes in the **uv**, not in the quad: the quad is always the whole panel, and the uv
picks the part of the texture that has the panel's aspect. The contract checks that what is
shown has the destination's aspect on every combination it tries, which is what makes it a
crop rather than a stretch.

## 2. `image.rs`

`label.rs` already decodes a PNG and box-filters it down by an integer factor. The wallpaper
wants exactly that and differs only in how big "too big" is, so the code moves here and both
call it. `label.rs` passes `plate * MAX_OVERSAMPLE`; the wallpaper passes the panel.

Keep the reasoning that is already in `label.rs` — the integer factor, and the guard against
reducing a side to nothing. A texture zero pixels tall is a GL error at boot, and a 2048x3
picture is not one anyone meant to make but it is one a card can hold.

## 3. `wallpaper.rs`

**The built-in ground.** Most cards will have no picture, so the fallback is the normal case.
A vertical gradient, darker at the foot than at the head: the slot is at the bottom and the
carts stand above it, so a face that darkens towards the machine reads as a surface going
back, and the row reads as standing on something instead of floating on a colour.

There are no gradient shaders here and there will not be for this. Build one column of
texels, `panel_h` tall, and stretch it across the panel — the stretch is along the axis where
every texel in a row is the same colour, so nearest filtering costs nothing. Rebuild it only
when the panel height changes, which is never on a device.

**The card's picture**, when there is one: uploaded once, drawn with `cover_uv`, kept while
the shelf that wants it is showing. `set_source` with the path it already has must not throw
the texture away — L1 and R1 walk the platforms, and a player holding R1 down would otherwise
decode a picture per shelf per press.

A picture that will not decode falls back to the ground, and is opened once rather than once
a frame. **A background is the one thing that cannot degrade to nothing**: whatever happens,
`draw` covers the panel.

## 4. `Card::wallpaper`

`Wallpapers/<PLAT>.png`, else `Wallpapers/default.png`, else `None`. Per platform first,
because L1 and R1 move between shelves and "a GB shelf looks like a GB shelf" is M3's
acceptance criterion; a card that wants one picture for everything names it `default`.

## 5. `app.rs`

`rescan` tells the wallpaper what this shelf's picture is, beside what it already does for
labels. `draw` paints it **first**, before anything on `List`, `Inserting`, `Ejecting`, and
before the shelf under a power menu. Painted after the carts it would be a wall in front of
the shelf.

Not on `Playing`: a running game clears to black and the surround of a letterboxed picture is
black, not a shelf's background showing around the edges of a game.

## The thing that will bite

Every task since 11 has passed its contract with a fault in a state the tests only saw at
rest or only saw indirectly. The candidates here: `set_source` called with the path it
already holds, which happens on every rescan and every platform walk that comes back round;
and the gradient being rebuilt because the panel height is compared as a float.

## House style

Comments explain *why*, never *what*. No `unwrap()` on a caller's arithmetic. No
`#[allow(...)]` to silence a lint — fix the cause.
