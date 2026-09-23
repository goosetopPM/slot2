# Task 13 — the shelf inside the app

Fourth piece of M3. `Screen::List` stops being a text list and becomes the shelf.

`crates/slot2/tests/shelf_app.rs` is the contract. **Do not edit it.** Iterate with
`cargo test -p slot2 --test shelf_app`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`, then
`cargo fmt --all`. The whole suite must stay green: `cargo test --workspace --features slot2-input/host`.

Files you may change: `crates/slot2/src/app.rs`, and `crates/slot2-ui/src/shelf.rs` only if
the clamp below needs it. Nothing else.

---

## What changes in `app.rs`

`App` gains a `slot2_ui::shelf_view::ShelfView` and keeps `GameList` only if something still
needs it — if nothing does, take the field out and leave the module alone.

New accessors the contract calls:

```rust
pub fn shelf_len(&self) -> usize;
pub fn selected(&self) -> usize;
/// True while the row is still sliding.
pub fn shelf_settling(&self) -> bool;
```

Input on `Screen::List`:

| button | was | becomes |
|---|---|---|
| Up / Down | move the list | nothing — a carousel is horizontal |
| Left / Right | nothing | move along the row |
| L1 / R1 | switch platform | unchanged |
| A | start the selected cart | unchanged, but the selection comes from the shelf |

`start_selected` must take its index from the shelf, not from `GameList`.

`rescan` and `switch_platform` must tell the shelf the row changed — `Shelf::set_len` — so a
shelf of seven does not keep showing a row of three. Switching platform also has to leave the
row **not chasing a target from the row that no longer exists**; `set_len` already recomputes
the target, so call it, and check with the contract's mid-slide test rather than assuming.

Drawing `Screen::List` draws the shelf. Titles come from `self.carts`.

## Advancing the row

The row is a spring and needs real time. `App::tick(now: Instant)` already runs every frame;
keep the previous `Instant` and hand the shelf the elapsed seconds.

**Clamp the step.** Loading a core can cost a second, and a frame that long integrated
against `OMEGA = 16` is numerically unstable — the row flies off and never returns. Clamp to
something like 1/30 s: a slow frame should make the animation lag, never explode. Put the
clamp where it is provable, say why in a comment, and let the contract's long-frame test hold
it. If you put it inside `Shelf::update`, note that the shelf tests in
`crates/slot2-ui/tests/shelf.rs` already pass 1/60 and must keep passing.

## The thing that will bite

Last three tasks each passed their contract and still had a real fault, every one of them in
a **state the tests only saw at rest**: a target read while the spring moved, a sort that
meant nothing mid-scroll, a cache key that only repeated while nothing was sliding. This
contract drives the row while it moves on purpose. Before you call it done, ask what is true
of this code only while the row is still.

## House style

Comments explain *why*, never *what*. No `unwrap()` on a caller's arithmetic. No
`#[allow(...)]` to silence a lint — fix the cause.
