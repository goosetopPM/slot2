//! The contract for slot2-input. Task 05 makes these pass without editing this file.

use std::time::{Duration, Instant};

use slot2_input::evdev::{codes, EV_ABS, EV_KEY, EV_SW, SW_LID};
use slot2_input::{
    parse_input_event, Action, Axis, Button, EvdevSource, Event, GestureConfig, Gestures, KeyMap,
    RawEvent, State, DEFAULT_H700_KEYMAP,
};

fn t0() -> Instant {
    Instant::now()
}
fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}
fn press(b: Button, at: Instant) -> Event {
    Event::Button {
        button: b,
        pressed: true,
        at,
    }
}
fn release(b: Button, at: Instant) -> Event {
    Event::Button {
        button: b,
        pressed: false,
        at,
    }
}

// ---------- evdev parsing and mapping ----------

fn raw_bytes(type_: u16, code: u16, value: i32) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&1_700_000_000i64.to_le_bytes());
    v.extend_from_slice(&123_456i64.to_le_bytes());
    v.extend_from_slice(&type_.to_le_bytes());
    v.extend_from_slice(&code.to_le_bytes());
    v.extend_from_slice(&value.to_le_bytes());
    v
}

#[test]
fn input_event_is_24_bytes_little_endian() {
    let b = raw_bytes(EV_KEY, codes::BTN_SOUTH, 1);
    assert_eq!(b.len(), 24);
    assert_eq!(
        parse_input_event(&b),
        Some(RawEvent {
            type_: EV_KEY,
            code: codes::BTN_SOUTH,
            value: 1
        })
    );
    assert_eq!(parse_input_event(&b[..23]), None);
    // Extra trailing bytes are fine (a buffer holds several events).
    let mut two = b.clone();
    two.extend(raw_bytes(EV_KEY, codes::BTN_SOUTH, 0));
    assert_eq!(
        parse_input_event(&two[24..]),
        Some(RawEvent {
            type_: EV_KEY,
            code: codes::BTN_SOUTH,
            value: 0
        })
    );
    assert_eq!(
        parse_input_event(&raw_bytes(EV_ABS, 0, -32768))
            .unwrap()
            .value,
        -32768
    );
}

#[test]
fn keymap_maps_keys_axes_and_lid() {
    let km = KeyMap::from_pairs(DEFAULT_H700_KEYMAP);
    let now = t0();
    let map = |type_, code, value| EvdevSource::map_raw(&km, RawEvent { type_, code, value }, now);
    assert_eq!(
        map(EV_KEY, codes::BTN_SOUTH, 1),
        Some(press(Button::A, now))
    );
    assert_eq!(
        map(EV_KEY, codes::BTN_SOUTH, 0),
        Some(release(Button::A, now))
    );
    assert_eq!(
        map(EV_KEY, codes::BTN_SOUTH, 2),
        None,
        "auto-repeat is dropped"
    );
    assert_eq!(
        map(EV_KEY, codes::BTN_MODE, 1),
        Some(press(Button::Menu, now))
    );
    assert_eq!(map(EV_KEY, codes::KEY_UP, 1), Some(press(Button::Up, now)));
    assert_eq!(
        map(EV_KEY, codes::BTN_DPAD_UP, 1),
        Some(press(Button::Up, now))
    );
    assert_eq!(map(EV_KEY, 9999, 1), None, "unknown code");
    assert_eq!(map(0, 0, 0), None, "EV_SYN");
    match map(EV_ABS, 0, 32767) {
        Some(Event::Axis {
            axis: Axis::LeftX,
            value,
            ..
        }) => assert!((value - 1.0).abs() < 1e-3),
        other => panic!("{other:?}"),
    }
    match map(EV_ABS, 1, -32768) {
        Some(Event::Axis {
            axis: Axis::LeftY,
            value,
            ..
        }) => assert!((value + 1.0).abs() < 1e-3),
        other => panic!("{other:?}"),
    }
    match map(EV_ABS, 5, 255) {
        Some(Event::Axis {
            axis: Axis::R2,
            value,
            ..
        }) => assert!((value - 1.0).abs() < 1e-3),
        other => panic!("{other:?}"),
    }
    match map(EV_ABS, 2, 0) {
        Some(Event::Axis {
            axis: Axis::L2,
            value,
            ..
        }) => assert_eq!(value, 0.0),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        map(EV_SW, SW_LID, 1),
        Some(Event::Lid {
            closed: true,
            at: now
        })
    );
    assert_eq!(
        map(EV_SW, SW_LID, 0),
        Some(Event::Lid {
            closed: false,
            at: now
        })
    );
    assert_eq!(map(EV_SW, 7, 1), None, "other switches ignored");
}

