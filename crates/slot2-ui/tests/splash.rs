//! The contract for slot2-ui's first pieces: faces, spans, splash. Task 04 makes these pass
//! without editing this file.

use std::path::PathBuf;

use slot2_gfx::{Canvas, Color, Op, RecordingCanvas, TexId};
use slot2_i18n::{Button, Span};
use slot2_platform::{by_target, Geometry};
use slot2_ui::{draw_spans, face, splash, Splash, UiCtx, SAFE_H, SAFE_W};

fn fonts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts")
}

fn ctx(target: &str, lang: &str) -> UiCtx {
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts_dir()], None)
}

fn images(ops: &[Op]) -> Vec<(TexId, f32, f32, f32, f32)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Image {
                tex, x, y, w, h, ..
            } => Some((*tex, *x, *y, *w, *h)),
            _ => None,
        })
        .collect()
}

// ---------- faces ----------

#[test]
fn faces_upload_once_and_draw_many() {
    let mut c = RecordingCanvas::new(720, 480);
    let mut ctx = ctx("rgsp", "en");
    let w1 = face::draw_text(&mut c, &mut ctx, "Tetris", 24.0, 10.0, 10.0, Color::WHITE);
    let w2 = face::draw_text(&mut c, &mut ctx, "Tetris", 24.0, 10.0, 50.0, Color::WHITE);
    let w3 = face::draw_text(&mut c, &mut ctx, "Tetris", 25.0, 10.0, 90.0, Color::WHITE);
    assert_eq!(w1, w2);
    assert!(w3 > w1);
    let uploads = c
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. }))
        .count();
    assert_eq!(uploads, 2, "one per distinct (text, px)");
    assert_eq!(ctx.faces.len(), 2);
    let imgs = images(&c.ops);
    assert_eq!(imgs.len(), 3);
    assert_eq!(imgs[0].0, imgs[1].0, "same face, same texture");
    assert_ne!(imgs[0].0, imgs[2].0);
    assert_eq!((imgs[0].1, imgs[0].2), (10.0, 10.0));
    assert_eq!(imgs[0].3, w1, "image width is the face width");
    assert!(
        imgs[0].4 > 20.0 && imgs[0].4 < 40.0,
        "line box height at 24px: {}",
        imgs[0].4
    );
    assert_eq!(face::measure(&mut ctx, "Tetris", 24.0), w1);
    assert_eq!(ctx.faces.len(), 2, "measure does not upload");
}

#[test]
fn faces_render_hangul_through_the_lazy_cjk_font() {
    let mut c = RecordingCanvas::new(720, 480);
    let mut ctx = ctx("rgsp", "ko");
    // Korean names Noto Sans KR as its preferred body font, so it is the chain's first slot
    // and is registered by path: nothing has read the file yet.
    assert!(
        !ctx.fonts.is_loaded(slot2_text::FontId(0)),
        "the context read the preferred font"
    );
    let w = face::draw_text(&mut c, &mut ctx, "포켓몬", 24.0, 0.0, 0.0, Color::WHITE);
    assert!(w > 30.0, "{w}");
    assert!(ctx.fonts.is_loaded(slot2_text::FontId(0)));
}

// ---------- spans ----------

