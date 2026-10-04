//! The contract for the platform skin table and the SVG rasteriser. Task 10 makes these
//! pass without editing this file.

use slot2_store::Platform;
use slot2_ui::insert::Curve;
use slot2_ui::shelf::{MOUTH_EXTRA, MOUTH_H};
use slot2_ui::skin::{self, Finish, PlatformSkin, SoundProfile};
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
    }
}

#[test]
fn every_shelf_has_its_own_artwork() {
    // Every platform ships a shell and a moulding of its own. `borrowed` is the flag that says a
    // shelf is wearing another platform's drawing, and no row sets it.
    for p in ALL {
        let s = skin::skin(p);
        assert!(!s.borrowed, "{p:?} is wearing another platform's artwork");
        assert!(!s.cart.is_empty(), "{p:?} has no shell artwork");
        assert!(!s.cart_detail.is_empty(), "{p:?} has no detail artwork");
    }
}

#[test]
fn every_shelf_drew_its_own() {
    // Four carts were drawn for SLOT2 after the ported three. Every one of the seven is a
    // distinct drawing: a copy-paste would show up as two platforms that look alike, and no
    // other test would notice.
    for (i, p) in ALL.iter().enumerate() {
        let a = skin::skin(*p);
        for q in &ALL[i + 1..] {
            let b = skin::skin(*q);
            assert_ne!(a.cart, b.cart, "{p:?} and {q:?} share a shell");
            assert_ne!(
                a.cart_detail, b.cart_detail,
                "{p:?} and {q:?} share a detail drawing"
            );
        }
    }
}

