//! Turning the cartridge artwork into something the canvas can draw.
//!
//! Every drawing in `assets/skins` is a white silhouette on transparency — the shell colour
//! is not in the file. So what comes back from here is *coverage*, not colour: the alpha
//! channel of the render, uploaded as a mask and tinted at draw time. That is what lets one
//! drawing serve a grey Game Pak, a black one and a clear one, and what lets a game's own
//! shell colour reach the shelf without a file per colour.
//!
//! It is also why the art stays vector. A cartridge is drawn at whatever size the panel
//! calls for — three geometries, and a different size again for the one in the slot and the
//! ones beside it — and a bitmap would have to be either huge or soft.

use resvg::tiny_skia;

/// A rasterised drawing, as coverage. One byte per pixel, row major, top row first.
pub struct Mask {
    pub w: u32,
    pub h: u32,
    pub a: Vec<u8>,
}

/// Draw `src` at exactly `w` by `h`. `None` for source that will not parse, or a zero size.
pub fn rasterize(src: &str, w: u32, h: u32) -> Option<Mask> {
    if w == 0 || h == 0 {
        return None;
    }

    // Default options deliberately: no font database. Nothing here has text in it, and
    // loading the system font stack would cost a second of start-up on the device for a
    // capability none of these drawings use.
    let tree = usvg::Tree::from_str(src, &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(w, h)?;

    let svg_w = tree.size().width();
    let svg_h = tree.size().height();
    let transform = tiny_skia::Transform::from_scale(w as f32 / svg_w, h as f32 / svg_h);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Premultiplied RGBA comes back; the colour is white everywhere it is anything, so the
    // alpha channel alone is the whole drawing.
    let a = pixmap.pixels().iter().map(|p| p.alpha()).collect();

    Some(Mask { w, h, a })
}

/// Draw `src` as large as fits inside `box_w` by `box_h` while keeping `natural`'s aspect.
/// `natural` is the artwork's own size, which the caller has from the skin table.
pub fn rasterize_fit(src: &str, natural: (f32, f32), box_w: u32, box_h: u32) -> Option<Mask> {
    if natural.0 <= 0.0 || natural.1 <= 0.0 || box_w == 0 || box_h == 0 {
        return None;
    }

    let ratio = (box_w as f32 / natural.0).min(box_h as f32 / natural.1);
    let w = (natural.0 * ratio).round() as u32;
    let h = (natural.1 * ratio).round() as u32;

    if w == 0 || h == 0 {
        return None;
    }

    rasterize(src, w, h)
}
