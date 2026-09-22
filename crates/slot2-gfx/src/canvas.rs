//! The drawing interface the UI sees, and a recording implementation for tests.
//!
//! Coordinates are panel pixels: `(0, 0)` top-left, `x` right, `y` down, in whatever
//! geometry the canvas was made with. Floats, so animation can sit between pixels; the GL
//! canvas does not snap them.

/// A texture handle. Only meaningful for the canvas that returned it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TexId(pub u32);

/// Straight (non-premultiplied) RGBA, 0..=1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    /// From 8-bit channels.
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn with_alpha(self, a: f32) -> Color {
        Color { a, ..self }
    }

    /// To 8-bit channels, rounded.
    pub fn to_u8(self) -> [u8; 4] {
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        [q(self.r), q(self.g), q(self.b), q(self.a)]
    }
}

pub trait Canvas {
    /// The panel size this canvas draws in.
    fn size(&self) -> (u32, u32);

    /// Fill the whole panel. Usually the first call of a frame.
    fn clear(&mut self, color: Color);

    /// Upload an RGBA8 image (straight alpha, row-major, top row first). `data.len()` must
    /// be `w * h * 4`.
    fn upload_rgba8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId;

    /// Upload an 8-bit coverage mask (what `slot2_text` rasterises). Drawn with `image`, the
    /// mask becomes the tint colour at that coverage: a text face in any colour from one
    /// upload. `data.len()` must be `w * h`.
    fn upload_alpha8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId;

    /// Release a texture. Using the id afterwards is a logic error (drawn as nothing).
    fn free(&mut self, tex: TexId);

    /// Solid rectangle, alpha-blended.
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color);

    /// The whole texture stretched to the rectangle, multiplied by `tint`, alpha-blended.
    /// For an alpha8 texture the result is `tint.rgb` at `coverage * tint.a`.
    fn image(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, tint: Color);

    /// Part of a texture: `uv` is `[u0, v0, u1, v1]` in 0..=1, `(0, 0)` the top-left texel.
    #[allow(clippy::too_many_arguments)]
    fn image_uv(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], tint: Color);
}

/// One recorded call. Textures are numbered from 1 in upload order.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Clear(Color),
    UploadRgba8 {
        id: TexId,
        w: u32,
        h: u32,
    },
    UploadAlpha8 {
        id: TexId,
        w: u32,
        h: u32,
    },
    Free(TexId),
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
    },
    Image {
        tex: TexId,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv: [f32; 4],
        tint: Color,
    },
}

/// A canvas that remembers what was asked of it. For testing draw code without a GPU.
#[derive(Debug)]
pub struct RecordingCanvas {
    size: (u32, u32),
    next: u32,
    pub ops: Vec<Op>,
}

impl RecordingCanvas {
    pub fn new(w: u32, h: u32) -> Self {
        RecordingCanvas {
            size: (w, h),
            next: 1,
            ops: Vec::new(),
        }
    }

    /// Everything drawn since the last `clear`, in order.
    pub fn frame(&self) -> &[Op] {
        let start = self
            .ops
            .iter()
            .rposition(|o| matches!(o, Op::Clear(_)))
            .map_or(0, |i| i + 1);
        &self.ops[start..]
    }
}

impl Canvas for RecordingCanvas {
    fn size(&self) -> (u32, u32) {
        self.size
    }

    fn clear(&mut self, color: Color) {
        self.ops.push(Op::Clear(color));
    }

    fn upload_rgba8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        assert_eq!(data.len(), (w * h * 4) as usize, "rgba8 upload size");
        let id = TexId(self.next);
        self.next += 1;
        self.ops.push(Op::UploadRgba8 { id, w, h });
        id
    }

    fn upload_alpha8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        assert_eq!(data.len(), (w * h) as usize, "alpha8 upload size");
        let id = TexId(self.next);
        self.next += 1;
        self.ops.push(Op::UploadAlpha8 { id, w, h });
        id
    }

    fn free(&mut self, tex: TexId) {
        self.ops.push(Op::Free(tex));
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.ops.push(Op::Rect { x, y, w, h, color });
    }

    fn image(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        self.image_uv(tex, x, y, w, h, [0.0, 0.0, 1.0, 1.0], tint);
    }

    #[allow(clippy::too_many_arguments)]
    fn image_uv(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], tint: Color) {
        self.ops.push(Op::Image {
            tex,
            x,
            y,
            w,
            h,
            uv,
            tint,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_round_trip_through_u8() {
        assert_eq!(Color::from_u8(255, 128, 0, 255).to_u8(), [255, 128, 0, 255]);
        assert_eq!(Color::WHITE.with_alpha(0.5).to_u8(), [255, 255, 255, 128]);
        assert_eq!(Color::rgba(2.0, -1.0, 0.5, 1.0).to_u8(), [255, 0, 128, 255]);
    }

    #[test]
    fn recording_canvas_numbers_textures_and_slices_frames() {
        let mut c = RecordingCanvas::new(720, 480);
        assert_eq!(c.size(), (720, 480));
        let a = c.upload_alpha8(2, 2, &[0; 4]);
        let b = c.upload_rgba8(1, 1, &[0; 4]);
        assert_eq!((a, b), (TexId(1), TexId(2)));
        c.clear(Color::BLACK);
        c.rect(1.0, 2.0, 3.0, 4.0, Color::WHITE);
        c.image(a, 0.0, 0.0, 2.0, 2.0, Color::WHITE);
        c.clear(Color::BLACK);
        c.image(b, 5.0, 5.0, 1.0, 1.0, Color::WHITE);
        assert_eq!(
            c.frame(),
            &[Op::Image {
                tex: b,
                x: 5.0,
                y: 5.0,
                w: 1.0,
                h: 1.0,
                uv: [0.0, 0.0, 1.0, 1.0],
                tint: Color::WHITE
            }]
        );
        assert_eq!(c.ops.len(), 7);
    }

    #[test]
    #[should_panic(expected = "alpha8 upload size")]
    fn recording_canvas_checks_upload_sizes() {
        RecordingCanvas::new(1, 1).upload_alpha8(2, 2, &[0; 3]);
    }
}
