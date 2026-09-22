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

use slot2_audio::Sink as _;
use slot2_gfx::{FbdevSurface, GlCanvas, Surface};
use slot2_ui::UiCtx;

use crate::app::{App, Exit, Screen, SinkRequest};
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

    let mut source = slot2_input::EvdevSource::open_all(slot2_input::KeyMap::from_pairs(
        slot2_input::DEFAULT_H700_KEYMAP,
    ));
    eprintln!("slot2: input: {} evdev devices", source.device_count());

    let card = slot2_store::Card::new(&boot.root);
    card.ensure_layout();
    let mut app = App::with_card(
        card,
        crate::core_dir(&boot.root),
        slot2_audio::DEVICE_RATE,
        true,
        Screen::List,
    );
    eprintln!(
        "slot2: {} carts on the {} shelf",
        app.carts().len(),
        app.platform().folder()
    );
    let mut sink: Option<slot2_audio::AlsaSink> = None;
    let deadline = Instant::now() + Duration::from_secs(1800);

    while Instant::now() < deadline {
        let began = Instant::now();

        for ev in source.poll(began) {
            app.feed(&ev);
        }
        app.tick(began);
        app.run_frame();
        match app.take_sink_request() {
            Some(SinkRequest::Open) => {
                if let Some(consumer) = app.take_consumer() {
                    // A game with no sound beats no game at all.
                    match slot2_audio::AlsaSink::open(slot2_audio::DEVICE_RATE, 1024, consumer) {
                        Ok(s) => sink = Some(s),
                        Err(e) => eprintln!("slot2: {e}; playing silently"),
                    }
                }
            }
            Some(SinkRequest::Close) => sink = None,
            None => {}
        }
        // A menu over the game silences it; closing the menu brings it back.
        if let Some(s) = sink.as_mut() {
            if matches!(app.screen, Screen::Power(_)) {
                s.pause();
            } else {
                s.resume();
            }
        }

        if let Some(exit) = app.exit() {
            let action = match exit {
                Exit::PowerOff => slot2_platform::PowerAction::PowerOff,
                Exit::Reboot => slot2_platform::PowerAction::Reboot,
            };
            if action.perform(true) {
                return;
            }
            return;
        }

        app.draw(&mut canvas, &mut ctx, began);
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
