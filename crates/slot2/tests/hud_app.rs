//! The contract for the corner furniture inside the app: which screens wear it, and how
//! often the gauge is actually asked. Task 20 makes these pass without editing this file.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot2_gfx::{Op, RecordingCanvas};
use slot2_input::{Button, Event};
use slot2_platform::{Gauge, Geometry};
use slot2_store::{Card, Platform};
use slot2_ui::hud::{GAUGE_H, GAUGE_W, HUD_H, HUD_MARGIN, WALL};

use slot2::app::{App, Screen, BATTERY_POLL_S};

static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

const GEOMETRY: Geometry = Geometry::W720H480;

fn scratch(tag: &str) -> PathBuf {
    let i = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-hudapp-{tag}-{}-{i}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn app(screen: Screen) -> (App, PathBuf) {
    let root = scratch("card");
    let card = Card::new(&root);
    card.ensure_layout();
    for p in Platform::ALL {
        let ext = p.extensions().first().copied().unwrap_or("bin");
        for g in 0..3 {
            let _ = fs::write(card.games_dir(p).join(format!("Game {g}.{ext}")), b"rom");
        }
    }
    let a = App::with_card(
        card,
        std::env::temp_dir().join("slot2-no-cores"),
        48_000,
        slot2_retro::Tuning::handheld((720, 480)),
        false,
        screen,
    );
    (a, root)
}

/// A sysfs root with one battery at `percent`, and the path to its `capacity`.
fn gauge_at(percent: u8) -> (Gauge, PathBuf) {
    let root = scratch("sysfs");
    let dir = root.join("class/power_supply/bat0");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("type"), "Battery").unwrap();
    fs::write(dir.join("capacity"), percent.to_string()).unwrap();
    fs::write(dir.join("status"), "Discharging").unwrap();
    (Gauge::probe(&root), dir.join("capacity"))
}

fn frame(a: &mut App, now: Instant) -> RecordingCanvas {
    let safe = slot2_ui::layout::SafeArea::for_geometry(GEOMETRY);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut ctx = slot2_ui::UiCtx::new(slot2_platform::detect().profile, "en", Vec::new(), None);
    ctx.safe = safe;
    a.draw(&mut canvas, &mut ctx, now);
    canvas
}

/// Rects that lie inside the band the HUD is pinned to. Everything else on screen belongs to
/// a screen underneath, and this is how the two are told apart without reaching into the app.
fn in_band(c: &RecordingCanvas) -> Vec<(f32, f32, f32, f32)> {
    c.frame()
        .iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .filter(|(_, y, _, h)| *y >= HUD_MARGIN - 1.0 && y + h <= HUD_MARGIN + HUD_H + 1.0)
        .collect()
}

/// Whether the capsule is on screen: a quad exactly as wide as the gauge and as thin as its
/// wall, inside the band. Nothing else on any screen is that shape in that place.
fn has_capsule(c: &RecordingCanvas) -> bool {
    in_band(c)
        .iter()
        .any(|(_, _, w, h)| (*w - GAUGE_W).abs() < 0.01 && (*h - WALL).abs() < 0.01)
}

