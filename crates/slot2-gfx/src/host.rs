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

use std::ffi::{c_void, CString};
use std::num::NonZeroU32;
use std::time::Duration;

use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext,
    PossiblyCurrentGlContext, Version,
};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
pub use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowId};

use crate::{GfxError, Surface};

/// What the window told us since the last `pump`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostEvent {
    CloseRequested,
    /// A physical key went down (`true`) or up (`false`). No auto-repeat events.
    Key {
        code: KeyCode,
        pressed: bool,
    },
    /// Client area is now this many pixels.
    Resized(u32, u32),
}

pub struct HostSurface {
    event_loop: EventLoop<()>,
    window: Window,
    display: Display,
    gl_surface: glutin::surface::Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    collected_events: Vec<HostEvent>,
}

fn err<E: std::fmt::Display>(e: E) -> GfxError {
    GfxError::Context(e.to_string())
}

impl HostSurface {
    /// Open a window `panel * scale` pixels big with a GL context. Must be called on the
    /// main thread.
    pub fn open(title: &str, panel: (u32, u32), scale: u32) -> Result<Self, GfxError> {
        // Tests open windows off the main thread; winit only allows that through a platform
        // extension, and each platform has its own.
        let mut builder = EventLoop::builder();
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::EventLoopBuilderExtWindows;
            builder.with_any_thread(true);
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // winit's default features carry both backends; each trait sets the same flag.
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            use winit::platform::x11::EventLoopBuilderExtX11;
            EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
            EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
        }
        let mut event_loop = builder.build().map_err(err)?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut init = InitApp {
            title: title.to_string(),
            w: panel.0 * scale,
            h: panel.1 * scale,
            result: None,
        };

        // Pump until we get the resumed event and create the surface.
        while init.result.is_none() {
            event_loop.pump_app_events(Some(Duration::ZERO), &mut init);
        }

        let state = init.result.unwrap()?;

        Ok(HostSurface {
            event_loop,
            window: state.window,
            display: state.display,
            gl_surface: state.gl_surface,
            context: state.context,
            collected_events: Vec::new(),
        })
    }

    /// Process pending window events without blocking and return them in order.
    pub fn pump(&mut self) -> Vec<HostEvent> {
        let mut handler = PumpHandler {
            events: &mut self.collected_events,
            gl_surface: &self.gl_surface,
            context: &self.context,
        };
        self.event_loop
            .pump_app_events(Some(Duration::ZERO), &mut handler);
        std::mem::take(&mut self.collected_events)
    }
}

impl Surface for HostSurface {
    fn make_current(&mut self) -> Result<(), GfxError> {
        self.context.make_current(&self.gl_surface).map_err(err)
    }
    fn size(&self) -> (u32, u32) {
        let s = self.window.inner_size();
        (s.width, s.height)
    }
    fn swap(&mut self) -> Result<(), GfxError> {
        self.gl_surface.swap_buffers(&self.context).map_err(err)
    }
    fn proc_address(&self, name: &str) -> *const c_void {
        let c = match CString::new(name) {
            Ok(c) => c,
            Err(_) => return std::ptr::null(),
        };
        self.display.get_proc_address(&c)
    }
}

struct HostSurfaceState {
    window: Window,
    display: Display,
    gl_surface: glutin::surface::Surface<WindowSurface>,
    context: PossiblyCurrentContext,
}

struct InitApp {
    title: String,
    w: u32,
    h: u32,
    result: Option<Result<HostSurfaceState, GfxError>>,
}

impl ApplicationHandler for InitApp {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        if self.result.is_some() {
            return;
        }
        self.result = Some(self.create_state(events));
    }
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}

impl InitApp {
    fn create_state(&self, events: &ActiveEventLoop) -> Result<HostSurfaceState, GfxError> {
        let attrs = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::PhysicalSize::new(self.w, self.h));

        let (window, config) = DisplayBuilder::new()
            .with_window_attributes(Some(attrs))
            .build(events, ConfigTemplateBuilder::new(), |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() < accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .map_err(err)?;

        let window = window.ok_or_else(|| GfxError::Context("no window".into()))?;
        let display = config.display();
        let handle = window.window_handle().map_err(err)?.as_raw();

        let context_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(Some(Version::new(2, 0))))
            .build(Some(handle));

        let mut context = unsafe { display.create_context(&config, &context_attrs) };
        if context.is_err() {
            let fallback_attrs = ContextAttributesBuilder::new().build(Some(handle));
            context = unsafe { display.create_context(&config, &fallback_attrs) };
        }
        let context = context.map_err(err)?;

        let surface_attrs = window
            .build_surface_attributes(Default::default())
            .map_err(err)?;
        let gl_surface =
            unsafe { display.create_window_surface(&config, &surface_attrs) }.map_err(err)?;
        let context = context.make_current(&gl_surface).map_err(err)?;

        let _ = gl_surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::MIN));

        Ok(HostSurfaceState {
            window,
            display,
            gl_surface,
            context,
        })
    }
}

struct PumpHandler<'a> {
    events: &'a mut Vec<HostEvent>,
    gl_surface: &'a glutin::surface::Surface<WindowSurface>,
    context: &'a PossiblyCurrentContext,
}

impl<'a> ApplicationHandler for PumpHandler<'a> {
    fn resumed(&mut self, _: &ActiveEventLoop) {}
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.events.push(HostEvent::CloseRequested),
            WindowEvent::Resized(s) => {
                if let (Some(w), Some(h)) = (NonZeroU32::new(s.width), NonZeroU32::new(s.height)) {
                    self.gl_surface.resize(self.context, w, h);
                }
                self.events.push(HostEvent::Resized(s.width, s.height));
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                self.events.push(HostEvent::Key {
                    code,
                    pressed: state == ElementState::Pressed,
                });
            }
            _ => {}
        }
    }
}
