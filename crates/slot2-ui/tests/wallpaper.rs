//! The contract for what the shelf stands on. Task 18 makes these pass without editing this
//! file.
//!
//! The bug behind it: nothing paints the whole panel. The slot covers its band, the carts
//! cover themselves, and neither loop clears — so the rest of the frame is whatever the last
//! frame left there. On a double-buffered device that is two frames of history alternating
//! behind a row that moves.

use std::path::PathBuf;

use slot2_gfx::{cover_uv, Op, RecordingCanvas};
use slot2_platform::Geometry;
use slot2_store::{Card, Platform};
use slot2_ui::layout::SafeArea;
use slot2_ui::wallpaper::{Wallpaper, BOTTOM, TOP};

const GEOMETRIES: [Geometry; 3] = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];

static NEXT_CARD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// A card with a wallpaper for each of `per_platform`, plus `default.png` if asked.
fn card(per_platform: &[Platform], default: bool, size: (u32, u32)) -> (Card, PathBuf) {
    let i = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-wall-{}-{i}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for p in per_platform {
        write_png(
            &root.join("Wallpapers").join(format!("{}.png", p.folder())),
            size,
        );
    }
    if default {
        write_png(&root.join("Wallpapers/default.png"), size);
    }
    (card, root)
}

/// A card with no wallpapers at all.
fn bare_card() -> (Card, PathBuf) {
    card(&[], false, (1, 1))
}

/// A solid magenta PNG. Nothing else here is that colour.
fn write_png(path: &std::path::Path, (w, h): (u32, u32)) {
    let f = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(
            &(0..w * h)
                .flat_map(|_| [0xFF, 0x00, 0xFF, 0xFF])
                .collect::<Vec<u8>>(),
        )
        .unwrap();
}

/// The area every op covers, as (left, top, right, bottom).
fn extent(canvas: &RecordingCanvas) -> (f32, f32, f32, f32) {
    let mut e = (
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    );
    for op in canvas.frame() {
        let (x, y, w, h) = match op {
            Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
            _ => continue,
        };
        e.0 = e.0.min(x);
        e.1 = e.1.min(y);
        e.2 = e.2.max(x + w);
        e.3 = e.3.max(y + h);
    }
    e
}

// ------------------------------------------------------------------ the ground

#[test]
fn the_wallpaper_covers_the_panel_with_no_card_picture() {
    // The whole point. A card with no wallpaper is the ordinary card, so the fallback has to
    // do the job, not merely exist.
    for g in GEOMETRIES {
        let safe = SafeArea::for_geometry(g);
        let mut w = Wallpaper::default();
        let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
        w.draw(&mut canvas, &safe);

        let (l, t, r, b) = extent(&canvas);
        assert!(l.is_finite(), "{g:?}: the wallpaper drew nothing");
        assert!(
            l <= 0.0 && t <= 0.0,
            "{g:?}: the wallpaper starts at {l},{t}"
        );
        assert!(
            r >= safe.panel_w as f32 && b >= safe.panel_h as f32,
            "{g:?}: the wallpaper reaches {r},{b} on a {}x{} panel",
            safe.panel_w,
            safe.panel_h
        );
    }
}

#[test]
fn the_built_in_ground_is_darker_at_the_foot() {
    // The slot is at the bottom and the carts stand above it. A face that darkens towards
    // the machine reads as a surface going back; a flat one reads as a colour the row floats
    // on.
    let lum = |c: slot2_gfx::Color| {
        let [r, g, b, _] = c.to_u8();
        0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
    };
    assert!(
        lum(TOP) > lum(BOTTOM) + 8.0,
        "the gradient is flat: {} then {}",
        lum(TOP),
        lum(BOTTOM)
    );
}

#[test]
fn the_ground_does_not_re_upload_every_frame() {
    // Sixty frames a second, for as long as the frontend is on the shelf.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut w = Wallpaper::default();
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);

    w.draw(&mut canvas, &safe);
    let warm = canvas.ops.len();
    for _ in 0..120 {
        w.draw(&mut canvas, &safe);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(warm)
        .filter(|o| matches!(o, Op::UploadRgba8 { .. } | Op::UploadAlpha8 { .. }))
        .count();
    assert_eq!(
        uploads, 0,
        "two seconds of background uploaded {uploads} times"
    );
}

// ------------------------------------------------------------------ the card's

