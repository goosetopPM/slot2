use slot2_gfx::{Canvas, Color};
use slot2_store::Platform;

use crate::art::ArtCache;
use crate::face;
use crate::layout::SafeArea;
use crate::shelf::{Shelf, LIP_H, MOUTH_EXTRA, MOUTH_H, SLIT_H};
use crate::skin;
use crate::UiCtx;

const BAND: Color = Color::from_rgb8(0x1E, 0x21, 0x26);
const LIP: Color = Color::from_rgb8(0x3A, 0x3F, 0x47);
const SLIT: Color = Color::from_rgb8(0x07, 0x08, 0x0A);

#[derive(Default)]
pub struct ShelfView {
    pub shelf: Shelf,
    cache: ArtCache,
}

impl ShelfView {
    /// Draw the row making way, and the chosen cart on its way into the slot.
    ///
    /// `seat` is `insert::seat_in(t)` on the way in and `insert::seat_out(t)` on the way
    /// out: this draws a position, not a direction, so one method covers both.
    ///
    /// At `seat == 0.0` what comes out must be what [`ShelfView::draw`] draws, because that
    /// is the frame the button was pressed on and nothing has moved yet.
    pub fn draw_insert(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        safe: &SafeArea,
        platform: Platform,
        titles: &[&str],
        seat: f32,
    ) {
        let skin = skin::skin(platform);
        let panel_w = safe.panel_w as f32;
        let panel_h = safe.panel_h as f32;

        let band_y = (panel_h - MOUTH_H).round();
        let mouth_w = skin.cart_size.0 + MOUTH_EXTRA;
        let mouth_x = ((panel_w - mouth_w) / 2.0).round();

        // 1. Back: the bay and the opening.
        self.draw_back(canvas, mouth_x, mouth_w, band_y);

        // 2. Parted row.
        let placements = self.shelf.parted(safe, skin.cart_size, seat);
        let (shell_tex, detail_tex) = self.prepare_textures(canvas, skin);
        self.draw_placements(
            canvas,
            ctx,
            &placements,
            skin,
            titles,
            shell_tex,
            detail_tex,
        );

        // 3. The travelling cart. Nothing goes into the slot off an empty shelf — there is
        // no cartridge to put there, and drawing one anyway conjures a game the card does
        // not have.
        if seat > 0.0 && !self.shelf.is_empty() {
            let rest_x = self.shelf.rest_x(safe, skin.cart_size);
            let t = crate::insert::travel(safe, skin.cart_size, rest_x, seat);
            self.draw_cart(
                canvas,
                ctx,
                t.x,
                t.y,
                t.w,
                t.h,
                1.0,
                self.shelf.selected(),
                self.shelf.selected(),
                skin,
                titles,
                shell_tex,
                detail_tex,
            );
        }

        // 4. Front: the plastic face of the machine.
        self.draw_front(canvas, panel_w, mouth_x, mouth_w, band_y);
    }

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
        let panel_h = safe.panel_h as f32;

        let band_y = (panel_h - MOUTH_H).round();
        let mouth_w = skin.cart_size.0 + MOUTH_EXTRA;
        let mouth_x = ((panel_w - mouth_w) / 2.0).round();

        // 1. Back: the bay and the opening.
        self.draw_back(canvas, mouth_x, mouth_w, band_y);

        // 2. The carts.
        let placements = self.shelf.placements(safe, skin.cart_size);
        let (shell_tex, detail_tex) = self.prepare_textures(canvas, skin);
        self.draw_placements(
            canvas,
            ctx,
            &placements,
            skin,
            titles,
            shell_tex,
            detail_tex,
        );

