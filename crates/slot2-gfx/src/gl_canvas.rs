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

use crate::{fit, glfn, Canvas, Color, GfxError, Image, Surface, TexId};
use std::collections::HashMap;
use std::ffi::CString;

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    col: [f32; 4],
}

pub struct GlCanvas {
    panel: (u32, u32),
    fbo: gl::types::GLuint,
    fbo_tex: gl::types::GLuint,
    white_tex: TexId,
    program: gl::types::GLuint,
    u_panel: gl::types::GLint,
    vbo: gl::types::GLuint,
    batch_tex: TexId,
    vertices: Vec<Vertex>,
    /// GL name and the size it was allocated at; the size is what lets `update_rgba8`
    /// tell a rewrite from a resize.
    textures: HashMap<TexId, (gl::types::GLuint, u32, u32)>,
    next_tex_id: u32,
    origin: (f32, f32),
}

impl GlCanvas {
    /// Make the context current, load GL, build the offscreen target and the program.
    pub fn new(surface: &mut dyn Surface, panel: (u32, u32)) -> Result<Self, GfxError> {
        surface.make_current()?;
        glfn::load(surface);

        let fbo_tex = unsafe {
            let mut tex = 0;
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                glfn::internal_format(),
                panel.0 as i32,
                panel.1 as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            tex
        };

        let fbo = unsafe {
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                fbo_tex,
                0,
            );
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                gl::DeleteFramebuffers(1, &fbo);
                gl::DeleteTextures(1, &fbo_tex);
                return Err(GfxError::Framebuffer(status));
            }
            fbo
        };

        let program = {
            let vs = shader(
                gl::VERTEX_SHADER,
                "
                attribute vec2 a_pos;
                attribute vec2 a_uv;
                attribute vec4 a_col;
                uniform vec2 u_panel;
                varying vec2 v_uv;
                varying vec4 v_col;
                void main() {
                    v_uv = a_uv;
                    v_col = a_col;
                    vec2 clip = (a_pos / u_panel) * 2.0 - 1.0;
                    gl_Position = vec4(clip.x, -clip.y, 0.0, 1.0);
                }
            ",
            )?;
            let fs = shader(
                gl::FRAGMENT_SHADER,
                "
                #ifdef GL_ES
                precision mediump float;
                #endif
                varying vec2 v_uv;
                varying vec4 v_col;
                uniform sampler2D u_tex;
                void main() {
                    gl_FragColor = texture2D(u_tex, v_uv) * v_col;
                }
            ",
            )?;
            let p = unsafe { gl::CreateProgram() };
            unsafe {
                gl::AttachShader(p, vs);
                gl::AttachShader(p, fs);
                gl::BindAttribLocation(p, 0, CString::new("a_pos").unwrap().as_ptr());
                gl::BindAttribLocation(p, 1, CString::new("a_uv").unwrap().as_ptr());
                gl::BindAttribLocation(p, 2, CString::new("a_col").unwrap().as_ptr());
                gl::LinkProgram(p);
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);
                let mut ok = 0;
                gl::GetProgramiv(p, gl::LINK_STATUS, &mut ok);
                if ok == 0 {
                    return Err(GfxError::Shader(info_log(
                        p,
                        gl::GetProgramiv,
                        gl::GetProgramInfoLog,
                    )));
                }
            }
            p
        };

        let u_panel = unsafe {
            gl::UseProgram(program);
            gl::Uniform1i(
                gl::GetUniformLocation(program, CString::new("u_tex").unwrap().as_ptr()),
                0,
            );
            gl::GetUniformLocation(program, CString::new("u_panel").unwrap().as_ptr())
        };

        let vbo = unsafe {
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            vbo
        };

        let mut canvas = GlCanvas {
            panel,
            fbo,
            fbo_tex,
            white_tex: TexId(0),
            program,
            u_panel,
            vbo,
            batch_tex: TexId(0),
            vertices: Vec::with_capacity(600),
            textures: HashMap::new(),
            next_tex_id: 1,
            origin: (0.0, 0.0),
        };

        // The offscreen target is bound but nothing has told GL how big it is. A fresh
        // context's viewport is the window's, so a panel smaller than the window was drawn
        // stretched to the window and then clipped to the panel — on a 720x480 panel in a
        // 720x720 window, the row of carts came out half a screen higher than it was asked
        // for and half again too tall.
        //
        // `present` sets this at the end of every frame, which is why the running frontend
        // looked right and every offscreen render did not: a screenshot test never calls it,
        // and neither does the first frame after boot.
        unsafe {
            gl::Viewport(0, 0, panel.0 as i32, panel.1 as i32);
        }

        canvas.white_tex = canvas.upload_rgba8(1, 1, &[255, 255, 255, 255]);
        canvas.batch_tex = canvas.white_tex;

        Ok(canvas)
    }

    fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let tex = self.textures.get(&self.batch_tex).map_or(0, |t| t.0);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<Vertex>()) as isize,
                self.vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 32, std::ptr::null());
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 32, 8 as *const _);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 32, 16 as *const _);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32);
        }
        self.vertices.clear();
    }

    /// Flush this frame to the surface: letterboxed/scaled blit of the panel, then swap.
    pub fn present(&mut self, surface: &mut dyn Surface) -> Result<(), GfxError> {
        self.flush();
        let rect = fit::integer_fit_rect(self.panel, surface.size());
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, surface.size().0 as i32, surface.size().1 as i32);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::Viewport(rect.x, rect.y, rect.w, rect.h);
            gl::UseProgram(self.program);
            gl::Uniform2f(self.u_panel, self.panel.0 as f32, self.panel.1 as f32);
            gl::Disable(gl::BLEND);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            // The offscreen texture was rendered with panel y=0 at the GL top, which is the
            // texture's LAST row (GL textures are bottom-up), so sampling it back onto the
            // window needs v flipped: v=1 at the top of the quad, v=0 at the bottom. Uploaded
            // images are the other way round (top row first in memory = row 0 = v=0), which
            // is why `image_uv` does not flip. `read_back` flips rows on the CPU for the
            // same reason.
            let (pw, ph) = (self.panel.0 as f32, self.panel.1 as f32);
            let quad = [
                Vertex {
                    pos: [0.0, 0.0],
                    uv: [0.0, 1.0],
                    col: [1.0; 4],
                },
                Vertex {
                    pos: [pw, 0.0],
                    uv: [1.0, 1.0],
                    col: [1.0; 4],
                },
                Vertex {
                    pos: [0.0, ph],
                    uv: [0.0, 0.0],
                    col: [1.0; 4],
                },
                Vertex {
                    pos: [pw, 0.0],
                    uv: [1.0, 1.0],
                    col: [1.0; 4],
                },
                Vertex {
                    pos: [pw, ph],
                    uv: [1.0, 0.0],
                    col: [1.0; 4],
                },
                Vertex {
                    pos: [0.0, ph],
                    uv: [0.0, 0.0],
                    col: [1.0; 4],
                },
            ];
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (6 * 32) as isize,
                quad.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 32, std::ptr::null());
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 32, 8 as *const _);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 32, 16 as *const _);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
            gl::BindTexture(gl::TEXTURE_2D, self.fbo_tex);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::DeleteBuffers(1, &vbo);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl::Viewport(0, 0, self.panel.0 as i32, self.panel.1 as i32);
        }
        surface.swap()?;
        Ok(())
    }

    /// The panel as drawn so far this frame (everything since `clear`), top row first.
    pub fn read_back(&mut self) -> Image {
        self.flush();
        let stride = self.panel.0 as usize * 4;
        let mut buf = vec![0u8; stride * self.panel.1 as usize];
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::ReadPixels(
                0,
                0,
                self.panel.0 as i32,
                self.panel.1 as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buf.as_mut_ptr() as *mut _,
            );
        }
        let mut flipped = Vec::with_capacity(buf.len());
        for row in buf.chunks_exact(stride).rev() {
            flipped.extend_from_slice(row);
        }
        Image {
            width: self.panel.0,
            height: self.panel.1,
            rgba: flipped,
        }
    }
}

