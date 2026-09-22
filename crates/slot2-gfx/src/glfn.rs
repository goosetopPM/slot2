//! GL function loading. Task 03 fills this in.
//!
//! `gl` (the `gl` crate, global function pointers) is loaded once per context with
//! `gl::load_with(|name| surface.proc_address(name))`. Both surfaces provide `proc_address`:
//! glutin's `display.get_proc_address`, and on the device `eglGetProcAddress` falling back
//! to `dlsym` on libGLESv2 (some Mali blobs return null from eglGetProcAddress for core
//! entry points, so the fallback is load-bearing). Port the approach from
//! `C:\Users\gyuha\slot-2\crates\slot-gfx\src\gl.rs`.

use crate::Surface;

/// Load every GL entry point from the surface's context. Call after `make_current`.
pub fn load(surface: &dyn Surface) {
    let _ = surface;
    todo!("task 03")
}