        // 3. Front: the plastic face of the machine.
        self.draw_front(canvas, panel_w, mouth_x, mouth_w, band_y);
    }

    fn prepare_textures(
        &mut self,
        canvas: &mut dyn Canvas,
        skin: &skin::PlatformSkin,
    ) -> (Option<slot2_gfx::TexId>, Option<slot2_gfx::TexId>) {
        let art_w = skin.cart_size.0.round() as u32;
        let art_h = skin.cart_size.1.round() as u32;
        let shell_tex = self.cache.mask(canvas, skin.cart, art_w, art_h);
        let detail_tex = (!skin.cart_detail.is_empty())
            .then(|| self.cache.mask(canvas, skin.cart_detail, art_w, art_h))
            .flatten();
        (shell_tex, detail_tex)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_placements(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        placements: &[crate::shelf::Placement],
        skin: &skin::PlatformSkin,
        titles: &[&str],
        shell_tex: Option<slot2_gfx::TexId>,
        detail_tex: Option<slot2_gfx::TexId>,
    ) {
        let selected_index = self.shelf.selected();
        for p in placements.iter() {
            self.draw_cart(
                canvas,
                ctx,
                p.x,
                p.y,
                p.w,
                p.h,
                p.alpha,
                p.index,
                selected_index,
                skin,
                titles,
                shell_tex,
                detail_tex,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_cart(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        alpha: f32,
        index: usize,
        selected_index: usize,
        skin: &skin::PlatformSkin,
        titles: &[&str],
        shell_tex: Option<slot2_gfx::TexId>,
        detail_tex: Option<slot2_gfx::TexId>,
    ) {
        let [r, g, b] = skin.shell.colour;
        let tint = Color::from_rgb8(r, g, b).with_alpha(alpha);

        if let Some(tex) = shell_tex {
            canvas.image(tex, x, y, w, h, tint);
        }
        if let Some(tex) = detail_tex {
            let detail_tint = Color::rgba(tint.r * 0.8, tint.g * 0.8, tint.b * 0.8, alpha);
            canvas.image(tex, x, y, w, h, detail_tint);
        }

        let scale_x = w / skin.cart_size.0;
        let scale_y = h / skin.cart_size.1;
        let lw = skin.label.w * scale_x;
        let lh = skin.label.h * scale_y;
        let lx = x + skin.label.x * scale_x;
        let ly = y + skin.label.y * scale_y;

        canvas.rect(lx, ly, lw, lh, Color::WHITE.with_alpha(0.2 * alpha));

        if index == selected_index && index < titles.len() {
            let title = titles[index];
            let base_px = crate::PX_TITLE;
            let f = face::face(canvas, ctx, title, base_px);
            let tw = face::measure(ctx, title, base_px);

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

    /// Everything you can see *into*: the bay the cart stands in and the opening at the foot
    /// of it. Both are holes, so the cart passes in front of them.
    ///
    /// The bay has to be here rather than in the front. Painted over the cart instead, a
    /// seated cartridge shows through the nine-pixel opening and nowhere else — a hundred
    /// and thirty-five pixels of cartridge arrive at the slot and disappear. What is meant
    /// to happen is that it stops with its top edge just inside the mouth and the rest of it
    /// sticking out, which is why `SEATED_BELOW_LIP` is four pixels and not the depth of a
    /// cartridge.
    fn draw_back(&self, canvas: &mut dyn Canvas, mouth_x: f32, mouth_w: f32, band_y: f32) {
        canvas.rect(mouth_x, band_y, mouth_w, MOUTH_H, BAND);
        canvas.rect(mouth_x, band_y + LIP_H + 5.0, mouth_w, SLIT_H, SLIT);
    }

    /// The plastic: the band either side of the mouth, and the lip along its top edge. This
    /// is all that occludes, and the mouth itself stays open.
    fn draw_front(
        &self,
        canvas: &mut dyn Canvas,
        panel_w: f32,
        mouth_x: f32,
        mouth_w: f32,
        band_y: f32,
    ) {
        canvas.rect(0.0, band_y + LIP_H, mouth_x, MOUTH_H - LIP_H, BAND);
        canvas.rect(
            mouth_x + mouth_w,
            band_y + LIP_H,
            panel_w - (mouth_x + mouth_w),
            MOUTH_H - LIP_H,
            BAND,
        );
        canvas.rect(0.0, band_y, panel_w, LIP_H, LIP);
    }
}