impl Canvas for GlCanvas {
    fn size(&self) -> (u32, u32) {
        self.panel
    }
    fn clear(&mut self, color: Color) {
        self.flush();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl::ClearColor(color.r, color.g, color.b, color.a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
    fn upload_rgba8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        let id = TexId(self.next_tex_id);
        self.next_tex_id += 1;
        let mut tex = 0;
        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                glfn::internal_format(),
                w as i32,
                h as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }
        self.textures.insert(id, (tex, w, h));
        id
    }
    fn upload_alpha8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        let mut rgba = Vec::with_capacity(data.len() * 4);
        for &a in data {
            rgba.push(255);
            rgba.push(255);
            rgba.push(255);
            rgba.push(a);
        }
        self.upload_rgba8(w, h, &rgba)
    }
    fn update_rgba8(&mut self, tex: TexId, w: u32, h: u32, data: &[u8]) -> bool {
        let Some(&(name, tw, th)) = self.textures.get(&tex) else {
            return false;
        };
        if (tw, th) != (w, h) {
            return false;
        }
        // Anything already queued against this texture was queued to draw the old pixels;
        // let it, before they are gone.
        if self.batch_tex == tex {
            self.flush();
        }
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, name);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                w as i32,
                h as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
        }
        true
    }

    fn free(&mut self, tex: TexId) {
        if let Some((t, _, _)) = self.textures.remove(&tex) {
            if self.batch_tex == tex {
                self.flush();
            }
            unsafe { gl::DeleteTextures(1, &t) };
        }
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.image_uv(self.white_tex, x, y, w, h, [0.0, 0.0, 1.0, 1.0], color);
    }
    fn image(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        self.image_uv(tex, x, y, w, h, [0.0, 0.0, 1.0, 1.0], tint)
    }
    fn set_origin(&mut self, x: f32, y: f32) {
        if self.origin != (x, y) {
            self.flush();
            self.origin = (x, y);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn image_uv(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], tint: Color) {
        if tex != self.batch_tex {
            self.flush();
            self.batch_tex = tex;
        }
        let c = [tint.r, tint.g, tint.b, tint.a];
        let [u0, v0, u1, v1] = uv;
        let (ox, oy) = self.origin;
        self.vertices.push(Vertex {
            pos: [x + ox, y + oy],
            uv: [u0, v0],
            col: c,
        });
        self.vertices.push(Vertex {
            pos: [x + w + ox, y + oy],
            uv: [u1, v0],
            col: c,
        });
        self.vertices.push(Vertex {
            pos: [x + ox, y + h + oy],
            uv: [u0, v1],
            col: c,
        });
        self.vertices.push(Vertex {
            pos: [x + w + ox, y + oy],
            uv: [u1, v0],
            col: c,
        });
        self.vertices.push(Vertex {
            pos: [x + w + ox, y + h + oy],
            uv: [u1, v1],
            col: c,
        });
        self.vertices.push(Vertex {
            pos: [x + ox, y + h + oy],
            uv: [u0, v1],
            col: c,
        });

        if self.vertices.len() >= 600 {
            self.flush();
        }
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFuncSeparate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
            gl::UseProgram(self.program);
            gl::Uniform2f(self.u_panel, self.panel.0 as f32, self.panel.1 as f32);
        }
    }
}

