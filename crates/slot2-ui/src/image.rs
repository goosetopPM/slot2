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

    let fw = (w as f32 / max.0 as f32).ceil() as u32;
    let fh = (h as f32 / max.1 as f32).ceil() as u32;
    let f = fw.max(fh).max(1).min(w.max(1)).min(h.max(1));

    let (nw, nh, filtered) = box_filter(w, h, &rgba, f);
    Some(Decoded {
        w: nw,
        h: nh,
        rgba: filtered,
    })
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
