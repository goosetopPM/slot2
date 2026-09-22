//! Surface (fbdev EGL on device, glutin on host), 2D canvas over GL ES 2, letterbox fit,
//! read-back.
//!
//! Two layers:
//!
//! - A [`Surface`] owns the GL context and the thing it presents to: a desktop window
//!   ([`HostSurface`]) or the device's framebuffer ([`FbdevSurface`], EGL over `/dev/fb0`
//!   with libEGL/libGLESv2 dlopen'd — nothing links at build time).
//! - A [`Canvas`] is what the UI draws with: clear, solid rects, textured quads, in the
//!   panel's own pixel space (one of the three geometries, origin top-left, y down).
//!   [`GlCanvas`] draws into an offscreen framebuffer of exactly the panel size and
//!   [`GlCanvas::present`] scales that to whatever the surface really is: 1:1 on the device,
//!   integer-scaled and letterboxed in a resizable host window. [`RecordingCanvas`] is the
//!   same interface with no GL, for tests of the code that draws.
//!
//! Shaders are GLSL ES 1.00 (`attribute`/`varying`/`gl_FragColor`) with the precision line
//! behind `#ifdef GL_ES`, so one source compiles on the Mali's GLES2 and on a desktop GL
//! context alike.
//!
//! Design: docs/DESIGN.md §5, decisions D-09, D-18.

mod canvas;
mod fit;
mod glfn;

pub use canvas::{Canvas, Color, Op, RecordingCanvas, TexId};
pub use fit::{fit_rect, integer_fit_rect, Rect};

mod gl_canvas;
pub use gl_canvas::GlCanvas;

// Compiled on the host too, so the device path stays under the type checker and clippy,
// which only ever run there. Opening it away from the device fails at the first dlopen.
mod fbdev;
pub use fbdev::FbdevSurface;

#[cfg(feature = "host")]
mod host;
#[cfg(feature = "host")]
pub use host::{HostEvent, HostSurface};

use std::ffi::c_void;
use std::fmt;

#[derive(Debug)]
pub enum GfxError {
    /// Could not create or activate a GL context / surface. Human-readable detail.
    Context(String),
    /// A shader failed to compile or link. Carries the driver's log.
    Shader(String),
    /// glCheckFramebufferStatus returned this instead of COMPLETE.
    Framebuffer(u32),
}

impl fmt::Display for GfxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GfxError::Context(m) => write!(f, "gl context: {m}"),
            GfxError::Shader(m) => write!(f, "shader: {m}"),
            GfxError::Framebuffer(s) => write!(f, "framebuffer incomplete: 0x{s:x}"),
        }
    }
}
impl std::error::Error for GfxError {}

/// Something with a GL context that can be presented to.
pub trait Surface {
    fn make_current(&mut self) -> Result<(), GfxError>;
    /// The real drawable size in pixels: the window's client area, or the panel.
    fn size(&self) -> (u32, u32);
    fn swap(&mut self) -> Result<(), GfxError>;
    /// For `gl::load_with`. Null for unknown names.
    fn proc_address(&self, name: &str) -> *const c_void;
}

/// A whole frame read back from a canvas: RGBA8, row-major, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }
}
