//! Card layout, scanning, saves and states, settings, atomic writes.
//!
//! The card is the only persistent thing SLOT2 has. Its layout (docs/DESIGN.md §9) is
//! compatible with the original slot's: `Games/<PLAT>/`, `Labels/`, `Saves/`, `States/`,
//! `BIOS/`, `System/`. Every write goes through [`atomic_write`] — a temp file beside the
//! target, then a rename — so a pulled card or a dead battery never leaves a half-written
//! save behind. Reads are best effort: a missing folder is an empty shelf, not an error.
//!
//! A numbered state can also be taken off the card into a [`StateBackup`] and put back later,
//! which is what the switcher's undo is built on. The bytes live in memory for as long as the
//! caller holds them: there is no trash folder, and nothing survives a power cut.

pub mod atomic;
pub mod card;
pub mod cheats;
pub mod ini;
pub mod platform;
pub mod settings;

pub use atomic::atomic_write;
pub use card::{Card, Cart, StateBackup, StateKind, StateNamespace, StateSlot, Thumb};
pub use cheats::Cheat;
pub use ini::Ini;
pub use platform::Platform;
pub use settings::{
    GameSettings, GlobalSettings, ScaleMode, ShaderPreset, DEFAULT_LANGUAGE,
    DEFAULT_UTC_OFFSET_MINUTES, DEFAULT_VOLUME_LEVEL, LANGUAGE_KEY, MAX_VOLUME_LEVEL,
    UTC_OFFSET_MINUTES_KEY, UTC_OFFSET_MINUTES_MAX, UTC_OFFSET_MINUTES_MIN,
};

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::path::PathBuf, std::io::Error),
    /// The data handed to a writer is not what the writer takes (wrong thumbnail size…).
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(p, e) => write!(f, "{}: {e}", p.display()),
            Error::Invalid(m) => write!(f, "invalid: {m}"),
        }
    }
}
impl std::error::Error for Error {}
