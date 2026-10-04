//! The built-in single-pass shader effects (D-10) and the GLSL ES 1.00 they are written in.
//!
//! Four presets, no more and no fewer: sharp-bilinear, LCD3x, zfast-crt and scanline. They are
//! the whole of what the renderer offers — user GLSL, `.glslp` and multi-pass chains are out of
//! scope (D-10) — so each one is a plain string in this file rather than an asset pipeline.
//!
//! Nothing here is a pass-through: a fragment shader that is not going to change the picture
//! is the ordinary image draw, and the caller should not have asked for an effect at all.
//!
//! The four share a prelude ([`COMMON`]) and differ in `main`, which keeps one copy of the
//! part that has to be exactly right: where a sample may come from. The crop matters more than
//! it looks — a game drawn with overscan has real picture outside the crop, and a neighbour tap
//! that steps over the edge pulls the hidden rows back in. Every tap below goes through
//! `crop_uv`, which clamps to the crop's own texels.

/// One of the built-in presets. There is no `Off`: turning shaders off is the ordinary image
/// draw, not a program, and giving it a variant here would make "no effect" a shader that
/// could fail to compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderEffect {
    SharpBilinear,
    Lcd3x,
    ZfastCrt,
    Scanline,
}

impl ShaderEffect {
    /// Every effect, in the order a menu should offer them.
    pub const ALL: [ShaderEffect; 4] = [
        ShaderEffect::SharpBilinear,
        ShaderEffect::Lcd3x,
        ShaderEffect::ZfastCrt,
        ShaderEffect::Scanline,
    ];

    /// Which of the two at a time this means. Not public: the ordering is an implementation
    /// detail of the array the programs live in.
    pub(crate) const fn index(self) -> usize {
        match self {
            ShaderEffect::SharpBilinear => 0,
            ShaderEffect::Lcd3x => 1,
            ShaderEffect::ZfastCrt => 2,
            ShaderEffect::Scanline => 3,
        }
    }
}

/// The vertex stage all four effect programs share. It differs from the default program's only
/// in `v_local`: the effects that work in whole destination pixels (the RGB stripes of LCD3x,
/// the beam width of zfast-crt) need to know where in the destination quad they are, and doing
/// that here with the vertex stage's `highp` is more precise than scaling `gl_FragCoord` back
/// from a viewport the fragment stage does not know.
pub(crate) const VERTEX_SOURCE: &str = r#"
attribute vec2 a_pos;
attribute vec2 a_uv;
attribute vec4 a_col;
uniform vec2 u_panel;
uniform vec2 u_dst;
uniform vec2 u_origin;
varying vec2 v_uv;
varying vec4 v_col;
varying vec2 v_local;
void main() {
    v_uv = a_uv;
    v_col = a_col;
    v_local = (a_pos - u_origin) / max(u_dst, vec2(1.0));
    vec2 clip = (a_pos / u_panel) * 2.0 - 1.0;
    gl_Position = vec4(clip.x, -clip.y, 0.0, 1.0);
}
"#;

/// What every effect fragment shader declares and shares.
///
/// `highp` matters: a 3-pixel stripe pattern is decided in the fragment stage, and `mediump`
/// there has about ten bits of mantissa, which is not enough to place a stripe on the right
/// pixel. The guard is the portable GLSL ES 1.00 way to ask for it — the qualifier does not
/// exist on desktop GL, where the precision block is skipped entirely and the driver's default
/// (already highp on the desktop) applies.
const COMMON: &str = r#"
#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif
varying vec2 v_uv;
varying vec4 v_col;
varying vec2 v_local;
uniform sampler2D u_tex;
uniform vec2 u_src;      // size of the source texture, in texels
uniform vec2 u_dst;      // size of the destination quad, in panel pixels
uniform vec4 u_uv_rect;  // the source crop this draw shows: u0, v0, u1, v1

