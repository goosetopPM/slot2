# Task 20 — the gauge and the clock

First half of the last piece of M3: what the machine knows about itself. Pure logic, no
drawing. The screen that shows it is task 21.

One contract, **do not edit** — not its assertions, not its formatting, not its lint
attributes:

- `crates/slot2-platform/tests/battery.rs`

Iterate with `cargo test -p slot2-platform --test battery`, then
`cargo clippy -p slot2-platform --all-targets -- -D warnings` exactly as written, with no
extra `-A` flags, then `cargo fmt --all`.

Files you may change — and nothing else:

- `crates/slot2-platform/src/battery.rs` — skeleton, eight `todo!()` bodies.
- `crates/slot2-platform/src/clock.rs` — skeleton, six `todo!()` bodies.

Both skeletons already carry the doc comment explaining each decision. The bodies are what
is missing; the reasoning is not.

`crates/slot2-ui/` and `crates/slot2/` are **not** part of this task. They will not build
until task 21 lands, and `cargo test --workspace` will fail for that reason alone — do not
try to fix it, and do not run it.

---

## 1. The gauge — `battery.rs`

Ported from the original `slot-power`, which had already met this hardware. Keep what it
learned:

- **Nothing is a hardcoded path.** `probe` walks `<sysfs>/class/power_supply` and takes the
  first entry *in name order* whose `type` is `Battery` and which has a `capacity`. V-2 saw
  `axp2202-battery` next to `axp2202-usb` on the RG SP: a charger is a power supply too and
  has no charge of its own to report.
- **Name order, not `read_dir` order.** Otherwise the supply that gets reported can change
  across a reboot.
- **An absent gauge is a device without one, not a failure.** Every host this is developed on
  is that device. A missing directory, an unreadable one, and a tree with no batteries in it
  all read the same.
- **A capacity that will not parse is `None`, never `0`.** Zero is a battery critical, which
  is a shutdown. This PMIC leaves `current_now` empty, so an advertised attribute being
  present is no promise it is populated.
- **Both halves are read on every `read`.** The charge state changes the instant a cable
  moves; caching it at probe time makes the display lie for a whole poll interval.
- `Gauge::none` already has a body. Leave it alone.

`SLOT2_BATTERY` is a host stand-in, in the same family as `SLOT2_TARGET` and
`SLOT2_GEOMETRY`: without it the corner is empty on the one machine this gets built on. It is
a development aid, not a test fixture — the contract exercises `parse_override` directly and
never sets the variable. `Gauge::detect` is the only place `/sys` is named.

## 2. The clock — `clock.rs`

There is no `tzdata` on the image and no network to ask, so local time is UTC plus a number
the player sets. The screen that sets it is M5; until then `SLOT2_UTC_OFFSET_MIN` holds it,
read once through the `OnceLock` the skeleton already declares, because `now_local` is called
every frame and the environment does not change under a running process.

No calendar. Hours and minutes need only division, and `hhmm` wraps with `rem_euclid` so a
time before the epoch is a time of day rather than a minus sign — which is exactly what a
negative offset on a board whose RTC never started produces on the first frame.

`is_set` is the other half of that: the board boots at the epoch and counts up, so within
minutes it would show a confident, wrong `00:04`. A clock still in 1970 was never set.

## What would make this wrong

The contract is mine and it is what the work is judged against. If one of its assertions
demands something the code should not do, **say so in the report and leave it failing.** Do
not widen a constant, do not special-case a value to satisfy an assertion, and do not touch
the test file. The last task to try that lost an afternoon and the contract turned out to be
the thing that was broken.
