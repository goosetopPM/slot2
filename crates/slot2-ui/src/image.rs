//! Reading a picture off the card.
//!
//! Shared by the labels and the wallpaper, which want the same three things and differ only
//! in how big "too big" is: decode the PNG, bring it down to a size worth uploading, hand
//! back RGBA8.

use std::path::Path;

/// A decoded picture, already brought down to size.
pub struct Decoded {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// Decode `path` and box-filter it down by an integer factor until neither side is larger
/// than `max`. `None` when the file is not there or will not decode — a caller that cares
/// about the difference should have checked the file first.
///
/// An integer factor rather than a resample: it is a handful of adds per output pixel, it
/// needs no filter kernel, and at the ratios a scan or a photograph arrives at it is
/// indistinguishable from anything cleverer. What it is not allowed to do is reduce a side
/// to nothing — a texture zero pixels tall is a GL error at boot, and a 2048x3 picture is
/// not a picture anyone meant to make but it is one a card can hold.
pub fn decode_png(path: &Path, max: (u32, u32)) -> Option<Decoded> {
    todo!()
}
