# Task 21 — the corner the machine talks from

Second half of the last piece of M3, and the drawing half. Task 20 built the gauge and the
clock; this puts them on screen. **Task 20 must be green before this starts.**

Two contracts, both **do not edit** — not their assertions, not their formatting, not their
lint attributes:

- `crates/slot2-ui/tests/hud.rs`
- `crates/slot2/tests/hud_app.rs`

Iterate with `cargo test -p slot2-ui --test hud` and `cargo test -p slot2 --test hud_app`,
then `cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`
exactly as written, with no extra `-A` flags, then `cargo fmt --all`. The whole suite must
stay green: `cargo test --workspace --features slot2-input/host`.

Files you may change — and nothing else:

- `crates/slot2-ui/src/hud.rs` — skeleton, five `todo!()` bodies.
- `crates/slot2/src/app.rs` — one `todo!()` body plus the wiring.
- `crates/slot2/src/host_app.rs`, `crates/slot2/src/device_app.rs` — one call each.

`assets/art/bolt.svg` already exists and is the drawing. Do not redraw it, do not move it.
`crates/slot2-platform/` is finished; do not touch it.

---

## 1. The HUD — `slot2-ui/src/hud.rs`

Clock top left, gauge top right, both held `HUD_MARGIN` off the **panel** corners.

**Off the panel, not off the safe area** (DESIGN 167). The safe area is where the layout
lives, because the row and the slot have to hold together on three panel shapes. The HUD
belongs to the screen: on the 720x720 it rides up into the extended band with the wallpaper,
clear of the shelf, instead of following the box 120 px down. A HUD placed with `safe.px` and
`safe.py` passes on the 640 and fails on the square — there is a test for exactly that.

Each cluster sits on a plate. A wallpaper is a photograph the player chose, and white ink on
an unknown photograph is ink that is sometimes not there at all.

The capsule is four walls, a nub on the positive end, and a fill proportional to the percent.
The bolt has a **slot of its own, reserved whether or not anything is charging** — the
original learned this twice, first with the bolt inside the capsule (a smudge, and a hole in
the fill) and then with the slot appearing only when charging (a capsule that jumps sideways
at the moment the player is looking at it).

`ink` is the warning rule: `LOW_INK` only when the pack is low **and** discharging. A pack at
8% with a cable in it is a pack on its way up, and a red gauge there warns about a problem
that is already being fixed.

Both corners are vertically centred in `HUD_H` on their own measured height — the capsule on
`GAUGE_H`, the text on whatever the font chain rasterised. Neither has to know the other is
there, and the digits do not sit on the plate edge.

Either half being `None` draws that corner not at all. An empty capsule says the battery is
flat, which is a different statement from having no gauge; and a clock that was never set is
worse than no clock.

Everything goes through `FaceCache` and `ArtCache`, so an unchanged HUD uploads nothing on
its second frame. The clock string changes once a minute and the percent less often; a face
uploaded per frame is a texture allocation sixty times a second for text that did not move.

## 2. The wiring — `slot2/src/app.rs`

`App` already has the fields: `gauge`, `battery`, `battery_read`, `hud`. What is missing is
`set_gauge`, the poll, and the draw call.

- `set_gauge` takes a reading **immediately**. Waiting out an interval first would leave the
  corner empty for the first ten seconds of every boot, which is most of the time anyone
  spends looking at a shelf they have just turned on.
- `BATTERY_POLL_S` is already declared. Measure against the `Instant` that `tick` is given,
  **not** against the `dt` the animations run on: that dt is clamped to 1/30 s so a slow
  frame cannot destabilise the row spring, and a poll counted in clamped dt would need three
  hundred ticks to reach ten seconds. This is the one thing in this task most likely to be
  got wrong, and it has its own test.
- Drawn on the shelf-side screens — `List`, `Inserting`, `Ejecting`, `Power` — and **not** on
  `Splash` or `Playing`. The splash is a composition with its own middle and nothing has been
  scanned yet; a running game owns the screen.
- The clock comes from `clock::now_local()`, passed as `Some` only when `clock::is_set` says
  so.
- `host_app` and `device_app` each call `app.set_gauge(Gauge::detect())` once at start-up.

## What would make this wrong

The contracts are mine and they are what the work is judged against. If one of them demands
something the drawing should not do, **say so in the report and leave it failing.** Do not
widen a constant, do not add a draw call to satisfy a count, and do not touch a test file.
The last task to try that lost an afternoon and the contract turned out to be the thing that
was broken.