// The crop, in texels, and one texel in UV.
vec2 crop_texels() { return (u_uv_rect.zw - u_uv_rect.xy) * u_src; }
vec2 texel_uv() { return 1.0 / u_src; }

// How many destination pixels one source texel covers. Effects use it to decide how much of a
// texel an output pixel may blend, so a picture drawn at 1:1 is left alone instead of blurred.
vec2 pixels_per_texel() { return u_dst / max(crop_texels(), vec2(1.0)); }

// A tap at `t`, in texel coordinates counted from the crop's first texel centre, clamped so it
// never leaves the crop: the rows a player cropped away are not ours to sample.
vec2 crop_uv(vec2 t) {
    vec2 last = max(crop_texels() - 0.5, vec2(0.5));
    return u_uv_rect.xy + clamp(t, vec2(0.5), last) * texel_uv();
}

// The texel this fragment sits on, its centre at +0.5.
vec2 center_texel() { return floor((v_uv - u_uv_rect.xy) * u_src) + 0.5; }
"#;

/// Sharp bilinear: flat inside a source texel, bilinear only across the seam between two.
///
/// Plain nearest makes a magnified picture a grid of blocks; plain bilinear smears every texel
/// over its neighbours even where nothing asked for it. This keeps the block and softens the
/// step, and it settles down to the ordinary bilinear blend when the picture is not magnified
/// (one destination pixel per texel has nothing to sharpen).
const SHARP_BILINEAR: &str = r#"
// How wide the seam between two texels is, in destination pixels. Narrower is sharper and
// more like nearest; this is about the width of the blur a hand-held CRT would have.
const float SEAM_PIXELS = 1.5;

void main() {
    vec2 t = (v_uv - u_uv_rect.xy) * u_src - 0.5;   // 0 at the crop's first texel centre
    vec2 base = floor(t);
    vec2 f = t - base;
    // Squeeze the blend weight away from the half-way point: the scale is how much of the
    // texel the seam is allowed to occupy, 1.0 being the whole of it (plain bilinear).
    vec2 k = clamp(pixels_per_texel() / SEAM_PIXELS, vec2(1.0), vec2(8.0));
    f = clamp((f - 0.5) * k + 0.5, 0.0, 1.0);
    vec4 c00 = texture2D(u_tex, crop_uv(base + vec2(0.5, 0.5)));
    vec4 c10 = texture2D(u_tex, crop_uv(base + vec2(1.5, 0.5)));
    vec4 c01 = texture2D(u_tex, crop_uv(base + vec2(0.5, 1.5)));
    vec4 c11 = texture2D(u_tex, crop_uv(base + vec2(1.5, 1.5)));
    vec4 c = mix(mix(c00, c10, f.x), mix(c01, c11, f.x), f.y);
    gl_FragColor = vec4(c.rgb * v_col.rgb, c.a * v_col.a);
}
"#;

/// LCD3x: the three-colour stripe of an LCD panel, plus the faint cell grid between source
/// pixels.
///
/// The three stripe gains add up to 3.0, so a flat colour comes back the brightness it went in
/// and only its subpixel balance changes — without that, every white pixel would come out a
/// third too dark. Alpha is carried through untouched: a transparent pixel stays transparent.
const LCD3X: &str = r#"
const vec3 STRIPE_RED   = vec3(1.30, 0.85, 0.85);
const vec3 STRIPE_GREEN = vec3(0.85, 1.30, 0.85);
const vec3 STRIPE_BLUE  = vec3(0.85, 0.85, 1.30);
// How much darker the seam between two source cells is at its darkest.
const float CELL_SEAM = 0.20;

