//! The contract for the gauge and the clock. Task 20 makes these pass without editing this
//! file.

use std::fs;
use std::path::PathBuf;

use slot2_platform::battery::{parse_charge, parse_override, Battery, Charge, Gauge};
use slot2_platform::clock::{hhmm, is_set, parse_offset_min, DAY, SET_AFTER};

static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// A sysfs root with the named supplies under `class/power_supply`. Each is
/// `(dir, type, files)`, and a file with a `None` body is not written at all.
fn sysfs(supplies: &[(&str, &str, &[(&str, Option<&str>)])]) -> PathBuf {
    let i = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-gauge-{}-{i}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for (name, kind, files) in supplies {
        let dir = root.join("class/power_supply").join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("type"), kind).unwrap();
        for (file, body) in *files {
            if let Some(b) = body {
                fs::write(dir.join(file), b).unwrap();
            }
        }
    }
    root
}

const DISCHARGING_64: &[(&str, Option<&str>)] = &[
    ("capacity", Some("64\n")),
    ("status", Some("Discharging\n")),
];

// ------------------------------------------------------------------ finding it

#[test]
fn a_charger_is_not_a_battery() {
    // V-2 saw both on the RG SP: `axp2202-battery` and `axp2202-usb`. The charger sorts
    // second by name here, but a walk that took the first directory it could open would
    // still be one rename away from reporting the cable as the battery.
    let root = sysfs(&[
        ("axp2202-battery", "Battery", DISCHARGING_64),
        ("axp2202-usb", "USB", &[("online", Some("1\n"))]),
    ]);
    let g = Gauge::probe(&root);
    assert_eq!(
        g.read(),
        Some(Battery {
            percent: 64,
            charge: Charge::Discharging
        })
    );
    assert!(g.path().is_some_and(|p| p.ends_with("axp2202-battery")));
}

#[test]
fn a_usb_supply_that_sorts_first_does_not_win() {
    let root = sysfs(&[
        ("aaa-usb", "USB", &[("online", Some("1\n"))]),
        ("zzz-battery", "Battery", DISCHARGING_64),
    ]);
    assert_eq!(Gauge::probe(&root).read().map(|b| b.percent), Some(64));
}

#[test]
fn two_batteries_pick_the_same_one_on_every_boot() {
    // `read_dir` order is the filesystem order, not the kernel order. A gauge that reports a
    // different supply after a reboot is a gauge nobody can chase a bug through.
    let root = sysfs(&[
        (
            "bat1",
            "Battery",
            &[("capacity", Some("11")), ("status", Some("Full"))],
        ),
        (
            "bat0",
            "Battery",
            &[("capacity", Some("22")), ("status", Some("Full"))],
        ),
    ]);
    assert_eq!(Gauge::probe(&root).read().map(|b| b.percent), Some(22));
}

#[test]
fn a_type_that_says_battery_but_has_no_capacity_is_not_a_gauge() {
    let root = sysfs(&[("bat0", "Battery", &[("status", Some("Full"))])]);
    assert_eq!(Gauge::probe(&root).read(), None);
    assert_eq!(Gauge::probe(&root).path(), None);
}

#[test]
fn no_power_supply_class_is_not_a_failure() {
    // A board with no gauge is a board with no gauge. Every other machine this runs on --
    // the host window, a desktop -- is that board.
    let root = sysfs(&[]);
    assert_eq!(Gauge::probe(&root).read(), None);
    assert_eq!(Gauge::probe(&PathBuf::from("/nowhere/at/all")).read(), None);
    assert_eq!(Gauge::none().read(), None);
}

// ------------------------------------------------------------------ reading it

#[test]
fn a_capacity_over_a_hundred_reads_as_full() {
    let root = sysfs(&[(
        "bat0",
        "Battery",
        &[("capacity", Some("137")), ("status", Some("Full"))],
    )]);
    assert_eq!(Gauge::probe(&root).read().map(|b| b.percent), Some(100));
}

#[test]
fn a_capacity_that_will_not_parse_is_no_reading_at_all() {
    // Not zero. Zero is a battery critical, which is a screen and a shutdown; an empty file
    // is a driver that has not populated an attribute it advertises, which this PMIC does.
    for body in ["", "\n", "  ", "unknown"] {
        let root = sysfs(&[(
            "bat0",
            "Battery",
            &[("capacity", Some(body)), ("status", Some("Full"))],
        )]);
        assert_eq!(Gauge::probe(&root).read(), None, "capacity {body:?}");
    }
}

