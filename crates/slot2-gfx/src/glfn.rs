//! GL function loading. Task 03 fills this in.
//!
//! `gl` (the `gl` crate, global function pointers) is loaded once per context with
//! `gl::load_with(|name| surface.proc_address(name))`. Both surfaces provide `proc_address`:
//! glutin's `display.get_proc_address`, and on the device `eglGetProcAddress` falling back
//! to `dlsym` on libGLESv2 (some Mali blobs return null from eglGetProcAddress for core
//! entry points, so the fallback is load-bearing). Port the approach from
//! `C:\Users\gyuha\slot-2\crates\slot-gfx\src\gl.rs`.

use crate::Surface;
use std::sync::atomic::{AtomicBool, Ordering};

static ES: AtomicBool = AtomicBool::new(false);

/// Load every GL entry point from the surface's context. Call after `make_current`.
pub fn load(surface: &dyn Surface) {
    gl::load_with(|name| surface.proc_address(name));

    let version = unsafe { gl::GetString(gl::VERSION) };
    let es = !version.is_null()
        && unsafe { std::ffi::CStr::from_ptr(version as *const std::ffi::c_char) }
            .to_string_lossy()
            .starts_with("OpenGL ES");
    ES.store(es, Ordering::Relaxed);
}

pub fn is_es() -> bool {
    ES.load(Ordering::Relaxed)
}
pub fn internal_format() -> gl::types::GLint {
    if is_es() {
        gl::RGBA as gl::types::GLint
    } else {
        gl::RGBA8 as gl::types::GLint
    }
}
