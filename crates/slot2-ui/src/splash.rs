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

use crate::UiCtx;

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
        let _ = (canvas, ctx);
        todo!("task 04")
    }
}
