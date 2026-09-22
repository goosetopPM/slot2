# Task 05 — input layer, power menu, app state machine, loop wiring

Repository: `C:\SLOT2` (Rust workspace, Windows host).

## Goal
All of these must succeed:

```
cargo test -p slot2-input --features host
cargo test -p slot2-ui
cargo test -p slot2                                    # app.rs unit tests (host feature)
cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings
cargo clippy -p slot2 -p slot2-gfx -p slot2-ui -p slot2-input --no-default-features --features slot2/device -- -D warnings
cargo build -p slot2
```

Then run `cargo run -p slot2`, hold **M** (or Backspace) for ~0.6 s: a progress bar grows
at the bottom, then the power menu appears; arrows move, **X** selects (Resume/Restart/
Power off), **Z** goes back. Choosing Power off or Restart ends the program.

## Rules
- **Do not edit** (contract): `crates/slot2-input/src/lib.rs`, `button.rs`,
  `tests/input.rs`; `crates/slot2-ui/src/lib.rs`, `layout.rs`, `face.rs`, `splash.rs`,
  `tests/*.rs`; `crates/slot2/src/main.rs`, `diag.rs`, the `#[cfg(test)] mod tests` block
  in `crates/slot2/src/app.rs`; `crates/slot2-platform/*`; `assets/*`; any `Cargo.toml`.
- Edit only, replacing every `todo!()` and `_todo` field:
  - `crates/slot2-input/src/evdev.rs` — `parse_input_event`, `EvdevSource`, `map_raw`
  - `crates/slot2-input/src/gestures.rs` — `Gestures` (add private fields/structs freely)
  - `crates/slot2-input/src/host.rs` — `host_map`
  - `crates/slot2-ui/src/power_menu.rs` — `PowerMenu::draw`
  - `crates/slot2/src/app.rs` — `App::{feed, tick, act, draw}` (not the tests)
  - `crates/slot2/src/host_app.rs` and `device_app.rs` — wire input and `App` into the
    existing loops (see below). These two are yours to restructure.
- Read the module doc at the top of every file you edit first; it is the spec. The
  gesture semantics in `gestures.rs` and the `tests/input.rs` cases are the same thing
  said twice; when in doubt the test is right.
- No new dependencies. No `libc`. `unsafe` only if truly unavoidable (it should not be:
  `std::fs::File` + `std::os::unix::fs::OpenOptionsExt::custom_flags(0o4000 /* O_NONBLOCK */)`
  is enough for non-blocking reads).

## Loop wiring
`host_app.rs`:
```text
let mut app = App::new(debug_frame);
loop {
    let now = Instant::now();
    for ev in surface.pump() {
        CloseRequested → return
        Key { code: Escape, pressed: true } → return
        Key { code, pressed } → if let Some(b) = slot2_input::host_map(code) {
            app.feed(&Event::Button { button: b, pressed, at: now })
        }
    }
    app.tick(now);
    if let Some(exit) = app.exit() {
        slot2_platform::PowerAction::{PowerOff|Reboot}.perform(false);   // logs, returns true
        return;
    }
    app.draw(&mut canvas, &mut ctx, now);
    canvas.present(...)?; frame sleep as before
}
```
`device_app.rs`: same shape with `EvdevSource::open_all(KeyMap::from_pairs(DEFAULT_H700_KEYMAP))`
polled each frame (`for ev in source.poll(now) { app.feed(&ev) }`), log
`slot2: input: N evdev devices` once after opening, keep the diag dump first, keep
`Splash { debug_frame: true }` semantics via `App::new(true)`, and on exit call
`PowerAction::perform(true)`; if it returns `false`, log and return (BaseOS respawns).
Replace the 5-minute deadline with a 30-minute one (a safety net only).

## Verification tips
- `hold_fires_once_and_swallows_the_release` and `double_tap_only_for_configured_buttons`
  pin the exact action order and timing edges (`>=` for hold at exactly 600 ms; the
  double-tap window is `<= 250 ms` after the first release, so a press at +250 counts and
  a tick at +301 after a release at +1050 emits the delayed tap).
- `draws_dim_box_three_labels_and_a_highlight_inside_the_safe_area` computes expected
  positions from `PowerMenu::box_origin` / `row_y` — use those same helpers in `draw`.
- `draw_shows_hold_progress_then_the_menu` expects the bar at `y = 476`, `h = 4`,
  `x = 40`, `w ≈ 320` at half progress on a 720x480 panel.

## Definition of done
Paste the last lines of each command's output and describe what happened in the window
when you held M. If a contract file must change, stop and explain which one and why.
