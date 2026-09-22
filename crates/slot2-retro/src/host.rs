//! The host: one loaded core, its callbacks, and the state those callbacks fill in.
//!
//! Implementation notes for task 06 (see tasks/06-retro.md). The original implementation
//! to port from is `C:\Users\gyuha\slot-2\crates\slot-retro\src\libretro.rs` (MIT, Brandon
//! T. Kowalski) — keep its callback plumbing and its care around lifetimes, drop its GBA-only
//! assumptions (fixed 240x160, XRGB-only, link/netpacket, rumble cell) and make everything
//! below core-agnostic.
//!
//! ## Callback state
//! libretro callbacks are plain C function pointers with no user data, so the host keeps a
//! thread-local (or a `static` guarded by the one-instance rule) `Slot` holding: pixel
//! format, `AvInfo`, last `Frame`, audio sample buffer (`Vec<i16>` interleaved stereo),
//! current `JoypadMask` per port (2 ports), the option table (`Vec<CoreOption>` + current
//! values + `variables_dirty` flag), the `Env`, the `EnvLog`, and the libretro log
//! callback's messages (last 64 lines, for `logs()`).
//!
//! ## Environment commands to answer (`retro_environment`)
//! Return `true` and act:
//! - `SET_PIXEL_FORMAT (10)` → store; refuse (`false`) formats other than the three known.
//! - `GET_SYSTEM_DIRECTORY (9)`, `GET_SAVE_DIRECTORY (31)` → `*const c_char` to a
//!   NUL-terminated copy kept alive in `Slot` (the core may hold the pointer).
//! - `GET_VARIABLE (15)` → look up `key`; `value` = current value's C string (kept alive
//!   in the table), or NULL + return `false` if unknown.
//! - `SET_VARIABLES (16)` → parse v0 format `"desc; a|b|c"` into `CoreOption`s, default =
//!   first value, keep any value the `Env` preset already gave.
//! - `GET_VARIABLE_UPDATE (17)` → `*bool = dirty; dirty = false`.
//! - `GET_CORE_OPTIONS_VERSION (52)` → `*u32 = 1` (accept v1 options: `SET_CORE_OPTIONS (53)`
//!   and `SET_CORE_OPTIONS_INTL (54)`, whose entries carry `key`, `desc`, `values[]` with
//!   `default_value`). Take the `us` table for INTL.
//! - `SET_CORE_OPTIONS_V2 (67)` / `V2_INTL (68)` → same, from the `definitions` array; ignore
//!   categories. `SET_CORE_OPTIONS_DISPLAY (55)` → `true`, ignore.
//! - `GET_LOG_INTERFACE (27)` → a `retro_log_printf_t` that formats the C varargs. Since
//!   Rust cannot take C varargs in a callback, implement the log callback as
//!   `unsafe extern "C" fn(level, fmt: *const c_char, ...)` — this IS allowed for
//!   `extern "C"` fns with `...` in stable Rust (`c_variadic` is only needed to *define*
//!   them... it is not stable). Therefore: return `false` for 27 (the core then prints to
//!   stderr, which BaseOS captures). Record it under `refused`.
//! - `SET_GEOMETRY (37)` → update `base_width/height/aspect_ratio` in `AvInfo`.
//! - `SET_SYSTEM_AV_INFO (32)` → replace the whole `AvInfo` (fps/sample rate included).
//! - `GET_INPUT_BITMASKS (51 | experimental)` → `true` (we implement the bitmask query).
//! - `SET_INPUT_DESCRIPTORS (11)`, `SET_CONTROLLER_INFO (35)`, `SET_SUPPORT_ACHIEVEMENTS (42)`,
//!   `SET_SERIALIZATION_QUIRKS (44)`, `SET_MEMORY_MAPS (36)`, `GET_LANGUAGE (39)` (write
//!   `env.language`), `GET_CAN_DUPE (3)` (`*bool = true`), `SET_SUPPORT_NO_GAME (18)` (accept,
//!   ignore), `GET_FASTFORWARDING (79)` (`*bool = false`), `GET_AUDIO_VIDEO_ENABLE (47)`
//!   (`*int = 3`), `SET_MINIMUM_AUDIO_LATENCY (63)` → `true`.
//! - `GET_RUMBLE_INTERFACE (23)` → fill with a callback that records the last strength per
//!   motor in `Slot` (exposed later; for now just accept), return `true`.
//! - `GET_VFS_INTERFACE (45)`, `GET_HW_RENDER (14)`, `SET_HW_RENDER`, `GET_PERF_INTERFACE (28)`,
//!   `GET_LOCATION_INTERFACE`, `GET_CAMERA_INTERFACE`, disk control, netpacket, and anything
//!   unlisted → `false`, recorded in `refused`.
//! Every command is recorded once in `answered` or `refused` (dedup).
//!
//! ## Other callbacks
//! - `video_refresh(data, w, h, pitch)`: `data == NULL` means "dupe" — keep the last frame.
//!   Otherwise copy `pitch * h` bytes into `Slot.frame` (reuse the allocation).
//! - `audio_sample(l, r)` → push both; `audio_sample_batch(data, frames)` → extend with
//!   `frames * 2` samples, return `frames`.
//! - `input_poll` → nothing (masks are set by the host before `run`).
//! - `input_state(port, device, index, id)`: for `device == RETRO_DEVICE_JOYPAD (1)`:
//!   `id == RETRO_DEVICE_ID_JOYPAD_MASK (256)` → the port's mask as i16; otherwise bit `id`
//!   of the mask (1/0). Ports ≥ 2 and other devices → 0.
//!
//! ## `Core` lifecycle
//! `load(dylib, rom, env)`: open the library (`libloading`), resolve every `retro_*` symbol
//! used (API version must be 1), `retro_set_environment` **before** `retro_init` (cores
//! declare options there), then the other `retro_set_*` callbacks, `retro_init`,
//! `retro_get_system_info` (honour `need_fullpath`: if true pass the path with `data ==
//! NULL`, else read the ROM into memory and pass `data`/`size` — keep that buffer alive for
//! the core's lifetime), `retro_load_game`, `retro_get_system_av_info`, then
//! `retro_set_controller_port_device(0, JOYPAD)` and `(1, JOYPAD)`. Any failure → `Error`
//! and the library is dropped after `retro_deinit` if `retro_init` ran.
//! `Drop`: `retro_unload_game`, `retro_deinit`, clear the slot, then drop the library.
//!
//! ## Frame conversion helpers
//! `frame_to_rgba8` and `frame_rgb` handle all three formats (5-bit/6-bit channels
//! expanded with `(v << 3) | (v >> 2)` / `(v << 2) | (v >> 4)`).

