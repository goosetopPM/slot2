# Task 17 — saying no, and saying why

Eighth piece of M3, and it closes a hole task 15 opened: a cart whose core is missing goes
all the way into the slot and comes back out with no explanation. That is the most common
thing to be wrong with a half-built card, and the player can fix it — if they are told what
it is.

Two contracts, both **do not edit** — not their assertions, not their formatting, not their
lint attributes:

- `crates/slot2-ui/tests/refusal.rs`
- `crates/slot2/tests/refusal_app.rs`

Iterate with `cargo test -p slot2-ui --test refusal` and
`cargo test -p slot2 --test refusal_app`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings` exactly
as written, with no extra `-A` flags, then `cargo fmt --all`. The whole suite must stay
green: `cargo test --workspace --features slot2-input/host`.

Files you may change — and nothing else:

- `crates/slot2-ui/src/refusal.rs` — skeleton, `todo!()` bodies.
- `crates/slot2-ui/src/toast.rs` — skeleton, `todo!()` bodies.
- `crates/slot2-gfx/src/canvas.rs` — `set_origin` has a default that ignores it; implement
  it for `RecordingCanvas` and `GlCanvas`.
- `crates/slot2/src/app.rs` — two `todo!()` accessors and the wiring.

The messages are already in `assets/lang/en.ftl` and `ko.ftl`. Do not add strings to either
file and do not put an English sentence in Rust.

---

## 1. The flinch — `refusal.rs`

Ported from the original, whose entire error UI was one curve: no words, a shake, the same
answer for everything the frontend will not do. A shake says *no* in less time than it takes
to read *no*, and it cannot be mistranslated.

```
offset(age) = cos(age · SHAKE_HZ · τ) · SHAKE_PX · (1 − age / REFUSAL_S)
```

Cosine, not sine. A flinch is a knock and everything after it is settling; starting at zero
would make the frame the button was pressed on the one frame that did not move, which reads
as the press being *ignored* rather than refused. Past `REFUSAL_S` the offset is exactly
zero — a screen left a few pixels off is a screen that looks broken.

`tick` takes seconds and must survive a caller handing it nonsense. `Refusal::default()` is
a refusal that has already finished.

## 2. The words — `toast.rs`

The original did not need these. It had one console and one core, so every refusal was "not
now". A card here can be missing the core for a whole shelf, which is a thing the player can
go and fix.

A toast holds a message **key** and its arguments, never a sentence. DESIGN §8: whole
sentences only, never a fragment the code glues together. `ctx.i18n.spans(key, args)` and
`crate::draw_spans` do the rest — that is also what makes `{ JOSA($title, "을/를") }` work in
Korean, which no amount of `format!` in Rust will.

Fade in, hold, fade out, on the constants in the skeleton. The plate goes down before the
words: a toast is drawn over a wallpaper and over the carts, and text alone on artwork is
text nobody can read.

Where it sits is measured from the slot, not from the panel — `TOAST_ABOVE_SLOT` — for the
same reason the row is (`shelf::CENTRE_ABOVE_SLOT`). The gap between the feet of the row and
the band is empty on every panel, and that is the space. The contract checks all three.

A long title must not push the toast off the side. Shrink it, wrap it, or clip it — the
contract only insists it stays on the panel and stays centred.

## 3. The whole screen moves — `set_origin`

`Canvas::set_origin` is new and defaults to ignoring the offset. Implement it for
`RecordingCanvas` (the recorded coordinates must already include it — a test that asked the
canvas what its origin was would prove nothing) and for `GlCanvas`.

This is how the flinch reaches the screen. Threading an offset through every screen's `draw`
down to every cart is a parameter each new screen has to remember to honour, and one of them
eventually will not.

## 4. The wiring — `app.rs`

```
A on an empty shelf     → refusal = Some(Refusal::new())     — no cart, so the screen flinches
Session::start fails    → Ejecting (already), plus a toast
    Error::NoCore(_)    → "core-missing"
    anything else       → "cart-broken", with $title
```

Two things the contract pins, both of them judgements rather than mechanics:

**A refused cart does not also make the screen flinch.** The cart is on screen and carries
the refusal by coming back out. A screen that flinched as well would read as two separate
failures. The flinch is for when there is nothing on screen to carry it.

**The message outlives the animation.** The eject is 0.45 s and the toast is three. A message
that left with the cart would be gone before a player who was watching the cartridge looked
up.

`refusal` and `toast` both advance on `tick`'s clamped `dt`, and both are dropped when they
are done — `toast_key()` returning `None` is how the contract sees that. `draw` puts the
toast over everything else on the shelf screens, and sets the origin from the refusal for
the whole frame.

A flinch is not a modal: the player can still press a shoulder and change shelf while it
decays.

## The thing that will bite

Every task since 11 has passed its contract with a fault in a state the tests only saw at
rest, or only saw indirectly. The candidates here: `set_origin` left set after a frame, so
the *next* frame inherits a shake that is already over; and a toast that is `done` but not
dropped, which draws nothing and still reports a key.

## House style

Comments explain *why*, never *what*. No `unwrap()` on a caller's arithmetic. No
`#[allow(...)]` to silence a lint — fix the cause.
