# Task 09 — play a game: registry, session, game list, app wiring

Repository: `C:\SLOT2` (Rust workspace, Windows host). This is the last piece of M1: after
it, `cargo run -p slot2` lists the GBA ROMs on a card and plays the one you pick.

## Goal
```
cargo test -p slot2-retro
cargo test -p slot2-ui
cargo test -p slot2
cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings
cargo clippy -p slot2 -p slot2-gfx -p slot2-ui -p slot2-input --no-default-features --features slot2/device -- -D warnings
cargo build -p slot2
```
Then, by hand:
```
mkdir -Force sdcard\Games\GBA
copy assets\test\arm.gba sdcard\Games\GBA\
cargo run -p slot2
```
A list appears with `arm`; **X** starts it, the test ROM's output fills the window,
**M held** stops it and returns to the list. **Q/W** (L1/R1) switch platform folders.

## Rules
- **Do not edit** (contract): `crates/slot2-retro/src/lib.rs`, `tests/registry.rs`,
  `tests/mgba.rs`; `crates/slot2-ui/src/lib.rs`, `layout.rs`, `face.rs`, `splash.rs`,
  `power_menu.rs`, `tests/*`; `crates/slot2/tests/session.rs`; `crates/slot2/src/diag.rs`;
  `crates/slot2-audio/*`, `crates/slot2-store/*`, `crates/slot2-gfx/*`, `crates/slot2-text/*`,
  `crates/slot2-i18n/*`, `crates/slot2-platform/*`; every `Cargo.toml`; `assets/*`.
- Edit only, replacing every `todo!()` / `_todo`:
  - `crates/slot2-retro/src/registry.rs` — `joypad_bit` (the table in `tests/registry.rs`
    is the spec; `mask_for` is already written)
  - `crates/slot2/src/session.rs` — the whole `Session` (read its module doc; it is the spec)
  - `crates/slot2-ui/src/game_list.rs` — `GameList` navigation and `draw`
  - `crates/slot2/src/app.rs` — extend the state machine (below); the existing
    `#[cfg(test)] mod tests` must keep passing unchanged
  - `crates/slot2/src/host_app.rs`, `device_app.rs` — wire the new screens and audio
- No new dependencies. No `unsafe`. No `unwrap()` on anything a card or a core can get
  wrong; `Session` logs with `eprintln!("slot2: …")` where the doc says so.

## App state machine after this task
```text
Screen::List(GameList)          ← the new start screen, replacing Splash
   Tap(Up)/Tap(Down)            → list.up()/down(len)
   Tap(L1)/Tap(R1)              → previous/next platform (Platform::ALL order, wrapping),
                                  rescan, list.clamp(len)
   Tap(A)                       → start a Session for the selected cart; on Err log and
                                  show nothing else (stay on the list)
   Hold(Menu)                   → Screen::Power (unchanged)
Screen::Playing                 ← a Session is running
   Hold(Menu)                   → session.stop(card), back to Screen::List, rescan
   Tap(Power)                   → exit (as before); stop the session first
   everything else              → nothing yet (the in-game menu is M4)
Screen::Power(menu)             ← unchanged, reachable from List
```
`App` grows: `card: Card`, `platform_index: usize`, `carts: Vec<Cart>`, `list: GameList`,
`session: Option<Session>`, `volume: Volume`, `core_dir: PathBuf`, `sink_rate: u32`.
Add `App::with_card(card, core_dir, sink_rate, debug_frame)`; keep `App::new(debug_frame)`
working (it can use an empty card at `.` and no core dir) so the existing tests compile.
Buttons held for the core come from `self.state.held()` filtered to `LogicalButton`
(a `From<slot2_input::Button>` style helper — Menu/Power/volume map to `None`).

`App::draw`: `Screen::List` → `list.draw(...)`; `Screen::Playing` → `session.draw(...)`;
`Screen::Power` → whatever is underneath, then the menu. The splash is gone from the loop
(keep `Splash` compiling; it returns in M3).

`App::run_frame(now)`: when `Screen::Playing`, `session.run_frame(&held, &volume)` once per
frame; otherwise nothing. The host/device loops call `feed`, `tick`, `run_frame`, `draw`,
`present` in that order.

## Audio wiring in the loops
The sink is created when a session starts and dropped when it stops:
`let (session, consumer) = Session::start(...)` → `HostSink::open(48_000, 1024, consumer)`
(device: `AlsaSink::open`). Keep the sink in the loop (an `Option<Box<dyn Sink>>`), drop it
when returning to the list. If the sink fails to open, log and play the game silently —
never refuse to start the game.

## Verification tips
- `tests/registry.rs` pins the per-platform button maps exactly (Game Boy has no X/Y/L/R,
  Mega Drive's Y/X/A → libretro Y/B/A).
- `tests/session.rs` runs the real mGBA core: audio must reach the ring (>10 000 samples in
  30 frames at 48 kHz), muting must give silence, the thumbnail must touch one axis of
  160 px, states must write a `.png` beside the `.state`, and after `stop` the library must
  be free for a second `Session::start`.
- The game list's exact layout numbers are in `game_list.rs`'s module doc.

## Definition of done
Paste the last lines of each command, and describe what happened in the window when you
ran the ROM. If a contract file must change, stop and explain which one and why.
