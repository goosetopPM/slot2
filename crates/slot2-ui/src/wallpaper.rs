//! What the shelf stands on.
//!
//! Not decoration. Nothing else paints the whole panel — the slot covers its band, the carts
//! cover themselves, and until now the rest of the frame was whatever the last frame left
//! there. On a double-buffered device that is two frames of history alternating behind a row
//! that moves. The wallpaper is the frame's ground, and the first rule is that it covers the
//! panel every time.

use std::path::PathBuf;

use slot2_gfx::{Canvas, Color, TexId};

use crate::layout::SafeArea;

/// The built-in background, for the card that has no picture — which is most of them.
///
/// A gradient rather than a flat fill, and darker at the foot than at the head. The slot is
/// at the bottom of the panel and the carts stand above it; a face that darkens towards the
/// machine reads as a surface going back, so the row reads as standing on something instead
/// of floating on a colour.
pub const TOP: Color = Color::from_rgb8(0x1B, 0x1F, 0x26);
pub const BOTTOM: Color = Color::from_rgb8(0x0B, 0x0C, 0x10);

/// The background, and whatever the card has to put there.
#[derive(Default)]
pub struct Wallpaper {
    /// The card's picture for the shelf being shown, if it has one.
    source: Option<PathBuf>,
    /// The uploaded picture, and the path it came from — so a platform switch back does not
    /// decode again.
    art: Option<(PathBuf, TexId, u32, u32)>,
    /// The built-in gradient, one column of texels tall as the panel, and the panel height it
    /// was built for.
    gradient: Option<(TexId, u32)>,
    /// A source that would not decode, remembered so a broken file is opened once.
    failed: Option<PathBuf>,
}

impl Wallpaper {
    /// Point it at the card's picture for the shelf now being shown, or at nothing.
    ///
    /// Called when the row changes, not per frame: whether a file exists is a question for a
    /// rescan. Pointing it at what it already has must not throw the texture away — L1 and R1
    /// walk the platforms, and a player holding R1 would otherwise decode a picture per
    /// shelf per press.
    pub fn set_source(&mut self, path: Option<PathBuf>) {
        todo!()
    }

    /// Paint the whole panel. Every pixel, every frame.
    pub fn draw(&mut self, canvas: &mut dyn Canvas, safe: &SafeArea) {
        todo!()
    }
}