#[test]
fn evdev_source_without_devices_is_quiet() {
    // On Windows there are no /dev/input nodes; on a Linux CI box there may be none we can
    // read. Either way: no panic, no events.
    let mut src = EvdevSource::open_all(KeyMap::from_pairs(DEFAULT_H700_KEYMAP));
    let _ = src.device_count();
    assert!(src.poll(t0()).is_empty() || cfg!(unix));
}

// ---------- state ----------

#[test]
fn state_tracks_buttons_axes_and_lid() {
    let mut s = State::default();
    let now = t0();
    assert!(!s.pressed(Button::A));
    s.feed(&press(Button::A, now));
    s.feed(&press(Button::Power, now));
    assert!(s.pressed(Button::A) && s.pressed(Button::Power));
    assert_eq!(s.held(), vec![Button::A, Button::Power]);
    s.feed(&release(Button::A, now));
    assert!(!s.pressed(Button::A));
    s.feed(&Event::Axis {
        axis: Axis::RightY,
        value: 2.0,
        at: now,
    });
    assert_eq!(s.axis(Axis::RightY), 1.0, "clamped");
    s.feed(&Event::Lid {
        closed: true,
        at: now,
    });
    assert!(s.lid_closed());
}

// ---------- gestures ----------

fn g() -> Gestures {
    Gestures::new(GestureConfig::default())
}

#[test]
fn tap_is_press_then_quick_release() {
    let mut g = g();
    let t = t0();
    assert_eq!(g.feed(&press(Button::A, t)), vec![Action::Down(Button::A)]);
    assert_eq!(
        g.feed(&release(Button::A, t + ms(80))),
        vec![Action::Up(Button::A), Action::Tap(Button::A)]
    );
    assert!(g.tick(t + ms(2000)).is_empty());
}

#[test]
fn hold_fires_once_and_swallows_the_release() {
    let mut g = g();
    let t = t0();
    g.feed(&press(Button::Menu, t));
    assert_eq!(g.hold_progress(Button::Menu, t), Some(0.0));
    assert!(g.tick(t + ms(300)).is_empty());
    let p = g.hold_progress(Button::Menu, t + ms(300)).unwrap();
    assert!((p - 0.5).abs() < 0.01, "{p}");
    assert_eq!(g.tick(t + ms(600)), vec![Action::Hold(Button::Menu)]);
    assert_eq!(
        g.hold_progress(Button::Menu, t + ms(700)),
        None,
        "fired: no ring"
    );
    assert!(g.tick(t + ms(900)).is_empty(), "fires once");
    assert_eq!(
        g.feed(&release(Button::Menu, t + ms(1000))),
        vec![Action::Up(Button::Menu)],
        "no tap after a hold"
    );
    assert_eq!(g.hold_progress(Button::Menu, t + ms(1100)), None);
}

#[test]
fn hold_can_be_detected_by_a_late_event_instead_of_tick() {
    let mut g = g();
    let t = t0();
    g.feed(&press(Button::Menu, t));
    // No tick happened; the next event arrives after the hold time.
    let out = g.feed(&press(Button::A, t + ms(700)));
    assert!(out.contains(&Action::Hold(Button::Menu)), "{out:?}");
    assert!(out.contains(&Action::Down(Button::A)));
}

#[test]
fn double_tap_only_for_configured_buttons() {
    let mut g = g();
    let t = t0();
    // R2 is configured: first tap is delayed, second press within the window = double.
    g.feed(&press(Button::R2, t));
    assert_eq!(
        g.feed(&release(Button::R2, t + ms(50))),
        vec![Action::Up(Button::R2)],
        "tap withheld"
    );
    assert!(g.tick(t + ms(200)).is_empty(), "still inside the window");
    let out = g.feed(&press(Button::R2, t + ms(250)));
    assert_eq!(
        out,
        vec![Action::Down(Button::R2), Action::DoubleTap(Button::R2)]
    );
    assert_eq!(
        g.feed(&release(Button::R2, t + ms(300))),
        vec![Action::Up(Button::R2)],
        "second tap suppressed"
    );
    // A lone R2 tap arrives once the window closes.
    g.feed(&press(Button::R2, t + ms(1000)));
    g.feed(&release(Button::R2, t + ms(1050)));
    assert!(g.tick(t + ms(1200)).is_empty());
    assert_eq!(g.tick(t + ms(1301)), vec![Action::Tap(Button::R2)]);
    // MENU is not configured: two quick taps are two taps, both immediate.
    assert_eq!(
        g.feed(&press(Button::Menu, t + ms(2000))),
        vec![Action::Down(Button::Menu)]
    );
    assert_eq!(
        g.feed(&release(Button::Menu, t + ms(2050))),
        vec![Action::Up(Button::Menu), Action::Tap(Button::Menu)]
    );
    g.feed(&press(Button::Menu, t + ms(2100)));
    assert_eq!(
        g.feed(&release(Button::Menu, t + ms(2150))),
        vec![Action::Up(Button::Menu), Action::Tap(Button::Menu)]
    );
}

