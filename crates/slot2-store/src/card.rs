//! The card: its folders, what is on the shelf, and where each cart's saves and states go.
//! Task 07 implements everything marked `todo!()`.
//!
//! Layout (relative to the root; `ensure_layout` creates all of these):
//! ```text
//! BIOS/  Games/<PLAT>/  Labels/<PLAT>/  Saves/  States/  System/  System/Fonts/  System/Lang/  Wallpapers/
//! ```
//! (`Saves/<PLAT>/` and `States/<PLAT>/<stem>/` are created on first write, not here.)
//!
//! Scanning (`scan`): every regular file directly in `Games/<PLAT>/` whose extension the
//! platform accepts, skipping names that start with `.` or `._`. `stem` is the file name
//! without its extension; `title` is the stem (a later task may read a titles file).
//! Sorted by `title` with plain `str` ordering (code points — Hangul syllables are already
//! in 가나다 order), stable, then by extension for equal stems.
//!
//! Saves: `Saves/<PLAT>/<stem>.sav`. `read_save` → `None` if absent. `write_save` →
//! `atomic_write`, returns whether bytes were written.
//!
//! States: `States/<PLAT>/<stem>/resume.state` and `States/<PLAT>/<stem>/<n>.state`
//! (`n` ≥ 1), each with an optional `<same name>.png` thumbnail. `write_state` writes the
//! state atomically, then the PNG (RGBA8, `thumb.width * thumb.height * 4` bytes; any other
//! length → `Error::Invalid`) via the `png` crate, also atomically (encode into a `Vec`,
//! then `atomic_write`). `list_states` returns every `.state` in the folder, resume first,
//! then numbered ascending, each with `modified` from the file's mtime and `thumb` = the
//! PNG path if it exists. `read_state` → `None` if absent. `next_state_number` = 1 + the
//! highest numbered state (1 when none). `delete_state` removes the state and its thumbnail.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::{Error, Platform};

/// Folders `ensure_layout` creates, parents first.
pub const DIRS: &[&str] = &[
    "BIOS",
    "Games",
    "Games/GB",
    "Games/GBC",
    "Games/GBA",
    "Games/NES",
    "Games/SNES",
    "Games/MD",
    "Games/SMS",
    "Labels",
    "Labels/GB",
    "Labels/GBC",
    "Labels/GBA",
    "Labels/NES",
    "Labels/SNES",
    "Labels/MD",
    "Labels/SMS",
    "Saves",
    "States",
    "System",
    "System/Fonts",
    "System/Lang",
    "Wallpapers",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cart {
    pub platform: Platform,
    /// File name without extension. The key for saves, states, labels and settings.
    pub stem: String,
    /// What the shelf shows.
    pub title: String,
    pub rom: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StateKind {
    Resume,
    Numbered(u32),
}

impl StateKind {
    /// `resume` or the number, as the file stem.
    pub fn file_stem(self) -> String {
        match self {
            StateKind::Resume => "resume".into(),
            StateKind::Numbered(n) => n.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateSlot {
    pub kind: StateKind,
    pub path: PathBuf,
    pub thumb: Option<PathBuf>,
    pub modified: SystemTime,
}

/// An RGBA8 image to store beside a state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thumb<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    root: PathBuf,
}

impl Card {
    pub fn new(root: impl Into<PathBuf>) -> Card {
        Card { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create every folder in `DIRS`. Best effort: a read-only card is an empty shelf, not
    /// a boot failure, so this never errors; it returns how many folders it created.
    pub fn ensure_layout(&self) -> usize {
        todo!("task 07")
    }

    pub fn games_dir(&self, p: Platform) -> PathBuf {
        self.root.join("Games").join(p.folder())
    }

    pub fn bios_dir(&self) -> PathBuf {
        self.root.join("BIOS")
    }

    pub fn system_dir(&self) -> PathBuf {
        self.root.join("System")
    }

    pub fn settings_path(&self) -> PathBuf {
        self.system_dir().join("slot2.ini")
    }

    /// Per-game settings: `System/games/<PLAT>/<stem>.ini`.
    pub fn game_settings_path(&self, cart: &Cart) -> PathBuf {
        self.system_dir()
            .join("games")
            .join(cart.platform.folder())
            .join(format!("{}.ini", cart.stem))
    }

    /// `Labels/<PLAT>/<stem>.png` if it exists.
    pub fn label(&self, cart: &Cart) -> Option<PathBuf> {
        let p = self
            .root
            .join("Labels")
            .join(cart.platform.folder())
            .join(format!("{}.png", cart.stem));
        p.is_file().then_some(p)
    }

    /// Everything on one platform's shelf, sorted. See the module doc.
    pub fn scan(&self, p: Platform) -> Vec<Cart> {
        let _ = p;
        todo!("task 07")
    }

    pub fn save_path(&self, cart: &Cart) -> PathBuf {
        self.root
            .join("Saves")
            .join(cart.platform.folder())
            .join(format!("{}.sav", cart.stem))
    }

    pub fn read_save(&self, cart: &Cart) -> Option<Vec<u8>> {
        let _ = cart;
        todo!("task 07")
    }

    pub fn write_save(&self, cart: &Cart, bytes: &[u8]) -> Result<bool, Error> {
        let _ = (cart, bytes);
        todo!("task 07")
    }

    pub fn states_dir(&self, cart: &Cart) -> PathBuf {
        self.root
            .join("States")
            .join(cart.platform.folder())
            .join(&cart.stem)
    }

    pub fn state_path(&self, cart: &Cart, kind: StateKind) -> PathBuf {
        self.states_dir(cart).join(format!("{}.state", kind.file_stem()))
    }

    pub fn list_states(&self, cart: &Cart) -> Vec<StateSlot> {
        let _ = cart;
        todo!("task 07")
    }

    pub fn next_state_number(&self, cart: &Cart) -> u32 {
        let _ = cart;
        todo!("task 07")
    }

    pub fn read_state(&self, cart: &Cart, kind: StateKind) -> Option<Vec<u8>> {
        let _ = (cart, kind);
        todo!("task 07")
    }

    pub fn write_state(&self, cart: &Cart, kind: StateKind, data: &[u8], thumb: Option<Thumb<'_>>) -> Result<(), Error> {
        let _ = (cart, kind, data, thumb);
        todo!("task 07")
    }

    pub fn delete_state(&self, cart: &Cart, kind: StateKind) -> Result<(), Error> {
        let _ = (cart, kind);
        todo!("task 07")
    }
}
