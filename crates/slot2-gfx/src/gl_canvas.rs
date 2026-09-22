//! `Canvas` over GL ES 2 (or a desktop GL context speaking the same subset).
//!
//! Implementation notes for task 03 (see tasks/03-gfx.md):
//!
//! - `GlCanvas::new(surface, panel)`: `surface.make_current()`, `glfn::load(surface)`, then
//!   create an offscreen framebuffer with one RGBA8 colour texture of exactly `panel` size
//!   (no depth, no stencil); check `gl::CheckFramebufferStatus` and return
//!   `GfxError::Framebuffer(status)` if not COMPLETE. Compile one program:
//!   ```text
//!   attribute vec2 a_pos; attribute vec2 a_uv; attribute vec4 a_col;
//!   uniform vec2 u_panel;                       // panel size, for pixel → clip
//!   varying vec2 v_uv; varying vec4 v_col;
//!   // clip = (pos / panel) * 2 - 1, with y flipped so y grows downward
//!   ```
//!   fragment: `gl_FragColor = texture2D(u_tex, v_uv) * v_col;` with the precision line behind
//!   `#ifdef GL_ES`. Solid rects use a 1x1 white RGBA texture uploaded at creation, so one
//!   program serves everything. Compile/link errors → `GfxError::Shader(log)`.
//! - Alpha8 uploads: store as `GL_LUMINANCE_ALPHA`? No — simplest portable route is to expand
//!   on the CPU to RGBA8 with rgb = 255 and a = coverage, then the fragment math
//!   `texel * tint` gives `tint.rgb` at `coverage * tint.a`, exactly the contract.
//!   Textures: `GL_NEAREST` min/mag, `CLAMP_TO_EDGE`, no mipmaps.
//! - Blending: `gl::BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA)`, enabled while drawing.
//! - Draw calls are batched into one vertex `Vec` per texture run: `rect`/`image`/`image_uv`
//!   append 6 vertices (two triangles) and flush when the texture changes or at `present`.
//!   Vertex layout: pos(2 f32) uv(2 f32) col(4 f32), one interleaved VBO, `STREAM_DRAW`.
//! - `present(surface)`: flush; bind the default framebuffer; `gl::Viewport` to the surface
//!   size; clear black; draw the offscreen texture as one quad at
//!   `fit::integer_fit_rect(panel, surface.size())` (host) — the device's surface *is* the
//!   panel so it comes out 1:1 through the same path; then `surface.swap()`. Return to the
//!   offscreen framebuffer afterwards so the next frame's draws land there.
//! - `read_back()`: flush, bind the offscreen framebuffer, `gl::ReadPixels(RGBA, UNSIGNED_BYTE)`
//!   the whole panel, then flip rows so the top row is first (GL reads bottom-up). Returns
//!   `Image`.
//! - `free(tex)`: `gl::DeleteTextures`. `TexId(0)` is never handed out.
//! - Every GL call is `unsafe`; keep them inside this module. No `unwrap()` on GL state.

use crate::{Canvas, Color, GfxError, Image, Surface, TexId};

pub struct GlCanvas {
    _todo: (),
}

impl GlCanvas {
    /// Make the context current, load GL, build the offscreen target and the program.
    pub fn new(surface: &mut dyn Surface, panel: (u32, u32)) -> Result<Self, GfxError> {
        let _ = (surface, panel);
        todo!("task 03")
    }

    /// Flush this frame to the surface: letterboxed/scaled blit of the panel, then swap.
    pub fn present(&mut self, surface: &mut dyn Surface) -> Result<(), GfxError> {
        let _ = surface;
        todo!("task 03")
    }

    /// The panel as drawn so far this frame (everything since `clear`), top row first.
    pub fn read_back(&mut self) -> Image {
        todo!("task 03")
    }
}

impl Canvas for GlCanvas {
    fn size(&self) -> (u32, u32) {
        todo!("task 03")
    }
    fn clear(&mut self, color: Color) {
        let _ = color;
        todo!("task 03")
    }
    fn upload_rgba8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        let _ = (w, h, data);
        todo!("task 03")
    }
    fn upload_alpha8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        let _ = (w, h, data);
        todo!("task 03")
    }
    fn free(&mut self, tex: TexId) {
        let _ = tex;
        todo!("task 03")
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let _ = (x, y, w, h, color);
        todo!("task 03")
    }
    fn image(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        self.image_uv(tex, x, y, w, h, [0.0, 0.0, 1.0, 1.0], tint)
    }
    fn image_uv(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], tint: Color) {
        let _ = (tex, x, y, w, h, uv, tint);
        todo!("task 03")
    }
}
