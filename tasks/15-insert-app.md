# Task 15 — the insert inside the app

Sixth piece of M3. Pressing A stops being a jump cut. Task 14 built the motion and the
drawing; this wires them to the button and to the core load.

`crates/slot2/tests/insert_app.rs` is the contract. **Do not edit it**, including its lint
attributes. Iterate with `cargo test -p slot2 --test insert_app`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings`, then
`cargo fmt --all`. The whole suite must stay green:
`cargo test --workspace --features slot2-input/host`.

Files you may change: `crates/slot2/src/app.rs` only.

---

## The shape of it

`Screen` gains two variants:

```rust
pub enum Screen {
    Splash,
    List,
    /// A cart on its way into the slot.
    Inserting,
    /// A cart on its way back out of it.
    Ejecting,
    Playing,
    Power(PowerMenu),
}
```

Keep `Screen` `Copy + Eq`. The animation's clock does **not** go inside the variant: an `f32`
in there costs `Screen` its `Eq`, and `resume_screen` and half of `act` compare screens with
`==`. Put it in a field beside it:

```rust
/// The insert or eject animation's clock, in seconds. Only meaningful while `screen` is
/// `Inserting` or `Ejecting`; the variant is the state and this is the data hanging off it.
anim: f32,
```

and one accessor, which is all the contract needs:

```rust
/// How far the cart is into the slot: 0.0 standing on the row, 1.0 seated. `None` when
/// nothing is going in or out.
pub fn insert_seat(&self) -> Option<f32>;
```

`Inserting` answers `insert::seat_in(self.anim)`, `Ejecting` answers
`insert::seat_out(self.anim)`, everything else answers `None`.

## The load happens when the cart seats

This is the decision the task turns on, so it gets said plainly.

`Session::start` blocks for about a second on the device: it dlopens the core and hands it a
ROM. The original frontend loaded on a worker thread and was told `on_core_ready`. SLOT2 has
no such thread and is not growing one for this — a libretro core is global mutable state and
moving one across a thread boundary to save a second of latency is a bad trade.

So the load runs on the frame the cart is seated, and the dwell that already exists is what
hides it:

```
A pressed    →  screen = Inserting, anim = 0
every tick   →  anim += dt                       (the same clamp the row uses)
anim ≥ SEATED_AT and no session yet
             →  start the session, here, now     (this frame costs a second)
anim ≥ INSERT_S and a session exists
             →  screen = Playing
```

The picture on screen while that second passes is a cartridge fully seated in the slot, which
is what a machine loading a cartridge looks like. Loading at the press instead would freeze
the *first* frame of the animation, which is the jump cut back again with extra steps.

Note what the second condition is: **a session exists**, not "the clock ran out". Without it
the screen freezes after the animation ends while the core is still coming up, which is the
fault the whole arrangement exists to avoid. `INSERT_HOLD_S` is 0.28 s and a cold core is
slower than that on the device, so this is the normal case, not the edge case.

Do not reset `last_tick` after the load. The clamp in `tick` already turns the stalled frame
into one 1/30 s step, it is documented there, and reaching for `Instant::now()` inside `tick`
would make the animation untestable.

## A core that will not load

A card with a game whose core is missing is an ordinary card — it is what a half-built SD
looks like. Sitting on the seated frame for ever is a dead handset. `Session::start` failing
sends the cart straight back out:

```
screen = Ejecting, anim = 0
```

`seat_out(0.0)` is 1.0, so it leaves from the slot rather than from wherever it happened to
be, which is right: it got all the way in before anything refused it.

`Ejecting` ticks the same clock and lands on `Screen::List` at `anim >= EJECT_S`.

There is no message yet. The refusal screen is later in M3; this task only has to not hang.

## Leaving a game

`(Screen::Playing, Hold(Menu))` currently stops the session and goes to `List`. It now stops
the session and goes to `Ejecting` with `anim = 0`. The cart comes back out the way it went
in. `rescan` still happens — do it when the eject reaches `List`, not at the start of it, or
the row rearranges behind a cart that is still on screen.

## The three places that must learn about the new screens

- **`run_frame`** runs the core whenever a session exists, and the loop calls it every frame
  regardless of screen. Once the core is loaded at the seat there is still a dwell to play
  out, and 17 frames of a game running behind a picture nobody can see is 17 frames of the
  game the player never gets to watch. Run the core only on `Screen::Playing`.
- **`frame_time`** returns the session's pace whenever a session exists. An insert is a UI
  animation and runs at the panel's rate; a core's 59.7275 fps is only right once the core
  is what is on screen. Same guard.
- **`act`** must ignore the row controls on `Inserting` and `Ejecting`. Left, Right, L1 and
  R1 scroll the row out from under a cart that is already on its way in, and the wrong game
  loads. `Tap(Power)` keeps working — whatever else is going on, the power button turns it
  off. A on an empty shelf still does nothing, as it already does.

## Drawing

```
Screen::Inserting => shelf_view.draw_insert(.., seat_in(anim))
Screen::Ejecting  => shelf_view.draw_insert(.., seat_out(anim))
```

`Screen::Power` draws whatever is underneath first; an animation underneath draws as the
insert, not as the resting shelf. `resume_screen` is what closing the power menu goes back
to — an animation that was interrupted has no sensible place to resume from, so send it to
`List` and let the player press A again.

## What is deliberately not here

The original carries a `resumed` flag on the insert: a cart already in the slot when the
frontend started is drawn seated from the first frame, with no shelf behind it, because a
resume is not a movement the user made and there is nothing for the cart to have come from.

SLOT2 has no boot resume — nothing records which game was playing across a restart — so the
flag would have no producer, and a flag with no producer is a thing the next person deletes
because it is always false. When boot resume lands, this is where it hooks in: start at
`Screen::Inserting` with `anim = SEATED_AT` and skip the row.

## The thing that will bite

Tasks 11 to 13 each passed their contract and still had a fault, every one in a state the
tests only saw at rest. The contract here drives every frame of an insert and every frame of
an eject. Before you call it done, ask what is true on the frame the core loads that is not
true on the frame before it.

## House style

Comments explain *why*, never *what*. No `unwrap()` on a caller's arithmetic. No
`#[allow(...)]` to silence a lint — fix the cause.
