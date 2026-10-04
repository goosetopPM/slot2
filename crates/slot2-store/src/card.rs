//! The card: its folders, what is on the shelf, and where each cart's saves and states go.
//! Task 07 implements everything marked `todo!()`.
//!
//! Layout (relative to the root; `ensure_layout` creates all of these):
//! ```text
//! BIOS/  Games/<PLAT>/  Labels/<PLAT>/  Saves/  States/  System/  System/Fonts/  System/Lang/  System/cheats/<PLAT>/  Wallpapers/
//! ```
//! (`Saves/<PLAT>/` and `States/<PLAT>/<stem>/` are created on first write, not here.)
//!
//! Scanning (`scan`): every regular file directly in `Games/<PLAT>/` whose *last* extension the
//! platform accepts (the extension is matched case-blind), skipping names that start with `.`
//! or `._`. `stem` is that file name with exactly that one extension removed and every other
//! character kept — spaces, dots, brackets and the script itself included — and `title` is the
//! stem. Sorted by `title` in Rust `str` order, which is Unicode scalar value order: ASCII
//! upper before lower, Hangul syllables in 가나다 order. No locale collation, no natural-number
//! ordering, no case folding, no NFC/NFD normalisation; equal titles are ordered by extension
//! bytes, so one stem's two accepted extensions are deterministic.
//!
//! That stem is the key for everything the card keeps for one game:
//! `Labels/<PLAT>/<stem>.png`, `Saves/<PLAT>/<stem>.sav`, `States/<PLAT>/<stem>/`,
//! `System/games/<PLAT>/<stem>.ini` and `System/cheats/<PLAT>/<stem>.cht`. Two platforms with
//! the same stem are kept apart by the platform folder. Two accepted extensions of one stem on
//! one platform (SNES `.sfc` and `.smc`) are two carts that share every one of those paths:
//! the layout has room for one save per stem, and this is that limit, not a bug to fix here.
//!
//! Saves: `Saves/<PLAT>/<stem>.sav`. `read_save` → `None` if absent. `write_save` →
//! `atomic_write`, returns whether bytes were written.
//!
//! States: `States/<PLAT>/<stem>/resume.state` and `States/<PLAT>/<stem>/<n>.state`
//! (`n` ≥ 1), each with an optional `<same name>.png` thumbnail. One core's states are one
//! level deeper still — `States/<PLAT>/<stem>/<core>/…`, a `StateNamespace` — because a
//! libretro state is a core's own serialization and another core cannot read it; the flat
//! layout above is what a card written before cores could be chosen still holds, and
//! `adopt_legacy_states` moves it into a namespace once. `write_state` writes the
//! state atomically, then the PNG (RGBA8, `thumb.width * thumb.height * 4` bytes; any other
//! length → `Error::Invalid`) via the `png` crate, also atomically (encode into a `Vec`,
//! then `atomic_write`). `list_states` returns every `.state` in the folder, resume first,
//! then numbered ascending, each with `modified` from the file's mtime and `thumb` = the
//! PNG path if it exists. `read_state` → `None` if absent. `next_state_number` = 1 + the
//! highest numbered state (1 when none). `delete_state` removes a state and its thumbnail for
//! good; `take_state` removes a numbered one and hands back a `StateBackup` that
//! `restore_state` puts back, which is what the switcher's undo is built on. The bytes of a
//! backup live in memory for as long as the caller holds them: there is no trash folder, and
//! nothing survives a power cut.

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
    "System/cheats",
    "System/cheats/GB",
    "System/cheats/GBC",
    "System/cheats/GBA",
    "System/cheats/NES",
    "System/cheats/SNES",
    "System/cheats/MD",
    "System/cheats/SMS",
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

/// One core's slice of a game's states: the directory under `States/<PLAT>/<stem>/`.
///
/// libretro state bytes are a core's own serialization, and no core can read another's: the
/// same game on mGBA and on Gambatte has two sets of states that must never be confused for
/// one. This is the name of the directory that keeps them apart, and it is the only way an
/// unvalidated string can reach a scoped path — a namespace that could hold `/`, `\`, `..` or
/// a space would be a path traversal or a name that cannot be typed into a shell.
///
/// What is allowed is what a core's base name is made of: lowercase ASCII letters, digits and
/// underscores, 1 to 64 bytes. `mgba_libretro`, `gambatte_libretro`, `gpsp_libretro`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateNamespace(String);