impl Drop for GlCanvas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteTextures(1, &self.fbo_tex);
            for (_, (t, _, _)) in self.textures.drain() {
                gl::DeleteTextures(1, &t);
            }
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.program);
        }
    }
}

fn shader(kind: gl::types::GLenum, src: &str) -> Result<gl::types::GLuint, GfxError> {
    let s = unsafe { gl::CreateShader(kind) };
    let c_src = CString::new(src.replace("\r\n", "\n")).unwrap();
    unsafe {
        gl::ShaderSource(s, 1, &c_src.as_ptr(), std::ptr::null());
        gl::CompileShader(s);
        let mut ok = 0;
        gl::GetShaderiv(s, gl::COMPILE_STATUS, &mut ok);
        if ok == 0 {
            let log = info_log(s, gl::GetShaderiv, gl::GetShaderInfoLog);
            gl::DeleteShader(s);
            return Err(GfxError::Shader(log));
        }
    }
    Ok(s)
}

fn info_log(
    id: gl::types::GLuint,
    get_iv: unsafe fn(gl::types::GLuint, gl::types::GLenum, *mut gl::types::GLint),
    get_log: unsafe fn(
        gl::types::GLuint,
        gl::types::GLsizei,
        *mut gl::types::GLsizei,
        *mut gl::types::GLchar,
    ),
) -> String {
    let mut len = 0;
    unsafe { get_iv(id, gl::INFO_LOG_LENGTH, &mut len) };
    if len <= 0 {
        return "no log".into();
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { get_log(id, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut _) };
    String::from_utf8_lossy(&buf).trim_end_matches('\0').into()
}