#[test]
fn the_cable_is_seen_the_moment_it_moves() {
    // Both halves are read on every call. A charge state cached at probe time is a gauge
    // that lies for as long as the poll interval, which is the whole visible behaviour of
    // the bolt.
    let root = sysfs(&[(
        "bat0",
        "Battery",
        &[("capacity", Some("50")), ("status", Some("Discharging"))],
    )]);
    let g = Gauge::probe(&root);
    assert_eq!(g.read().map(|b| b.charge), Some(Charge::Discharging));
    fs::write(root.join("class/power_supply/bat0/status"), "Charging\n").unwrap();
    assert_eq!(g.read().map(|b| b.charge), Some(Charge::Charging));
    fs::write(root.join("class/power_supply/bat0/capacity"), "51\n").unwrap();
    assert_eq!(g.read().map(|b| b.percent), Some(51));
}

#[test]
fn a_missing_status_leaves_the_percent_readable() {
    let root = sysfs(&[(
        "bat0",
        "Battery",
        &[("capacity", Some("42")), ("status", None)],
    )]);
    assert_eq!(
        Gauge::probe(&root).read(),
        Some(Battery {
            percent: 42,
            charge: Charge::Unknown
        })
    );
}

#[test]
fn the_kernel_words_decode_to_the_three_that_matter() {
    assert_eq!(parse_charge("Charging\n"), Charge::Charging);
    assert_eq!(parse_charge("Discharging"), Charge::Discharging);
    assert_eq!(parse_charge(" Full \n"), Charge::Full);
    // Everything else is the reading that leaves every policy where it was.
    for s in ["Not charging", "Unknown", "", "\n", "charging"] {
        assert_eq!(parse_charge(s), Charge::Unknown, "{s:?}");
    }
}

// ------------------------------------------------------------------ the host stand-in

#[test]
fn the_host_override_stands_in_for_a_gauge() {
    assert_eq!(
        parse_override("42"),
        Some(Battery {
            percent: 42,
            charge: Charge::Unknown
        })
    );
    assert_eq!(
        parse_override("8,discharging").map(|b| b.charge),
        Some(Charge::Discharging)
    );
    assert_eq!(
        parse_override("99,CHARGING").map(|b| b.charge),
        Some(Charge::Charging)
    );
    assert_eq!(
        parse_override(" 100 , full ").map(|b| b.charge),
        Some(Charge::Full)
    );
    assert_eq!(parse_override("0").map(|b| b.percent), Some(0));
    assert_eq!(parse_override("250").map(|b| b.percent), Some(100));
    // A typo is an absent gauge, not a gauge reading zero.
    for s in ["", "abc", "-1", "50%", "50,", "50,sideways"] {
        assert_eq!(parse_override(s), None, "{s:?}");
    }
}

#[test]
fn a_fixed_reading_is_a_gauge_that_never_changes() {
    let b = Battery {
        percent: 7,
        charge: Charge::Charging,
    };
    let g = Gauge::fixed(b);
    assert_eq!(g.read(), Some(b));
    assert_eq!(g.read(), Some(b));
    assert_eq!(g.path(), None, "a fixed reading came from no file");
}

// ------------------------------------------------------------------ the clock

#[test]
fn the_clock_wraps_and_pads() {
    assert_eq!(hhmm(0), "00:00");
    assert_eq!(hhmm(61), "00:01");
    assert_eq!(hhmm(3661), "01:01");
    assert_eq!(hhmm(DAY - 1), "23:59");
    assert_eq!(hhmm(DAY), "00:00");
    assert_eq!(hhmm(DAY * 20_000 + 9 * 3600 + 5 * 60), "09:05");
    // A time before the epoch is a time of day, not a minus sign: a negative offset on a
    // board whose RTC never started puts the clock there on the first frame.
    assert_eq!(hhmm(-1), "23:59");
    assert_eq!(hhmm(-DAY), "00:00");
    assert_eq!(hhmm(-DAY - 60), "23:59");
}

#[test]
fn an_offset_is_whole_minutes_within_reach_of_utc() {
    assert_eq!(parse_offset_min("0"), Some(0));
    assert_eq!(parse_offset_min("540"), Some(540));
    assert_eq!(parse_offset_min("-480"), Some(-480));
    assert_eq!(parse_offset_min(" 60 \n"), Some(60));
    assert_eq!(parse_offset_min("840"), Some(840));
    assert_eq!(parse_offset_min("-720"), Some(-720));
    // Outside the inhabited world, or not a whole number of minutes.
    for s in ["841", "-721", "9.5", "", "east", "+", "1e3"] {
        assert_eq!(parse_offset_min(s), None, "{s:?}");
    }
}

#[test]
fn a_clock_that_was_never_set_says_nothing() {
    // The board boots at the epoch and counts up, so within minutes it shows a confident,
    // wrong time. An empty corner is the honest reading.
    assert!(!is_set(0));
    assert!(!is_set(SET_AFTER - 1));
    assert!(!is_set(-1));
    assert!(is_set(SET_AFTER));
    assert!(is_set(SET_AFTER + DAY * 4000));
}
