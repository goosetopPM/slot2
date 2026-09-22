//! EGL on the device framebuffer. This is how the H700 presents: `/dev/fb0` opened directly,
//! libEGL and libGLESv2 loaded with `libloading` at runtime so the binary links against
//! nothing the build box lacks.
//!
//! Implementation notes for task 03 (see tasks/03-gfx.md):
//!
//! - **Port `C:\Users\gyuha\slot-2\crates\slot-gfx\src\fbdev.rs`** (MIT, Brandon T. Kowalski;
//!   keep a one-line attribution comment). It already: dlopens `libEGL.so.1`/`libEGL.so` and
//!   `libGLESv2.so.2`/`libGLESv2.so`; reads the panel size from `/dev/fb0` via
//!   `FBIOGET_VSCREENINFO`; creates a display with `eglGetDisplay(EGL_DEFAULT_DISPLAY)`, a
//!   config (RGB888, ES2 renderable), a window surface on the fbdev native window, and an
//!   ES 2.0 context; sets swap interval 1; formats EGL errors readably.
//! - Adapt the names to this crate's `Surface` trait (`size()` instead of `window_size()`)
//!   and `GfxError`. `proc_address` = `eglGetProcAddress`, and if that returns null, `dlsym`
//!   the name on the GLESv2 library handle (keep the handle alive in the struct).
//! - `FbdevSurface::open()` must compile on the host (it is `cfg`-free); on a machine with no
//!   `/dev/fb0` or no libEGL it fails at runtime with `GfxError::Context(..)`, never at
//!   build time. Use `#[cfg(unix)]` only around the ioctl itself if the host toolchain needs
//!   it (Windows has no `ioctl`); keep everything else portable.
//! - `panel_size()` is also exposed on its own so the boot log can print it before EGL is
//!   touched.

use std::ffi::c_void;

use crate::{GfxError, Surface};

pub struct FbdevSurface {
    _todo: (),
}

impl FbdevSurface {
    /// Open `/dev/fb0`, bring up EGL on it, make the ES 2.0 context current.
    pub fn open() -> Result<Self, GfxError> {
        todo!("task 03")
    }

    /// The panel's size from `/dev/fb0`'s variable screen info, without touching EGL.
    pub fn panel_size() -> Result<(u32, u32), GfxError> {
        todo!("task 03")
    }
}

impl Surface for FbdevSurface {
    fn make_current(&mut self) -> Result<(), GfxError> {
        todo!("task 03")
    }
    fn size(&self) -> (u32, u32) {
        todo!("task 03")
    }
    fn swap(&mut self) -> Result<(), GfxError> {
        todo!("task 03")
    }
    fn proc_address(&self, name: &str) -> *const c_void {
        let _ = name;
        todo!("task 03")
    }
}
