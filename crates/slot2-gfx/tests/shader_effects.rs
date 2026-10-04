//! The effect draw contract, checked with no GPU at all: what a canvas is asked to do, and
//! what a canvas that cannot do it does instead.
//!
//! The GLSL sources themselves are checked in `src/shader.rs`, where they live; whether the
//! four effects actually change a picture is the GL phase in `tests/screenshot.rs`. None of
//! the three stands in for another.

use slot2_gfx::{Canvas, Color, Op, RecordingCanvas, ShaderEffect, TexId};

fn rgba8() -> Vec<u8> {
    vec![255u8; 8 * 8 * 4]
}

/// One plain image draw, as the fallback canvas received it.
#[derive(Clone, Debug, PartialEq)]
struct PlainImage {
    tex: TexId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    uv: [f32; 4],
    tint: Color,
}

/// A canvas with no effect support of its own: it inherits the trait's fallback, which is what
/// any future canvas implementation will do until it has programs of its own.
#[derive(Default)]
struct FlatCanvas {
    images: Vec<PlainImage>,
}

impl Canvas for FlatCanvas {
    fn size(&self) -> (u32, u32) {
        (720, 480)
    }
    fn clear(&mut self, _: Color) {}
    fn upload_rgba8(&mut self, _w: u32, _h: u32, _data: &[u8]) -> TexId {
        TexId(1)
    }
    fn upload_alpha8(&mut self, _w: u32, _h: u32, _data: &[u8]) -> TexId {
        TexId(2)
    }
    fn free(&mut self, _: TexId) {}
    fn rect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _color: Color) {}
    fn image(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        self.image_uv(tex, x, y, w, h, [0.0, 0.0, 1.0, 1.0], tint);
    }
    fn image_uv(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], tint: Color) {
        self.images.push(PlainImage {
            tex,
            x,
            y,
            w,
            h,
            uv,
            tint,
        });
    }
}

#[test]
fn the_four_effects_are_the_four_presets_and_nothing_else() {
    assert_eq!(
        ShaderEffect::ALL,
        [
            ShaderEffect::SharpBilinear,
            ShaderEffect::Lcd3x,
            ShaderEffect::ZfastCrt,
            ShaderEffect::Scanline,
        ]
    );
    // The type is `Copy + Eq + Hash`, which is what a menu key and a settings comparison need.
    let copied = ShaderEffect::Lcd3x;
    let mut seen = std::collections::HashSet::new();
    for effect in ShaderEffect::ALL {
        assert!(seen.insert(effect), "{effect:?} appears twice");
    }
    assert_eq!(copied, ShaderEffect::Lcd3x);
}

#[test]
fn a_recording_canvas_keeps_every_effect_draw_whole() {
    let mut c = RecordingCanvas::new(720, 480);
    let tex = c.upload_rgba8(8, 8, &rgba8());
    let uv = [0.25, 0.125, 0.75, 0.875];
    let tint = Color::rgba(1.0, 0.5, 0.25, 0.5);

    for (i, effect) in ShaderEffect::ALL.iter().enumerate() {
        let (x, y) = (10.0 + i as f32, 20.0);
        c.clear(Color::BLACK);
        c.image_effect_uv(tex, x, y, 100.0, 50.0, uv, tint, *effect);
        // Nothing is lost on the way in: the effect, the texture, the geometry, the crop and
        // the tint all arrive together, and one draw makes one op.
        assert_eq!(
            c.frame(),
            &[Op::ImageEffect {
                tex,
                x,
                y,
                w: 100.0,
                h: 50.0,
                uv,
                tint,
                effect: *effect,
            }],
            "{effect:?} was not recorded whole"
        );
    }

    // The whole-texture convenience is the same draw with the full crop.
    c.clear(Color::BLACK);
    c.image_effect(
        tex,
        1.0,
        2.0,
        3.0,
        4.0,
        Color::WHITE,
        ShaderEffect::Scanline,
    );
    assert_eq!(
        c.frame(),
        &[Op::ImageEffect {
            tex,
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
            uv: [0.0, 0.0, 1.0, 1.0],
            tint: Color::WHITE,
            effect: ShaderEffect::Scanline,
        }]
    );
}

