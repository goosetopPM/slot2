use slot2_gfx::{Canvas, Color};
use slot2_store::Platform;

use crate::art::ArtCache;
use crate::face;
use crate::layout::SafeArea;
use crate::shelf::Shelf;
use crate::skin;
use crate::UiCtx;

/// How much wider than a cartridge the slot's mouth is.
const MOUTH_EXTRA: f32 = 14.0;
/// The band of chrome across the foot of the panel that the mouth is cut into.
const MOUTH_H: f32 = 58.0;
/// The opening itself.
const SLIT_H: f32 = 9.0;
/// A highlight along the top edge of the band, which is what makes it read as a surface
/// standing proud of the background rather than a painted rectangle.
const LIP_H: f32 = 2.0;

const BAND: Color = Color::from_rgb8(0x1E, 0x21, 0x26);
const LIP: Color = Color::from_rgb8(0x3A, 0x3F, 0x47);
const SLIT: Color = Color::from_rgb8(0x07, 0x08, 0x0A);

#[derive(Default)]
pub struct ShelfView {
    pub shelf: Shelf,
    cache: ArtCache,
}

impl ShelfView {
    /// Draw the row and the slot it sits above.
    ///
    /// `titles` is one per cart, in the same order as the row. The caller owns the
    /// selection and the scroll; this only draws what they say.
    pub fn draw(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        safe: &SafeArea,
        platform: Platform,
        titles: &[&str],
    ) {
        let skin = skin::skin(platform);
        let panel_w = safe.panel_w as f32;

        // 1. The slot, which is chrome rather than artwork: a band across the foot of the
        // panel, a mouth a little wider than a cartridge, and a lip above it. Ported from
        // the original's `slot_chrome`, whose proportions are in the same units the cart
        // drawings are, so they carry across unscaled.
        //
        // Drawn even when the row is empty — a shelf with no games is not a blank screen,
        // and the slot is what says something is meant to go here.
        let panel_h = safe.panel_h as f32;
        let band_y = panel_h - MOUTH_H;
        let mouth_w = skin.cart_size.0 + MOUTH_EXTRA;
        let mouth_x = ((panel_w - mouth_w) / 2.0).round();

        canvas.rect(0.0, band_y, panel_w, MOUTH_H, BAND);
        canvas.rect(0.0, band_y, panel_w, LIP_H, LIP);
        canvas.rect(mouth_x, band_y + LIP_H + 5.0, mouth_w, SLIT_H, SLIT);

        // 2. The carts.
        //
        // Artwork is rasterised once, at the size the *selection* is drawn, and the
        // neighbours are that same texture scaled down by the canvas. Asking the cache for
        // each placement's own size instead would rasterise an SVG and upload a texture on
        // nearly every frame of every scroll — a neighbour's size lerps continuously as the
        // row slides — and nothing evicts them. A 0.78x downscale of a quad costs nothing
        // and looks the same.
        let art_w = skin.cart_size.0.round() as u32;
        let art_h = skin.cart_size.1.round() as u32;
        let shell_tex = self.cache.mask(canvas, skin.cart, art_w, art_h);
        let detail_tex = (!skin.cart_detail.is_empty())
            .then(|| self.cache.mask(canvas, skin.cart_detail, art_w, art_h))
            .flatten();

        let placements = self.shelf.placements(safe, skin.cart_size);
        let selected_index = self.shelf.selected();

        for p in placements.iter() {
            let alpha = p.alpha;
            let [r, g, b] = skin.shell.colour;
            let tint = Color::from_rgb8(r, g, b).with_alpha(alpha);

            // Finish::Translucent is not honoured yet: a clear shell should lighten toward
            // the rim, and there is no cheap way to do that with a flat tint. Drawn solid
            // until the shader work of M5 gives it somewhere to live.
            if let Some(tex) = shell_tex {
                canvas.image(tex, p.x, p.y, p.w, p.h, tint);
            }
            if let Some(tex) = detail_tex {
                // Darker than the shell, so the moulded ribs and recesses read as shadow.
                let detail_tint = Color::rgba(tint.r * 0.8, tint.g * 0.8, tint.b * 0.8, alpha);
                canvas.image(tex, p.x, p.y, p.w, p.h, detail_tint);
            }

            // Label plate: a lightened area where the sticker goes.
            let scale_x = p.w / skin.cart_size.0;
            let scale_y = p.h / skin.cart_size.1;
            let lw = skin.label.w * scale_x;
            let lh = skin.label.h * scale_y;
            let lx = p.x + skin.label.x * scale_x;
            let ly = p.y + skin.label.y * scale_y;

            canvas.rect(lx, ly, lw, lh, Color::WHITE.with_alpha(0.2 * alpha));

            // 3. The title of the selection only — a side cart is small and dim and its
            // title would be illegible. Keyed on the index rather than on "is this the last
            // placement", because mid-scroll the last one is whichever cart is nearest the
            // middle, and the title would blink out exactly when it is being read.
            if p.index == selected_index && p.index < titles.len() {
                let title = titles[p.index];

                // Nothing may rasterise or upload on a redraw. face() caches based on (text, px).
                // Using a constant PX_TITLE ensures the cache is hit even while the cart
                // is scaling during a scroll; we scale the resulting quad instead.
                let base_px = crate::PX_TITLE;
                let f = face::face(canvas, ctx, title, base_px);
                let tw = face::measure(ctx, title, base_px);

                // Scale to match the cart's current size, then shrink further if the title
                // is too wide for the label plate.
                let mut s = scale_x;
                if tw * s > lw {
                    s = lw / tw;
                }

                let dw = tw * s;
                let dh = f.h as f32 * s;

                canvas.image(
                    f.tex,
                    lx + (lw - dw) / 2.0,
                    ly + (lh - dh) / 2.0,
                    dw,
                    dh,
                    Color::WHITE.with_alpha(alpha),
                );
            }
        }
    }
}
