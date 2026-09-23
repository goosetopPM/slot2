//! The contract for the platform skin table and the SVG rasteriser. Task 10 makes these
//! pass without editing this file.

use slot2_store::Platform;
use slot2_ui::skin::{self, Finish, PlatformSkin};
use slot2_ui::svg;

const ALL: [Platform; 7] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Nes,
    Platform::Snes,
    Platform::Md,
    Platform::Sms,
];

/// Pull `viewBox="a b w h"` out of SVG source, so the table can be checked against the file
/// it describes rather than against itself.
fn view_box(src: &str) -> (f32, f32) {
    let at = src.find("viewBox=\"").expect("a viewBox");
    let rest = &src[at + 9..];
    let end = rest.find('"').expect("a closed viewBox");
    let n: Vec<f32> = rest[..end]
        .split_whitespace()
        .map(|v| v.parse().expect("a number"))
        .collect();
    assert_eq!(n.len(), 4, "viewBox should have four numbers");
    (n[2], n[3])
}

#[test]
fn every_platform_has_a_skin() {
    for p in ALL {
        let s: &PlatformSkin = skin::skin(p);
        assert_eq!(s.platform, p, "the table is out of order or keyed wrong");
        assert!(!s.cart.is_empty(), "{p:?} has no cart artwork");
        assert!(!s.port.is_empty(), "{p:?} has no slot artwork");
    }
}

#[test]
fn the_four_shelves_without_artwork_borrow_and_admit_it() {
    // DESIGN §7: the Game Boys and the GBA have drawings; NES, SNES, Mega Drive and Master
    // System fall back to the GBA's until someone draws them. A skin that borrowed quietly
    // would leave nobody able to tell which shelves still need art.
    let gba = skin::skin(Platform::Gba);
    for p in [Platform::Gb, Platform::Gbc, Platform::Gba] {
        assert!(!skin::skin(p).borrowed, "{p:?} has its own artwork");
    }
    for p in [Platform::Nes, Platform::Snes, Platform::Md, Platform::Sms] {
        let s = skin::skin(p);
        assert!(s.borrowed, "{p:?} claims artwork it does not have");
        assert_eq!(s.cart, gba.cart, "{p:?} should fall back to the GBA cart");
        assert_eq!(s.cart_size, gba.cart_size);
    }
}

#[test]
fn the_declared_sizes_are_the_ones_in_the_files() {
    // A mistyped size here draws every cart on that shelf at the wrong aspect, and nothing
    // else would catch it.
    for p in ALL {
        let s = skin::skin(p);
        assert_eq!(view_box(s.cart), s.cart_size, "{p:?} cart size");
        assert_eq!(view_box(s.port), s.port_size, "{p:?} port size");
        if !s.cart_detail.is_empty() {
            assert_eq!(
                view_box(s.cart_detail),
                s.cart_size,
                "{p:?} detail art must be drawn on the same grid as the shell"
            );
        }
    }
}

#[test]
fn the_label_sits_on_the_cart() {
    for p in ALL {
        let s = skin::skin(p);
        let (w, h) = s.cart_size;
        assert!(
            s.label.w > 0.0 && s.label.h > 0.0,
            "{p:?} has no label area"
        );
        assert!(
            s.label.x >= 0.0
                && s.label.y >= 0.0
                && s.label.x + s.label.w <= w
                && s.label.y + s.label.h <= h,
            "{p:?} label {:?} falls off a {w}x{h} cart",
            s.label
        );
        // A label covering the whole shell means the shell is never seen, which is a
        // mistyped rect rather than a design.
        assert!(
            s.label.w * s.label.h < w * h * 0.9,
            "{p:?} label covers the whole cart"
        );
    }
}

#[test]
fn a_shell_colour_is_always_available() {
    for p in ALL {
        let s = skin::skin(p);
        assert!(
            matches!(s.shell.finish, Finish::Solid | Finish::Translucent),
            "{p:?}"
        );
    }
    // The Game Boy's grey pak and the GBA's dark shell should not be the same colour, or
    // switching shelves would look like nothing happened.
    assert_ne!(
        skin::skin(Platform::Gb).shell.colour,
        skin::skin(Platform::Gba).shell.colour
    );
}

// --- the rasteriser ------------------------------------------------------------------

#[test]
fn artwork_rasterises_to_the_size_asked_for() {
    let s = skin::skin(Platform::Gba);
    let m = svg::rasterize(s.cart, 240, 135).expect("the GBA cart rasterises");
    assert_eq!((m.w, m.h), (240, 135));
    assert_eq!(m.a.len(), 240 * 135, "one coverage byte per pixel");
    assert!(m.a.iter().any(|v| *v > 200), "the drawing came out blank");
    assert!(
        m.a.iter().any(|v| *v < 50),
        "the drawing has no transparency"
    );
}

#[test]
fn a_bigger_raster_carries_more_ink() {
    // The art is vector, so it is drawn at the size the panel wants rather than scaled from
    // one bitmap. Coverage should grow with the area, near enough.
    let s = skin::skin(Platform::Gba);
    let ink = |w: u32, h: u32| {
        svg::rasterize(s.cart, w, h)
            .unwrap()
            .a
            .iter()
            .map(|v| *v as u64)
            .sum::<u64>()
    };
    let small = ink(120, 68) as f64;
    let big = ink(480, 270) as f64;
    let ratio = big / small;
    assert!(
        (10.0..=22.0).contains(&ratio),
        "four times the width should be about sixteen times the ink, got {ratio:.1}"
    );
}

#[test]
fn fitting_keeps_the_aspect_and_stays_inside_the_box() {
    let s = skin::skin(Platform::Gb); // 240x253, taller than wide
    let m = svg::rasterize_fit(s.cart, s.cart_size, 200, 200).expect("fits");
    assert!(m.w <= 200 && m.h <= 200, "{}x{} escaped the box", m.w, m.h);
    assert_eq!(m.h, 200, "the tall side should be the one that fills");
    let want = s.cart_size.0 / s.cart_size.1;
    let got = m.w as f32 / m.h as f32;
    assert!((got - want).abs() < 0.02, "aspect {got} should be {want}");
}

#[test]
fn nonsense_is_refused_rather_than_fatal() {
    // Artwork can be overridden from the card later, so a broken file must cost a missing
    // cartridge, not the frontend.
    assert!(svg::rasterize("not an svg at all", 32, 32).is_none());
    assert!(svg::rasterize("<svg", 32, 32).is_none());
    assert!(svg::rasterize("", 32, 32).is_none());
    let s = skin::skin(Platform::Gba);
    assert!(svg::rasterize(s.cart, 0, 32).is_none(), "zero width");
    assert!(svg::rasterize(s.cart, 32, 0).is_none(), "zero height");
    assert!(
        svg::rasterize_fit(s.cart, (0.0, 0.0), 100, 100).is_none(),
        "artwork with no size"
    );
}

#[test]
fn every_skins_artwork_actually_parses() {
    // The table points at files; this is what says the files are real and drawable.
    for p in ALL {
        let s = skin::skin(p);
        for (what, src) in [
            ("cart", s.cart),
            ("detail", s.cart_detail),
            ("port", s.port),
        ] {
            if src.is_empty() {
                continue;
            }
            assert!(
                svg::rasterize(src, 64, 64).is_some(),
                "{p:?} {what} artwork does not rasterise"
            );
        }
    }
}
