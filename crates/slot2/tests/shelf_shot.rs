//! Draws the shelf at all three panel geometries through real GL and writes the results to
//! `target/shelf-<w>x<h>.png`, so M3's first acceptance criterion — the shelf does not break
//! on any of the three — can be looked at rather than argued about.
//!
//! Needs a display and a GL driver, so it only runs with `SLOT2_GFX_TEST=1`; without it the
//! test passes trivially, which is what keeps CI green on a runner with no GPU.
//!
//! One test, not three: winit allows one event loop per process and a second one fails at
//! run time.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2::app::{App, Screen};
use slot2_gfx::{Canvas, GlCanvas, HostSurface, Image};
use slot2_platform::Geometry;
use slot2_store::{Card, Platform};
use slot2_ui::layout::SafeArea;

fn enabled() -> bool {
    std::env::var("SLOT2_GFX_TEST")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn write_png(path: &PathBuf, img: &Image) {
    let f = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), img.width, img.height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(&img.rgba)
        .unwrap();
}

/// A card with a few carts on two shelves, named the way real ones are.
fn card() -> Card {
    let root = std::env::temp_dir().join(format!("slot2-shot-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for (p, names) in [
        (
            Platform::Gba,
            &[
                "Advance Wars",
                "Golden Sun",
                "Metroid Fusion",
                "Mother 3",
                "Wario Land 4",
            ][..],
        ),
        (
            Platform::Gb,
            &["Link's Awakening", "Tetris", "Super Mario Land"][..],
        ),
    ] {
        let ext = p.extensions()[0];
        for n in names {
            std::fs::write(card.games_dir(p).join(format!("{n}.{ext}")), b"rom").unwrap();
        }
    }
    card
}

#[test]
fn the_shelf_holds_up_on_every_panel() {
    if !enabled() {
        eprintln!("SLOT2_GFX_TEST not set; skipping GL test");
        return;
    }
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target");
    // The window is only somewhere for GL to live; every panel is drawn into the canvas's
    // own offscreen buffer and read back from there.
    let mut surface = HostSurface::open("slot2 shelf", (720, 720), 1).unwrap();

    for (g, platform) in [
        (Geometry::W640H480, Platform::Gba),
        (Geometry::W720H480, Platform::Gba),
        (Geometry::W720H720, Platform::Gb),
    ] {
        let safe = SafeArea::for_geometry(g);
        let panel = (safe.panel_w, safe.panel_h);
        let mut canvas = GlCanvas::new(&mut surface, panel).unwrap();

        let mut profile = slot2_platform::detect().profile;
        profile.geometry = g;
        let mut ctx = slot2_ui::UiCtx::new(profile, "en", Vec::new(), None);

        let mut app = App::with_card(
            card(),
            std::env::temp_dir().join("slot2-shot-no-cores"),
            48_000,
            slot2_retro::Tuning::handheld(panel),
            false,
            Screen::List,
        );
        // Walk to the shelf we want the way a player does, rather than opening a hole in
        // the App's API for a screenshot.
        let mut at = Instant::now();
        while app.platform() != platform {
            app.feed(&slot2_input::Event::Button {
                button: slot2_input::Button::R1,
                pressed: true,
                at,
            });
            app.feed(&slot2_input::Event::Button {
                button: slot2_input::Button::R1,
                pressed: false,
                at: at + Duration::from_millis(40),
            });
            at += Duration::from_millis(120);
            app.tick(at);
        }

        // Let the row settle where it starts, then draw the frame a player would see.
        let mut now = at;
        for _ in 0..30 {
            now += Duration::from_millis(16);
            app.tick(now);
        }
        canvas.clear(slot2_gfx::Color::from_u8(0x12, 0x14, 0x18, 255));
        app.draw(&mut canvas, &mut ctx, now);

        let img = canvas.read_back();
        assert_eq!((img.width, img.height), panel);

        // Something was actually drawn: a frame of flat background means the shelf failed
        // silently, which a file on disk would not tell anyone.
        let bg = [0x12u8, 0x14, 0x18];
        let ink = img
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| [p[0], p[1], p[2]] != bg)
            .count();
        assert!(
            ink > (panel.0 * panel.1 / 100) as usize,
            "{g:?}: only {ink} pixels differ from the background"
        );

        let path = out.join(format!("shelf-{}x{}.png", panel.0, panel.1));
        write_png(&path, &img);
        eprintln!(
            "{g:?} {platform:?}: wrote {} ({ink} pixels of ink)",
            path.display()
        );

        // And the insert, at the four moments worth looking at: standing on the row, caught
        // on the lip, being pushed through, and seated. Testing told us the chrome was drawn
        // after the cart and told us nothing about whether the cart could still be seen — it
        // could not, and only a picture said so.
        let titles: Vec<String> = app.carts().iter().map(|c| c.title.clone()).collect();
        let titles: Vec<&str> = titles.iter().map(|s| s.as_str()).collect();
        for seat in [0.0f32, 0.5, 0.8, 1.0] {
            canvas.clear(slot2_gfx::Color::from_u8(0x12, 0x14, 0x18, 255));
            app.shelf_view.draw_insert(
                &mut canvas,
                &mut ctx,
                &safe,
                platform,
                &titles,
                slot2_ui::insert::Insertion {
                    seat,
                    motion: slot2_ui::insert::Motion::Insert,
                },
            );
            let img = canvas.read_back();
            let path = out.join(format!(
                "insert-{}x{}-{:02}.png",
                panel.0,
                panel.1,
                (seat * 10.0) as u32
            ));
            write_png(&path, &img);
        }
    }
}
