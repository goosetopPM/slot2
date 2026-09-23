//! What is on the front of a cartridge.
//!
//! A card can carry a scan of the real sticker at `Labels/<PLAT>/<stem>.png`. Most will not,
//! for most games, so the fallback is not an error state — it is the normal one, and it has
//! to look deliberate. A *printed* label is a plate in a colour derived from the title with
//! the title on it: no two games alike, nothing to download, and a shelf that reads at a
//! glance because the colours differ even when the words are too small.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::image;
use slot2_gfx::{Canvas, TexId};

/// How many label textures are kept at once.
pub const CACHE_MAX: usize = 12;

/// The largest a decoded label is kept at, as a multiple of the plate it is drawn on.
pub const MAX_OVERSAMPLE: u32 = 2;

/// A dumped filename carries region and revision tags, and they are facts about the dump
/// rather than about the game. `Advance Wars (USA, Europe) (Rev 1)` is `Advance Wars` on the
/// shelf, and the filename keeps the rest.
///
/// A bare hyphen is part of a word — `Spider-Man` keeps its own — but a spaced one separates
/// a title from a subtitle, and the shelf has room for the title.
///
/// A name that cleans away to nothing comes back as the stem. A messy name is worse than a
/// tidy one and better than a blank where a game should be.
pub fn clean_title(stem: &str) -> String {
    let mut bare = String::with_capacity(stem.len());
    let mut depth = 0u32;
    for ch in stem.chars() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => bare.push(ch),
            _ => {}
        }
    }

    let mut out = String::with_capacity(bare.len());
    for word in bare.split_whitespace().filter(|w| *w != "-") {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    if out.is_empty() {
        stem.to_string()
    } else {
        out
    }
}

/// The colour of a printed label. Derived from the title, so it is the same every boot and
/// two games share one only by coincidence.
///
/// Plain forward FNV-1a. An earlier version hashed the bytes backwards with a comment saying
/// it separated *these test titles* better, which is what a hash tuned to a test looks like:
/// it says nothing about any other eight names, and the test it was tuned to was itself a
/// coin toss. What the colours have to do is fill the wheel, and any decent hash does that.
///
/// Mid saturation and value, because a label is a background for dark text: fully saturated
/// at handheld brightness is a warning light, and near-white is a blank sticker.
pub fn printed_colour(title: &str) -> [u8; 3] {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &b in title.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hsv_to_rgb((hash % 360) as f32, 0.52, 0.74)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}

/// Decoded label art, uploaded once and kept while it is being drawn.
#[derive(Default)]
pub struct LabelCache {
    art: HashMap<PathBuf, (Option<Art>, u64)>,
    /// Bumped on every lookup.
    clock: u64,
}

/// One label's texture and the size it was decoded at.
#[derive(Clone, Copy, Debug)]
pub struct Art {
    pub tex: TexId,
    pub w: u32,
    pub h: u32,
}

impl LabelCache {
    pub fn art(&mut self, canvas: &mut dyn Canvas, path: &Path, plate: (u32, u32)) -> Option<Art> {
        if let Some((art, used)) = self.art.get_mut(path) {
            *used = self.clock;
            self.clock += 1;
            return *art;
        }

        if self.art.len() >= CACHE_MAX {
            let oldest = self
                .art
                .iter()
                .min_by_key(|(_, (_, used))| *used)
                .map(|(p, _)| p.clone());
            if let Some(p) = oldest {
                if let Some((Some(art), _)) = self.art.remove(&p) {
                    canvas.free(art.tex);
                }
            }
        }

        let res = load_png(
            canvas,
            path,
            (plate.0 * MAX_OVERSAMPLE, plate.1 * MAX_OVERSAMPLE),
        );
        self.art.insert(path.to_path_buf(), (res, self.clock));
        self.clock += 1;
        res
    }

    pub fn len(&self) -> usize {
        self.art.len()
    }

    pub fn is_empty(&self) -> bool {
        self.art.is_empty()
    }
}

fn load_png(canvas: &mut dyn Canvas, path: &Path, max: (u32, u32)) -> Option<Art> {
    let decoded = image::decode_png(path, max)?;
    let tex = canvas.upload_rgba8(decoded.w, decoded.h, &decoded.rgba);
    Some(Art {
        tex,
        w: decoded.w,
        h: decoded.h,
    })
}