void main() {
    vec4 c = texture2D(u_tex, crop_uv(center_texel()));
    // Which third of a destination pixel this is, counting the quad from its left edge, so the
    // stripe period is three output pixels wherever the panel puts them.
    float phase = mod(floor(v_local.x * u_dst.x), 3.0);
    vec3 gain = STRIPE_BLUE;
    if (phase < 1.0) {
        gain = STRIPE_RED;
    } else if (phase < 2.0) {
        gain = STRIPE_GREEN;
    }
    // Distance to the edge of the source cell this pixel came from, in destination pixels: the
    // grid is drawn from the source grid, not from the output grid, so it does not crawl.
    vec2 cell = max(pixels_per_texel(), vec2(1.0));
    vec2 within = fract((v_uv - u_uv_rect.xy) * u_src) * cell;
    vec2 edge = min(within, cell - within) / cell;
    float grid = 1.0 - CELL_SEAM * (1.0 - clamp(min(edge.x, edge.y) * 3.0, 0.0, 1.0));
    gl_FragColor = vec4(c.rgb * gain * grid * v_col.rgb, c.a * v_col.a);
}
"#;

/// Zfast CRT: a beam wider than one texel, and a scan that follows the source rows.
///
/// Both are tied to the source grid rather than the output grid, so changing the destination
/// size does not make the pattern crawl. There is no history texture, no curvature and no
/// second pass: this is the cheap end of the CRT look, which is the only end a Mali G31 MP2
/// gets to have (D-10).
const ZFAST_CRT: &str = r#"
// How much of a pixel its neighbours may take, at full magnification, and how much of its
// brightness a row seam loses.
const float BEAM = 0.35;
const float SCAN = 0.22;

void main() {
    vec2 t = center_texel();
    // A picture drawn at 1:1 has nothing to soften: the beam is turned down as the
    // magnification falls, so this effect does not blur what a plain draw would have shown
    // crisply.
    float beam = BEAM * clamp(pixels_per_texel().x - 1.0, 0.0, 1.0);
    vec4 c = (1.0 - beam) * texture2D(u_tex, crop_uv(t))
           + (beam * 0.5) * texture2D(u_tex, crop_uv(t - vec2(1.0, 0.0)))
           + (beam * 0.5) * texture2D(u_tex, crop_uv(t + vec2(1.0, 0.0)));
    // Bright in the middle of a source row, dark at the seam between two. The cosine is what
    // makes the scan soft rather than a second copy of the scanline effect.
    float row = fract((v_uv.y - u_uv_rect.xy.y) * u_src.y);
    float scan = 1.0 - SCAN * (0.5 + 0.5 * cos(6.2831853 * row));
    gl_FragColor = vec4(c.rgb * scan * v_col.rgb, c.a * v_col.a);
}
"#;

/// Scanline: a flat dark band along half of every source row, and nothing else.
///
/// No colour mask, no blur, no hue change — one multiply on all three channels. A source row
/// finer than a destination pixel cannot be drawn as a band at all, so the effect fades out
/// there instead of turning a minified picture into moire.
const SCANLINE: &str = r#"
// How much of the band's brightness is taken, at full strength.
const float BAND = 0.35;

void main() {
    vec4 c = texture2D(u_tex, crop_uv(center_texel()));
    // The band belongs to the source row, not to the output row: the pattern stays where the
    // game put it when the destination size changes.
    float row = fract((v_uv.y - u_uv_rect.xy.y) * u_src.y);
    float strength = BAND * clamp(pixels_per_texel().y - 1.0, 0.0, 1.0);
    float dark = (row < 0.5) ? 1.0 : (1.0 - strength);
    gl_FragColor = vec4(c.rgb * dark * v_col.rgb, c.a * v_col.a);
}
"#;

/// The complete fragment source for one effect: the shared prelude, then its `main`.
pub(crate) fn fragment_source(effect: ShaderEffect) -> String {
    let body = match effect {
        ShaderEffect::SharpBilinear => SHARP_BILINEAR,
        ShaderEffect::Lcd3x => LCD3X,
        ShaderEffect::ZfastCrt => ZFAST_CRT,
        ShaderEffect::Scanline => SCANLINE,
    };
    format!("{COMMON}{body}")
}

