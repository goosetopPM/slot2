# Task 16 — what is on the front of a cartridge

Seventh piece of M3. The label plate stops being a white rectangle.

`crates/slot2-ui/tests/label.rs` is the contract. **Do not edit it** — not its assertions,
not its formatting, not its lint attributes. Iterate with
`cargo test -p slot2-ui --test label`, then
`cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings` exactly
as written, with no extra `-A` flags, then `cargo fmt --all`. The whole suite must stay
green: `cargo test --workspace --features slot2-input/host`.

Files you may change — and nothing else:

- `crates/slot2-store/src/card.rs` — one new `todo!()` method, `label_path`.
- `crates/slot2-ui/src/label.rs` — a skeleton with the constants, the doc comments and
  `todo!()` bodies. Fill the bodies in. The signatures are what the contract calls.
- `crates/slot2-ui/src/shelf_view.rs` — one new `todo!()` method, `set_labels`, and the
  label plate in `draw_cart`.

Nothing in `crates/slot2` yet. Handing the card's label paths to the view is task 17.

---

## 1. The card

`Labels/<PLAT>/<stem>.png`, which `ensure_layout` already creates. `label_path` returns the
path only when the file is there, so the view never has to ask.

Only `.png`. Adding formats later is one line; guessing at a set of them now is four
untested branches.

## 2. The printed label

Most games on most cards will never have a scan. The fallback is therefore the **normal**
case, not an error state, and it has to look like something somebody chose.

A printed label is a plate in a colour derived from the title. Two rules decide the colour,
and the contract checks both:

- **Derived, not assigned.** The same title gives the same colour every boot, or a shelf
  teaches a player nothing. A hash of the title's bytes into a hue is enough — the original
  used FNV-1a, `hsv_to_rgb((h % 360), 0.52, 0.74)`, and those two numbers are the second
  rule.
- **It is a background for dark text.** Mid saturation and value. A fully saturated colour at
  handheld brightness reads as a warning light; a near-white one reads as a blank sticker.

`clean_title` is the other half. A dumped filename carries region and revision tags, and
those are facts about the dump rather than about the game: `Advance Wars (USA, Europe)
(Rev 1)` is `Advance Wars` on a shelf, and the filename keeps the rest. Watch the two cases
the contract names:

- a bare hyphen is inside a word — `Spider-Man` keeps its own — while a spaced one separates
  a title from a subtitle and goes;
- a name that cleans away to nothing falls back to the stem. A blank where a game should be
  is worse than a messy name.

Korean and Japanese titles go through unchanged. The card is Korean before it is anything
else.

## 3. The cache

`LabelCache` decodes a PNG, uploads it once, and holds it while it is being drawn. Three
things it must get right, each with a test:

**Decode once.** Including the misses: a file that will not decode, or is not there, is
remembered as such, or a broken label costs a file open sixty times a second.

**Scale before uploading.** Scans come at whatever size a scanner produced. A 1024-square PNG
drawn two hundred pixels wide costs four megabytes to look no better. Box-filter it down by
an **integer** factor until it fits `MAX_OVERSAMPLE` times the plate, and no further — the
contract checks both ends, because a label reduced to the plate's exact size is mush the
moment the cart becomes the selection and grows.

Note what the plate does *not* decide: the texture's size is not the drawn size. A label is a
picture and the canvas scales the quad, exactly as the cart artwork already does. Keying the
cache on the drawn size is the fault task 12 had, and a cart's drawn size lerps continuously
while the row slides.

**Cap it.** `CACHE_MAX` textures, dropping the least recently asked for. Seven carts are ever
on screen; a card with two hundred games otherwise holds two hundred textures, and on a Mali
sharing system memory with everything else that is a leak with a polite name. Evicting means
`canvas.free(tex)` as well as dropping the entry, or the cap saves nothing.

## 4. The shelf

`ShelfView::set_labels(Vec<Option<PathBuf>>)`, one entry per cart in the row's order. Set
when the row changes, never per frame: whether a file exists is a question for a rescan, and
asking it sixty times a second for every cart on screen is sixty stat calls a second at the
one moment the device can least afford them.

The list can be shorter than the row, or never set at all — a rescan that has not happened
yet. That is not a reason for a cart to have no front; fall through to the printed label.

In `draw_cart`, where the white plate is drawn today:

- art → `canvas.image` at the plate rect, tinted **white**. It is a picture. Tinting it with
  the shell colour would stain a scan of a real sticker.
- no art, or art that would not decode → the printed plate.

The title stays on the selection only, as it is now. A side cart is drawn at 0.78 and dimmed
to 0.55 and its title would be illegible; what tells it apart at the edge of the row is the
colour, which is what the printed label is for.

## The thing that will bite

Every task since 11 has passed its contract and still had a fault in a state the tests only
saw at rest or only saw indirectly. Two candidates here: the cache key, which only repeats
while nothing is sliding; and eviction, which only happens on a card bigger than the tests
that pass quickest. Before you call it done, scroll a shelf of forty labelled games and ask
what the cache is holding.

## House style

Comments explain *why*, never *what*. No `unwrap()` on a caller's arithmetic. No
`#[allow(...)]` to silence a lint — fix the cause.
