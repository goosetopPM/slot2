//! The fontdue side. Implementation notes for task 02 (see tasks/02-text.md):
//!
//! - `Inner` holds `Vec<Slot>` where a slot is either `Loaded(name, fontdue::Font)`,
//!   `Lazy(PathBuf)` or `Failed(PathBuf)`, plus a glyph cache
//!   `HashMap<(FontId, char, u32 /* px*100 */), Glyph>` with `Glyph { metrics: fontdue::Metrics, data: Vec<u8> }`.
//! - `resolve(ch)`: walk slots in order; a `Lazy` slot is read + parsed on the way
//!   (`std::fs::read`, `Font::from_bytes(bytes, FontSettings::default())`); on failure mark
//!   `Failed`, `eprintln!("slot2-text: {path}: {err}")` once, and continue. A slot has the
//!   glyph when `font.lookup_glyph_index(ch) != 0`.
//! - Line metrics come from the FIRST loaded font: `font.horizontal_line_metrics(px)` →
//!   `ascent`, `descent` (fontdue's descent is negative; report it positive), and
//!   `line_height = (ascent + descent).ceil() as u32`. If the first slot is lazy and not yet
//!   loaded, load it for this purpose. If no font at all: ascent = descent = 0, height 0.
//! - Each char: `resolve` → glyph via cache or `font.rasterize(ch, px)`; advance is
//!   `metrics.advance_width`. Missing everywhere → try `TOFU` the same way; missing again →
//!   advance `px * 0.5`, draw nothing.
//! - `rasterize`: width = `measure.width.ceil().max(1.0) as u32`, height = `line_height`,
//!   baseline = `ascent.round() as u32`. For each glyph place its bitmap at
//!   `x = pen + metrics.xmin`, `y = baseline - metrics.height as i32 - metrics.ymin`
//!   (fontdue's `ymin` is the offset of the bitmap's bottom edge above the baseline, may be
//!   negative). Clip to the bitmap; composite with `max`. Then `pen += advance`.
//!   Keep `pen` as f32 and round each glyph's x placement.
//! - `measure` must not rasterise; use `font.metrics(ch, px)` for advances and cache
//!   nothing (or cache advances separately) — but it is fine to share the glyph cache if
//!   simpler. `cache_len` counts rasterised glyph entries only.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use fontdue::{Font, FontSettings, Metrics as FontMetrics};

use crate::{Bitmap, Error, FontId, Metrics, TOFU};

enum Slot {
    Loaded(Font),
    Lazy(PathBuf),
    Failed,
}

struct Glyph {
    metrics: FontMetrics,
    data: Vec<u8>,
}

pub struct Inner {
    slots: Vec<Slot>,
    cache: HashMap<(FontId, char, u32), Glyph>,
}

impl Inner {
    pub fn new() -> Self {
        Inner {
            slots: Vec::new(),
            cache: HashMap::new(),
        }
    }

    pub fn push_bytes(&mut self, name: &str, bytes: Vec<u8>) -> Result<FontId, Error> {
        let font = Font::from_bytes(bytes, FontSettings::default())
            .map_err(|_| Error::Parse(name.to_string()))?;
        let id = FontId(self.slots.len());
        self.slots.push(Slot::Loaded(font));
        Ok(id)
    }

    pub fn push_file(&mut self, path: &Path) -> Result<FontId, Error> {
        let bytes = fs::read(path).map_err(|e| Error::Io(path.to_path_buf(), e.to_string()))?;
        self.push_bytes(&path.to_string_lossy(), bytes)
    }

