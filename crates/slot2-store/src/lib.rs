//! Card layout, scanning, saves and states, settings, atomic writes.
//!
//! The card is the only persistent thing SLOT2 has. Its layout (docs/DESIGN.md §9) is
//! compatible with the original slot's: `Games/<PLAT>/`, `Labels/`, `Saves/`, `States/`,
//! `BIOS/`, `System/`. Every write goes through [`atomic_write`] — a temp file beside the
//! target, then a rename — so a pulled card or a dead battery never leaves a half-written
//! save behind. Reads are best effort: a missing folder is an empty shelf, not an error.

pub mod atomic;
pub mod card;
pub mod ini;
pub mod platform;

pub use atomic::atomic_write;
pub use card::{Card, Cart, StateKind, StateSlot, Thumb};
pub use ini::Ini;
pub use platform::Platform;

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