use std::path::Path;

use crate::{AvInfo, CoreOption, Env, EnvLog, Error, Frame, JoypadMask, Memory, PixelFormat};

pub struct Core {
    _todo: (),
}

impl Core {
    /// Open `dylib`, initialise it with `env`, load `rom`. See the module doc.
    pub fn load(dylib: &Path, rom: &Path, env: Env) -> Result<Self, Error> {
        let _ = (dylib, rom, env);
        todo!("task 06")
    }

    /// `retro_get_system_info`'s library name and version, e.g. `("mGBA", "0.11-dev")`.
    pub fn name(&self) -> (String, String) {
        todo!("task 06")
    }

    pub fn av_info(&self) -> AvInfo {
        todo!("task 06")
    }

    pub fn pixel_format(&self) -> PixelFormat {
        todo!("task 06")
    }

    /// Set the joypad state for `port` (0 or 1) that the next `run` will report.
    pub fn set_input(&mut self, port: usize, mask: JoypadMask) {
        let _ = (port, mask);
        todo!("task 06")
    }

    /// One `retro_run`. Afterwards `frame()` has the picture (or the previous one, if the
    /// core duped) and `take_audio()` the samples produced.
    pub fn run(&mut self) {
        todo!("task 06")
    }

    /// The last frame delivered. `None` before the first non-dupe `video_refresh`.
    pub fn frame(&self) -> Option<&Frame> {
        todo!("task 06")
    }

    /// Interleaved stereo i16 produced since the last take. At `av_info().sample_rate`.
    pub fn take_audio(&mut self) -> Vec<i16> {
        todo!("task 06")
    }

    pub fn serialize_size(&self) -> usize {
        todo!("task 06")
    }

    pub fn serialize(&mut self) -> Result<Vec<u8>, Error> {
        todo!("task 06")
    }

    pub fn unserialize(&mut self, data: &[u8]) -> Result<(), Error> {
        let _ = data;
        todo!("task 06")
    }

    /// A copy of a memory region, `None` if the core exposes none of that kind.
    pub fn memory(&self, which: Memory) -> Option<Vec<u8>> {
        let _ = which;
        todo!("task 06")
    }

    /// Overwrite a memory region in place (e.g. restore save RAM). Errors if the sizes
    /// differ or the region does not exist.
    pub fn write_memory(&mut self, which: Memory, data: &[u8]) -> Result<(), Error> {
        let _ = (which, data);
        todo!("task 06")
    }

    /// Reset the game (`retro_reset`).
    pub fn reset(&mut self) {
        todo!("task 06")
    }

    /// The options the core declared, with current values applied.
    pub fn options(&self) -> Vec<CoreOption> {
        todo!("task 06")
    }

    pub fn option(&self, key: &str) -> Option<String> {
        let _ = key;
        todo!("task 06")
    }

    /// Change an option; the core picks it up at its next `GET_VARIABLE_UPDATE` poll.
    pub fn set_option(&mut self, key: &str, value: &str) {
        let _ = (key, value);
        todo!("task 06")
    }

    pub fn env_log(&self) -> EnvLog {
        todo!("task 06")
    }

    /// Whether the core wanted the ROM passed by path (`need_fullpath`).
    pub fn need_fullpath(&self) -> bool {
        todo!("task 06")
    }
}

pub(crate) fn frame_to_rgba8(f: &Frame) -> Vec<u8> {
    let _ = f;
    todo!("task 06")
}

pub(crate) fn frame_rgb(f: &Frame, x: u32, y: u32) -> [u8; 3] {
    let _ = (f, x, y);
    todo!("task 06")
}