    pub fn push_lazy_file(&mut self, path: &Path) -> FontId {
        let id = FontId(self.slots.len());
        self.slots.push(Slot::Lazy(path.to_path_buf()));
        id
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_loaded(&self, id: FontId) -> bool {
        matches!(self.slots.get(id.0), Some(Slot::Loaded(..)))
    }

    pub fn resolve(&mut self, ch: char) -> Option<FontId> {
        for i in 0..self.slots.len() {
            let id = FontId(i);
            if let Some(font) = self.ensure_loaded(id) {
                if font.lookup_glyph_index(ch) != 0 {
                    return Some(id);
                }
            }
        }
        None
    }

    fn ensure_loaded(&mut self, id: FontId) -> Option<&Font> {
        let slot = self.slots.get_mut(id.0)?;
        if let Slot::Lazy(path) = slot {
            let p = path.clone();
            match fs::read(&p) {
                Ok(bytes) => match Font::from_bytes(bytes, FontSettings::default()) {
                    Ok(font) => {
                        *slot = Slot::Loaded(font);
                    }
                    Err(err) => {
                        eprintln!("slot2-text: {}: {}", p.display(), err);
                        *slot = Slot::Failed;
                    }
                },
                Err(err) => {
                    eprintln!("slot2-text: {}: {}", p.display(), err);
                    *slot = Slot::Failed;
                }
            }
        }

        match self.slots.get(id.0)? {
            Slot::Loaded(font) => Some(font),
            _ => None,
        }
    }

    fn line_metrics(&mut self, px: f32) -> (f32, f32, u32) {
        if self.slots.is_empty() {
            return (0.0, 0.0, 0);
        }
        if let Some(font) = self.ensure_loaded(FontId(0)) {
            if let Some(m) = font.horizontal_line_metrics(px) {
                let ascent = m.ascent;
                let descent = -m.descent;
                let line_height = (ascent + descent).ceil() as u32;
                return (ascent, descent, line_height);
            }
        }
        (0.0, 0.0, 0)
    }

    pub fn measure(&mut self, text: &str, px: f32) -> Metrics {
        let (ascent, descent, line_height) = self.line_metrics(px);
        let mut width = 0.0;
        for ch in text.chars() {
            if let Some(id) = self.resolve(ch) {
                if let Some(Slot::Loaded(font)) = self.slots.get(id.0) {
                    width += font.metrics(ch, px).advance_width;
                }
            } else if let Some(id) = self.resolve(TOFU) {
                if let Some(Slot::Loaded(font)) = self.slots.get(id.0) {
                    width += font.metrics(TOFU, px).advance_width;
                }
            } else {
                width += px * 0.5;
            }
        }
        Metrics {
            width,
            ascent,
            descent,
            line_height,
        }
    }

    pub fn rasterize(&mut self, text: &str, px: f32) -> Bitmap {
        let metrics = self.measure(text, px);
        let width = metrics.width.ceil().max(1.0) as u32;
        let height = metrics.line_height;
        let baseline = metrics.ascent.round() as u32;
        let mut data = vec![0u8; (width * height) as usize];

        let mut pen = 0.0f32;
        let px_key = (px * 100.0).round() as u32;

        for ch in text.chars() {
            let (res_ch, res_id) = if let Some(id) = self.resolve(ch) {
                (ch, Some(id))
            } else if let Some(id) = self.resolve(TOFU) {
                (TOFU, Some(id))
            } else {
                (ch, None)
            };

            if let Some(id) = res_id {
                let key = (id, res_ch, px_key);
                if !self.cache.contains_key(&key) {
                    if let Some(Slot::Loaded(font)) = self.slots.get(id.0) {
                        let (m, d) = font.rasterize(res_ch, px);
                        self.cache.insert(
                            key,
                            Glyph {
                                metrics: m,
                                data: d,
                            },
                        );
                    }
                }

                if let Some(glyph) = self.cache.get(&key) {
                    let m = &glyph.metrics;
                    let x0 = (pen.round() as i32) + m.xmin;
                    let y0 = baseline as i32 - m.height as i32 - m.ymin;

                    for row in 0..m.height {
                        for col in 0..m.width {
                            let dx = x0 + col as i32;
                            let dy = y0 + row as i32;
                            if dx >= 0 && dx < width as i32 && dy >= 0 && dy < height as i32 {
                                let src = glyph.data[row * m.width + col];
                                let dst = &mut data[(dy as u32 * width + dx as u32) as usize];
                                *dst = (*dst).max(src);
                            }
                        }
                    }
                    pen += m.advance_width;
                }
            } else {
                pen += px * 0.5;
            }
        }

        Bitmap {
            width,
            height,
            baseline,
            data,
        }
    }

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
}
