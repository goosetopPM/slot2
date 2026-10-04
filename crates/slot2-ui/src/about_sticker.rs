//! The About sticker: the build a machine is running, and where its notices live.
//!
//! A screen of its own, in the style of the other menus. What is on it is what the caller hands
//! over and nothing else: the version and the target arrive as arguments, and every word around
//! them — the product's name, the licence line, the path to the notices — is a sentence in the
//! language pack. The crate reads no version of its own, no host name and no environment: a
//! sticker that printed the UI crate's own version where the frontend's belongs would be a lie
//! about what the machine is running. There is no git revision because none is compiled into the
//! binary; one is not guessed at.
//!
//! This is a sticker rather than a licence viewer: it says where `System/licenses` is, which is
//! where the distribution script puts the font notices and the cores' own stamps, and promises
//! nothing about what that folder will hold when M7 finishes it.
//!
//! Draw order, mirroring the other menus (same panel, palette and text sizes): full-panel dim
//! (BLACK at alpha 0.6), box centred in the safe area (BACKDROP), the title at PX_BODY in
//! INK_DIM, the product's wordmark at [`WORDMARK_PX`] in INK, the version, the target, the
//! licence line and the notices, each centred, then the way out at PX_HINT in INK_DIM at the
//! bottom of the box. Every line is centred on its own measured width: nothing is placed by
//! counting characters, and nothing is clipped to hide a line that grew.
//!
//! draw overlays whatever is underneath and does not clear.

use slot2_gfx::{Canvas, Color};
use slot2_i18n::Arg;

use crate::UiCtx;

/// The panel's title.
pub const TITLE_KEY: &str = "about-title";
/// The product's own name, as its own message rather than as letters glued onto a word.
pub const WORDMARK_KEY: &str = "about-wordmark";
/// The build, with the version handed in.
pub const VERSION_KEY: &str = "about-version";
/// The device image this build is, with the target handed in.
pub const TARGET_KEY: &str = "about-target";
/// SLOT2 and the licence it ships under. Fixed product information, not a translation.
pub const LICENSE_KEY: &str = "about-license";
/// Where the font and core notices are on a distributed card.
pub const NOTICES_KEY: &str = "about-notices";
/// Leaving the sticker. The other menus' own back hint.
pub const HINT_KEY: &str = "hint-back";

pub const BOX_W: f32 = 440.0;
pub const BOX_H: f32 = 260.0;
pub const PAD: f32 = 16.0;
/// The wordmark's size. Twice the body text, which is what makes it the sticker's one big mark
/// without pushing the lines under it out of a 260-pixel box.
pub const WORDMARK_PX: f32 = 32.0;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);

/// Top of each line inside the box, in draw order. Spaced by hand rather than stacked, so the
/// rows stay put however tall a translation's own line box is, and far enough apart that no two
/// of them touch.
pub const TITLE_Y: f32 = PAD;
pub const WORDMARK_Y: f32 = 44.0;
pub const VERSION_Y: f32 = 96.0;
pub const TARGET_Y: f32 = 124.0;
pub const LICENSE_Y: f32 = 152.0;
pub const NOTICES_Y: f32 = 180.0;
/// The bottom line, measured up from the box's own bottom edge so a taller box moves it with it.
pub const HINT_Y: f32 = BOX_H - PAD - crate::PX_HINT;

/// What the sticker says about the build, handed over by the frontend.
///
/// Two borrowed strings and nothing else: the screen owns neither, and a caller that has no
/// version to give passes an empty one rather than something invented here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AboutInfo<'a> {
    pub version: &'a str,
    pub target: &'a str,
}

/// The About sticker.
///
/// A value with no state at all: what it draws comes from the arguments of `draw`, so there is
/// nothing to keep in step with the frontend's own idea of the build. It handles no input and
/// reads no file, clock or environment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AboutSticker;

impl AboutSticker {
    pub const fn new() -> Self {
        AboutSticker
    }

    /// Top-left of the box on the panel, centred in the safe area.
    ///
    /// The safe area and not the panel: on a 720x720 the layout region is centred but it is not
    /// the whole screen, and a box placed against the panel would drift out of the region the
    /// rest of the UI is laid out in.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, info: AboutInfo<'_>) {
        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        line(
            canvas,
            ctx,
            TITLE_KEY,
            &[],
            crate::PX_BODY,
            by + TITLE_Y,
            crate::splash::INK_DIM,
        );
        line(
            canvas,
            ctx,
            WORDMARK_KEY,
            &[],
            WORDMARK_PX,
            by + WORDMARK_Y,
            crate::splash::INK,
        );
        // The two things the caller knows are the message's own arguments: the words around them,
        // and where they go, belong to the language pack.
        line(
            canvas,
            ctx,
            VERSION_KEY,
            &[("version", Arg::from(info.version))],
            crate::PX_BODY,
            by + VERSION_Y,
            crate::splash::INK,
        );
        line(
            canvas,
            ctx,
            TARGET_KEY,
            &[("target", Arg::from(info.target))],
            crate::PX_BODY,
            by + TARGET_Y,
            crate::splash::INK,
        );
        line(
            canvas,
            ctx,
            LICENSE_KEY,
            &[],
            crate::PX_BODY,
            by + LICENSE_Y,
            crate::splash::INK,
        );
        line(
            canvas,
            ctx,
            NOTICES_KEY,
            &[],
            crate::PX_BODY,
            by + NOTICES_Y,
            crate::splash::INK_DIM,
        );
        line(
            canvas,
            ctx,
            HINT_KEY,
            &[],
            crate::PX_HINT,
            by + HINT_Y,
            crate::splash::INK_DIM,
        );
    }
}

/// One whole message, centred in the box on the width it actually measures, in `color`.
fn line(
    canvas: &mut dyn Canvas,
    ctx: &mut UiCtx,
    key: &str,
    args: &[(&str, Arg)],
    px: f32,
    y: f32,
    color: Color,
) {
    // The box is the layout's, read here rather than passed in: every line is centred in the same
    // one, and a caller with its own idea of where the box is would be a second layout.
    let bx = AboutSticker::box_origin(ctx).0;
    let spans = ctx.i18n.spans(key, args);
    let w = crate::face::spans_width(ctx, &spans, px);
    crate::draw_spans(canvas, ctx, &spans, px, bx + (BOX_W - w) / 2.0, y, color);
}
