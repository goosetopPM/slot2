# Task 19 — the slot's own noises

Tenth piece of M3. The cart going in and coming out get the sound they were recorded with.

Two contracts, both **do not edit** — not their assertions, not their formatting, not their
lint attributes:

- `crates/slot2-audio/tests/sfx.rs`
- `crates/slot2/tests/sfx_app.rs`

Iterate with `cargo test -p slot2-audio --test sfx` and `cargo test -p slot2 --test sfx_app`,
then `cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`
exactly as written, with no extra `-A` flags, then `cargo fmt --all`. The whole suite must
stay green: `cargo test --workspace --features slot2-input/host`.

Files you may change — and nothing else:

- `crates/slot2-audio/src/sfx.rs` — skeleton, three `todo!()` bodies.
- `crates/slot2/src/app.rs` — `play`, `todo!()`, and the wiring.

The assets are already at `assets/sfx/` with their provenance. Do not touch them, and do not
add any others.

---

## 1. `sfx.rs`

The clips are raw mono signed 16-bit little endian at 48 kHz — no header, nothing to parse.
`seconds` is therefore `bytes / 2 / ASSET_HZ`, **measured rather than written down**: a
constant that drifts from the file is a lead that points at the wrong part of the sound.

`tail` is `seconds - lead`.

`render` gives interleaved stereo from mono content: the same sample in both channels,
because the sink is stereo and a slot is in the middle of the machine. At the asset's own
rate — which is what every device here runs at — nothing is resampled. At another rate, what
must not change is **how long the sound takes**, because the picture it goes with is on a
clock of its own; a linear resample is plenty, and `resample.rs` next door already has one if
it fits.

The sink reports the rate it actually opened at and nothing here may assume that is sensible.
Zero is a rate a broken backend will hand you.

## 2. `play` in `app.rs`

Writes the whole clip into a fresh ring, hands the consumer to the loop and asks for a sink —
the mechanism that already exists for a session's audio (`pending_consumer` +
`SinkRequest::Open`). The loop does `sink = Some(new)`, so an `Open` replaces whatever was
there; there is no second device and no mixing.

**The whole clip at once, not a frame at a time.** The frame after the insert's clip starts
may be the one that seats the cart, and that frame loads a core — about a second inside
`dlopen` on the device. The sink has its own thread and plays what is in the ring regardless
of what the main thread is doing, so a clip that is already there survives the stall. A clip
being dripped in would stop dead at the seat, which is the exact moment it is making its
point.

Size the ring from the clip, not from a guess.

## 3. When

This is the whole of the work, and it is one line of arithmetic:

```
insert → play when anim >= SEATED_AT - Sfx::Insert.lead()
eject  → play when the eject begins
```

The lead is how far into the clip the contacts are. Starting it when the cart *reaches* them
puts the whole sound a tenth of a second late — late enough to sound like a cheap recording
rather than like a bug, which is worse, because nobody files it and everybody hears it.

Once per animation. `sfx_fired` is on `App` for that; clear it when an insert or an eject
begins, not when one ends.

Nothing else makes a noise. Walking the row is not an insert, and a refused press on an empty
shelf is answered by the screen flinching — a sound as well would claim something happened.

## 4. What must not break

`Session::start` succeeding sets `pending_consumer` and `SinkRequest::Open` for the game. By
then the insert's clip is long over: it ends at `SEATED_AT - lead + seconds`, which is about
0.14 s past the seat, and the core load that follows the seat takes a second. The game's
`Open` replaces the clip's sink, which is correct — but do not *rely* on the ordering by
leaving the two to race. Whatever you do, a running game's audio is never replaced by a clip,
because nothing inserts or ejects while one is playing.

`crates/slot2/tests/insert_app.rs` and `refusal_app.rs` call `take_sink_request` and
`take_consumer` already and must stay green. Read what they assert before you change when
those are set.

## The thing that will bite

Every task since 11 has passed its contract with a fault in a state the tests only saw at
rest or only saw indirectly. The candidate here is `sfx_fired`: cleared in the wrong place it
either fires every frame from the threshold to the end of the animation, or fires once and
never again for the rest of the session. The contract counts, so both are caught — but only
for the paths it walks.

## House style

Comments explain *why*, never *what*. No `unwrap()` on a caller's arithmetic. No
`#[allow(...)]` to silence a lint — fix the cause.
