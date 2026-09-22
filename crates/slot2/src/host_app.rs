//! The PC window. Implementation notes for task 04 (see tasks/04-splash.md):
//!
//! ```text
//! let panel = boot.detected.profile.geometry.size();
//! let mut surface = HostSurface::open("SLOT2", panel, 2)?;     // 2x on a desktop monitor
//! let mut canvas = GlCanvas::new(&mut surface, panel)?;
//! let mut ctx = UiCtx::new(profile, &boot.lang, crate::font_dirs(&boot.root), Some(&boot.root.join("System/Lang")));
//! let splash = Splash { debug_frame: std::env::var_os("SLOT2_DEBUG_FRAME").is_some() };
//! loop {
//!     let began = Instant::now();
//!     for ev in surface.pump() {
//!         CloseRequested                       → return
//!         Key { code: KeyCode::Escape, pressed: true } → return
//!         (log other key presses at trace level? no — ignore for M0)
//!     }
//!     splash.draw(&mut canvas, &mut ctx);
//!     canvas.present(&mut surface)?;          // on Err: eprintln and return
//!     sleep the rest of a 16.667 ms frame (vsync usually already waits; this caps a
//!     driver that ignores the swap interval)
//! }
//! ```
//! Any `Err` from open/new/present is printed as `slot2: <err>` and ends the program;
//! there is nothing to recover to on a desktop.

use crate::Boot;

pub fn run(boot: Boot) {
    let _ = boot;
    todo!("task 04")
}
