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

// Ported from slot-gfx/fbdev.rs (MIT, Brandon T. Kowalski)

use crate::{GfxError, Surface};
use libloading::Library;
use std::ffi::{c_char, c_void, CString};

const FALLBACK_PANEL: (u32, u32) = (720, 480);
const FB0_SYS: &str = "/sys/class/graphics/fb0";

const EGL_NONE: i32 = 0x3038;
const EGL_ALPHA_SIZE: i32 = 0x3021;
const EGL_BLUE_SIZE: i32 = 0x3022;
const EGL_GREEN_SIZE: i32 = 0x3023;
const EGL_RED_SIZE: i32 = 0x3024;
const EGL_SURFACE_TYPE: i32 = 0x3033;
const EGL_HEIGHT: i32 = 0x3056;
const EGL_WIDTH: i32 = 0x3057;
const EGL_RENDERABLE_TYPE: i32 = 0x3040;
const EGL_CONTEXT_CLIENT_VERSION: i32 = 0x3098;
const EGL_OPENGL_ES_API: u32 = 0x30A0;
const EGL_WINDOW_BIT: i32 = 0x0004;
const EGL_OPENGL_ES2_BIT: i32 = 0x0004;

type Ptr = *mut c_void;
type GetDisplay = unsafe extern "C" fn(Ptr) -> Ptr;
type Initialize = unsafe extern "C" fn(Ptr, *mut i32, *mut i32) -> u32;
type BindApi = unsafe extern "C" fn(u32) -> u32;
type ChooseConfig = unsafe extern "C" fn(Ptr, *const i32, *mut Ptr, i32, *mut i32) -> u32;
type CreateWindowSurface = unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr;
type CreateContext = unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr;
type MakeCurrent = unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32;
type QuerySurface = unsafe extern "C" fn(Ptr, Ptr, i32, *mut i32) -> u32;
type SwapBuffers = unsafe extern "C" fn(Ptr, Ptr) -> u32;
type SwapInterval = unsafe extern "C" fn(Ptr, i32) -> u32;
type GetProcAddress = unsafe extern "C" fn(*const c_char) -> *const c_void;
type GetError = unsafe extern "C" fn() -> i32;
type Terminate = unsafe extern "C" fn(Ptr) -> u32;

struct Egl {
    get_display: GetDisplay,
    initialize: Initialize,
    bind_api: BindApi,
    choose_config: ChooseConfig,
    create_window_surface: CreateWindowSurface,
    create_context: CreateContext,
    make_current: MakeCurrent,
    query_surface: QuerySurface,
    swap_buffers: SwapBuffers,
    swap_interval: SwapInterval,
    get_proc_address: GetProcAddress,
    get_error: GetError,
    terminate: Terminate,
    _lib: Library,
}

#[repr(C)]
struct FbdevWindow {
    width: u16,
    height: u16,
}

pub struct FbdevSurface {
    egl: Egl,
    gles: Library,
    display: Ptr,
    surface: Ptr,
    context: Ptr,
    size: (u32, u32),
    _window: Box<FbdevWindow>,
}

impl FbdevSurface {
    /// Open `/dev/fb0`, bring up EGL on it, make the ES 2.0 context current.
    pub fn open() -> Result<Self, GfxError> {
        let hint = Self::panel_size().unwrap_or(FALLBACK_PANEL);
        let egl = Egl::load()?;
        let gles = open_lib("libGLESv2.so.2").or_else(|_| open_lib("libGLESv2.so"))?;
        unsafe {
            let display = (egl.get_display)(std::ptr::null_mut());
            if display.is_null() {
                return Err(egl.fail("eglGetDisplay"));
            }
            if (egl.initialize)(display, std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
                return Err(egl.fail("eglInitialize"));
            }
            if (egl.bind_api)(EGL_OPENGL_ES_API) == 0 {
                return Err(egl.fail("eglBindAPI"));
            }
            let attrs = [
                EGL_SURFACE_TYPE,
                EGL_WINDOW_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                0,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES2_BIT,
                EGL_NONE,
            ];
            let mut config: Ptr = std::ptr::null_mut();
            let mut found = 0;
            if (egl.choose_config)(display, attrs.as_ptr(), &mut config, 1, &mut found) == 0 {
                return Err(egl.fail("eglChooseConfig"));
            }
            if found == 0 {
                return Err(GfxError::Context("no es2 config".into()));
            }
            let mut window = Box::new(FbdevWindow {
                width: hint.0 as u16,
                height: hint.1 as u16,
            });
            let native = &mut *window as *mut _ as Ptr;
            let mut surface =
                (egl.create_window_surface)(display, config, native, std::ptr::null());
            if surface.is_null() {
                surface = (egl.create_window_surface)(
                    display,
                    config,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                );
            }
            if surface.is_null() {
                return Err(egl.fail("eglCreateWindowSurface"));
            }
            let ctx_attrs = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
            let context =
                (egl.create_context)(display, config, std::ptr::null_mut(), ctx_attrs.as_ptr());
            if context.is_null() {
                return Err(egl.fail("eglCreateContext"));
            }
            if (egl.make_current)(display, surface, surface, context) == 0 {
                return Err(egl.fail("eglMakeCurrent"));
            }
            (egl.swap_interval)(display, 1);
            let size = query_size(&egl, display, surface).unwrap_or(hint);
            Ok(FbdevSurface {
                egl,
                gles,
                display,
                surface,
                context,
                size,
                _window: window,
            })
        }
    }

