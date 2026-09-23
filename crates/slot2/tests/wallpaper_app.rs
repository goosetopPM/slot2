//! The contract for the ground under the shelf, inside the app. Task 18 makes these pass
//! without editing this file.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};

use slot2::app::{App, Screen};

static NEXT_CARD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn card_with(wallpapers: &[Platform]) -> (Card, PathBuf) {
    let i = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-wallapp-{}-{i}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for p in Platform::ALL {
        for g in 0..3 {
            let ext = p.extensions().first().copied().unwrap_or("bin");
            let _ = std::fs::write(card.games_dir(p).join(format!("Game {g}.{ext}")), b"rom");
        }
    }
    for p in wallpapers {
        let f = std::fs::File::create(root.join("Wallpapers").join(format!("{}.png", p.folder())))
            .unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), 64, 64);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .unwrap()
            .write_image_data(&vec![0x80u8; 64 * 64 * 4])
            .unwrap();
    }
    (card, root)
}

fn app(wallpapers: &[Platform]) -> (App, PathBuf) {
    let (card, root) = card_with(wallpapers);
    let a = App::with_card(
        card,
        std::env::temp_dir().join("slot2-no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        Screen::List,
    );
    (a, root)
}

fn ui() -> slot2_ui::UiCtx {
    slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None)
}

fn tap(a: &mut App, b: Button, at: Instant) {
    a.feed(&Event::Button {
        button: b,
        pressed: true,
        at,
    });
    a.feed(&Event::Button {
        button: b,
        pressed: false,
        at: at + Duration::from_millis(40),
    });
    a.tick(at + Duration::from_millis(60));
}

fn frame(a: &mut App, now: Instant) -> RecordingCanvas {
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    a.draw(&mut canvas, &mut ctx, now);
    canvas
}

/// True when this op covers the whole panel.
fn covers_panel(op: &Op, panel: (f32, f32)) -> bool {
    let (x, y, w, h) = match op {
        Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
        _ => return false,
    };
    x <= 0.0 && y <= 0.0 && x + w >= panel.0 && y + h >= panel.1
}

const PANEL: (f32, f32) = (720.0, 480.0);

#[test]
fn every_shelf_screen_paints_the_whole_panel() {
    // The bug this closes: neither loop clears, and nothing painted the panel, so the frame
    // was whatever the last one left there. Behind a row that moves, that is a smear.
    let (mut a, _root) = app(&[]);
    let t = Instant::now();

    let c = frame(&mut a, t);
    assert!(
        c.frame().iter().any(|o| covers_panel(o, PANEL)),
        "the shelf left the panel bare"
    );

    tap(&mut a, Button::A, t);
    assert_eq!(a.screen, Screen::Inserting);
    let c = frame(&mut a, t + Duration::from_millis(100));
    assert!(
        c.frame().iter().any(|o| covers_panel(o, PANEL)),
        "an insert left the panel bare"
    );

    // Stop on the frame the refusal happens. The whole insert-and-eject is 0.9s, so a
    // fixed run long enough to be sure of reaching the eject is also long enough to be past
    // it, and the test would be asserting on the shelf again.
    let mut now = t + Duration::from_millis(60);
    for _ in 0..120 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        if a.screen == Screen::Ejecting {
            break;
        }
    }
    assert_eq!(a.screen, Screen::Ejecting, "expected the refused eject");
    let c = frame(&mut a, now);
    assert!(
        c.frame().iter().any(|o| covers_panel(o, PANEL)),
        "an eject left the panel bare"
    );
}

#[test]
fn the_ground_goes_down_before_anything_stands_on_it() {
    // Painted after the carts it would be a wall in front of the shelf.
    let (mut a, _root) = app(&[]);
    let t = Instant::now();
    let c = frame(&mut a, t);
    // The first thing *drawn*, not the first op: a texture upload is not a mark on the
    // screen, and the ground's own gradient is uploaded before it is drawn. The first
    // version of this test looked at op zero, which let a redundant full-panel rect in
    // ahead of the real background and hid that it was asking the wrong question.
    let first = c
        .frame()
        .iter()
        .find(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
        .expect("the shelf drew nothing at all");
    assert!(
        covers_panel(first, PANEL),
        "the first thing drawn was not the ground: {first:?}"
    );
}

#[test]
fn switching_shelf_changes_the_ground() {
    // M3's acceptance criterion reaches the background too: a Game Boy shelf looks like one.
    let (mut a, _root) = app(&[Platform::Gba, Platform::Gb]);
    let t = Instant::now();
    assert_eq!(a.platform(), Platform::Gba);

    // Both shelves are drawn into *one* canvas and the frame is split at an index. Texture
    // ids are handed out per canvas and start again at 1 for each, so comparing ids across
    // two canvases compares nothing — an earlier version of this test did exactly that, and
    // what came back was an implementation that uploaded junk 1x1 textures to push the
    // numbering apart until the ids differed.
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();

    a.draw(&mut canvas, &mut ctx, t);
    let split = canvas.ops.len();

    // Walk to the Game Boy shelf.
    let mut now = t;
    for _ in 0..Platform::ALL.len() {
        if a.platform() == Platform::Gb {
            break;
        }
        tap(&mut a, Button::R1, now);
        now += Duration::from_millis(100);
    }
    assert_eq!(a.platform(), Platform::Gb, "never reached the GB shelf");
    a.draw(&mut canvas, &mut ctx, now);

    let ground = |ops: &[Op]| -> Option<slot2_gfx::TexId> {
        ops.iter().find_map(|o| match o {
            Op::Image { tex, .. } if covers_panel(o, PANEL) => Some(*tex),
            _ => None,
        })
    };
    let gba_tex = ground(&canvas.ops[..split]);
    let gb_tex = ground(&canvas.ops[split..]);
    assert!(
        gba_tex.is_some(),
        "the GBA shelf drew no background picture"
    );
    assert!(gb_tex.is_some(), "the GB shelf drew no background picture");
    assert_ne!(gba_tex, gb_tex, "both shelves stand on the same picture");
}

#[test]
fn a_shelf_with_no_picture_still_stands_on_something() {
    let (mut a, _root) = app(&[]);
    let t = Instant::now();
    let c = frame(&mut a, t);
    assert!(
        c.frame().iter().any(|o| covers_panel(o, PANEL)),
        "a card with no wallpapers left the shelf on nothing"
    );
}

#[test]
fn the_shelf_does_not_repaint_its_ground_from_scratch_every_frame() {
    // Two seconds of shelf, including a scroll.
    let (mut a, _root) = app(&[Platform::Gba]);
    let t = Instant::now();
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();

    a.draw(&mut canvas, &mut ctx, t);
    let warm = canvas.ops.len();

    tap(&mut a, Button::Right, t);
    let mut now = t + Duration::from_millis(100);
    for _ in 0..120 {
        now += Duration::from_micros(16_667);
        a.tick(now);
        a.draw(&mut canvas, &mut ctx, now);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(warm)
        .filter(|o| matches!(o, Op::UploadRgba8 { .. } | Op::UploadAlpha8 { .. }))
        .count();
    assert!(
        uploads <= 4,
        "two seconds of shelf uploaded {uploads} textures"
    );
}
