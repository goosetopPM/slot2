//! The device loop. Implementation notes for task 04 (see tasks/04-splash.md):
//!
//! ```text
//! eprint!("{}", crate::diag::report(&boot.root));   // first, before any hardware is touched
//! let mut surface = match FbdevSurface::open() { Ok(s) => s, Err(e) => { eprintln!("slot2: {e}"); sleep 60 s; return } };
//! let panel = surface.size();
//! // Trust the framebuffer over the profile table if they disagree, and say so.
//! if panel != profile.geometry.size() { eprintln!("slot2: panel {panel:?} differs from profile {:?}", ...) }
//! let mut canvas = GlCanvas::new(&mut surface, panel)?  (on Err: eprintln, sleep 60 s, return)
//! let mut ctx = UiCtx::new(profile, &boot.lang, crate::font_dirs(&boot.root), Some(&boot.root.join("System/Lang")));
//! let splash = Splash { debug_frame: true };          // on the device we want to see the box
//! let deadline = Instant::now() + Duration::from_secs(300);
//! while Instant::now() < deadline {
//!     let began = Instant::now();
//!     splash.draw(&mut canvas, &mut ctx);
//!     if let Err(e) = canvas.present(&mut surface) { eprintln!("slot2: {e}"); break; }
//!     sleep the rest of a 16.667 ms frame
//! }
//! eprintln!("slot2: idle period over, exiting (BaseOS respawns)");
//! ```
//! The 60 s sleeps on failure keep busybox init's respawn from spinning the box while still
//! letting a fixed binary on the card take over at the next boot. No input yet: that is
//! task 05.
//!
//! Note: `UiCtx::new` needs a `Profile` whose geometry matches `panel` so the safe area is
//! placed right; if the framebuffer disagrees with the table, override
//! `profile.geometry` with the matching `Geometry` when one of the three exists, else keep
//! the profile's and log it.

use std::thread::sleep;
use std::time::{Duration, Instant};

use slot2_gfx::{FbdevSurface, GlCanvas, Surface};
use slot2_ui::{Splash, UiCtx};

use crate::Boot;

pub fn run(boot: Boot) {
    eprint!("{}", crate::diag::report(&boot.root));

    let mut surface = match FbdevSurface::open() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("slot2: {e}");
            sleep(Duration::from_secs(60));
            return;
        }
    };
    let panel = surface.size();
    let mut profile = boot.detected.profile;

    if panel != profile.geometry.size() {
        eprintln!(
            "slot2: panel {panel:?} differs from profile {:?}",
            profile.geometry.size()
        );
        // Try to find a geometry that matches the panel
        if let Some(g) = slot2_platform::Geometry::parse(&format!("{}x{}", panel.0, panel.1)) {
            profile.geometry = g;
        }
    }

    let mut canvas = match GlCanvas::new(&mut surface, panel) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("slot2: {e}");
            sleep(Duration::from_secs(60));
            return;
        }
    };

    let mut ctx = UiCtx::new(
        profile,
        &boot.lang,
        crate::font_dirs(&boot.root),
        Some(&boot.root.join("System/Lang")),
    );

    let splash = Splash { debug_frame: true };
    let deadline = Instant::now() + Duration::from_secs(300);

    while Instant::now() < deadline {
        let began = Instant::now();
        splash.draw(&mut canvas, &mut ctx);
        if let Err(e) = canvas.present(&mut surface) {
            eprintln!("slot2: {e}");
            break;
        }
        let elapsed = began.elapsed();
        let frame_time = Duration::from_secs_f64(1.0 / 60.0);
        if elapsed < frame_time {
            sleep(frame_time - elapsed);
        }
    }

    eprintln!("slot2: idle period over, exiting (BaseOS respawns)");
}
