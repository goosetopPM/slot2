//! The GL contract, checked by drawing into the offscreen panel and reading pixels back.
//! Needs a display and a GL driver, so it only runs when `SLOT2_GFX_TEST=1` is set; without
//! it the test passes trivially (CI runners have no GPU). It also writes what it drew to
//! `target/gfx-screenshot.png` so a human can look.

use slot2_gfx::{Canvas, Color, GlCanvas, HostSurface, Image};

fn enabled() -> bool {
    std::env::var("SLOT2_GFX_TEST")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn near(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
}

fn write_png(path: &std::path::Path, img: &Image) {
    let f = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), img.width, img.height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(&img.rgba)
        .unwrap();
}

#[test]
fn draws_rects_and_masks_where_asked() {
    if !enabled() {
        eprintln!("SLOT2_GFX_TEST not set; skipping GL test");
        return;
    }
    let panel = (720u32, 480u32);
    let mut surface = HostSurface::open("slot2-gfx test", panel, 1).unwrap();
    let mut canvas = GlCanvas::new(&mut surface, panel).unwrap();
    assert_eq!(canvas.size(), panel);

    // Frame 1: a dark clear, a red rect, a green half-transparent rect over black, and a
    // 4x4 alpha mask (top half opaque, bottom half clear) tinted blue, scaled to 40x40.
    canvas.clear(Color::from_u8(16, 16, 16, 255));
    canvas.rect(100.0, 100.0, 50.0, 30.0, Color::from_u8(255, 0, 0, 255));
    canvas.rect(300.0, 100.0, 50.0, 30.0, Color::rgba(0.0, 1.0, 0.0, 0.5));
    let mut mask = vec![0u8; 16];
    for v in &mut mask[..8] {
        *v = 255;
    }
    let tex = canvas.upload_alpha8(4, 4, &mask);
    canvas.image(
        tex,
        500.0,
        100.0,
        40.0,
        40.0,
        Color::from_u8(0, 0, 255, 255),
    );
    // An RGBA texture: 2x1, left magenta, right cyan, drawn 20x10 at (600, 300).
    let rgba = [255, 0, 255, 255, 0, 255, 255, 255];
    let tex2 = canvas.upload_rgba8(2, 1, &rgba);
    canvas.image(tex2, 600.0, 300.0, 20.0, 10.0, Color::WHITE);
    // Only the right texel, via uv.
    canvas.image_uv(
        tex2,
        650.0,
        300.0,
        10.0,
        10.0,
        [0.5, 0.0, 1.0, 1.0],
        Color::WHITE,
    );

    let img = canvas.read_back();
    assert_eq!((img.width, img.height), panel);
    assert_eq!(img.rgba.len(), (720 * 480 * 4) as usize);
    let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/gfx-screenshot.png");
    write_png(&out, &img);
    eprintln!("wrote {}", out.display());

    assert!(
        near(img.pixel(10, 10), [16, 16, 16, 255], 1),
        "clear colour {:?}",
        img.pixel(10, 10)
    );
    assert!(
        near(img.pixel(120, 110), [255, 0, 0, 255], 1),
        "red rect {:?}",
        img.pixel(120, 110)
    );
    assert!(
        near(img.pixel(99, 110), [16, 16, 16, 255], 1),
        "left of red rect"
    );
    assert!(
        near(img.pixel(120, 130), [16, 16, 16, 255], 1),
        "below red rect (y grows down)"
    );
    // 50% green over (16,16,16): 8 + 127 ≈ 135 green, 8 red/blue.
    assert!(
        near(img.pixel(320, 110), [8, 135, 8, 255], 3),
        "blended green {:?}",
        img.pixel(320, 110)
    );
    // Mask: top half of the 40x40 is solid blue, bottom half shows the clear colour.
    assert!(
        near(img.pixel(520, 105), [0, 0, 255, 255], 1),
        "mask top {:?}",
        img.pixel(520, 105)
    );
    assert!(
        near(img.pixel(520, 135), [16, 16, 16, 255], 1),
        "mask bottom {:?}",
        img.pixel(520, 135)
    );
    // RGBA texture halves, nearest-sampled.
    assert!(
        near(img.pixel(603, 305), [255, 0, 255, 255], 1),
        "magenta {:?}",
        img.pixel(603, 305)
    );
    assert!(
        near(img.pixel(617, 305), [0, 255, 255, 255], 1),
        "cyan {:?}",
        img.pixel(617, 305)
    );
    assert!(
        near(img.pixel(655, 305), [0, 255, 255, 255], 1),
        "uv crop {:?}",
        img.pixel(655, 305)
    );

    // Present once so the window path (letterbox blit + swap) is exercised too.
    canvas.present(&mut surface).unwrap();
    let _ = surface.pump();

    // Frame 2: clear only, then read back — nothing from frame 1 may remain.
    canvas.clear(Color::BLACK);
    let img2 = canvas.read_back();
    assert!(
        near(img2.pixel(120, 110), [0, 0, 0, 255], 0),
        "frame 1 leaked into frame 2"
    );
    canvas.free(tex);
    canvas.free(tex2);

    check_texture_rewrite(&mut canvas);
}

/// A phase of the GL test above rather than a test of its own: winit allows one event loop
/// per process, so everything that needs a real context shares this one.
///
/// The game texture is rewritten every frame instead of being reallocated, so what
/// glTexSubImage2D leaves behind is exactly what the player sees.
fn check_texture_rewrite(canvas: &mut GlCanvas) {
    let red = [255u8, 0, 0, 255].repeat(4);
    let tex = canvas.upload_rgba8(2, 2, &red);
    canvas.clear(Color::from_u8(0, 0, 0, 255));
    canvas.image(tex, 0.0, 0.0, 64.0, 64.0, Color::WHITE);
    let img = canvas.read_back();
    assert!(
        near(img.pixel(32, 32), [255, 0, 0, 255], 4),
        "expected red, got {:?}",
        img.pixel(32, 32)
    );

    let blue = [0u8, 0, 255, 255].repeat(4);
    assert!(
        canvas.update_rgba8(tex, 2, 2, &blue),
        "a same-size rewrite must be accepted"
    );
    canvas.clear(Color::from_u8(0, 0, 0, 255));
    canvas.image(tex, 0.0, 0.0, 64.0, 64.0, Color::WHITE);
    let img = canvas.read_back();
    assert!(
        near(img.pixel(32, 32), [0, 0, 255, 255], 4),
        "expected blue after the rewrite, got {:?}",
        img.pixel(32, 32)
    );

    // A different size is a different allocation, and must be refused rather than quietly
    // stretched into the old one.
    assert!(!canvas.update_rgba8(tex, 4, 4, &[0u8; 4 * 4 * 4]));
    canvas.free(tex);
}
