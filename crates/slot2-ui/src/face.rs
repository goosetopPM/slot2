//! Text → texture, cached. Implementation notes for task 04 (see tasks/04-splash.md):
//!
//! - `FaceCache` maps `(text, px_key)` → `Face { tex, w, h, baseline }` where
//!   `px_key = (px * 100.0).round() as u32`. `face()` rasterises through
//!   `ctx.fonts.rasterize(text, px)` and uploads with `canvas.upload_alpha8` on a miss.
//!   Faces are never freed in M0 (a later task adds eviction); `len()` reports entries.
//! - `draw_text` draws the face at `(x, y)` = top-left of the line box, tinted `color`,
//!   and returns the face width as f32. It must call `upload_alpha8` at most once per
//!   distinct `(text, px)` for the cache's lifetime.
//! - `measure` returns the width without drawing (via `ctx.fonts.measure`), no upload.
//! - `draw_spans`: walk spans left to right. `Span::Text(t)` → `draw_text`. `Span::Btn(b)` →
//!   let `label = b.label()`, `lw = measure(label, px)`, `pad = px * 0.4`, `gap = px * 0.3`;
//!   draw `canvas.rect(pen + gap, y, lw + 2*pad, line_h, color.with_alpha(0.25))` then
//!   `draw_text(label, px, pen + gap + pad, y, color)`; advance `pen` by
//!   `gap + lw + 2*pad + gap`. `line_h` is `ctx.fonts.measure("", px).line_height as f32`.
//!   Return `pen` (total width).

use std::collections::HashMap;

use slot2_gfx::{Canvas, Color, TexId};
use slot2_i18n::Span;

use crate::UiCtx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face {
    pub tex: TexId,
    pub w: u32,
    pub h: u32,
    pub baseline: u32,
}

#[derive(Default)]
pub struct FaceCache {
    faces: HashMap<(String, u32), Face>,
}

impl FaceCache {
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}

/// The face for `text` at `px`, uploading it on first use.
pub fn face(canvas: &mut dyn Canvas, ctx: &mut UiCtx, text: &str, px: f32) -> Face {
    let px_key = (px * 100.0).round() as u32;
    if let Some(f) = ctx.faces.faces.get(&(text.to_string(), px_key)) {
        return *f;
    }
    let b = ctx.fonts.rasterize(text, px);
    let tex = canvas.upload_alpha8(b.width, b.height, &b.data);
    let face = Face {
        tex,
        w: b.width,
        h: b.height,
        baseline: b.baseline,
    };
    ctx.faces.faces.insert((text.to_string(), px_key), face);
    face
}

/// Width of `text` at `px` without drawing or uploading.
pub fn measure(ctx: &mut UiCtx, text: &str, px: f32) -> f32 {
    ctx.fonts.measure(text, px).width
}

/// Draw `text` with its top-left line-box corner at `(x, y)`. Returns the width.
pub fn draw_text(
    canvas: &mut dyn Canvas,
    ctx: &mut UiCtx,
    text: &str,
    px: f32,
    x: f32,
    y: f32,
    color: Color,
) -> f32 {
    let w = measure(ctx, text, px);
    let f = face(canvas, ctx, text, px);
    canvas.image(f.tex, x, y, w, f.h as f32, color);
    w
}

/// Width of `spans` at `px` without drawing or uploading.
pub fn spans_width(ctx: &mut UiCtx, spans: &[Span], px: f32) -> f32 {
    let mut width = 0.0;
    for span in spans {
        match span {
            Span::Text(t) => {
                width += measure(ctx, t, px);
            }
            Span::Btn(b) => {
                let label = b.label();
                let lw = measure(ctx, label, px);
                let pad = px * 0.4;
                let gap = px * 0.3;
                width += gap + lw + 2.0 * pad + gap;
            }
        }
    }
    width
}

pub fn draw_spans(
    canvas: &mut dyn Canvas,
    ctx: &mut UiCtx,
    spans: &[Span],
    px: f32,
    x: f32,
    y: f32,
    color: Color,
) -> f32 {
    let line_h = ctx.fonts.measure("", px).line_height as f32;
    let mut pen = 0.0;
    for span in spans {
        match span {
            Span::Text(t) => {
                pen += draw_text(canvas, ctx, t, px, x + pen, y, color);
            }
            Span::Btn(b) => {
                let label = b.label();
                let lw = measure(ctx, label, px);
                let pad = px * 0.4;
                let gap = px * 0.3;
                canvas.rect(
                    x + pen + gap,
                    y,
                    lw + 2.0 * pad,
                    line_h,
                    color.with_alpha(0.25),
                );
                draw_text(canvas, ctx, label, px, x + pen + gap + pad, y, color);
                pen += gap + lw + 2.0 * pad + gap;
            }
        }
    }
    pen
}
