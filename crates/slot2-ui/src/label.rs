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

        let res = load_png(canvas, path, plate);
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

fn load_png(canvas: &mut dyn Canvas, path: &Path, (pw, ph): (u32, u32)) -> Option<Art> {
    let file = std::fs::File::open(path).ok()?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width, info.height);

    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity((w * h * 4) as usize);
            for i in 0..(w * h) as usize {
                out.push(buf[i * 3]);
                out.push(buf[i * 3 + 1]);
                out.push(buf[i * 3 + 2]);
                out.push(0xFF);
            }
            out
        }
        // Grey is a colour type a scanner produces, not a broken file. `EXPAND` has already
        // widened a sub-8-bit grey to 8 and split out any tRNS alpha, so what is left is one
        // or two channels to spread across three.
        png::ColorType::Grayscale | png::ColorType::GrayscaleAlpha => {
            let step = if info.color_type == png::ColorType::Grayscale {
                1
            } else {
                2
            };
            let mut out = Vec::with_capacity((w * h * 4) as usize);
            for i in 0..(w * h) as usize {
                let g = buf[i * step];
                out.extend_from_slice(&[g, g, g]);
                out.push(if step == 2 { buf[i * step + 1] } else { 0xFF });
            }
            out
        }
        _ => return None,
    };

    let fw = (w as f32 / (pw * MAX_OVERSAMPLE) as f32).ceil() as u32;
    let fh = (h as f32 / (ph * MAX_OVERSAMPLE) as f32).ceil() as u32;
    // Never past the point where a side disappears. A 2048x3 label is not a label anyone
    // meant to make, but uploading a texture zero pixels tall is a GL error at boot.
    let f = fw.max(fh).max(1).min(w.max(1)).min(h.max(1));

    let (nw, nh, filtered) = box_filter(w, h, &rgba, f);
    let tex = canvas.upload_rgba8(nw, nh, &filtered);
    Some(Art { tex, w: nw, h: nh })
}

fn box_filter(w: u32, h: u32, rgba: &[u8], f: u32) -> (u32, u32, Vec<u8>) {
    if f <= 1 {
        return (w, h, rgba.to_vec());
    }
    let nw = w / f;
    let nh = h / f;
    let mut out = Vec::with_capacity((nw * nh * 4) as usize);
    for y in 0..nh {
        for x in 0..nw {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut a = 0u32;
            for fy in 0..f {
                for fx in 0..f {
                    let i = (((y * f + fy) * w + (x * f + fx)) * 4) as usize;
                    r += rgba[i] as u32;
                    g += rgba[i + 1] as u32;
                    b += rgba[i + 2] as u32;
                    a += rgba[i + 3] as u32;
                }
            }
            let den = f * f;
            out.push((r / den) as u8);
            out.push((g / den) as u8);
            out.push((b / den) as u8);
            out.push((a / den) as u8);
        }
    }
    (nw, nh, out)
}
