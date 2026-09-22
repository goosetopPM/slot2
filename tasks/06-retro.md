# Task 06 — implement `slot2-retro` (core-agnostic libretro host)

Repository: `C:\SLOT2` (Rust workspace, Windows host). Reference implementation to port from:
`C:\Users\gyuha\slot-2\crates\slot-retro\src\{libretro.rs,ffi.rs,core.rs}` (MIT, Brandon T.
Kowalski). Copying with a one-line attribution comment is expected. That code is GBA-only
(fixed 240x160, XRGB8888 only, link cable, rumble cell); this port must be core-agnostic as
the module docs describe.

Test fixtures already present: `vendor/mgba_libretro.dll` (real core) and
`assets/test/arm.gba` (MIT test ROM).

## Goal
All of these must succeed:

```
cargo test -p slot2-retro -- --nocapture          # all 10 tests run for real (the core dll is present)
cargo clippy -p slot2-retro --all-targets -- -D warnings
cargo build -p slot2-retro --target aarch64-unknown-linux-gnu   # cross-compiles (no linking of the core; dlopen at runtime)
```

## Rules
- **Do not edit** (contract): `crates/slot2-retro/src/lib.rs`, `crates/slot2-retro/tests/mgba.rs`,
  `Cargo.toml`, `assets/*`, `vendor/*`, other crates.
- Edit only `crates/slot2-retro/src/host.rs` (replace `_todo` and every `todo!()`; add private
  items freely, including the callback state) and `crates/slot2-retro/src/ffi.rs` (fill per
  its doc). You may split host.rs into private submodules under `src/host/` if you prefer,
  as long as `host.rs`/`host/mod.rs` still exposes `Core`, `frame_to_rgba8`, `frame_rgb`.
- Dependencies are fixed: `libloading 0.8` + std. No `libc`, no `once_cell`
  (`std::sync::OnceLock`/`Mutex` are available).
- `unsafe` is expected around FFI; keep each block small and comment what invariant it
  relies on (pointer lifetime, one instance rule). No `unwrap()` on anything the core or the
  filesystem can get wrong — return `Error`.
- `GET_LOG_INTERFACE` (27) must be refused: stable Rust cannot define a variadic callback.
  The core then logs to stderr on its own; that is fine.
- The one-instance rule: a process-wide registry (e.g. `static LOADED: Mutex<Option<PathBuf>>`)
  makes `Core::load` return `Error::Busy` while another `Core` from the same library path
  is alive, and `Drop` clears it. Callback state lives in a `static` behind that same rule
  (a `Mutex<Slot>` or `thread_local!`), since libretro callbacks carry no user pointer.
- mGBA specifics you will observe (do not special-case them; the generic paths must
  handle them): it sets XRGB8888 or RGB565 via `SET_PIXEL_FORMAT`, declares options through
  `SET_CORE_OPTIONS_V2` (67) if you answer `GET_CORE_OPTIONS_VERSION` with 2, or v1 (53) if
  you answer 1 — answer **1** as host.rs says (v2 handling is still required for other cores),
  asks `GET_INPUT_BITMASKS`, `GET_VARIABLE` many times per frame, and `SET_MEMORY_MAPS`.
  `need_fullpath` is false: pass the ROM bytes.
- `serialize` must allocate exactly `retro_serialize_size()` and fail cleanly if the core
  returns false. `unserialize` with a wrong-sized buffer must return `Error::State` (check
  the size against `serialize_size()` before calling the core: mGBA does not validate).

## Verification tips
- If `loads_and_reports_itself` fails on `fps`, you read `retro_system_av_info` before
  `retro_load_game`; libretro only defines it after the game is loaded.
- If frames are all black, check that `video_refresh` copies `pitch * height` bytes and
  that a NULL `data` keeps the previous frame rather than clearing it.
- If `options_are_declared_and_settable` finds no options, `retro_set_environment` was
  called after `retro_init`; it must be first.
- Run with `--nocapture` to see mGBA's own stderr lines.

## Definition of done
Paste the last lines of each command's output. If a contract file must change, stop and
explain which one and why instead of editing it.