impl StateNamespace {
    pub fn new(name: &str) -> Result<Self, Error> {
        if name.is_empty() || name.len() > 64 {
            return Err(Error::Invalid(format!(
                "a state namespace is 1 to 64 bytes, not {}",
                name.len()
            )));
        }
        if !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(Error::Invalid(
                "a state namespace is lowercase ASCII letters, digits and underscores".into(),
            ));
        }
        Ok(StateNamespace(name.to_owned()))
    }

    /// The directory name, read-only: a namespace that could be changed after it was checked
    /// would be no check at all.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The `*.state` file a kind lives in, inside one directory.
fn state_file(dir: &Path, kind: StateKind) -> PathBuf {
    dir.join(format!("{}.state", kind.file_stem()))
}

/// Every state in one directory, sorted. See the module doc for the rules.
fn states_in(dir: &Path) -> Vec<StateSlot> {
    let mut slots = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return slots;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("state") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let kind = if stem == "resume" {
            StateKind::Resume
        } else if let Ok(n) = stem.parse::<u32>() {
            StateKind::Numbered(n)
        } else {
            continue;
        };

        let thumb_path = path.with_extension("png");
        let thumb = thumb_path.exists().then_some(thumb_path);
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };

        slots.push(StateSlot {
            kind,
            path,
            thumb,
            modified,
        });
    }

    slots.sort_by_key(|s| s.kind);
    slots
}

/// The next numbered slot in one directory: one past the highest number there, or 1.
fn next_number_in(dir: &Path) -> u32 {
    states_in(dir)
        .iter()
        .filter_map(|s| match s.kind {
            StateKind::Numbered(n) => Some(n),
            _ => None,
        })
        .max()
        .map_or(1, |n| n + 1)
}

/// Write a state and, if one is given, its picture, both atomically.
fn write_state_files(path: &Path, data: &[u8], thumb: Option<Thumb<'_>>) -> Result<(), Error> {
    crate::atomic::atomic_write(path, data)?;

    if let Some(t) = thumb {
        if t.rgba.len() as u32 != t.width * t.height * 4 {
            return Err(Error::Invalid(format!(
                "thumb size mismatch: {} vs {}x{}x4",
                t.rgba.len(),
                t.width,
                t.height
            )));
        }
        let mut buf = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut buf, t.width, t.height);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            let mut wr = enc
                .write_header()
                .map_err(|e| Error::Invalid(e.to_string()))?;
            wr.write_image_data(t.rgba)
                .map_err(|e| Error::Invalid(e.to_string()))?;
        }
        crate::atomic::atomic_write(&path.with_extension("png"), &buf)?;
    }
    Ok(())
}

/// Remove a state and its picture, if they are there.
fn delete_state_files(path: &Path) -> Result<(), Error> {
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
    }
    let thumb = path.with_extension("png");
    if thumb.exists() {
        let _ = std::fs::remove_file(thumb);
    }
    Ok(())
}

/// The number a `*.state` stem names, when the plain listing would call it that: plain decimal
/// digits, positive, with no leading zero and no sign.
///
/// `01.state` and `+1.state` are not this, and are left where they are by an adoption rather
/// than renamed to a name they never had.
fn canonical_number(stem: &str) -> Option<u32> {
    let n: u32 = stem.parse().ok()?;
    (n >= 1 && stem == n.to_string()).then_some(n)
}

