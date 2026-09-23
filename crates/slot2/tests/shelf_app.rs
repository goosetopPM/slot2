//! The contract for the shelf inside the app. Task 13 makes these pass without editing
//! this file.
//!
//! Everything here drives `App` the way the loop does — feed an event, tick a clock, draw —
//! so what it checks is the frontend a player touches rather than a layout function.

use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_store::{Card, Platform};

use slot2::app::{App, Screen};

/// Tests run in parallel, so each one needs a card of its own. Naming the directory after
/// anything two tests could share — the number of platforms, say — has them deleting each
/// other's ROMs halfway through.
static NEXT_CARD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn card_with(counts: &[(Platform, usize)]) -> (Card, std::path::PathBuf) {
    let n = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-shelfapp-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for (p, n) in counts {
        for i in 0..*n {
            let ext = p.extensions().first().copied().unwrap_or("bin");
            let rom = card.games_dir(*p).join(format!("Game {i:02}.{ext}"));
            std::fs::write(&rom, b"rom").unwrap();
        }
    }
    (card, root)
}

fn app(counts: &[(Platform, usize)]) -> (App, std::path::PathBuf) {
    let (card, root) = card_with(counts);
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

/// Run the row forward the way the loop does, a frame at a time.
fn settle(a: &mut App, from: Instant) -> Instant {
    let mut now = from;
    for _ in 0..240 {
        now += Duration::from_millis(16);
        a.tick(now);
    }
    now
}

#[test]
fn the_shelf_holds_the_games_on_the_card() {
    let (a, _root) = app(&[(Platform::Gba, 5)]);
    assert_eq!(a.carts().len(), 5);
    assert_eq!(a.shelf_len(), 5, "the shelf and the card disagree");
    assert_eq!(a.selected(), 0);
}

#[test]
fn left_and_right_move_along_the_row() {
    // A carousel is horizontal. Up and down are not how you walk a shelf.
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let t = Instant::now();
    tap(&mut a, Button::Right, t);
    assert_eq!(a.selected(), 1);
    tap(&mut a, Button::Left, t + Duration::from_millis(200));
    assert_eq!(a.selected(), 0);
    tap(&mut a, Button::Left, t + Duration::from_millis(400));
    assert_eq!(a.selected(), 3, "the row should wrap");
}

#[test]
fn the_row_slides_and_then_stops() {
    // The gap that let a real bug through last time: check the moving state, not only the
    // resting one.
    let (mut a, _root) = app(&[(Platform::Gba, 6)]);
    let t = Instant::now();
    tap(&mut a, Button::Right, t);
    assert!(a.shelf_settling(), "the row did not start moving");

    let now = settle(&mut a, t + Duration::from_millis(100));
    assert!(!a.shelf_settling(), "the row never came to rest");
    let _ = now;
}

#[test]
fn a_long_frame_does_not_throw_the_row_off_the_screen() {
    // Loading a core can cost a second. Integrating a spring with dt that large is
    // unstable, and the row would fly off and never come back.
    let (mut a, _root) = app(&[(Platform::Gba, 6)]);
    let t = Instant::now();
    tap(&mut a, Button::Right, t);
    let mut now = t + Duration::from_millis(100);
    for _ in 0..5 {
        now += Duration::from_secs(1);
        a.tick(now);
    }
    let now = settle(&mut a, now);
    assert!(!a.shelf_settling(), "the row never recovered");

    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    a.draw(&mut canvas, &mut ctx, now);
    let drew = canvas.frame().iter().any(|o| matches!(o, Op::Image { .. }));
    assert!(drew, "nothing was drawn after a stalled frame");
}

#[test]
fn the_shoulders_change_shelf() {
    // M3's acceptance criterion. L1/R1 walks the platforms, and the row that comes back is
    // the new platform's.
    let (mut a, _root) = app(&[(Platform::Gba, 3), (Platform::Gb, 7)]);
    let t = Instant::now();
    let first = a.platform();

    for i in 0..Platform::ALL.len() {
        tap(
            &mut a,
            Button::R1,
            t + Duration::from_millis(100 * i as u64),
        );
        assert_eq!(
            a.shelf_len(),
            a.carts().len(),
            "{:?}: the shelf and the card disagree after a switch",
            a.platform()
        );
        assert!(
            a.selected() == 0 || a.selected() < a.carts().len(),
            "{:?}: selection {} is off a row of {}",
            a.platform(),
            a.selected(),
            a.carts().len()
        );
    }
    assert_eq!(a.platform(), first, "walking every shelf should return");
}

#[test]
fn switching_shelf_mid_slide_does_not_leave_the_row_chasing() {
    // Press right, then change platform before the row has settled. The old target belongs
    // to a row that no longer exists.
    let (mut a, _root) = app(&[(Platform::Gba, 9), (Platform::Gb, 2)]);
    let t = Instant::now();
    tap(&mut a, Button::Right, t);
    a.tick(t + Duration::from_millis(116));
    assert!(a.shelf_settling());

    tap(&mut a, Button::R1, t + Duration::from_millis(130));
    let now = settle(&mut a, t + Duration::from_millis(200));
    assert!(
        !a.shelf_settling(),
        "the row is still chasing a dead target"
    );
    assert!(
        a.selected() < a.carts().len().max(1),
        "selection {} is off the new row",
        a.selected()
    );
    let _ = now;
}

#[test]
fn the_shelf_is_what_gets_drawn() {
    let (mut a, _root) = app(&[(Platform::Gba, 4)]);
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    a.draw(&mut canvas, &mut ctx, Instant::now());

    let images = canvas
        .frame()
        .iter()
        .filter(|o| matches!(o, Op::Image { .. }))
        .count();
    assert!(images >= 3, "the shelf screen drew only {images} things");
}

#[test]
fn an_empty_shelf_is_a_screen_not_a_crash() {
    let (mut a, _root) = app(&[(Platform::Gba, 0)]);
    let t = Instant::now();
    assert_eq!(a.shelf_len(), 0);

    // Every button a player might press on an empty shelf.
    for b in [
        Button::Left,
        Button::Right,
        Button::A,
        Button::R1,
        Button::L1,
    ] {
        tap(&mut a, b, t);
    }
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    a.draw(&mut canvas, &mut ctx, t);
    assert!(!canvas.frame().is_empty(), "an empty shelf drew nothing");
}

#[test]
fn drawing_the_shelf_does_not_upload_every_frame() {
    // The whole screen, sixty times a second, including while the row slides.
    let (mut a, _root) = app(&[(Platform::Gba, 8)]);
    let safe = slot2_ui::layout::SafeArea::for_geometry(slot2_platform::Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = ui();
    let t = Instant::now();

    a.draw(&mut canvas, &mut ctx, t);
    let warm = canvas.ops.len();

    tap(&mut a, Button::Right, t);
    let mut now = t + Duration::from_millis(100);
    for _ in 0..60 {
        now += Duration::from_millis(16);
        a.tick(now);
        a.draw(&mut canvas, &mut ctx, now);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(warm)
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count();
    assert!(
        uploads <= 4,
        "a second of shelf uploaded {uploads} textures"
    );
}