#[test]
fn the_origin_moves_an_effect_draw_the_way_it_moves_an_image() {
    let mut c = RecordingCanvas::new(100, 100);
    let tex = c.upload_alpha8(1, 1, &[255]);
    c.set_origin(7.0, -3.0);
    c.clear(Color::BLACK);
    c.image(tex, 1.0, 2.0, 3.0, 4.0, Color::WHITE);
    c.image_effect(
        tex,
        1.0,
        2.0,
        3.0,
        4.0,
        Color::WHITE,
        ShaderEffect::ZfastCrt,
    );

    let ops = c.frame();
    assert_eq!(
        ops[0],
        Op::Image {
            tex,
            x: 8.0,
            y: -1.0,
            w: 3.0,
            h: 4.0,
            uv: [0.0, 0.0, 1.0, 1.0],
            tint: Color::WHITE,
        }
    );
    assert_eq!(
        ops[1],
        Op::ImageEffect {
            tex,
            x: 8.0,
            y: -1.0,
            w: 3.0,
            h: 4.0,
            uv: [0.0, 0.0, 1.0, 1.0],
            tint: Color::WHITE,
            effect: ShaderEffect::ZfastCrt,
        },
        "the offset landed somewhere other than on the plain draw"
    );
}

#[test]
fn an_effect_draw_is_one_draw_and_does_not_spread() {
    // The HUD, the menus, the letterbox bars and the text over a frozen frame are all drawn
    // after the game picture. If the effect reached them, a shader would distort the UI.
    let mut c = RecordingCanvas::new(720, 480);
    let tex = c.upload_rgba8(2, 2, &[255u8; 16]);
    let mask = c.upload_alpha8(2, 2, &[255u8; 4]);
    c.clear(Color::BLACK);
    c.image_effect(
        tex,
        0.0,
        0.0,
        720.0,
        480.0,
        Color::WHITE,
        ShaderEffect::Lcd3x,
    );
    c.rect(10.0, 10.0, 5.0, 5.0, Color::WHITE);
    c.image(mask, 20.0, 20.0, 5.0, 5.0, Color::WHITE);
    c.image_uv(
        tex,
        30.0,
        30.0,
        5.0,
        5.0,
        [0.5, 0.5, 1.0, 1.0],
        Color::WHITE,
    );

    let ops = c.frame();
    assert_eq!(ops.len(), 4, "{ops:?}");
    assert!(matches!(
        ops[0],
        Op::ImageEffect {
            effect: ShaderEffect::Lcd3x,
            ..
        }
    ));
    assert!(matches!(ops[1], Op::Rect { .. }), "{:?}", ops[1]);
    assert!(matches!(ops[2], Op::Image { tex: t, .. } if t == mask));
    assert!(matches!(ops[3], Op::Image { tex: t, .. } if t == tex));
}

#[test]
fn a_canvas_without_effects_draws_the_same_picture_plainly() {
    let mut c = FlatCanvas::default();
    let tex = TexId(7);
    let uv = [0.1, 0.2, 0.3, 0.4];
    let tint = Color::rgba(0.25, 0.5, 0.75, 1.0);

    for effect in ShaderEffect::ALL {
        c.images.clear();
        c.image_effect_uv(tex, 5.0, 6.0, 70.0, 80.0, uv, tint, effect);
        assert_eq!(
            c.images,
            vec![PlainImage {
                tex,
                x: 5.0,
                y: 6.0,
                w: 70.0,
                h: 80.0,
                uv,
                tint,
            }],
            "{effect:?} did not fall back to a plain image draw"
        );
    }
}
