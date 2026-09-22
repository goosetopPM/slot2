//! A desktop window with a GL context, driven by the caller's own loop.
//!
//! Implementation notes for task 03 (see tasks/03-gfx.md):
//!
//! - winit 0.30 + glutin 0.32 + glutin-winit 0.5. Port the context creation from
//!   `C:\Users\gyuha\slot-2\crates\slot-gfx\src\host.rs` (it already does the
//!   `DisplayBuilder` / `ConfigTemplate` / `ContextAttributes` dance and picks GLES-first
//!   with a GL fallback). Keep that ordering: request an ES 2.0 context first
//!   (`ContextApi::Gles(Some(Version::new(2, 0)))`), fall back to desktop GL if the
//!   platform refuses.
//! - The window is the panel size times `scale` (2 on a normal monitor, so a 720x480 panel
//!   becomes 1440x960), resizable, titled `title`, vsync requested via
//!   `Surface::set_swap_interval(SwapInterval::Wait(1))` (ignore failure).
//! - **The caller owns the loop.** Use `winit::platform::pump_events::EventLoopExtPumpEvents`:
//!   `pump()` calls `event_loop.pump_app_events(Some(Duration::ZERO), &mut app)` and
//!   returns the events collected by a small `ApplicationHandler` that pushes `HostEvent`s
//!   into a `Vec`. Key events use `winit::keyboard::PhysicalKey::Code(KeyCode)`; ignore
//!   repeats (`event.repeat`) and non-`Code` keys. `Resized` carries the new client size.
//! - `EventLoop::with_user_event()` is not needed; `EventLoop::new()` + `set_control_flow(Poll)`.
//! - `Surface::size()` is the current client size in physical pixels; `swap()` is
//!   `gl_surface.swap_buffers(&context)`; `proc_address` is `display.get_proc_address(CStr)`.
//! - On `Resized`, call `gl_surface.resize(&context, w, h)` (non-zero only).

use std::ffi::c_void;

use crate::{GfxError, Surface};

pub use winit::keyboard::KeyCode;

/// What the window told us since the last `pump`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostEvent {
    CloseRequested,
    /// A physical key went down (`true`) or up (`false`). No auto-repeat events.
    Key { code: KeyCode, pressed: bool },
    /// Client area is now this many pixels.
    Resized(u32, u32),
}

pub struct HostSurface {
    _todo: (),
}

impl HostSurface {
    /// Open a window `panel * scale` pixels big with a GL context. Must be called on the
    /// main thread.
    pub fn open(title: &str, panel: (u32, u32), scale: u32) -> Result<Self, GfxError> {
        let _ = (title, panel, scale);
        todo!("task 03")
    }

    /// Process pending window events without blocking and return them in order.
    pub fn pump(&mut self) -> Vec<HostEvent> {
        todo!("task 03")
    }
}

impl Surface for HostSurface {
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