#[cfg(test)]
mod tests {
    //! What can be checked without a GPU: the sources are complete, distinct, and inside the
    //! GLSL ES 1.00 subset the Mali accepts. Whether they *look* right is the GL phase in
    //! `tests/screenshot.rs`, and neither check stands in for the other.

    use super::*;

    #[test]
    fn every_effect_has_a_complete_source() {
        for effect in ShaderEffect::ALL {
            let src = fragment_source(effect);
            for needed in [
                "#ifdef GL_ES",
                "precision highp float;",
                "varying vec2 v_uv;",
                "varying vec4 v_col;",
                "uniform sampler2D u_tex;",
                "uniform vec2 u_src;",
                "uniform vec2 u_dst;",
                "uniform vec4 u_uv_rect;",
                "void main()",
                "gl_FragColor",
                "texture2D(",
            ] {
                assert!(src.contains(needed), "{effect:?} is missing {needed:?}");
            }
            assert!(!src.contains('\0'), "{effect:?} carries a NUL");
        }
    }

    #[test]
    fn the_sources_stay_inside_gles2() {
        // Anything from a later GLSL, or anything a GLES2 fragment stage has no equivalent
        // for, would compile here on the desktop and fail on the device.
        for effect in ShaderEffect::ALL {
            let src = fragment_source(effect);
            for forbidden in [
                "#version",
                "textureSize",
                "dFdx",
                "dFdy",
                "fwidth",
                "discard",
                "1u",
                "uint",
                "<<",
                ">>",
            ] {
                assert!(
                    !src.contains(forbidden),
                    "{effect:?} uses {forbidden:?}, which GLES2 does not have"
                );
            }
            // No loop of any kind: the four are single-pass by design.
            assert!(!src.contains("for ("), "{effect:?} loops");
            assert!(!src.contains("while ("), "{effect:?} loops");
        }
        assert!(VERTEX_SOURCE.contains("attribute vec2 a_pos;"));
        assert!(VERTEX_SOURCE.contains("uniform vec2 u_panel;"));
        assert!(VERTEX_SOURCE.contains("uniform vec2 u_origin;"));
        assert!(VERTEX_SOURCE.contains("uniform vec2 u_dst;"));
        assert!(!VERTEX_SOURCE.contains('\0'));
    }

    #[test]
    fn the_four_are_not_one_shader_with_four_names() {
        let mut seen: Vec<(ShaderEffect, String)> = Vec::new();
        for effect in ShaderEffect::ALL {
            let src = fragment_source(effect);
            for (other, prev) in &seen {
                assert_ne!(&src, prev, "{effect:?} and {other:?} are the same source");
            }
            seen.push((effect, src));
        }

        // Each one does its own thing: a seam blend, an RGB stripe phase, a scan, a band.
        // This is a shape check, not a picture check — the GL phase is where a picture is
        // actually compared.
        assert!(fragment_source(ShaderEffect::SharpBilinear).contains("mix("));
        assert!(fragment_source(ShaderEffect::Lcd3x).contains("mod("));
        assert!(fragment_source(ShaderEffect::ZfastCrt).contains("cos("));
        let scanline = fragment_source(ShaderEffect::Scanline);
        for absent in ["mix(", "mod(", "cos("] {
            assert!(!scanline.contains(absent), "scanline gained {absent:?}");
        }
    }

    #[test]
    fn all_is_every_effect_in_a_fixed_order() {
        assert_eq!(
            ShaderEffect::ALL,
            [
                ShaderEffect::SharpBilinear,
                ShaderEffect::Lcd3x,
                ShaderEffect::ZfastCrt,
                ShaderEffect::Scanline,
            ]
        );
        for (i, effect) in ShaderEffect::ALL.iter().enumerate() {
            assert_eq!(effect.index(), i, "{effect:?} is not where it says it is");
        }
    }
}
