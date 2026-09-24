# Task 20 — what the machine says about itself

Last piece of M3. Everything else on the shelf is about the games; this is the two things
that are about the box: how much charge is left, and what time it is.

Three contracts, all **do not edit** — not their assertions, not their formatting, not their
lint attributes:

- `crates/slot2-platform/tests/battery.rs`
- `crates/slot2-ui/tests/hud.rs`
- `crates/slot2/tests/hud_app.rs`

Iterate with `cargo test -p slot2-platform --test battery`,
`cargo test -p slot2-ui --test hud` and `cargo test -p slot2 --test hud_app`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings` exactly
as written, with no extra `-A` flags, then `cargo fmt --all`. The whole suite must stay
green: `cargo test --workspace --features slot2-input/host`.

Files you may change — and nothing else:

- `crates/slot2-platform/src/battery.rs` — skeleton, `todo!()` bodies.
- `crates/slot2-platform/src/clock.rs` — skeleton, `todo!()` bodies.
- `crates/slot2-ui/src/hud.rs` — skeleton, `todo!()` bodies.
- `crates/slot2/src/app.rs` — the wiring.
- `crates/slot2/src/host_app.rs`, `crates/slot2/src/device_app.rs` — one call each.
- `crates/slot2/src/diag.rs` — one line, if the gauge path is worth reporting there.

`assets/art/bolt.svg` already exists and is the drawing; do not redraw it.

---

## 1. The gauge — `slot2-platform/src/battery.rs`

Ported from the original `slot-power`, which had already met this hardware. Keep what it
learned:

- **Nothing is a hardcoded path.** `probe` walks `<sysfs>/class/power_supply` and takes the
  first entry *in name order* whose `type` is `Battery` and which has a `capacity`. V-2 saw
  `axp2202-battery` next to `axp2202-usb` on the RG SP: a charger is a power supply too and
  has no charge of its own to report.
- **Name order, not `read_dir` order.** Otherwise the supply that gets reported can change
  across a reboot.
- **An absent gauge is a device without one, not a failure.** Every host this is developed on
  is that device.
- **A capacity that will not parse is `None`, never `0`.** Zero is a battery critical, which
  is a shutdown. This PMIC leaves `current_now` empty, so an advertised attribute being
  present is no promise it is populated.
- **Both halves are read on every `read`.** The charge state changes the instant a cable
  moves; caching it at probe time makes the bolt lie for a whole poll interval.

`SLOT2_BATTERY` is a host stand-in, in the same family as `SLOT2_TARGET` and
`SLOT2_GEOMETRY`: without it the corner is empty on the one machine the HUD gets built on.
It is a development aid, not a test fixture — the contract exercises `parse_override`
directly and never sets the variable.

## 2. The clock — `slot2-platform/src/clock.rs`

There is no `tzdata` on the image and no network to ask, so local time is UTC plus a number
the player sets. The clock screen that sets it is M5; until then `SLOT2_UTC_OFFSET_MIN`
holds it, read once through a `OnceLock` because `now_local` is called every frame and the
environment does not change under a running process.

No calendar. Hours and minutes need only division, and `hhmm` wraps with `rem_euclid` so a
time before the epoch is a time of day rather than a minus sign — which is exactly what a
negative offset on a board whose RTC never started produces on the first frame.

`is_set` is the other half of that: the board boots at the epoch and counts up, so within
minutes it would show a confident, wrong `00:04`. A clock still in 1970 was never set, and an
empty corner says so better than a number does.

## 3. The HUD — `slot2-ui/src/hud.rs`

Clock top left, gauge top right, both held `HUD_MARGIN` off the **panel** corners.

**Off the panel, not off the safe area** (DESIGN 167). The safe area is where the layout
lives, because the row and the slot have to hold together on three panel shapes. The HUD
belongs to the screen: on the 720x720 it rides up into the extended band with the wallpaper,
clear of the shelf, instead of following the box 120 px down. A HUD placed with `safe.px` and
`safe.py` passes on the 640 and fails on the square.

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

Everything goes through `FaceCache` and `ArtCache`, so an unchanged HUD uploads nothing on
its second frame. The clock string changes once a minute and the percent less often; a face
uploaded per frame is a texture allocation sixty times a second for text that did not move.

## 4. The wiring — `slot2/src/app.rs`

- `App` owns a `Gauge`, the last `Option<Battery>`, and a `Hud`. Default is `Gauge::none()`,
  so a test sees no battery until it says otherwise.
- `pub fn set_gauge(&mut self, gauge: Gauge)` takes a reading **immediately**. Waiting out an
  interval first would leave the corner empty for the first ten seconds of every boot.
- `pub const BATTERY_POLL_S: f32` — ten seconds. Measured against the `Instant` that `tick`
  is given, **not** against the `dt` the animations run on: that dt is clamped to 1/30 s so a
  slow frame cannot destabilise the row spring, and a poll counted in clamped dt would need
  three hundred ticks to reach ten seconds.
- Drawn on the shelf-side screens — `List`, `Inserting`, `Ejecting`, `Power` — and **not** on
  `Splash` or `Playing`. The splash is a composition with its own middle and nothing has been
  scanned yet; a running game owns the screen.
- The clock comes from `clock::now_local()`, passed as `Some` only when `clock::is_set` says
  so.
- `host_app` and `device_app` each call `app.set_gauge(Gauge::detect())` once at start-up.
  `Gauge::detect` is the only place `/sys` is named.

## What would make this wrong

The contracts are mine and they are what the work is judged against. If one of them is wrong
— if an assertion demands something the drawing should not do — **say so in the report and
leave it failing.** Do not widen a constant, do not add a draw call to satisfy a count, and
do not touch a test file. The last task lost an afternoon to exactly that, and the contract
turned out to be the thing that was broken.
