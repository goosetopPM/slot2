# The slot's own noises

Ported from [brandonkowalski/slot](https://github.com/brandonkowalski/slot) (MIT), where they
live in `crates/slot/assets/`. Unmodified.

Cut from a recording of the real thing, **mono signed 16-bit little endian at 48 kHz**, raw —
no header, no metadata, nothing to parse. One continuous take per direction, played as
recorded: no stretching, no envelope, nothing joined. Everything the sound needs to do it
already does, and keeping the two in step is a question of *when the clip starts*, not of
what is done to it.

| file | frames | seconds | what it is |
|---|---|---|---|
| `insert.pcm` | 11520 | 0.240 | a cart going in: the shell down the rails, then the contacts |
| `eject.pcm` | 15120 | 0.315 | one coming out: the contacts letting go, then the shell back up |

Each clip has a **lead**: how far into it the contacts are. The caller starts the clip that
long before the cart reaches them, which is the whole of how the picture and the sound are
kept together.

| | lead | tail |
|---|---|---|
| insert | 0.097 s | the shell settling after the contacts |
| eject | 0.021 s | the shell still moving after they let go |

Nothing should cut across a tail. On the way in that matters more here than it did in the
original: SLOT2 loads the core on the frame the cart seats, and that call blocks for about a
second on the device. The clip is written into the ring in one go before then, so the sink's
own thread keeps playing it while the main thread is inside `dlopen`.
