//! The GL contract, checked by drawing into the offscreen panel and reading pixels back.
//! Needs a display and a GL driver, so it only runs when `SLOT2_GFX_TEST=1` is set; without
//! it the test passes trivially (CI runners have no GPU). It also writes what it drew to
//! `target/gfx-screenshot.png` so a human can look.

use slot2_gfx::{Canvas, Color, GlCanvas, HostSurface, Image, ShaderEffect, TexId};

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
    check_shader_effects(&mut canvas);
}

/// The effect programs, and that drawing through one changes the picture without changing
/// anything drawn after it.
///
/// A phase of the GL test above rather than a test of its own: one event loop per process.
fn check_shader_effects(canvas: &mut GlCanvas) {
    for effect in ShaderEffect::ALL {
        assert!(
            canvas.shader_available(effect),
            "{effect:?} did not build: {}",
            canvas.shader_error(effect).unwrap_or("no log")
        );
        assert!(
            canvas.shader_error(effect).is_none(),
            "{effect:?} built and still reported a failure"
        );
    }

    // A hard-edged checker, drawn far bigger than it is: every one of the four has something
    // to do to a boundary or to a row, so a picture that comes back byte-identical means the
    // program never ran.
    let mut checker = vec![0u8; 8 * 8 * 4];
    for y in 0..8u32 {
        for x in 0..8u32 {
            let i = ((y * 8 + x) * 4) as usize;
            let light = (x + y) % 2 == 0;
            let v = if light { 255 } else { 32 };
            checker[i..i + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    let tex = canvas.upload_rgba8(8, 8, &checker);
    let quad = (100.0f32, 80.0f32, 400.0f32, 300.0f32);

    let base = draw_effect_scene(canvas, tex, quad, None);
    let mut shots: Vec<(ShaderEffect, Image)> = Vec::new();
    for effect in ShaderEffect::ALL {
        let img = draw_effect_scene(canvas, tex, quad, Some(effect));
        // The rect drawn after the effect, and only after it, lands on the same colour in all
        // five runs: the effect's program and uniforms did not leak into the ordinary path.
        for (px, py) in [(30u32, 405u32), (45, 415)] {
            assert!(
                near(img.pixel(px, py), [0, 200, 255, 255], 1),
                "{effect:?} changed the rect drawn after it: {:?}",
                img.pixel(px, py)
            );
        }
        let d = pixel_difference(&base, &img, quad);
        assert!(
            d > 200_000,
            "{effect:?} left the picture unchanged (difference {d})"
        );
        shots.push((effect, img));
    }

    // And the four are not the same effect four times over.
    for i in 0..shots.len() {
        for j in i + 1..shots.len() {
            let (a, img_a) = &shots[i];
            let (b, img_b) = &shots[j];
            let d = pixel_difference(img_a, img_b, quad);
            assert!(
                d > 200_000,
                "{a:?} and {b:?} drew the same picture (difference {d})"
            );
        }
    }
    canvas.free(tex);

    // A crop with a loud ring around it. A neighbour tap that steps outside the crop pulls
    // the ring back into the picture, which is the one way an effect can invent detail a
    // player cropped away.
    let sentinel = [255u8, 0, 0, 255];
    let grey = [128u8, 128, 128, 255];
    let mut ringed = vec![0u8; 8 * 8 * 4];
    for y in 0..8u32 {
        for x in 0..8u32 {
            let i = ((y * 8 + x) * 4) as usize;
            let inside = (2..6).contains(&x) && (2..6).contains(&y);
            ringed[i..i + 4].copy_from_slice(if inside { &grey } else { &sentinel });
        }
    }
    let ring = canvas.upload_rgba8(8, 8, &ringed);
    for effect in ShaderEffect::ALL {
        canvas.clear(Color::BLACK);
        canvas.image_effect_uv(
            ring,
            100.0,
            80.0,
            400.0,
            300.0,
            [0.25, 0.25, 0.75, 0.75],
            Color::WHITE,
            effect,
        );
        let img = canvas.read_back();
        for py in 80..380u32 {
            for px in 100..500u32 {
                let p = img.pixel(px, py);
                // Grey survives the stripes in every one of them; red only gets in by
                // sampling outside the crop. The two are 57 apart at worst, so 100 leaves
                // room for rounding on both sides.
                let red = p[0] as i32 - p[1].max(p[2]) as i32;
                assert!(
                    red < 100,
                    "{effect:?} pulled the cropped-away ring in at ({px}, {py}): {p:?}"
                );
            }
        }
    }
    canvas.free(ring);
}

/// One scene drawn with or without an effect, read back whole. The rect at the foot is the
/// after-the-effect draw every run has to agree on.
fn draw_effect_scene(
    canvas: &mut GlCanvas,
    tex: TexId,
    quad: (f32, f32, f32, f32),
    effect: Option<ShaderEffect>,
) -> Image {
    let (x, y, w, h) = quad;
    canvas.clear(Color::from_u8(0, 0, 0, 255));
    match effect {
        Some(e) => canvas.image_effect_uv(tex, x, y, w, h, [0.0, 0.0, 1.0, 1.0], Color::WHITE, e),
        None => canvas.image_uv(tex, x, y, w, h, [0.0, 0.0, 1.0, 1.0], Color::WHITE),
    }
    canvas.rect(30.0, 400.0, 20.0, 20.0, Color::from_u8(0, 200, 255, 255));
    canvas.read_back()
}

/// Total channel difference between two read-backs, over the quad the effect drew into.
fn pixel_difference(a: &Image, b: &Image, quad: (f32, f32, f32, f32)) -> u64 {
    let (x, y, w, h) = quad;
    let mut sum = 0u64;
    for py in y as u32..(y + h) as u32 {
        for px in x as u32..(x + w) as u32 {
            let (p, q) = (a.pixel(px, py), b.pixel(px, py));
            for c in 0..4 {
                sum += p[c].abs_diff(q[c]) as u64;
            }
        }
    }
    sum
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