#[test]
fn a_card_picture_is_used_and_still_covers_the_panel() {
    // A picture that is the wrong shape for the panel — which it always is for two of the
    // three — must still leave no bare ground.
    for g in GEOMETRIES {
        for size in [(320u32, 240u32), (1024, 1024), (1920, 1080), (200, 900)] {
            let (card, _root) = card(&[Platform::Gba], false, size);
            let safe = SafeArea::for_geometry(g);
            let mut w = Wallpaper::default();
            w.set_source(card.wallpaper(Platform::Gba));

            let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
            w.draw(&mut canvas, &safe);

            let drew_image = canvas.frame().iter().any(|o| matches!(o, Op::Image { .. }));
            assert!(
                drew_image,
                "{g:?} {size:?}: the card's picture was not drawn"
            );

            let (l, t, r, b) = extent(&canvas);
            assert!(
                l <= 0.0 && t <= 0.0 && r >= safe.panel_w as f32 && b >= safe.panel_h as f32,
                "{g:?} {size:?}: the panel is not covered — {l},{t} to {r},{b}"
            );
        }
    }
}

#[test]
fn a_picture_is_cropped_rather_than_squashed() {
    // Cover, not fit and not stretch. A background stretched to the panel puts a face out of
    // shape; one that is letterboxed is a picture in a frame rather than the ground.
    let (card, _root) = card(&[Platform::Gba], false, (1920, 1080));
    let safe = SafeArea::for_geometry(Geometry::W720H720);
    let mut w = Wallpaper::default();
    w.set_source(card.wallpaper(Platform::Gba));
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    w.draw(&mut canvas, &safe);

    // The quad fills the panel, and the crop is in the uv rather than in the quad: a
    // 16:9 picture on a square panel shows the middle of it, so the u range is narrower
    // than the whole width and v is the whole height.
    let uv = canvas.frame().iter().find_map(|o| match o {
        Op::Image { uv, .. } => Some(*uv),
        _ => None,
    });
    let Some([u0, v0, u1, v1]) = uv else {
        panic!("no picture was drawn");
    };
    assert!(
        (u1 - u0) < 0.99,
        "a 16:9 picture on a square panel used the whole width: u {u0}..{u1}"
    );
    assert!(
        (v1 - v0) > 0.99,
        "it cropped the height as well: v {v0}..{v1}"
    );
    // Centred: what is dropped comes off both sides equally.
    assert!(
        ((u0) - (1.0 - u1)).abs() < 0.01,
        "the crop is off centre: u {u0}..{u1}"
    );
}

#[test]
fn the_card_prefers_this_shelfs_picture_and_falls_back_to_the_default() {
    let (card, _root) = card(&[Platform::Gb], true, (64, 64));
    let gb = card.wallpaper(Platform::Gb).expect("no GB wallpaper");
    let gba = card.wallpaper(Platform::Gba).expect("no fallback");
    assert!(gb.ends_with("GB.png"), "GB shelf got {}", gb.display());
    assert!(
        gba.ends_with("default.png"),
        "GBA shelf got {} instead of the default",
        gba.display()
    );

    // And a card with neither has none, rather than a path to a file that is not there.
    let (bare, _root2) = bare_card();
    assert_eq!(bare.wallpaper(Platform::Gba), None);
}

#[test]
fn switching_back_to_a_shelf_does_not_decode_again() {
    // L1 and R1 walk the platforms. A player holding one down would otherwise decode a
    // picture per shelf per press.
    let (card, _root) = card(&[Platform::Gba, Platform::Gb], false, (256, 256));
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut w = Wallpaper::default();
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);

    let gba = card.wallpaper(Platform::Gba);
    w.set_source(gba.clone());
    w.draw(&mut canvas, &safe);
    let after_first = canvas.ops.len();

    // Told the same thing again, ten times, with a draw after each.
    for _ in 0..10 {
        w.set_source(gba.clone());
        w.draw(&mut canvas, &safe);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(after_first)
        .filter(|o| matches!(o, Op::UploadRgba8 { .. }))
        .count();
    assert_eq!(
        uploads, 0,
        "the same wallpaper was uploaded {uploads} more times"
    );
}