/// How wide the capsule fill is, in the band. `None` when there is no capsule.
fn fill_w(c: &RecordingCanvas) -> Option<f32> {
    let inner_h = GAUGE_H - 4.0 * WALL;
    in_band(c)
        .into_iter()
        .find(|(_, _, _, h)| (*h - inner_h).abs() < 0.01)
        .map(|(_, _, w, _)| w)
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

// ------------------------------------------------------------------ who wears it

#[test]
fn the_shelf_wears_the_hud() {
    let (mut a, _root) = app(Screen::List);
    let (g, _) = gauge_at(64);
    a.set_gauge(g);
    let t = Instant::now();
    a.tick(t);
    assert!(
        has_capsule(&frame(&mut a, t)),
        "the shelf had no gauge on it"
    );
}

#[test]
fn an_insert_keeps_it() {
    // The cart is on its way in and the shelf is still the screen. A HUD that blinked out
    // for the length of the animation would read as a glitch, not as a mode change.
    let (mut a, _root) = app(Screen::List);
    let (g, _) = gauge_at(64);
    a.set_gauge(g);
    let t = Instant::now();
    a.tick(t);
    tap(&mut a, Button::A, t);
    assert_eq!(a.screen, Screen::Inserting);
    assert!(
        has_capsule(&frame(&mut a, t + Duration::from_millis(100))),
        "the gauge went out during the insert"
    );
}

#[test]
fn a_running_game_does_not() {
    // The game owns the screen. Corner furniture over someone else's picture is the thing
    // every frontend that does it gets complained about.
    let (mut a, _root) = app(Screen::Playing);
    let (g, _) = gauge_at(64);
    a.set_gauge(g);
    let t = Instant::now();
    a.tick(t);
    assert!(
        !has_capsule(&frame(&mut a, t)),
        "the gauge drew over a game"
    );
}

#[test]
fn the_boot_screen_does_not() {
    // The splash is a composition with its own middle, and it is on screen for a second and
    // a half. Nothing has been scanned yet either.
    let (mut a, _root) = app(Screen::Splash);
    let (g, _) = gauge_at(64);
    a.set_gauge(g);
    let t = Instant::now();
    a.tick(t);
    assert!(
        !has_capsule(&frame(&mut a, t)),
        "the gauge drew on the splash"
    );
}

#[test]
fn a_machine_with_no_gauge_still_draws_a_shelf() {
    let (mut a, _root) = app(Screen::List);
    let t = Instant::now();
    a.tick(t);
    let c = frame(&mut a, t);
    assert!(
        !has_capsule(&c),
        "a capsule appeared with no gauge behind it"
    );
    assert!(!c.frame().is_empty(), "the shelf stopped drawing entirely");
}

// ------------------------------------------------------------------ how often it asks

#[test]
fn the_first_frame_after_a_gauge_arrives_already_shows_it() {
    // Waiting out a poll interval before the first reading would leave the corner empty for
    // the first ten seconds of every boot, which is most of the time anyone looks at it.
    let (mut a, _root) = app(Screen::List);
    let (g, _) = gauge_at(100);
    a.set_gauge(g);
    let t = Instant::now();
    a.tick(t);
    assert!(fill_w(&frame(&mut a, t)).is_some_and(|w| w > 0.0));
}

#[test]
fn the_gauge_is_read_on_its_own_clock_not_on_the_frame_clock() {
    // Sysfs is a file read and the frontend draws sixty times a second. What is on screen is
    // the last reading, not a fresh one, and the interval is what makes that true.
    //
    // Measured against `now`, not against the `dt` the animations run on: that dt is clamped
    // to 1/30 s so a slow frame cannot destabilise the row spring, and a poll counted in
    // clamped dt would take three hundred ticks to reach ten seconds. This test jumps the
    // clock by whole intervals, which is what a real device does when a frame stalls.
    let (mut a, _root) = app(Screen::List);
    let (g, capacity) = gauge_at(100);
    a.set_gauge(g);
    let t = Instant::now();
    a.tick(t);
    let full = fill_w(&frame(&mut a, t)).expect("no fill on the first frame");

    fs::write(&capacity, "10").unwrap();
    let soon = t + Duration::from_secs_f32(BATTERY_POLL_S * 0.5);
    a.tick(soon);
    assert_eq!(
        fill_w(&frame(&mut a, soon)),
        Some(full),
        "the gauge was re-read inside its own interval"
    );

    let later = t + Duration::from_secs_f32(BATTERY_POLL_S * 1.5);
    a.tick(later);
    let low = fill_w(&frame(&mut a, later)).expect("no fill after the interval");
    assert!(
        low < full,
        "the gauge never noticed the pack had dropped: {low} vs {full}"
    );
}

#[test]
fn the_poll_interval_is_seconds_not_frames() {
    // A number small enough to be a per-frame read in disguise is the defect above, spelled
    // as a constant instead of as a call.
    assert!(
        (1.0..=60.0).contains(&BATTERY_POLL_S),
        "a {BATTERY_POLL_S}s poll is not a poll"
    );
}