/// Put back what a failed adoption had already moved, newest first.
///
/// Best effort by design: the caller is being handed the error that happened, and a card that
/// refuses a rename back is broken in a way this cannot repair.
fn roll_back(moved: &[(PathBuf, PathBuf)]) {
    for (from, to) in moved.iter().rev() {
        let _ = std::fs::rename(to, from);
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

/// A numbered state that has been taken off the card, kept in memory so it can be put back.
///
/// The switcher's undo holds one of these for thirty seconds, so a delete is one backup, not a
/// history: there is no trash folder and nothing survives a power cut. It is opaque on purpose
/// — `kind()` is all a caller needs, and the bytes are the card's business.
pub struct StateBackup {
    /// Where it came from. Restoring anywhere else would be putting a game's save into another
    /// game's folder, so the origin travels with the bytes.
    root: PathBuf,
    cart: Cart,
    kind: StateKind,
    /// The core's namespace it was taken from, or `None` for a state that was still in the
    /// game's own flat directory when it was taken. A backup belongs to the one place it came
    /// from, so this travels with it like the cart does.
    namespace: Option<StateNamespace>,
    state: Vec<u8>,
    /// The thumbnail exactly as it was on disk: bytes, not a decoded picture. A card may hold
    /// something that is not a PNG at all — a screenshot from another frontend, a half-written
    /// file — and this layer has no business decoding or rewriting it.
    thumb: Option<Vec<u8>>,
}

impl StateBackup {
    /// Which numbered slot this was.
    pub fn kind(&self) -> StateKind {
        self.kind
    }
}

impl std::fmt::Debug for StateBackup {
    /// The sizes, not the bytes: a state is megabytes, and a log line is not.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateBackup")
            .field("kind", &self.kind)
            .field(
                "namespace",
                &self.namespace.as_ref().map(StateNamespace::as_str),
            )
            .field("state", &self.state.len())
            .field("thumb", &self.thumb.as_ref().map(Vec::len))
            .finish_non_exhaustive()
    }
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
        let mut count = 0;
        for dir in DIRS {
            let p = self.root.join(dir);
            if !p.exists() && std::fs::create_dir_all(&p).is_ok() {
                count += 1;
            }
        }
        count
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

    /// Per-game cheats: `System/cheats/<PLAT>/<stem>.cht` (D-21).
    pub fn cheat_path(&self, cart: &Cart) -> PathBuf {
        self.system_dir()
            .join("cheats")
            .join(cart.platform.folder())
            .join(format!("{}.cht", cart.stem))
    }

    /// The cheats this game's file describes, in the order the file indexes them.
    ///
    /// A game with no cheat file has no cheats, which is not a failure. A file that is there
    /// and cannot be read or understood is: half a list of cheats would be a lie about the
    /// file. See [`crate::cheats`] for the format.
    pub fn read_cheats(&self, cart: &Cart) -> Result<Vec<crate::Cheat>, Error> {
        crate::cheats::read(&self.cheat_path(cart))
    }

    /// The background for one shelf: `Wallpapers/<PLAT>.png`, or `Wallpapers/default.png`
    /// when the card has no picture for that platform.
    ///
    /// Per platform first, because L1 and R1 move between shelves and a Game Boy shelf that
    /// looks like one is M3's acceptance criterion. A card that wants one picture for
    /// everything names it `default` and stops there.
    pub fn wallpaper(&self, p: Platform) -> Option<PathBuf> {
        let p = self
            .root
            .join("Wallpapers")
            .join(format!("{}.png", p.folder()));
        if p.is_file() {
            return Some(p);
        }
        let default = self.root.join("Wallpapers").join("default.png");
        default.is_file().then_some(default)
    }

    /// `Labels/<PLAT>/<stem>.png` if it exists.
    ///
    /// Answered from the card rather than assembled by the caller, because whether the file
    /// is there is what decides between a scan of the real sticker and a printed one, and a
    /// path handed over unchecked would have the shelf find that out once a frame.
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
        let dir = self.games_dir(p);
        let mut carts = Vec::new();
        let Ok(read_dir) = std::fs::read_dir(dir) else {
            return carts;
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with('.') || name.starts_with("._") {
                continue;
            }
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if !p.accepts(ext) {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            carts.push(Cart {
                platform: p,
                stem: stem.to_owned(),
                title: stem.to_owned(),
                rom: path,
            });
        }

        carts.sort_by(|a, b| {
            a.title
                .cmp(&b.title)
                .then_with(|| a.rom.extension().cmp(&b.rom.extension()))
        });
        carts
    }

    pub fn save_path(&self, cart: &Cart) -> PathBuf {
        self.root
            .join("Saves")
            .join(cart.platform.folder())
            .join(format!("{}.sav", cart.stem))
    }

    pub fn read_save(&self, cart: &Cart) -> Option<Vec<u8>> {
        std::fs::read(self.save_path(cart)).ok()
    }

    pub fn write_save(&self, cart: &Cart, bytes: &[u8]) -> Result<bool, Error> {
        crate::atomic::atomic_write(&self.save_path(cart), bytes)
    }

    pub fn states_dir(&self, cart: &Cart) -> PathBuf {
        self.root
            .join("States")
            .join(cart.platform.folder())
            .join(&cart.stem)
    }

    pub fn state_path(&self, cart: &Cart, kind: StateKind) -> PathBuf {
        state_file(&self.states_dir(cart), kind)
    }

    pub fn list_states(&self, cart: &Cart) -> Vec<StateSlot> {
        states_in(&self.states_dir(cart))
    }

    pub fn next_state_number(&self, cart: &Cart) -> u32 {
        next_number_in(&self.states_dir(cart))
    }

    pub fn read_state(&self, cart: &Cart, kind: StateKind) -> Option<Vec<u8>> {
        std::fs::read(self.state_path(cart, kind)).ok()
    }

    pub fn write_state(
        &self,
        cart: &Cart,
        kind: StateKind,
        data: &[u8],
        thumb: Option<Thumb<'_>>,
    ) -> Result<(), Error> {
        write_state_files(&self.state_path(cart, kind), data, thumb)
    }

    pub fn delete_state(&self, cart: &Cart, kind: StateKind) -> Result<(), Error> {
        delete_state_files(&self.state_path(cart, kind))
    }

    // ------------------------------------------------------------ one core's slice

    /// Where one core keeps this game's states: `States/<PLAT>/<stem>/<namespace>/`.
    pub fn scoped_states_dir(&self, cart: &Cart, namespace: &StateNamespace) -> PathBuf {
        self.states_dir(cart).join(namespace.as_str())
    }

    pub fn scoped_state_path(
        &self,
        cart: &Cart,
        namespace: &StateNamespace,
        kind: StateKind,
    ) -> PathBuf {
        state_file(&self.scoped_states_dir(cart, namespace), kind)
    }

    /// This core's states for this game: never another core's, and never the flat ones.
    pub fn scoped_list_states(&self, cart: &Cart, namespace: &StateNamespace) -> Vec<StateSlot> {
        states_in(&self.scoped_states_dir(cart, namespace))
    }

    pub fn scoped_next_state_number(&self, cart: &Cart, namespace: &StateNamespace) -> u32 {
        next_number_in(&self.scoped_states_dir(cart, namespace))
    }

    pub fn scoped_read_state(
        &self,
        cart: &Cart,
        namespace: &StateNamespace,
        kind: StateKind,
    ) -> Option<Vec<u8>> {
        std::fs::read(self.scoped_state_path(cart, namespace, kind)).ok()
    }

    pub fn scoped_write_state(
        &self,
        cart: &Cart,
        namespace: &StateNamespace,
        kind: StateKind,
        data: &[u8],
        thumb: Option<Thumb<'_>>,
    ) -> Result<(), Error> {
        write_state_files(&self.scoped_state_path(cart, namespace, kind), data, thumb)
    }

    pub fn scoped_delete_state(
        &self,
        cart: &Cart,
        namespace: &StateNamespace,
        kind: StateKind,
    ) -> Result<(), Error> {
        delete_state_files(&self.scoped_state_path(cart, namespace, kind))
    }

    /// Take one core's numbered state off the card, keeping where it came from.
    pub fn scoped_take_state(
        &self,
        cart: &Cart,
        namespace: &StateNamespace,
        kind: StateKind,
    ) -> Result<Option<StateBackup>, Error> {
        let dir = self.scoped_states_dir(cart, namespace);
        self.take_state_from(cart, &dir, Some(namespace.clone()), kind)
    }

    /// Move this game's flat states into one core's namespace, once.
    ///
    /// The states a card already has were written by whichever core ran the game before cores
    /// could be chosen; they belong to that core, and the card cannot say which one it was. So
    /// the caller decides — the follow-up task passes the platform's default core — and this
    /// moves them there, once, the next time that core starts the game.
    ///
    /// Recognised are `resume.state`, a canonical positive `<n>.state` and each one's sibling
    /// PNG, directly in the game's own directory. Everything else is left where it is: an
    /// orphan picture with no state, a name that is not a number, `0.state`, a number that
    /// overflowed, another extension, and any subdirectory — including the namespaces
    /// themselves.
    ///
    /// Nothing is moved unless every destination is free: a refusal happens before the first
    /// rename, so a game's states are never split across two places by a half-done move. A
    /// rename that fails part way is rolled back, and the error is returned — never swallowed.
    /// The moves are renames inside one card, so the bytes and the timestamps are the ones that
    /// were already there.
    ///
    /// Returns how many slots moved. No states at all is `Ok(0)` and does not create the
    /// namespace directory.
    pub fn adopt_legacy_states(
        &self,
        cart: &Cart,
        namespace: &StateNamespace,
    ) -> Result<usize, Error> {
        let flat = self.states_dir(cart);
        let ns_dir = self.scoped_states_dir(cart, namespace);

        // No directory at all means no legacy states, and no directory to make. Anything else
        // that stops the directory being read — a file where the folder belongs, an entry that
        // cannot be opened — is a card this cannot answer for, and is reported rather than
        // hidden behind an empty result.
        if !flat.exists() {
            return Ok(0);
        }
        if !flat.is_dir() {
            return Err(Error::Io(
                flat.clone(),
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "the game's state directory is not a directory",
                ),
            ));
        }
        let entries = std::fs::read_dir(&flat).map_err(|e| Error::Io(flat.clone(), e))?;
        let mut legacy: Vec<(StateKind, PathBuf, bool)> = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| Error::Io(flat.clone(), e))?;
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("state") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let kind = if stem == "resume" {
                StateKind::Resume
            } else if let Some(n) = canonical_number(stem) {
                StateKind::Numbered(n)
            } else {
                continue;
            };
            let has_thumb = path.with_extension("png").is_file();
            legacy.push((kind, path, has_thumb));
        }
        if legacy.is_empty() {
            return Ok(0);
        }
        legacy.sort_by_key(|(kind, ..)| *kind);

        // Every destination, before the first rename: half a game's states in the old place and
        // half in the new one is the one outcome worse than either. Both the state and the
        // picture are checked for every slot — including a slot with no picture of its own, so
        // that an orphan `.png` already in the namespace cannot end up beside a state that just
        // arrived from somewhere else and be listed as its thumbnail.
        for (kind, _, _) in &legacy {
            let dest = state_file(&ns_dir, *kind);
            for path in [dest.clone(), dest.with_extension("png")] {
                if path.exists() {
                    return Err(Error::Invalid(format!(
                        "{} is already there",
                        path.display()
                    )));
                }
            }
        }

        std::fs::create_dir_all(&ns_dir).map_err(|e| Error::Io(ns_dir.clone(), e))?;

        let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
        for (kind, from, has_thumb) in &legacy {
            let to = state_file(&ns_dir, *kind);
            // The picture first: a `.state` is what a listing sees, so a slot becomes visible
            // in its new home only once the picture beside it is already there.
            if *has_thumb {
                let from_thumb = from.with_extension("png");
                let to_thumb = to.with_extension("png");
                if let Err(e) = std::fs::rename(&from_thumb, &to_thumb) {
                    roll_back(&moved);
                    return Err(Error::Io(from_thumb, e));
                }
                moved.push((from_thumb, to_thumb));
            }
            if let Err(e) = std::fs::rename(from, &to) {
                roll_back(&moved);
                return Err(Error::Io(from.clone(), e));
            }
            moved.push((from.clone(), to));
        }
        Ok(legacy.len())
    }

    /// Take a numbered state off the card, keeping it so it can be put back.
    ///
    /// The bytes are read before anything is removed, so a card pulled mid-delete cannot lose
    /// a state that never made it into the backup. `Ok(None)` when there is no state file; an
    /// orphan thumbnail beside a state that is not there is left alone — it is not this call's
    /// to remove.
    ///
    /// Resume is not a numbered state and is refused outright: it belongs to stopping and
    /// starting the frontend, and nothing a player deletes may take it with it.
    pub fn take_state(&self, cart: &Cart, kind: StateKind) -> Result<Option<StateBackup>, Error> {
        let dir = self.states_dir(cart);
        self.take_state_from(cart, &dir, None, kind)
    }

    /// The body of `take_state` and `scoped_take_state`: one directory, one origin.
    fn take_state_from(
        &self,
        cart: &Cart,
        dir: &Path,
        namespace: Option<StateNamespace>,
        kind: StateKind,
    ) -> Result<Option<StateBackup>, Error> {
        let StateKind::Numbered(_) = kind else {
            return Err(Error::Invalid("resume is not a numbered state".into()));
        };
        let state_path = state_file(dir, kind);
        let thumb_path = state_path.with_extension("png");

        let state = match std::fs::read(&state_path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(Error::Io(state_path, e)),
        };
        // Read before deleting anything: a thumbnail that cannot be read is a thumbnail that
        // cannot be put back, and the state is still on the card while this is asked.
        let thumb = match std::fs::read(&thumb_path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(Error::Io(thumb_path, e)),
        };

        // The picture goes first. If the state cannot be removed after it, putting the picture
        // back leaves the slot exactly as it was; the other order would leave the state gone
        // and its picture behind.
        if thumb.is_some() {
            std::fs::remove_file(&thumb_path).map_err(|e| Error::Io(thumb_path.clone(), e))?;
        }
        if let Err(e) = std::fs::remove_file(&state_path) {
            if let Some(bytes) = thumb.as_ref() {
                // Best effort: the caller is being handed the error that matters.
                let _ = crate::atomic::atomic_write(&thumb_path, bytes);
            }
            return Err(Error::Io(state_path, e));
        }

        Ok(Some(StateBackup {
            root: self.root.clone(),
            cart: cart.clone(),
            kind,
            namespace,
            state,
            thumb,
        }))
    }

    /// Put a taken state back, exactly as it was.
    ///
    /// Takes the backup by reference, so a refused or failed restore leaves it usable: the
    /// undo can try again, or keep offering itself.
    ///
    /// Nothing is overwritten. A state or a picture already at the target is somebody else's —
    /// a newer save, or what a half-finished delete left — and refusing is the only honest
    /// answer to that. The picture goes down first and the `.state` last, because the state
    /// file is what `list_states` sees: a slot becomes visible only once the picture beside it
    /// is already there.
    pub fn restore_state(&self, backup: &StateBackup) -> Result<(), Error> {
        if self.root != backup.root {
            return Err(Error::Invalid(format!(
                "a state from {} cannot be restored to {}",
                backup.root.display(),
                self.root.display()
            )));
        }
        // Back to the directory it came from, and only there: a core's namespace for a state
        // that was taken out of it, the game's own for one taken before namespaces existed.
        let dir = match backup.namespace.as_ref() {
            Some(namespace) => self.scoped_states_dir(&backup.cart, namespace),
            None => self.states_dir(&backup.cart),
        };
        let state_path = state_file(&dir, backup.kind);
        let thumb_path = state_path.with_extension("png");
        if state_path.exists() {
            return Err(Error::Invalid(format!(
                "{} is already there",
                state_path.display()
            )));
        }
        if thumb_path.exists() {
            return Err(Error::Invalid(format!(
                "{} is already there",
                thumb_path.display()
            )));
        }

        let wrote_thumb = match backup.thumb.as_ref() {
            Some(bytes) => {
                crate::atomic::atomic_write(&thumb_path, bytes)?;
                true
            }
            None => false,
        };
        if let Err(e) = crate::atomic::atomic_write(&state_path, &backup.state) {
            // The picture was this attempt's doing and the state never landed: take it out
            // again, so no thumbnail is left standing for a state nobody has.
            if wrote_thumb {
                let _ = std::fs::remove_file(&thumb_path);
            }
            return Err(e);
        }
        Ok(())
    }
}