#[test]
fn spans_lay_out_text_and_button_caps_in_order() {
    let mut c = RecordingCanvas::new(720, 480);
    let mut ctx = ctx("rgsp", "en");
    let spans = [
        Span::Text("Hold ".into()),
        Span::Btn(Button::Menu),
        Span::Text(" to eject".into()),
    ];
    let total = draw_spans(&mut c, &mut ctx, &spans, 16.0, 100.0, 200.0, Color::WHITE);
    let hold = face::measure(&mut ctx, "Hold ", 16.0);
    let menu = face::measure(&mut ctx, "MENU", 16.0);
    let eject = face::measure(&mut ctx, " to eject", 16.0);
    let (pad, gap) = (16.0 * 0.4, 16.0 * 0.3);
    let expect = hold + gap + pad + menu + pad + gap + eject;
    assert!(
        (total - expect).abs() < 0.5,
        "total {total} expect {expect}"
    );
    // Exactly one rect (the cap) and three images (Hold, MENU, to eject), left to right.
    let rects: Vec<_> = c
        .ops
        .iter()
        .filter(|o| matches!(o, Op::Rect { .. }))
        .collect();
    assert_eq!(rects.len(), 1);
    if let Op::Rect { x, y, w, color, .. } = rects[0] {
        assert!((x - (100.0 + hold + gap)).abs() < 0.5);
        assert_eq!(*y, 200.0);
        assert!((w - (menu + 2.0 * pad)).abs() < 0.5);
        assert!((color.a - 0.25).abs() < 1e-6);
    }
    let imgs = images(&c.ops);
    assert_eq!(imgs.len(), 3);
    assert!(imgs[0].1 < imgs[1].1 && imgs[1].1 < imgs[2].1);
    assert!(
        (imgs[1].1 - (100.0 + hold + gap + pad)).abs() < 0.5,
        "label sits inside the cap"
    );
}

#[test]
fn korean_spans_put_the_cap_first() {
    let mut c = RecordingCanvas::new(720, 480);
    let mut ctx = ctx("rgsp", "ko");
    let spans = ctx.i18n.spans("hint-eject", &[]);
    assert!(matches!(spans[0], Span::Btn(Button::Menu)));
    draw_spans(&mut c, &mut ctx, &spans, 16.0, 0.0, 0.0, Color::WHITE);
    let first_rect = c
        .ops
        .iter()
        .position(|o| matches!(o, Op::Rect { .. }))
        .unwrap();
    let first_img = c
        .ops
        .iter()
        .position(|o| matches!(o, Op::Image { .. }))
        .unwrap();
    assert!(
        first_rect < first_img,
        "cap background is drawn before its label"
    );
}

// ---------- splash ----------

fn splash_on(target: &str, lang: &str) -> (RecordingCanvas, UiCtx) {
    let profile = by_target(target).unwrap();
    let (w, h) = profile.geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    let mut ctx = UiCtx::new(profile, lang, vec![fonts_dir()], None);
    Splash::default().draw(&mut c, &mut ctx);
    (c, ctx)
}

#[test]
fn splash_clears_then_draws_inside_the_safe_area_on_every_geometry() {
    for target in ["rgsp", "rg35xxsp", "rgcubexx"] {
        let (c, ctx) = splash_on(target, "en");
        assert!(matches!(
            c.ops.first(),
            Some(Op::UploadAlpha8 { .. }) | Some(Op::Clear(_))
        ));
        assert!(
            c.frame().iter().any(|o| matches!(o, Op::Image { .. })),
            "{target}: nothing drawn"
        );
        let safe = ctx.safe;
        for (tex, x, y, w, h) in images(c.frame()) {
            // The wordmark is centred on the panel, which on 720-wide panels still lies in
            // the safe area; everything must.
            assert!(
                safe.contains(x, y, w, h),
                "{target}: face {tex:?} at {x},{y} {w}x{h} leaves the safe area {safe:?}"
            );
        }
        let g = ctx.profile.geometry;
        assert_eq!((safe.panel_w, safe.panel_h), g.size());
        let _ = (SAFE_W, SAFE_H, Geometry::W640H480);
    }
}

#[test]
fn splash_wordmark_is_centred_and_greeting_is_localised() {
    let (c, mut ctx) = splash_on("rgsp", "ko");
    let imgs = images(c.frame());
    let word_w = face::measure(&mut ctx, splash::WORDMARK, slot2_ui::PX_WORDMARK);
    let wm = imgs
        .iter()
        .find(|(_, _, _, w, _)| (*w - word_w).abs() < 0.5)
        .expect("wordmark face drawn");
    let centre = wm.1 + wm.3 / 2.0;
    assert!((centre - 360.0).abs() < 1.0, "wordmark centre {centre}");
    assert!((wm.2 - ctx.safe.py(splash::Y_WORDMARK)).abs() < 0.5);
    let greeting = ctx.i18n.t("splash-hello");
    let greet_w = face::measure(&mut ctx, &greeting, slot2_ui::PX_TITLE);
    assert!(
        imgs.iter()
            .any(|(_, _, _, w, _)| (*w - greet_w).abs() < 0.5),
        "korean greeting face drawn"
    );
}

