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

use std::thread::sleep;
use std::time::{Duration, Instant};

use slot2_gfx::{GlCanvas, HostEvent, HostSurface, KeyCode};
use slot2_ui::UiCtx;

use crate::app::{App, Exit};
use crate::Boot;

pub fn run(boot: Boot) {
    let panel = boot.detected.profile.geometry.size();
    let mut surface = match HostSurface::open("SLOT2", panel, 2) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("slot2: {e}");
            return;
        }
    };
    let mut canvas = match GlCanvas::new(&mut surface, panel) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("slot2: {e}");
            return;
        }
    };
    let mut ctx = UiCtx::new(
        boot.detected.profile,
        &boot.lang,
        crate::font_dirs(&boot.root),
        Some(&boot.root.join("System/Lang")),
    );
    let mut app = App::new(std::env::var_os("SLOT2_DEBUG_FRAME").is_some());

    loop {
        let began = Instant::now();
        for ev in surface.pump() {
            match ev {
                HostEvent::CloseRequested => return,
                HostEvent::Key {
                    code: KeyCode::Escape,
                    pressed: true,
                } => return,
                HostEvent::Key { code, pressed } => {
                    if let Some(b) = slot2_input::host_map(code) {
                        app.feed(&slot2_input::Event::Button {
                            button: b,
                            pressed,
                            at: began,
                        });
                    }
                }
                _ => {}
            }
        }

        app.tick(began);
        if let Some(exit) = app.exit() {
            let action = match exit {
                Exit::PowerOff => slot2_platform::PowerAction::PowerOff,
                Exit::Reboot => slot2_platform::PowerAction::Reboot,
            };
            action.perform(false);
            return;
        }

        app.draw(&mut canvas, &mut ctx, began);

        if let Err(e) = canvas.present(&mut surface) {
            eprintln!("slot2: {e}");
            return;
        }

        let elapsed = began.elapsed();
        let frame_time = Duration::from_secs_f64(1.0 / 60.0);
        if elapsed < frame_time {
            sleep(frame_time - elapsed);
        }
    }
}
