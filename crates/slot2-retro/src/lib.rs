//! Core-agnostic libretro host.
//!
//! A [`Core`] is one loaded libretro dynamic library with one game in it. The host owns the
//! callbacks the core needs (environment, video, audio, input, log), keeps the last frame,
//! buffers audio, and answers the core's questions from a small, explicit table: system
//! directory, save directory, variables (core options), pixel format, geometry changes. It
//! knows nothing about any particular core; what mGBA or snes9x need beyond the standard
//! lives in `quirks` (M2) as option presets and input maps handed in through this API.
//!
//! Threading: a `Core` is `Send` but not `Sync`; the frame loop owns it. libretro cores
//! use global state, so at most one instance per library should be alive at a time
//! (`Core::load` refuses a second load of the same path while the first is alive).
//!
//! Design: docs/DESIGN.md §6, decisions D-03, D-05.

mod ffi;
mod host;
pub mod registry;

pub use host::Core;
pub use registry::{
    def, joypad_bit, mask_for, Core as CoreId, LogicalButton, Platform, PlatformDef, PLATFORMS,
};

use std::fmt;
use std::path::PathBuf;

impl fmt::Debug for Core {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, version) = self.name();
        f.debug_struct("Core")
            .field("name", &name)
            .field("version", &version)
            .finish_non_exhaustive()
    }
}

/// libretro `RETRO_DEVICE_ID_JOYPAD_*` bit order, as a mask for `retro_input_state` and the
/// bitmask extension. One per port.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct JoypadMask(pub u16);

impl JoypadMask {
    pub const B: u16 = 1 << 0;
    pub const Y: u16 = 1 << 1;
    pub const SELECT: u16 = 1 << 2;
    pub const START: u16 = 1 << 3;
    pub const UP: u16 = 1 << 4;
    pub const DOWN: u16 = 1 << 5;
    pub const LEFT: u16 = 1 << 6;
    pub const RIGHT: u16 = 1 << 7;
    pub const A: u16 = 1 << 8;
    pub const X: u16 = 1 << 9;
    pub const L: u16 = 1 << 10;
    pub const R: u16 = 1 << 11;
    pub const L2: u16 = 1 << 12;
    pub const R2: u16 = 1 << 13;
    pub const L3: u16 = 1 << 14;
    pub const R3: u16 = 1 << 15;

    pub fn with(self, bit: u16) -> Self {
        JoypadMask(self.0 | bit)
    }
}

/// The pixel format a core delivers frames in. Set once by the core via
/// `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT`; the default when it never asks is `Rgb1555`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// 16-bit `0RRRRRGGGGGBBBBB`.
    Rgb1555,
    /// 32-bit little-endian, bytes B G R X in memory.
    Xrgb8888,
    /// 16-bit `RRRRRGGGGGGBBBBB`.
    Rgb565,
}

impl PixelFormat {
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            PixelFormat::Rgb1555 | PixelFormat::Rgb565 => 2,
            PixelFormat::Xrgb8888 => 4,
        }
    }
}

/// The last picture the core produced. A copy: libretro only guarantees the core's buffer
/// during the callback. `data` is `pitch * height` bytes in `format`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// Bytes per row in `data`, ≥ `width * bytes_per_pixel`.
    pub pitch: usize,
    pub format: PixelFormat,
    pub data: Vec<u8>,
}

impl Frame {
    /// Tightly packed RGBA8 (straight alpha 255), row-major, top row first — what
    /// `Canvas::upload_rgba8` wants. Converts from any `format`.
    pub fn to_rgba8(&self) -> Vec<u8> {
        host::frame_to_rgba8(self)
    }

    /// The pixel at `(x, y)` as RGB8, for tests and debugging.
    pub fn rgb(&self, x: u32, y: u32) -> [u8; 3] {
        host::frame_rgb(self, x, y)
    }
}

/// Timing and geometry the core reports. `SET_GEOMETRY` updates the size fields mid-game;
/// `SET_SYSTEM_AV_INFO` can update all of them.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AvInfo {
    pub fps: f64,
    pub sample_rate: f64,
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    /// Pixel aspect ratio the core wants: `display width / display height` of the image
    /// (libretro's `aspect_ratio`), or 0 when the core leaves it to width/height.
    pub aspect_ratio: f32,
}

/// One core option (variable) as the core declared it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreOption {
    pub key: String,
    /// Human description, without the `; a|b|c` tail of the v0 format.
    pub description: String,
    pub values: Vec<String>,
    pub default: String,
}

/// Memory regions a core may expose via `retro_get_memory_data/size`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Memory {
    SaveRam,
    Rtc,
    SystemRam,
    VideoRam,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    /// The library could not be opened or lacks a libretro symbol.
    Load(String),
    /// The core refused the game, or a required environment call.
    Game(String),
    /// Serialize/unserialize failed.
    State(String),
    /// A second instance of the same library while one is alive.
    Busy(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io: {e}"),
            Error::Load(m) => write!(f, "load: {m}"),
            Error::Game(m) => write!(f, "game: {m}"),
            Error::State(m) => write!(f, "state: {m}"),
            Error::Busy(p) => write!(f, "{} is already loaded", p.display()),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// What the host tells the core about its surroundings before the game loads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Env {
    /// `RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY`: BIOS files live here.
    pub system_dir: PathBuf,
    /// `RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY`.
    pub save_dir: PathBuf,
    /// Core options to set before load: `(key, value)`. Unknown keys are kept and reported
    /// through `GET_VARIABLE` anyway (cores query by key).
    pub options: Vec<(String, String)>,
    /// `RETRO_ENVIRONMENT_GET_LANGUAGE` answer (libretro `retro_language`; 0 = English).
    pub language: u32,
}

impl Default for Env {
    fn default() -> Self {
        Env {
            system_dir: PathBuf::from("."),
            save_dir: PathBuf::from("."),
            options: Vec::new(),
            language: 0,
        }
    }
}

/// Host-side log of what the core asked for, so a test (or the boot log) can see which
/// environment calls a core depends on. Numbers are `RETRO_ENVIRONMENT_*` commands with
/// the experimental bit masked off.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvLog {
    pub answered: Vec<u32>,
    pub refused: Vec<u32>,
}
