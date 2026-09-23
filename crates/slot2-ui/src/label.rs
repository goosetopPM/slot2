//! What is on the front of a cartridge.
//!
//! A card can carry a scan of the real sticker at `Labels/<PLAT>/<stem>.png`. Most will not,
//! for most games, so the fallback is not an error state — it is the normal one, and it has
//! to look deliberate. A *printed* label is a plate in a colour derived from the title with
//! the title on it: no two games alike, nothing to download, and a shelf that reads at a
//! glance because the colours differ even when the words are too small.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use slot2_gfx::{Canvas, TexId};

/// How many label textures are kept at once.
///
/// Seven carts can be on screen and the row draws them from a ring, so a handful more than
/// that covers a scroll without thrashing. It is a cap rather than a total: a card with two
/// hundred games would otherwise hold two hundred textures, and on a Mali sharing system
/// memory with everything else that is not a cache, it is a leak with a polite name.
pub const CACHE_MAX: usize = 12;

/// The largest a decoded label is kept at, as a multiple of the plate it is drawn on.
///
/// Scans come at whatever size a scanner produced. Uploading a 2048-square PNG to draw it
/// two hundred pixels wide costs sixteen megabytes to look no better, so anything larger is
/// box-filtered down by an integer factor first. Two rather than one because the plate grows
/// with the panel and a cart at rest is drawn larger than one beside it.
pub const MAX_OVERSAMPLE: u32 = 2;

/// A dumped filename carries region and revision tags, and they are facts about the dump
/// rather than about the game. `Advance Wars (USA, Europe) (Rev 1)` is `Advance Wars` on the
/// shelf, and the filename keeps the rest.
///
/// A bare hyphen is part of a word — `Spider-Man` keeps its own — but a spaced one separates
/// a title from a subtitle and goes.
pub fn clean_title(stem: &str) -> String {
    todo!()
}

/// The colour of a printed label. Derived from the title, so it is the same every boot and
/// two games are the same colour only by coincidence.
///
/// Mid saturation and value: a label is a background for dark text, and a fully saturated
/// one at handheld brightness reads as a warning light rather than a sticker.
pub fn printed_colour(title: &str) -> [u8; 3] {
    todo!()
}

/// Decoded label art, uploaded once and kept while it is being drawn.
#[derive(Default)]
pub struct LabelCache {
    art: HashMap<PathBuf, Option<Art>>,
    /// Bumped on every lookup; the least recently asked for is what gets dropped.
    clock: u64,
}

/// One label's texture and the size it was decoded at.
#[derive(Clone, Copy, Debug)]
pub struct Art {
    pub tex: TexId,
    pub w: u32,
    pub h: u32,
    used: u64,
}

impl LabelCache {
    /// The art at `path`, decoding and uploading on first ask. `None` when there is no file
    /// or it will not decode — remembered either way, so a broken file is opened once rather
    /// than every frame.
    ///
    /// `plate` is the size it will be drawn at, which decides how far the decoded image is
    /// scaled down before it is uploaded. It does not decide the texture's size: a label is
    /// a picture and gets scaled by the canvas like every other quad, so a cart that grows
    /// as it becomes the selection does not re-upload.
    pub fn art(&mut self, canvas: &mut dyn Canvas, path: &Path, plate: (u32, u32)) -> Option<Art> {
        todo!()
    }

    /// How many labels are held, counting the ones that turned out not to decode.
    pub fn len(&self) -> usize {
        self.art.len()
    }

    pub fn is_empty(&self) -> bool {
        self.art.is_empty()
    }
}