#[test]
fn a_broken_picture_is_the_built_in_ground_not_a_hole() {
    // Half a file, a JPEG renamed, a card pulled mid-copy. The panel still has to be covered
    // — a background is the one thing that cannot degrade to nothing.
    let (card, root) = card(&[], false, (1, 1));
    let bad = root.join("Wallpapers/GBA.png");
    std::fs::write(&bad, b"not a png").unwrap();
    let found = card
        .wallpaper(Platform::Gba)
        .expect("the file is there, whatever is in it");

    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut w = Wallpaper::default();
    w.set_source(Some(found));
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    w.draw(&mut canvas, &safe);

    let (l, t, r, b) = extent(&canvas);
    assert!(
        l <= 0.0 && t <= 0.0 && r >= safe.panel_w as f32 && b >= safe.panel_h as f32,
        "a broken picture left the panel bare: {l},{t} to {r},{b}"
    );

    // And it is opened once, not once a frame.
    let before = canvas.ops.len();
    for _ in 0..60 {
        w.draw(&mut canvas, &safe);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(before)
        .filter(|o| matches!(o, Op::UploadRgba8 { .. }))
        .count();
    assert_eq!(uploads, 0, "a broken file was re-read {uploads} times");
}

#[test]
fn dropping_the_card_picture_goes_back_to_the_ground() {
    let (card, _root) = card(&[Platform::Gba], false, (256, 256));
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut w = Wallpaper::default();
    w.set_source(card.wallpaper(Platform::Gba));
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    w.draw(&mut canvas, &safe);
    assert!(canvas.frame().iter().any(|o| matches!(o, Op::Image { .. })));

    w.set_source(None);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    w.draw(&mut canvas, &safe);
    let (l, t, r, b) = extent(&canvas);
    assert!(
        l <= 0.0 && t <= 0.0 && r >= safe.panel_w as f32 && b >= safe.panel_h as f32,
        "a shelf with no picture was left bare: {l},{t} to {r},{b}"
    );
}

#[test]
fn a_huge_picture_is_not_uploaded_at_source_resolution() {
    // A 4K photograph drawn on a 720x480 panel costs 33 MB to look no better, on a Mali
    // sharing system memory with the cores.
    let (card, _root) = card(&[Platform::Gba], false, (3840, 2160));
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut w = Wallpaper::default();
    w.set_source(card.wallpaper(Platform::Gba));
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    w.draw(&mut canvas, &safe);

    let uploaded = canvas
        .ops
        .iter()
        .find_map(|o| match o {
            Op::UploadRgba8 { w, h, .. } => Some((*w, *h)),
            _ => None,
        })
        .expect("nothing was uploaded");
    assert!(
        uploaded.0 <= safe.panel_w * 2 && uploaded.1 <= safe.panel_h * 2,
        "a 3840x2160 picture was uploaded at {}x{}",
        uploaded.0,
        uploaded.1
    );
    // And not so far down that the background is mush.
    assert!(
        uploaded.0 >= safe.panel_w / 2,
        "it was reduced to {}x{}",
        uploaded.0,
        uploaded.1
    );
}

// ------------------------------------------------------------------ the crop

#[test]
fn cover_uv_never_leaves_a_gap_and_never_inverts() {
    for src in [
        (1u32, 1u32),
        (320, 240),
        (1920, 1080),
        (200, 900),
        (1024, 1024),
    ] {
        for dest in [(640u32, 480u32), (720, 480), (720, 720)] {
            let [u0, v0, u1, v1] = cover_uv(src, dest);
            for v in [u0, v0, u1, v1] {
                assert!(v.is_finite(), "{src:?}->{dest:?}: {u0},{v0},{u1},{v1}");
                assert!(
                    (0.0..=1.0).contains(&v),
                    "{src:?}->{dest:?}: {v} is off the texture"
                );
            }
            assert!(u1 > u0 && v1 > v0, "{src:?}->{dest:?}: inverted");

            // Whatever is shown has the destination's aspect, which is what makes it a crop
            // rather than a stretch.
            let shown = ((u1 - u0) * src.0 as f32, (v1 - v0) * src.1 as f32);
            let want = dest.0 as f32 / dest.1 as f32;
            let got = shown.0 / shown.1;
            assert!(
                (got / want - 1.0).abs() < 0.02,
                "{src:?}->{dest:?}: showing {shown:?}, aspect {got} against {want}"
            );
        }
    }
    // A source with no area is not a crash.
    let uv = cover_uv((0, 0), (720, 480));
    assert!(uv.iter().all(|v| v.is_finite()));
}