#[test]
fn splash_is_idempotent_once_warm() {
    let profile = by_target("rgsp").unwrap();
    let mut c = RecordingCanvas::new(720, 480);
    let mut ctx = UiCtx::new(profile, "en", vec![fonts_dir()], None);
    let s = Splash::default();
    s.draw(&mut c, &mut ctx);
    let uploads_after_first = c
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. }))
        .count();
    assert!(
        uploads_after_first >= 4,
        "wordmark, greeting, device, hint text"
    );
    let n = c.ops.len();
    s.draw(&mut c, &mut ctx);
    let second: Vec<_> = c.ops[n..].to_vec();
    assert!(
        !second
            .iter()
            .any(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. })),
        "warm frame must not upload"
    );
    assert!(matches!(second[0], Op::Clear(_)));
    assert_eq!(c.size(), (720, 480));
}

#[test]
fn splash_debug_frame_outlines_the_safe_area() {
    let profile = by_target("rgcubexx").unwrap();
    let mut c = RecordingCanvas::new(720, 720);
    let mut ctx = UiCtx::new(profile, "en", vec![fonts_dir()], None);
    Splash { debug_frame: true }.draw(&mut c, &mut ctx);
    let rects: Vec<_> = c
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .collect();
    // Four edges of the 640x480 box at (40,120): top, bottom, left, right — plus any cap.
    assert!(
        rects.contains(&(40.0, 120.0, 640.0, 1.0)),
        "top edge {rects:?}"
    );
    assert!(rects.contains(&(40.0, 599.0, 640.0, 1.0)), "bottom edge");
    assert!(rects.contains(&(40.0, 120.0, 1.0, 480.0)), "left edge");
    assert!(rects.contains(&(679.0, 120.0, 1.0, 480.0)), "right edge");
}

// ---------- real GL, opt in ----------

#[test]
fn splash_renders_for_real() {
    if std::env::var("SLOT2_GFX_TEST")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        use slot2_gfx::{GlCanvas, HostSurface};
        let profile = by_target("rgsp").unwrap();
        let panel = profile.geometry.size();
        let mut surface = HostSurface::open("slot2 splash test", panel, 1).unwrap();
        let mut gl = GlCanvas::new(&mut surface, panel).unwrap();
        let mut ctx = UiCtx::new(profile, "ko", vec![fonts_dir()], None);
        Splash { debug_frame: true }.draw(&mut gl, &mut ctx);
        let img = gl.read_back();
        let out =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/splash-screenshot.png");
        let f = std::fs::File::create(&out).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), img.width, img.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .unwrap()
            .write_image_data(&img.rgba)
            .unwrap();
        eprintln!("wrote {}", out.display());
        // Ink in the wordmark band, none in the top-left corner.
        let band: usize = (150..214)
            .map(|y| (200..520).filter(|&x| img.pixel(x, y)[0] > 128).count())
            .sum();
        assert!(band > 500, "wordmark ink {band}");
        assert_eq!(img.pixel(5, 5), splash::BACKDROP.to_u8());
        // Greeting band (Korean) has ink too.
        let greet: usize = (235..265)
            .map(|y| (100..620).filter(|&x| img.pixel(x, y)[0] > 128).count())
            .sum();
        assert!(greet > 200, "greeting ink {greet}");
        gl.present(&mut surface).unwrap();
    } else {
        eprintln!("SLOT2_GFX_TEST not set; skipping GL splash test");
    }
}
