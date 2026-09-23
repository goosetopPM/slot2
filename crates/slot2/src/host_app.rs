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
use std::time::Instant;

use slot2_audio::Sink as _;
use slot2_gfx::{GlCanvas, HostEvent, HostSurface, KeyCode};
use slot2_ui::UiCtx;

use crate::app::{App, Exit, Screen, SinkRequest};
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
    let card = slot2_store::Card::new(&boot.root);
    card.ensure_layout();
    let mut app = App::with_card(
        card,
        crate::core_dir(&boot.root),
        slot2_audio::DEVICE_RATE,
        slot2::tuning_for(&boot.detected.profile),
        std::env::var_os("SLOT2_DEBUG_FRAME").is_some(),
        Screen::List,
    );
    eprintln!(
        "slot2: {} carts on the {} shelf",
        app.carts().len(),
        app.platform().folder()
    );
    let mut sink: Option<slot2_audio::HostSink> = None;

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
        app.run_frame();
        match app.take_sink_request() {
            Some(SinkRequest::Open) => {
                if let Some(consumer) = app.take_consumer() {
                    match slot2_audio::HostSink::open(slot2_audio::DEVICE_RATE, 1024, consumer) {
                        Ok(s) => sink = Some(s),
                        // A game with no sound beats no game at all.
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
            action.perform(false);
            return;
        }

        app.draw(&mut canvas, &mut ctx, began);

        if let Err(e) = canvas.present(&mut surface) {
            eprintln!("slot2: {e}");
            return;
        }

        let elapsed = began.elapsed();
        let frame_time = app.frame_time();
        if elapsed < frame_time {
            sleep(frame_time - elapsed);
        }
    }
}
