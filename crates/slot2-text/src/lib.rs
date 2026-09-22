//! Font fallback chain, glyph cache, width measurement, line rasterisation.
//!
//! A `FontChain` is an ordered list of fonts. Each character is drawn by the first font in
//! the chain that has a glyph for it — Latin from the UI font, Hangul from the CJK font,
//! icons from the symbol font — which is what lets one line mix scripts without the UI
//! knowing which font a character came from (docs/DESIGN.md §8, D-14).
//!
//! Fonts may be *lazy*: registered by path and only parsed on the first character no earlier
//! font could draw. The CJK font is several megabytes and fontdue parses every glyph up
//! front, so a boot that never shows a Korean or Japanese title never pays for it.
//!
//! Rendering is CPU: `rasterize` returns an 8-bit coverage bitmap for one line of text,
//! which the caller uploads as a texture ("face"). Glyph bitmaps are cached per
//! (font, char, size) so a line that changes one character re-rasterises one glyph.

use std::fmt;
use std::path::{Path, PathBuf};

/// Position of a font in the chain. Stable for the chain's lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub usize);

/// What a missing glyph is drawn as: WHITE SQUARE, from the first font that has it. If no
/// font has it either, the character takes half an em of blank advance and draws nothing.
pub const TOFU: char = '\u{25A1}';

#[derive(Debug)]
pub enum Error {
    /// The file could not be read. Carries the path and the io error text.
    Io(PathBuf, String),
    /// The bytes are not a font fontdue can parse. Carries the font's name.
    Parse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(p, e) => write!(f, "{}: {e}", p.display()),
            Error::Parse(n) => write!(f, "{n}: not a font"),
        }
    }
}
impl std::error::Error for Error {}

/// Size of one line of text at a pixel size, before rasterising.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// Sum of advances, in pixels. Fractional; callers round up for a texture.
    pub width: f32,
    /// Distance from baseline to the top of the line box, from the *first* font in the
    /// chain (the UI font sets the rhythm; fallback fonts must fit inside it).
    pub ascent: f32,
    /// Distance from baseline to the bottom of the line box, positive.
    pub descent: f32,
    /// `ascent + descent`, rounded up to whole pixels.
    pub line_height: u32,
}

/// One rasterised line: 8-bit coverage, row-major, `width * height` bytes, 0 = transparent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    /// Row of the baseline, counted from the top. Glyphs sit on it.
    pub baseline: u32,
    pub data: Vec<u8>,
}

impl Bitmap {
    pub fn get(&self, x: u32, y: u32) -> u8 {
        self.data[(y * self.width + x) as usize]
    }

    /// Number of pixels with any coverage. A sanity measure for tests.
    pub fn ink(&self) -> usize {
        self.data.iter().filter(|&&a| a > 0).count()
    }
}

/// The chain. Owns its fonts and its glyph cache.
pub struct FontChain {
    inner: imp::Inner,
}

impl Default for FontChain {
    fn default() -> Self {
        Self::new()
    }
}

impl FontChain {
    pub fn new() -> Self {
        FontChain {
            inner: imp::Inner::new(),
        }
    }

    /// Append a font from bytes already in memory (e.g. `include_bytes!`). Parsed now.
    pub fn push_bytes(&mut self, name: &str, bytes: Vec<u8>) -> Result<FontId, Error> {
        self.inner.push_bytes(name, bytes)
    }

    /// Append a font from a file. Parsed now.
    pub fn push_file(&mut self, path: &Path) -> Result<FontId, Error> {
        self.inner.push_file(path)
    }

    /// Append a font by path without reading it. It is read and parsed the first time a
    /// character reaches it in the chain. A file that then fails to load is skipped for the
    /// rest of the chain's life (logged once on stderr), never retried.
    pub fn push_lazy_file(&mut self, path: &Path) -> FontId {
        self.inner.push_lazy_file(path)
    }

    /// Fonts registered, lazy ones included.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether a font's bytes have been parsed yet. Always true for eager fonts.
    pub fn is_loaded(&self, id: FontId) -> bool {
        self.inner.is_loaded(id)
    }

    /// The font that will draw `ch`: the first in the chain with a glyph for it, loading
    /// lazy fonts on the way. `None` when no font has it (the caller gets `TOFU`).
    pub fn resolve(&mut self, ch: char) -> Option<FontId> {
        self.inner.resolve(ch)
    }

    /// Measure one line. Newlines are not special; they measure as a missing glyph.
    /// An empty string has width 0 but still the first font's line height.
    pub fn measure(&mut self, text: &str, px: f32) -> Metrics {
        self.inner.measure(text, px)
    }

    /// Rasterise one line at `px`. The bitmap is `ceil(measure.width)` wide (at least 1)
    /// and `measure.line_height` tall, glyphs placed left to right on the baseline by
    /// their advances (no kerning, no shaping — enough for UI labels). Glyph coverage is
    /// composited with `max`, so overlapping glyphs never exceed 255.
    pub fn rasterize(&mut self, text: &str, px: f32) -> Bitmap {
        self.inner.rasterize(text, px)
    }

    /// Entries in the glyph cache. Grows by one per new (font, char, size) rasterised.
    pub fn cache_len(&self) -> usize {
        self.inner.cache_len()
    }
}

impl fmt::Debug for FontChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontChain")
            .field("fonts", &self.len())
            .field("cache", &self.cache_len())
            .finish()
    }
}

mod imp;
