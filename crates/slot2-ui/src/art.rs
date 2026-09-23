//! Rasterised artwork, held for as long as something is drawing it.
//!
//! A cart is redrawn sixty times a second, and rasterising an SVG or uploading a texture at
//! that rate is not an option. The art is vector, so a different size is a different drawing
//! rather than a scaled one, and the size is part of what is cached.

use crate::svg;
use slot2_gfx::{Canvas, TexId};
use std::collections::HashMap;

/// Rasterised artwork, kept as long as it is being drawn at that size.
#[derive(Default)]
pub struct ArtCache {
    /// Keyed on (src_ptr, src_len, w, h).
    ///
    /// Keying on a `&str`'s contents means hashing the whole SVG on every lookup, sixty times a
    /// second. Key on the pointer and length instead, which is stable for the `include_str!`
    /// constants every caller passes; the size is part of the key either way.
    ///
    /// Note: A caller passing a freshly built `String` each frame would defeat this.
    textures: HashMap<(usize, usize, u32, u32), Option<TexId>>,
}

impl ArtCache {
    /// The texture for this drawing at this size, rasterising and uploading on first ask.
    /// `None` when the source will not parse — remembered, so a broken file is parsed once
    /// rather than every frame.
    pub fn mask(&mut self, canvas: &mut dyn Canvas, src: &str, w: u32, h: u32) -> Option<TexId> {
        let key = (src.as_ptr() as usize, src.len(), w, h);
        if let Some(tex) = self.textures.get(&key) {
            return *tex;
        }

        let tex = svg::rasterize(src, w, h).map(|mask| canvas.upload_alpha8(w, h, &mask.a));

        self.textures.insert(key, tex);
        tex
    }

    /// How many drawings have been asked for, counting the ones that turned out not to
    /// parse — those occupy the cache too, which is the point of remembering them.
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// Whether a source is known not to parse at this size. For the contract; it is also
    /// the honest way to ask "did we already give up on this".
    pub fn remembers_failure(&self, src: &str, w: u32, h: u32) -> bool {
        let key = (src.as_ptr() as usize, src.len(), w, h);
        matches!(self.textures.get(&key), Some(None))
    }

    /// Drop every texture. Call before the canvas goes away.
    pub fn clear(&mut self, canvas: &mut dyn Canvas) {
        for tex in self.textures.values().flatten() {
            canvas.free(*tex);
        }
        self.textures.clear();
    }
}
