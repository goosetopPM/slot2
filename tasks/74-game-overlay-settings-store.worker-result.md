# Task 74 — Per-game overlay settings store: worker result

- Status: success
- Cumulative calls used: 1 of 2 (this is the first call)

## The three states and their canonical spelling

- `GameSettings` gained `pub overlay: Option<bool>` and `GameSettings::KEY_OVERLAY = "overlay"`.
  `is_default()` is still `*self == GameSettings::default()`, so the field counts only while it is
  `None`.
- `from_ini` reuses the file's existing bool parser (`parse_bool`), so the spellings are exactly the
  other bool keys' ones: `on`/`true`/`yes`/`1` and `off`/`false`/`no`/`0`, with surrounding
  whitespace and ASCII case ignored. A missing key, an empty value and anything unrecognised are
  `None`.
- `apply_to` writes the canonical `overlay = on` / `overlay = off` and removes the key only for
  `None`. No new enum, path, name or file: `Card::read_settings`/`write_settings` are the only
  paths, and `lib.rs` needed no change (`GameSettings` was already exported).
- The three meanings stay three: absent inherits (today that inherits "no overlay anywhere"), `on`
  asks for this game, `off` refuses it. A round trip of all three through the card was verified,
  along with every accepted spelling being rewritten canonically on the next save.

## Safe write, preservation, defaults, unreadable originals

- An overlay change preserves core/scale/overscan/rewind/shader and hand-written unknown keys
  (`future_filter`), and a core/scale/shader change preserves a stored overlay and the unknown key.
- Clearing the choice removes only the overlay key; a file left with nothing goes, and a clear with
  no file is a no-op success rather than an error.
- An invalid-UTF-8 settings file reads as no settings (forgiving) and refuses both set and clear
  with `Error::Io(path)`, byte-for-byte preserved and no `.tmp` left behind. A directory where the
  file belongs is refused the same way and is not removed.
- The store never asks whether a picture exists: `overlay = on` on a card with no overlay folder
  reads back as `Some(true)`, creates nothing, and the settings folder still holds only the game's
  own ini. No PNG fixture, fake resolver or production test branch was added; the failure fixtures
  are the portable invalid-UTF-8 file and the directory path, never a Windows-meaningless
  permission bit.

## Acceptance commands (run last, in this order, after the final code change)

| command | exit | last result line |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | (no output) |
| `cargo test -p slot2-store` | 0 | last crate `Doc-tests slot2_store`: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`; summed 89 passed / 0 failed / 0 ignored (the new `overlay_settings` suite is 10 of them, `card` 19 and `shader_settings` 13 unchanged) |
| `cargo check -p slot2 --tests` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.94s` |
| `cargo clippy -p slot2-store --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 4.68s` |

`cargo fmt --all` was applied once (my new test file's long `assert_eq!` lines), then the whole
sequence above ran in order and nothing was edited afterwards. Workspace tests and device
deployment were not run.

## Files

- Modified: `crates/slot2-store/src/settings.rs` (field, key, `from_ini`, `apply_to`).
- New: `crates/slot2-store/tests/overlay_settings.rs` (10 tests).
- Narrow literal fixes for the new field only, with no changed meaning or assertion:
  `crates/slot2-store/tests/shader_settings.rs` (`overlay: None` added),
  `crates/slot2-store/tests/card.rs` (`overlay: None` added to the round-trip `want`),
  `crates/slot2/tests/display_menu_app.rs` (`overlay: None` added to an existing full literal).
- No other file was touched; unrelated uncommitted changes were left alone.

## Out of scope, confirmed not implemented

- No PNG discovery, decode, texture upload/free, draw order, geometry-based asset selection or
  overlay renderer work; no Display/Overlay menu, translation, instant preview or save-failure
  toast; no `slot2-gfx`, `slot2-ui`, `slot2-retro`, `slot2-platform` or registry change.

## Contract concern / remaining risk (1 line)

The registry has no overlay default type yet, so `None` currently inherits a "no overlay" default
that lives nowhere; when that platform default is added it must be resolved by the loader (as the
shader default is), not by this store, or an inherited `on` and an explicit `off` could be folded
together.

## Time

Start ~13:53, finish ~13:58 (Asia/Seoul) — about 6 minutes, one call.
