# Task 14 — the cart goes into the slot

Fifth piece of M3. Pressing A stops being a jump cut and becomes a cartridge being pushed
into the machine.

`crates/slot2-ui/tests/insert.rs` is the contract. **Do not edit it.** Iterate with
`cargo test -p slot2-ui --test insert`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`, then
`cargo fmt --all`. The whole suite must stay green:
`cargo test --workspace --features slot2-input/host`.

Files you may change — and nothing else:

- `crates/slot2-ui/src/insert.rs` — a skeleton with the constants, the doc comments and
  `todo!()` bodies. Fill the bodies in. The signatures are what the contract calls; do not
  change them.
- `crates/slot2-ui/src/shelf.rs` — two new `todo!()` methods, `rest_x` and `parted`.
- `crates/slot2-ui/src/shelf_view.rs` — one new `todo!()` method, `draw_insert`, and the
  chrome split described below.

Nothing in `crates/slot2` yet. Wiring the animation into `App` is task 15.

---

## 1. `insert.rs` — the motion

This is a port, not a design. The original frontend's travel is in
`C:\Users\gyuha\slot-2\crates\slot-ui\src\slot_chrome.rs`; the constants are already
copied into the skeleton with their reasons. What you are writing is the arithmetic.

```
seat_in(t)  = (t / SEATED_AT).clamp(0, 1)
seat_out(t) = 1 - (t / EJECT_S).clamp(0, 1)
ease(u)     = u³(u(6u − 15) + 10)          // smootherstep
```

The geometry, all of it measured off the slot:

```
band_y    = panel_h − MOUTH_H                    // the front of the machine
lip_y     = band_y                               // what the cart's foot catches on
seated_y  = band_y + LIP_H + SEATED_BELOW_LIP    // the cart's TOP edge when seated
row_centre(safe) = panel_h − MOUTH_H − CENTRE_ABOVE_SLOT
rest_y(safe, h)  = row_centre(safe) − h / 2
```

`rest_y` must agree with `Shelf::placements` exactly. `placements` computes
`floor = centre + natural_h / 2` and then `y = floor − h`, which for the selection (drawn at
its natural height) is `centre − h / 2`. If the two ever disagree the cart jumps on the frame
A is pressed, and the contract's first drawing test is there to catch it.

The catch, and the three-part travel:

```
catch_at(safe, h) = (lip_y − (rest_y + h)) / (seated_y − rest_y)

journey(seat) =
    seat < CATCH_IN   →  catch · ease(seat / CATCH_IN)
    seat < CATCH_OUT  →  catch + CREEP · (seat − CATCH_IN) / (CATCH_OUT − CATCH_IN)
    otherwise         →  let c = catch + CREEP
                         c + (1 − c) · ease((seat − CATCH_OUT) / (1 − CATCH_OUT))
```

and then

```
travel(safe, cart, rest_x, seat):
    j = journey(safe, cart.1, seat)
    x = rest_x   + (centred_x − rest_x) * j       // centred_x = (panel_w − cart.0) / 2
    y = rest_y   + (seated_y − rest_y) * j
    w, h = cart                                   // natural size, never scaled
```

Across and down on the one progress, so the cart arrives over the mouth exactly as it
reaches it.

`seat` arrives clamped from `seat_in`/`seat_out`, but `travel` is public and must not produce
nonsense if it is handed something outside 0..1. Clamp where it is provable and say why.

### What porting this into SLOT2 changes

The original measured the row from the middle of the panel (`rest_y(h) = (OUT_H − h) / 2`).
SLOT2 measures it from the slot, because a 720-tall panel drops the slot to the foot and a
row centred on the panel would be left hanging unrelated to it. So every number here is
measured off `band_y` rather than off `panel_h / 2`.

Done right, this makes the insert *more* stable than the original's: the catch lands on the
same frame of the animation on all three panels instead of drifting as the panel gets taller.
On a 480-tall panel a GBA cart falls 114.5 px to the lip out of a 251.5 px journey, exactly
as it did in the original — the contract checks the spread across the three geometries, so if
you find yourself reaching for `panel_h / 2` anywhere, that is the test that will fail.

## 2. `shelf.rs` — the row making way

`rest_x(safe, cart)` is where the row would draw the selection this frame, at the cartridge's
natural width. It is `placements`' own arithmetic for offset `target − scroll`; take it from
there rather than assuming the middle of the panel, or pressing A mid-slide teleports the cart
sideways on the first frame.

`parted(safe, cart, recede)` is `placements` with two changes:

- every cart is pushed outwards by `away · PART · recede`, where
  `away = offset.signum() · (1 + offset.abs())` — further out the further out it already was,
  so the row *opens* rather than sliding sideways;
- alpha is multiplied by `(1 − recede)`;
- and the selection is left out entirely once `recede > 0`, because `draw_insert` is drawing
  that cart. Leaving it in the row as well puts two of one cart on screen and the travel reads
  as a copy sliding away from the original.

`placements(safe, cart)` becomes `parted(safe, cart, 0.0)`. Its existing contract in
`crates/slot2-ui/tests/shelf_draw.rs` must keep passing untouched — that is the check that
`parted` at rest really is the old behaviour.

Carts that end up entirely off the panel, or at zero alpha, are dropped exactly as
`placements` already drops them. A fully receded row draws nothing, and that is right.

## 3. `shelf_view.rs` — drawing it

Split the slot chrome in two, because the insert needs to draw *between* the halves:

- **back** — the dark opening (`SLIT`) alone. It is a hole, so the cart passes in front of it
  rather than behind a painted bar.
- **front** — everything that is the plastic face of the machine: the `LIP` strip across the
  full width, and the `BAND` in the pieces that are left once the mouth is cut out of it
  (left of the mouth, right of the mouth, and the strip below the opening). All of it after
  the cart.

`ShelfView::draw` then becomes back → row → front with nothing in between, and
`draw_insert` is back → `parted` row → the travelling cart → front. `draw`'s own contract
(`shelf_draw.rs`, `an_empty_shelf_still_draws_its_slot`) still wants at least three rects and
the lowest one ending exactly at the foot of the panel, so keep the pieces flush.

`draw_insert(canvas, ctx, safe, platform, titles, seat)`:

- `recede` for the row is `seat` — the row makes way in step with the cart going in.
- The travelling cart is drawn like the selection is in `draw`: shell, then detail at 0.8
  tint, then the label plate, then the title. Same textures, from the same cache, at the same
  natural size, scaled by the canvas. Forty-four frames of an insert must upload nothing, and
  the size changes on every one of them, so asking the cache for the drawn size is the fault
  task 12 had.
- At `seat == 0.0` the output must be frame-for-frame what `draw` produces. That is not a
  nicety: it is the frame the button was pressed on, and a difference is a visible flicker as
  the animation starts.

## The thing that will bite

Tasks 11, 12 and 13 each passed their contract and still had a real fault, every one of them
in a state the tests only saw at rest. This contract is almost entirely intermediate states
on purpose: forty-four frames, three geometries, two cartridge shapes. Before you call it
done, run one insert and ask what is true on frame 22 that is not true on frame 0 or 44.

## House style

Comments explain *why*, never *what*. No `unwrap()` on a caller's arithmetic. No
`#[allow(...)]` to silence a lint — fix the cause. Keep the skeleton's doc comments; they
carry the reasoning the constants came with.