#[test]
fn the_declared_sizes_are_the_ones_in_the_files() {
    // A mistyped size here draws every cart on that shelf at the wrong aspect, and nothing
    // else would catch it.
    for p in ALL {
        let s = skin::skin(p);
        assert_eq!(view_box(s.cart), s.cart_size, "{p:?} cart size");
        assert_eq!(
            view_box(s.cart_detail),
            s.cart_size,
            "{p:?} detail art must be drawn on the same grid as the shell"
        );
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

// --- the carts SLOT2 drew for itself --------------------------------------------------

/// The four carts that are not ported: the NES, Super Nintendo, Mega Drive and Master System
/// ones. The other three came from the original project.
const DRAWN: [Platform; 4] = [Platform::Nes, Platform::Snes, Platform::Md, Platform::Sms];

/// The label rect each of those four was laid out around.
fn drawn_label(p: Platform) -> (f32, f32, f32, f32) {
    match p {
        Platform::Nes => (20.0, 28.0, 170.0, 142.0),
        Platform::Snes => (25.0, 34.0, 190.0, 92.0),
        Platform::Md => (32.0, 34.0, 186.0, 84.0),
        Platform::Sms => (24.0, 32.0, 152.0, 118.0),
        other => panic!("{other:?} is not one of the carts drawn here"),
    }
}

/// Everything a mask paints, added up: the ink. Enough to compare two drawings of the same
/// size without depending on any one pixel.
fn ink(m: &svg::Mask) -> u64 {
    m.a.iter().map(|v| *v as u64).sum()
}

/// Pixels with any coverage at all, and pixels with none.
fn opaque(m: &svg::Mask) -> usize {
    m.a.iter().filter(|v| **v > 0).count()
}

/// The box the mask actually paints in, as `(left, top, right, bottom)` in mask pixels.
fn painted_box(m: &svg::Mask) -> (u32, u32, u32, u32) {
    let (mut l, mut t, mut r, mut b) = (m.w, m.h, 0u32, 0u32);
    for y in 0..m.h {
        for x in 0..m.w {
            if m.a[(y * m.w + x) as usize] > 0 {
                l = l.min(x);
                t = t.min(y);
                r = r.max(x);
                b = b.max(y);
            }
        }
    }
    (l, t, r, b)
}

fn natural(p: Platform) -> (u32, u32) {
    let s = skin::skin(p);
    (s.cart_size.0 as u32, s.cart_size.1 as u32)
}

#[test]
fn each_drawn_cart_declares_the_label_rect_it_was_drawn_for() {
    // The artwork leaves a well for the label; the table is what says where. These are the
    // numbers the four drawings were laid out around.
    for p in DRAWN {
        let s = skin::skin(p);
        assert_eq!(
            (s.label.x, s.label.y, s.label.w, s.label.h),
            drawn_label(p),
            "the {p:?} label rect moved"
        );

        let (w, h) = natural(p);
        let shell = svg::rasterize(s.cart, w, h).expect("the shell rasterises");
        let (l, t, r, b) = painted_box(&shell);
        // Natural size, so artwork units are mask pixels and the rect can be compared directly.
        assert!(
            s.label.x as u32 > l
                && s.label.y as u32 > t
                && (s.label.x + s.label.w) as u32 <= r
                && (s.label.y + s.label.h) as u32 <= b,
            "{p:?} label {:?} is not inside the shell it was drawn on: painted {l},{t}..{r},{b}",
            s.label
        );
        assert!(
            s.label.w * s.label.h < w as f32 * h as f32 * 0.9,
            "{p:?} label leaves no shell to see"
        );
    }
}

#[test]
fn each_drawn_cart_rasterises_at_both_sizes() {
    for p in DRAWN {
        let s = skin::skin(p);
        let (w, h) = natural(p);
        for (what, src) in [("cart", s.cart), ("detail", s.cart_detail)] {
            let m = svg::rasterize(src, w, h)
                .unwrap_or_else(|| panic!("{p:?} {what} does not rasterise at its own size"));
            assert_eq!((m.w, m.h), (w, h), "{p:?} {what}");
            assert!(opaque(&m) > 0, "{p:?} {what} came out blank");

            // The shelf draws these as small previews as well as at full size.
            let small = svg::rasterize_fit(src, s.cart_size, 48, 48)
                .unwrap_or_else(|| panic!("{p:?} {what} does not rasterise small"));
            assert!(small.w > 0 && small.h > 0 && small.w <= 48 && small.h <= 48);
        }
    }
}

#[test]
fn a_drawn_shell_is_a_shape_not_a_rectangle_filling_the_canvas() {
    for p in DRAWN {
        let s = skin::skin(p);
        let (w, h) = natural(p);
        let m = svg::rasterize(s.cart, w, h).expect("the shell rasterises");
        let total = m.a.len();
        assert!(
            opaque(&m) * 4 > total,
            "{p:?} shell paints less than a quarter of its canvas"
        );
        let clear = total - opaque(&m);
        assert!(
            clear * 20 > total,
            "{p:?} shell paints the whole canvas ({clear} clear of {total})"
        );
    }
}

#[test]
fn every_detail_is_a_smaller_shape_than_its_shell() {
    // A detail that painted as much as the shell would be a second shell, and the shelf would
    // draw a bearing surface with no flat plastic on it. Every platform is held to the same
    // rule, ported artwork and new alike.
    for p in ALL {
        let s = skin::skin(p);
        let (w, h) = natural(p);
        let shell = svg::rasterize(s.cart, w, h).expect("the shell rasterises");
        let detail = svg::rasterize(s.cart_detail, w, h).expect("the detail rasterises");
        assert!(ink(&detail) > 0, "{p:?} detail is blank");
        assert!(
            ink(&detail) < ink(&shell),
            "{p:?} detail paints {} against the shell's {}",
            ink(&detail),
            ink(&shell)
        );
        assert_ne!(detail.a, shell.a, "{p:?} detail is the shell");
    }
}

#[test]
fn the_drawn_carts_have_their_own_proportions() {
    // Tall or wide is what the shelf reads first: the NES and Master System carts stand up, the
    // Super Nintendo and Mega Drive ones lie down. The four are not four of the same box.
    let ratio = |p: Platform| {
        let s = skin::skin(p);
        s.cart_size.0 / s.cart_size.1
    };
    for (p, wide) in [
        (Platform::Nes, false),
        (Platform::Snes, true),
        (Platform::Md, true),
        (Platform::Sms, false),
    ] {
        assert_eq!(
            ratio(p) > 1.0,
            wide,
            "{p:?} is {} than it is tall",
            if wide { "no wider" } else { "not taller" }
        );
    }
    assert!(
        (ratio(Platform::Md) - ratio(Platform::Sms)).abs() > 0.25,
        "the Mega Drive and Master System carts have the same proportions: {} vs {}",
        ratio(Platform::Md),
        ratio(Platform::Sms)
    );
    assert!(
        (ratio(Platform::Nes) - ratio(Platform::Snes)).abs() > 0.2,
        "the NES and Super Nintendo carts have the same proportions: {} vs {}",
        ratio(Platform::Nes),
        ratio(Platform::Snes)
    );

    // And a fitted raster keeps whichever one it was asked for, in both directions.
    for p in DRAWN {
        let s = skin::skin(p);
        let want = ratio(p);
        let m = svg::rasterize_fit(s.cart, s.cart_size, 96, 96).expect("fits");
        assert!(m.w <= 96 && m.h <= 96, "{p:?} escaped the box");
        let got = m.w as f32 / m.h as f32;
        assert!(
            (got - want).abs() < 0.05,
            "{p:?} aspect {got} should be {want}"
        );
    }
}

// --- the travel profiles ---------------------------------------------------------------

#[test]
fn every_shelf_has_its_own_travel_profiles() {
    // Fourteen profiles, all different: an insert and an eject per platform, and a platform's
    // two are not each other. A shelf sharing another's rhythm would be a shelf that reads as
    // the same machine.
    let mut inserts: Vec<(Platform, Curve)> = Vec::new();
    let mut ejects: Vec<(Platform, Curve)> = Vec::new();
    for p in ALL {
        let s = skin::skin(p);
        assert_ne!(
            s.insert, s.eject,
            "{p:?} goes in and comes out on one profile"
        );
        for (q, c) in &inserts {
            assert_ne!(&s.insert, c, "{p:?} and {q:?} share an insert profile");
        }
        for (q, c) in &ejects {
            assert_ne!(&s.eject, c, "{p:?} and {q:?} share an eject profile");
        }
        inserts.push((p, s.insert));
        ejects.push((p, s.eject));
    }
}

#[test]
fn every_travel_profile_is_the_one_the_table_declares() {
    // The beats themselves, as the design states them, through the accessors rather than
    // through any drawn pixel: contact, release, creep.
    type Beats = (f32, f32, f32);
    let table: [(Platform, Beats, Beats); 7] = [
        (Platform::Gb, (0.40, 0.64, 0.025), (0.36, 0.58, 0.025)),
        (Platform::Gbc, (0.42, 0.65, 0.030), (0.38, 0.60, 0.030)),
        (Platform::Gba, (0.38, 0.58, 0.035), (0.34, 0.54, 0.035)),
        (Platform::Nes, (0.46, 0.70, 0.020), (0.42, 0.68, 0.020)),
        (Platform::Snes, (0.36, 0.56, 0.040), (0.32, 0.52, 0.040)),
        (Platform::Md, (0.34, 0.52, 0.045), (0.30, 0.50, 0.045)),
        (Platform::Sms, (0.44, 0.67, 0.028), (0.40, 0.64, 0.028)),
    ];
    let beats = |c: Curve| (c.contact(), c.release(), c.creep());
    for (p, want_in, want_out) in table {
        let s = skin::skin(p);
        assert_eq!(beats(s.insert), want_in, "{p:?} insert beats");
        assert_eq!(beats(s.eject), want_out, "{p:?} eject beats");
        // And the bounds the sampler's own contract depends on.
        for c in [s.insert, s.eject] {
            assert!(
                0.0 < c.contact() && c.contact() < c.release() && c.release() < 1.0,
                "{p:?}: contact {} release {}",
                c.contact(),
                c.release()
            );
            assert!(
                0.0 < c.creep() && c.creep() < 0.08,
                "{p:?}: creep {}",
                c.creep()
            );
        }
    }
}

// --- the sound profiles ----------------------------------------------------------------

/// The two numbers the design gives each shelf for the two recordings: `(speed, gain)` for the
/// insert and for the eject. Written out here as a table of its own so the row order can be
/// read against the design in one place.
type Sound = (f32, f32);

const SOUNDS: [(Platform, Sound, Sound); 7] = [
    (Platform::Gb, (0.86, 0.92), (0.89, 0.88)),
    (Platform::Gbc, (0.93, 0.88), (0.96, 0.84)),
    (Platform::Gba, (1.10, 0.86), (1.13, 0.82)),
    (Platform::Nes, (0.82, 1.00), (0.85, 0.96)),
    (Platform::Snes, (1.00, 0.96), (1.03, 0.92)),
    (Platform::Md, (1.16, 0.90), (1.19, 0.86)),
    (Platform::Sms, (0.96, 0.82), (0.99, 0.78)),
];

#[test]
fn every_shelf_plays_the_recordings_its_row_declares() {
    // Direction matters here: `sfx_in` is the cart going in and `sfx_out` the cart coming out,
    // and the two columns of the design's table are not interchangeable.
    for (p, want_in, want_out) in SOUNDS {
        let s = skin::skin(p);
        assert_eq!(
            (s.sfx_in.speed(), s.sfx_in.gain()),
            want_in,
            "{p:?} insert profile"
        );
        assert_eq!(
            (s.sfx_out.speed(), s.sfx_out.gain()),
            want_out,
            "{p:?} eject profile"
        );
        for (what, prof) in [("insert", s.sfx_in), ("eject", s.sfx_out)] {
            assert!(
                (0.80..=1.20).contains(&prof.speed()),
                "{p:?} {what}: {}x is outside the range a clip may be read at",
                prof.speed()
            );
            assert!(
                prof.gain() > 0.0 && prof.gain() <= 1.0,
                "{p:?} {what}: gain {} would not be a level the recording can take",
                prof.gain()
            );
        }
    }
}

#[test]
fn every_shelf_has_its_own_sound_profiles() {
    // Fourteen profiles, all different, and a platform's two are not each other. Two shelves
    // sharing one would be two shelves the ear cannot tell apart, which is the whole of what
    // this is for.
    let mut inserts: Vec<(Platform, SoundProfile)> = Vec::new();
    let mut ejects: Vec<(Platform, SoundProfile)> = Vec::new();
    for p in ALL {
        let s = skin::skin(p);
        assert_ne!(
            s.sfx_in, s.sfx_out,
            "{p:?} goes in and comes out on one profile"
        );
        for (q, c) in &inserts {
            assert_ne!(&s.sfx_in, c, "{p:?} and {q:?} share an insert profile");
        }
        for (q, c) in &ejects {
            assert_ne!(&s.sfx_out, c, "{p:?} and {q:?} share an eject profile");
        }
        inserts.push((p, s.sfx_in));
        ejects.push((p, s.sfx_out));
    }
}

#[test]
fn a_profile_out_of_range_does_not_build() {
    // The table is `const`, so a typo in it is a compile error and never reaches a test. This
    // is the same guard reached at run time, where the NaN case is what proves the range check
    // is not quietly passing everything without an order to it.
    for (speed, gain) in [
        (0.79, 1.0),
        (1.21, 1.0),
        (f32::NAN, 1.0),
        (f32::INFINITY, 1.0),
        (1.0, f32::NAN),
        (1.0, 0.69),
        (1.0, 1.01),
    ] {
        assert!(
            std::panic::catch_unwind(|| SoundProfile::new(speed, gain)).is_err(),
            "{speed}x at gain {gain} built"
        );
    }
}

// --- the mouth trims -----------------------------------------------------------------

/// Every port trim is its cart's width plus the mouth's chrome, with 20px of trim either side.
fn port_mouth(p: Platform) -> f32 {
    skin::skin(p).cart_size.0 + MOUTH_EXTRA
}

#[test]
fn every_shelf_has_its_own_port_trim() {
    for (i, p) in ALL.iter().enumerate() {
        let a = skin::skin(*p);
        assert!(!a.port.is_empty(), "{p:?} has no port trim");
        for q in &ALL[i + 1..] {
            assert_ne!(
                a.port,
                skin::skin(*q).port,
                "{p:?} and {q:?} share a port trim"
            );
        }
    }
}

#[test]
fn a_port_trim_fits_the_mouth_it_frames() {
    for p in ALL {
        let s = skin::skin(p);
        assert_eq!(view_box(s.port), s.port_size, "{p:?} port size");
        assert_eq!(
            s.port_size.1, MOUTH_H,
            "{p:?} trim is not the height of the mouth"
        );
        assert_eq!(
            s.port_size.0,
            s.cart_size.0 + MOUTH_EXTRA + 40.0,
            "{p:?} trim is not the mouth plus 20px either side"
        );
    }
}

#[test]
fn every_port_trim_rasterises_at_both_sizes() {
    for p in ALL {
        let s = skin::skin(p);
        let (w, h) = (s.port_size.0 as u32, s.port_size.1 as u32);
        let m = svg::rasterize(s.port, w, h)
            .unwrap_or_else(|| panic!("{p:?} port trim does not rasterise"));
        assert_eq!((m.w, m.h), (w, h), "{p:?} port trim");
        assert!(opaque(&m) > 0, "{p:?} port trim came out blank");
        assert!(
            opaque(&m) * 3 < m.a.len(),
            "{p:?} port trim fills its whole canvas"
        );

        // The shelf draws it at the panel's size as well as at this one.
        let small = svg::rasterize_fit(s.port, s.port_size, 64, 58)
            .unwrap_or_else(|| panic!("{p:?} port trim does not rasterise small"));
        assert!(small.w > 0 && small.h > 0 && small.w <= 64 && small.h <= 58);
    }
}

#[test]
fn every_port_mask_is_its_own_drawing() {
    // Different source strings are not enough: two shelves whose trim rasterises to the same
    // coverage are one drawing with a comment on top.
    let mut seen: Vec<(Platform, svg::Mask)> = Vec::new();
    for p in ALL {
        let s = skin::skin(p);
        let m = svg::rasterize(s.port, s.port_size.0 as u32, s.port_size.1 as u32)
            .expect("the trim rasterises");
        for (q, other) in &seen {
            assert!(
                !(m.w == other.w && m.h == other.h && m.a == other.a),
                "{p:?} and {q:?} draw the same trim"
            );
        }
        seen.push((p, m));
    }
}

#[test]
fn a_ports_middle_is_the_open_mouth() {
    // The trim goes on over everything, so the cartridge in the slot is visible only because
    // nothing is painted in the middle of the mouth. And a trim that painted nothing at all
    // either side of it would not be trimming anything.
    for p in ALL {
        let s = skin::skin(p);
        let (w, h) = (s.port_size.0 as u32, s.port_size.1 as u32);
        let m = svg::rasterize(s.port, w, h).expect("the trim rasterises");
        let at = |x: u32, y: u32| m.a[(y * m.w + x) as usize];

        let cx = m.w / 2;
        for y in ((m.h as f32 * 0.65) as u32)..((m.h as f32 * 0.85) as u32) {
            for x in cx - 8..=cx + 8 {
                assert_eq!(
                    at(x, y),
                    0,
                    "{p:?} trim paints the middle of the mouth at {x},{y}"
                );
            }
        }

        let side = |from: u32, to: u32| -> usize {
            (from..to)
                .map(|x| (0..m.h).filter(|y| at(x, *y) > 0).count())
                .sum()
        };
        let left = side(2, 18);
        let right = side(m.w - 18, m.w - 2);
        assert!(left > 100, "{p:?} has no trim left of the mouth ({left})");
        assert!(
            right > 100,
            "{p:?} has no trim right of the mouth ({right})"
        );
        assert!(
            m.w as f32 > port_mouth(p),
            "{p:?} has no trim outside the mouth at all"
        );
    }
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
        for (what, src) in [("cart", s.cart), ("detail", s.cart_detail)] {
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
