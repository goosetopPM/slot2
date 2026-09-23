//! What the shelf stands on.
//!
//! Not decoration. Nothing else paints the whole panel — the slot covers its band, the carts
//! cover themselves, and until now the rest of the frame was whatever the last frame left
//! there. On a double-buffered device that is two frames of history alternating behind a row
//! that moves. The wallpaper is the frame's ground, and the first rule is that it covers the
//! panel every time.

use std::collections::HashMap;
use std::path::PathBuf;

use slot2_gfx::{cover_uv, Canvas, Color, TexId};

use crate::image;
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
    /// The uploaded pictures, by path — so a platform switch back does not decode again.
    ///
    /// Every shelf's picture is kept, rather than only the one on screen. There are seven
    /// platforms, so this is bounded at seven plus the default; at panel size that is about
    /// ten megabytes of texture against a gigabyte of shared memory. The alternative costs a
    /// PNG decode on every press of L1 or R1, which on an A53 is a fifth of a second of
    /// nothing happening each time.
    art: HashMap<PathBuf, (TexId, u32, u32)>,
    /// The built-in gradient, one column of texels tall as the panel, and the panel height it
    /// was built for.
    gradient: Option<(TexId, u32)>,
    /// Sources that would not decode, remembered so a broken file is opened once.
    failed: std::collections::HashSet<PathBuf>,
}

impl Wallpaper {
    /// Point it at the card's picture for the shelf now being shown, or at nothing.
    ///
    /// Called when the row changes, not per frame: whether a file exists is a question for a
    /// rescan. Pointing it at what it already has must not throw the texture away — L1 and R1
    /// walk the platforms, and a player holding R1 would otherwise decode a picture per
    /// shelf per press.
    pub fn set_source(&mut self, path: Option<PathBuf>) {
        if self.source == path {
            return;
        }
        self.source = path;
    }

    /// Paint the whole panel. Every pixel, every frame.
    pub fn draw(&mut self, canvas: &mut dyn Canvas, safe: &SafeArea) {
        let (pw, ph) = (safe.panel_w as f32, safe.panel_h as f32);

        // The gradient is one column of texels as tall as the panel, so a panel that changed
        // height needs a new one. Nothing on a device ever does, but the host window walks
        // all three geometries in one process.
        if let Some((tex, h)) = self.gradient {
            if h != safe.panel_h {
                canvas.free(tex);
                self.gradient = None;
            }
        }

        if let Some(path) = self.source.clone() {
            if !self.art.contains_key(&path) && !self.failed.contains(&path) {
                match image::decode_png(&path, (safe.panel_w, safe.panel_h)) {
                    Some(d) => {
                        let tex = canvas.upload_rgba8(d.w, d.h, &d.rgba);
                        self.art.insert(path.clone(), (tex, d.w, d.h));
                    }
                    None => {
                        self.failed.insert(path.clone());
                    }
                }
            }
            if let Some(&(tex, w, h)) = self.art.get(&path) {
                // The quad is the whole panel and the crop is in the uv, so the picture
                // covers whatever shape the panel is without being stretched to it.
                let uv = cover_uv((w, h), (safe.panel_w, safe.panel_h));
                canvas.image_uv(tex, 0.0, 0.0, pw, ph, uv, Color::WHITE);
                return;
            }
        }

        if self.gradient.is_none() {
            let h = safe.panel_h;
            let mut data = Vec::with_capacity((h * 4) as usize);
            for y in 0..h {
                let t = if h > 1 {
                    y as f32 / (h - 1) as f32
                } else {
                    0.0
                };
                let c = Color {
                    r: TOP.r + (BOTTOM.r - TOP.r) * t,
                    g: TOP.g + (BOTTOM.g - TOP.g) * t,
                    b: TOP.b + (BOTTOM.b - TOP.b) * t,
                    a: 1.0,
                };
                data.extend_from_slice(&c.to_u8());
            }
            let tex = canvas.upload_rgba8(1, h, &data);
            self.gradient = Some((tex, h));
        }

        // Stretched across the panel. Every texel in a row is the same colour, so nearest
        // filtering along that axis costs nothing and the column is 1:1 down the other.
        if let Some((tex, _)) = self.gradient {
            canvas.image(tex, 0.0, 0.0, pw, ph, Color::WHITE);
        } else {
            // The upload failed, which on a canvas that can upload nothing is the only way
            // here. The ground is the one thing that cannot degrade to nothing.
            canvas.rect(0.0, 0.0, pw, ph, BOTTOM);
        }
    }
}
