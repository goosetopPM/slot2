# The slot's own noises

Ported from [brandonkowalski/slot](https://github.com/brandonkowalski/slot) (MIT), where they
live in `crates/slot/assets/`. Unmodified.

Cut from a recording of the real thing, **mono signed 16-bit little endian at 48 kHz**, raw —
no header, no metadata, nothing to parse. One continuous take per direction, unchanged from
the original: nothing joined into the file, no envelope, no stretching.

SLOT2 does not alter either file, and still ships the two recordings for what they are. What
the frontend does at runtime is *read* them at a per-platform rate and level (`PlatformSkin`'s
`sfx_in`/`sfx_out`, which is the single source of truth for the numbers and is not copied
here). A pak drops down a long rail and a GBA cart clicks into a connector, and the same
take has to sound like both; reading the clip faster shortens it and raises its pitch, which
is the whole of the variation. The base `Sfx::render` is the recording untouched. The contact
lead and the clip's duration are computed at the same effective speed the clip was rendered
at, so the contacts still land on the frame the cart reaches them.

| file | frames | seconds | what it is |
|---|---|---|---|
| `insert.pcm` | 11520 | 0.240 | a cart going in: the shell down the rails, then the contacts |
| `eject.pcm` | 15120 | 0.315 | one coming out: the contacts letting go, then the shell back up |

Each clip has a **lead**: how far into it the contacts are. The caller starts the clip that
long before the cart reaches them, which is the whole of how the picture and the sound are
kept together. Read at a platform's speed the lead moves in the file, so the caller asks for
the styled lead (`Sfx::lead_at_speed`) rather than the recorded one.

| | recorded lead | tail |
|---|---|---|
| insert | 0.097 s | the shell settling after the contacts |
| eject | 0.021 s | the shell still moving after they let go |

Nothing should cut across a tail. On the way in that matters more here than it did in the
original: SLOT2 loads the core on the frame the cart seats, and that call blocks for about a
second on the device. The clip is written into the ring in one go before then, so the sink's
own thread keeps playing it while the main thread is inside `dlopen`.