#[test]
fn select_chords_suppress_both_taps() {
    let mut g = g();
    let t = t0();
    assert_eq!(
        g.feed(&press(Button::Select, t)),
        vec![Action::Down(Button::Select)]
    );
    assert_eq!(
        g.feed(&press(Button::R1, t + ms(100))),
        vec![Action::Down(Button::R1), Action::Chord(Button::R1)]
    );
    assert_eq!(
        g.feed(&release(Button::R1, t + ms(150))),
        vec![Action::Up(Button::R1)]
    );
    // A second chord while SELECT is still held.
    assert_eq!(
        g.feed(&press(Button::L1, t + ms(200))),
        vec![Action::Down(Button::L1), Action::Chord(Button::L1)]
    );
    g.feed(&release(Button::L1, t + ms(250)));
    assert_eq!(
        g.feed(&release(Button::Select, t + ms(300))),
        vec![Action::Up(Button::Select)],
        "used modifier: no tap"
    );
    // SELECT alone is still a tap.
    g.feed(&press(Button::Select, t + ms(400)));
    assert_eq!(
        g.feed(&release(Button::Select, t + ms(450))),
        vec![Action::Up(Button::Select), Action::Tap(Button::Select)]
    );
    // Holding SELECT long without a chord is a hold, not a tap.
    g.feed(&press(Button::Select, t + ms(1000)));
    assert_eq!(g.tick(t + ms(1600)), vec![Action::Hold(Button::Select)]);
}

#[test]
fn chorded_button_never_holds() {
    let mut g = g();
    let t = t0();
    g.feed(&press(Button::Select, t));
    g.feed(&press(Button::Menu, t + ms(10)));
    assert!(
        g.tick(t + ms(2000))
            .iter()
            .all(|a| *a != Action::Hold(Button::Menu)),
        "chorded MENU must not eject"
    );
}

#[test]
fn stray_releases_and_non_button_events_are_ignored() {
    let mut g = g();
    let t = t0();
    assert!(g.feed(&release(Button::B, t)).is_empty());
    assert!(g
        .feed(&Event::Axis {
            axis: Axis::LeftX,
            value: 0.5,
            at: t
        })
        .is_empty());
    assert!(g
        .feed(&Event::Lid {
            closed: true,
            at: t
        })
        .is_empty());
    assert_eq!(g.hold_progress(Button::B, t), None);
}

#[cfg(feature = "host")]
#[test]
fn host_keys_map_to_buttons() {
    use slot2_input::host::{host_map, KeyCode};
    assert_eq!(host_map(KeyCode::ArrowUp), Some(Button::Up));
    assert_eq!(host_map(KeyCode::KeyX), Some(Button::A));
    assert_eq!(host_map(KeyCode::KeyZ), Some(Button::B));
    assert_eq!(host_map(KeyCode::KeyS), Some(Button::X));
    assert_eq!(host_map(KeyCode::KeyA), Some(Button::Y));
    assert_eq!(host_map(KeyCode::KeyQ), Some(Button::L1));
    assert_eq!(host_map(KeyCode::KeyW), Some(Button::R1));
    assert_eq!(host_map(KeyCode::Digit1), Some(Button::L2));
    assert_eq!(host_map(KeyCode::Digit2), Some(Button::R2));
    assert_eq!(host_map(KeyCode::Enter), Some(Button::Start));
    assert_eq!(host_map(KeyCode::ShiftRight), Some(Button::Select));
    assert_eq!(host_map(KeyCode::KeyM), Some(Button::Menu));
    assert_eq!(host_map(KeyCode::Backspace), Some(Button::Menu));
    assert_eq!(host_map(KeyCode::PageUp), Some(Button::VolUp));
    assert_eq!(host_map(KeyCode::PageDown), Some(Button::VolDown));
    assert_eq!(host_map(KeyCode::KeyP), Some(Button::Power));
    assert_eq!(host_map(KeyCode::Escape), None);
    assert_eq!(host_map(KeyCode::F1), None);
}