    /// The panel's size from `/dev/fb0`'s variable screen info, without touching EGL.
    pub fn panel_size() -> Result<(u32, u32), GfxError> {
        let attr = |name: &str| std::fs::read_to_string(format!("{FB0_SYS}/{name}")).ok();
        let size = attr("mode")
            .and_then(|s| parse_mode(&s))
            .or_else(|| attr("modes").and_then(|s| parse_mode(&s)))
            .or_else(|| attr("virtual_size").and_then(|s| parse_vsize(&s)));
        size.ok_or_else(|| GfxError::Context("could not read fb size".into()))
    }
}

fn parse_mode(s: &str) -> Option<(u32, u32)> {
    let body = s.lines().next()?.rsplit(':').next()?;
    let (w, rest) = body.split_once('x')?;
    let h: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    Some((w.trim().parse().ok()?, h.parse().ok()?))
}

fn parse_vsize(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.trim().split_once(',')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

fn open_lib(name: &str) -> Result<Library, GfxError> {
    unsafe { Library::new(name) }.map_err(|e| GfxError::Context(format!("{name}: {e}")))
}

unsafe fn sym<T: Copy>(lib: &Library, name: &str) -> Result<T, GfxError> {
    lib.get::<T>(name.as_bytes())
        .map(|s| *s)
        .map_err(|e| GfxError::Context(format!("{name}: {e}")))
}

impl Egl {
    fn load() -> Result<Self, GfxError> {
        let lib = open_lib("libEGL.so.1").or_else(|_| open_lib("libEGL.so"))?;
        unsafe {
            Ok(Egl {
                get_display: sym(&lib, "eglGetDisplay")?,
                initialize: sym(&lib, "eglInitialize")?,
                bind_api: sym(&lib, "eglBindAPI")?,
                choose_config: sym(&lib, "eglChooseConfig")?,
                create_window_surface: sym(&lib, "eglCreateWindowSurface")?,
                create_context: sym(&lib, "eglCreateContext")?,
                make_current: sym(&lib, "eglMakeCurrent")?,
                query_surface: sym(&lib, "eglQuerySurface")?,
                swap_buffers: sym(&lib, "eglSwapBuffers")?,
                swap_interval: sym(&lib, "eglSwapInterval")?,
                get_proc_address: sym(&lib, "eglGetProcAddress")?,
                get_error: sym(&lib, "eglGetError")?,
                terminate: sym(&lib, "eglTerminate")?,
                _lib: lib,
            })
        }
    }
    fn fail(&self, what: &str) -> GfxError {
        let code = unsafe { (self.get_error)() };
        GfxError::Context(format!("{what}: egl error 0x{code:x}"))
    }
}

fn query_size(egl: &Egl, display: Ptr, surface: Ptr) -> Option<(u32, u32)> {
    let (mut w, mut h) = (0, 0);
    unsafe {
        if (egl.query_surface)(display, surface, EGL_WIDTH, &mut w) == 0
            || (egl.query_surface)(display, surface, EGL_HEIGHT, &mut h) == 0
        {
            return None;
        }
    }
    Some((w as u32, h as u32))
}

impl Surface for FbdevSurface {
    fn make_current(&mut self) -> Result<(), GfxError> {
        if unsafe {
            (self.egl.make_current)(self.display, self.surface, self.surface, self.context)
        } == 0
        {
            return Err(self.egl.fail("eglMakeCurrent"));
        }
        Ok(())
    }
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn swap(&mut self) -> Result<(), GfxError> {
        if unsafe { (self.egl.swap_buffers)(self.display, self.surface) } == 0 {
            return Err(self.egl.fail("eglSwapBuffers"));
        }
        Ok(())
    }
    fn proc_address(&self, name: &str) -> *const c_void {
        let Ok(c) = CString::new(name) else {
            return std::ptr::null();
        };
        if let Ok(f) = unsafe { self.gles.get::<unsafe extern "C" fn()>(name.as_bytes()) } {
            return *f as *const _;
        }
        unsafe { (self.egl.get_proc_address)(c.as_ptr()) }
    }
}

impl Drop for FbdevSurface {
    fn drop(&mut self) {
        unsafe {
            (self.egl.make_current)(
                self.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            (self.egl.terminate)(self.display);
        }
    }
}
