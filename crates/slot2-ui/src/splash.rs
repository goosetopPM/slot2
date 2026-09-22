//! The first screen. Implementation notes for task 04 (see tasks/04-splash.md):
//!
//! Layout, all in safe-area coordinates unless noted (use `ctx.safe.px/py/centre_x`):
//!
//! ```text
//!   clear            BACKDROP (whole panel)
//!   wordmark "SLOT2" PX_WORDMARK, INK, centred on the WHOLE PANEL (`panel_centre_x`),
//!                    line box top at safe y = 150
//!   greeting         i18n `splash-hello`, PX_TITLE, INK, centred in safe area, top at 235
//!   device line      i18n `splash-device` with args target=profile.target,
//!                    panel="{w}x{h}", PX_BODY, INK_DIM, centred in safe area, top at 275
//!   hint row         i18n spans of `hint-menu`, PX_HINT, INK_DIM, centred in safe area,
//!                    top at 440. Centre it by measuring first: width = draw once into a
//!                    `RecordingCanvas`? No — simpler: compute width with `face::measure` for
//!                    text spans and the button formula from face.rs for Btn spans; expose
//!                    that as `spans_width(ctx, spans, px)` in face.rs if convenient.
//!   safe-area frame  when `debug_frame` is true: a 1px INK_DIM rect outline around the safe
//!                    area (four thin rects), so the three geometries can be checked by eye.
//! ```
//!
//! `Splash::draw` must be idempotent frame to frame: with a warm `FaceCache` it performs no
//! uploads, only `clear`, `rect` and `image` calls.

use slot2_gfx::{Canvas, Color};
use slot2_i18n::Arg;

use crate::{face, UiCtx, PX_BODY, PX_HINT, PX_TITLE, PX_WORDMARK};

pub const BACKDROP: Color = Color::from_rgb8(0x10, 0x10, 0x12);
pub const INK: Color = Color::from_rgb8(0xF2, 0xF2, 0xF0);
pub const INK_DIM: Color = Color::from_rgb8(0x9A, 0x9A, 0xA0);

pub const WORDMARK: &str = "SLOT2";
pub const Y_WORDMARK: f32 = 150.0;
pub const Y_GREETING: f32 = 235.0;
pub const Y_DEVICE: f32 = 275.0;
pub const Y_HINT: f32 = 440.0;

#[derive(Default)]
pub struct Splash {
    pub debug_frame: bool,
}

impl Splash {
    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        canvas.clear(BACKDROP);

        // wordmark "SLOT2" PX_WORDMARK, INK, centred on the WHOLE PANEL (`panel_centre_x`),
        // line box top at safe y = 150
        let wordmark_w = face::measure(ctx, WORDMARK, PX_WORDMARK);
        face::draw_text(
            canvas,
            ctx,
            WORDMARK,
            PX_WORDMARK,
            ctx.safe.panel_centre_x(wordmark_w),
            ctx.safe.py(Y_WORDMARK),
            INK,
        );

        // greeting i18n `splash-hello`, PX_TITLE, INK, centred in safe area, top at 235
        let hello = ctx.i18n.t("splash-hello");
        let hello_w = face::measure(ctx, &hello, PX_TITLE);
        face::draw_text(
            canvas,
            ctx,
            &hello,
            PX_TITLE,
            ctx.safe.centre_x(hello_w),
            ctx.safe.py(Y_GREETING),
            INK,
        );

        // device line i18n `splash-device` with args target=profile.target,
        // panel="{w}x{h}", PX_BODY, INK_DIM, centred in safe area, top at 275
        let (pw, ph) = canvas.size();
        let panel_str = format!("{}x{}", pw, ph);
        let args: &[(&str, Arg)] = &[
            ("target", ctx.profile.target.into()),
            ("panel", panel_str.into()),
        ];
        let device = ctx.i18n.t_args("splash-device", args);
        let device_w = face::measure(ctx, &device, PX_BODY);
        face::draw_text(
            canvas,
            ctx,
            &device,
            PX_BODY,
            ctx.safe.centre_x(device_w),
            ctx.safe.py(Y_DEVICE),
            INK_DIM,
        );

        // hint row i18n spans of `hint-menu`, PX_HINT, INK_DIM, centred in safe area, top at 440
        let spans = ctx.i18n.spans("hint-menu", &[]);
        let hint_w = face::spans_width(ctx, &spans, PX_HINT);
        face::draw_spans(
            canvas,
            ctx,
            &spans,
            PX_HINT,
            ctx.safe.centre_x(hint_w),
            ctx.safe.py(Y_HINT),
            INK_DIM,
        );

        // safe-area frame when `debug_frame` is true
        if self.debug_frame {
            let s = &ctx.safe;
            // top at y, bottom at y+479, left at x, right at x+639
            canvas.rect(s.x as f32, s.y as f32, 640.0, 1.0, INK_DIM); // top
            canvas.rect(s.x as f32, (s.y + 479) as f32, 640.0, 1.0, INK_DIM); // bottom
            canvas.rect(s.x as f32, s.y as f32, 1.0, 480.0, INK_DIM); // left
            canvas.rect((s.x + 639) as f32, s.y as f32, 1.0, 480.0, INK_DIM); // right
        }
    }
}
