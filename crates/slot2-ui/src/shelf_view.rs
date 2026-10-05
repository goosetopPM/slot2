use slot2_gfx::{Canvas, Color};
use slot2_store::Platform;

use crate::art::ArtCache;
use crate::face;
use crate::insert::{Insertion, Motion};
use crate::layout::SafeArea;
use crate::shelf::{Shelf, LIP_H, MOUTH_EXTRA, MOUTH_H, SLIT_H};
use crate::skin;
use crate::UiCtx;

const BAND: Color = Color::from_rgb8(0x1E, 0x21, 0x26);
const LIP: Color = Color::from_rgb8(0x3A, 0x3F, 0x47);
const SLIT: Color = Color::from_rgb8(0x07, 0x08, 0x0A);

/// Between one shelf hint and the next, and between the line and the slot below it.
const HINT_GAP: f32 = crate::PX_HINT;
const HINT_PAD: f32 = 8.0;

#[derive(Default)]
pub struct ShelfView {
    pub shelf: Shelf,
    cache: ArtCache,
    labels: Vec<Option<std::path::PathBuf>>,
    label_cache: crate::label::LabelCache,
    /// Whether each cart has a state to go back to, in the row's order. Set with
    /// `set_resume_available`, for the same reason the labels are: whether a file is there is
    /// a question for a rescan, not for a frame.
    resume_available: Vec<bool>,
}

impl ShelfView {
    /// Where the card's label art is, one entry per cart in the row's order, `None` for a
    /// cart the card has no scan of.
    ///
    /// Set when the row changes rather than passed to `draw`, because whether a file exists
    /// is a question for a rescan and not for a frame: asking it sixty times a second, for
    /// every cart on screen, is sixty stat calls a second at the one moment the device is
    /// least able to afford them.
    pub fn set_labels(&mut self, labels: Vec<Option<std::path::PathBuf>>) {
        self.labels = labels;
    }

    /// Which carts have a resume state, one entry per cart in the row's order.
    ///
    /// Set when the card is scanned, like the labels: the hint under the row says what A will
    /// do, and asking the filesystem that question once a frame is the sixty-stat-a-second
    /// the label cache exists to avoid.
    pub fn set_resume_available(&mut self, flags: Vec<bool>) {
        self.resume_available = flags;
    }

    /// Draw the row making way, and the chosen cart on its way into the slot.
    ///
    /// `at` is `insert::seat_in(t)` on the way in and `insert::seat_out(t)` on the way out,
    /// with the direction that position is going: this draws a position, not a direction, so
    /// one method covers both. The direction picks the platform's profile — the caller knows
    /// the screen, the skin knows the rhythm, and neither has to know both.
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
        at: Insertion,
    ) {
        let Insertion { seat, motion } = at;
        let skin = skin::skin(platform);
        let panel_w = safe.panel_w as f32;
        let panel_h = safe.panel_h as f32;

        let band_y = (panel_h - MOUTH_H).round();
        let mouth_w = skin.cart_size.0 + MOUTH_EXTRA;
        let mouth_x = ((panel_w - mouth_w) / 2.0).round();

        // 1. Back: the bay and the opening.
        self.draw_back(canvas, mouth_x, mouth_w, band_y);

        // 2. Where the row stands this frame. Measured before anything is drawn, because the
        // line under it is placed clear of the carts and only the row knows how tall they are.
        let placements = self.shelf.parted(safe, skin.cart_size, seat);

        // 3. What A does — but only on the frame the button was pressed on. The line
        // describes an action nobody has taken yet; once the cart is travelling the action
        // has been accepted, and a tall cartridge would either stand across the line or
        // flash it back on its way past where the line used to be.
        if seat == 0.0 {
            self.draw_hint(canvas, ctx, safe, &placements);
        }

        // 4. Parted row.
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

        // 5. The travelling cart. Nothing goes into the slot off an empty shelf — there is
        // no cartridge to put there, and drawing one anyway conjures a game the card does
        // not have.
        if seat > 0.0 && !self.shelf.is_empty() {
            let rest_x = self.shelf.rest_x(safe, skin.cart_size);
            let curve = match motion {
                Motion::Insert => skin.insert,
                Motion::Eject => skin.eject,
            };
            let t = crate::insert::travel(safe, curve, skin.cart_size, rest_x, seat);
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

        // 5. And the trim on that face. Last of all: it sits on the band, and the cart on its
        // way in passes behind it rather than over it.
        self.draw_port(canvas, skin, panel_w, band_y);
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

        // 2. Where the row stands. Measured before anything is drawn, because the line under
        // it is placed clear of the carts and only the row knows how tall they are.
        let placements = self.shelf.placements(safe, skin.cart_size);

        // 3. What A does. The same line `draw_insert` puts on its first frame: seat 0.0 is the
        // frame the button was pressed on, and a hint that appeared only once the cart had
        // moved would be a flicker at the start of every launch.
        self.draw_hint(canvas, ctx, safe, &placements);

        // 4. The carts.
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

        // 6. Front: the plastic face of the machine, and the trim on it.
        self.draw_front(canvas, panel_w, mouth_x, mouth_w, band_y);
        self.draw_port(canvas, skin, panel_w, band_y);
    }

    /// The line under the row: what the button means for the cart on show.
    ///
    /// Nothing on an empty shelf — there is no cart to play — and one line otherwise: A plays,
    /// or A goes back to where the player left off and a hold starts over.
    ///
    /// `placements` is the row exactly as this frame draws it, because where the line can go
    /// is a question about the cartridges standing on the shelf rather than about the panel.
    fn draw_hint(
        &self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        safe: &SafeArea,
        placements: &[crate::shelf::Placement],
    ) {
        if self.shelf.is_empty() {
            return;
        }
        let keys: &[&str] = if self.resume_available.get(self.shelf.selected()) == Some(&true) {
            &["hint-resume", "hint-new-game"]
        } else {
            &["hint-play"]
        };

        let widths: Vec<f32> = keys
            .iter()
            .map(|key| {
                let spans = ctx.i18n.spans(key, &[]);
                face::spans_width(ctx, &spans, crate::PX_HINT)
            })
            .collect();
        let total = widths.iter().sum::<f32>() + HINT_GAP * (keys.len() - 1) as f32;
        let mut x = safe.centre_x(total);

        let line_h = ctx.fonts.measure("", crate::PX_HINT).line_height as f32;
        // Above the slot, like the toast, and never past the safe area: on the square panel
        // the machine hangs below the layout and a line pinned to the slot would sit off it.
        let floor = (safe.panel_h as f32 - MOUTH_H).min(safe.py(crate::SAFE_H as f32));
        let mut y = floor - HINT_PAD - line_h;

        // A cartridge standing across that line hides it: the line is centred on the row, and
        // the selection is the tallest cart there. Which carts reach the line is the row's own
        // geometry — a shorter one leaves the line exactly where it was, which is why nothing
        // here tests for a platform name or a panel size. The row's carts all stand on the
        // same line, so clearing the tallest of the ones the line would cross clears them all.
        let mut covered = f32::INFINITY;
        for p in placements {
            let across = p.x < x + total && p.x + p.w > x;
            if across && p.y < y + line_h && p.y + p.h > y {
                covered = covered.min(p.y);
            }
        }
        if covered.is_finite() {
            y = covered - HINT_PAD - line_h;
        }

        for (key, width) in keys.iter().zip(&widths) {
            let spans = ctx.i18n.spans(key, &[]);
            crate::draw_spans(
                canvas,
                ctx,
                &spans,
                crate::PX_HINT,
                x,
                y,
                crate::splash::INK_DIM,
            );
            x += width + HINT_GAP;
        }
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

        let label_path = self.labels.get(index).and_then(|p| p.as_ref());
        let art = label_path.and_then(|p| {
            self.label_cache
                .art(canvas, p, (skin.label.w as u32, skin.label.h as u32))
        });

        if let Some(art) = art {
            canvas.image(art.tex, lx, ly, lw, lh, Color::WHITE.with_alpha(alpha));
        } else {
            let title = titles.get(index).copied().unwrap_or("");
            let [pr, pg, pb] = crate::label::printed_colour(title);
            canvas.rect(
                lx,
                ly,
                lw,
                lh,
                Color::from_rgb8(pr, pg, pb).with_alpha(alpha),
            );
        }

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

    /// The trim around the mouth: a drawing per platform, rasterised once and tinted with the
    /// machine's own lip colour, so it reads as part of the face rather than a sticker on it.
    ///
    /// It goes on last — over the band, over the lip and over the carts — and its middle is
    /// transparent, which is the whole trick: the trim dresses the mouth without covering the
    /// cartridge standing in it. Drawn at the mouth's own height, centred on the panel, so the
    /// trim lines up with the slot the carts actually go into.
    fn draw_port(
        &mut self,
        canvas: &mut dyn Canvas,
        skin: &skin::PlatformSkin,
        panel_w: f32,
        band_y: f32,
    ) {
        let art_w = skin.port_size.0.round() as u32;
        let art_h = skin.port_size.1.round() as u32;
        let Some(tex) = self.cache.mask(canvas, skin.port, art_w, art_h) else {
            return;
        };
        let w = skin.port_size.0;
        let x = ((panel_w - w) / 2.0).round();
        canvas.image(tex, x, band_y, w, MOUTH_H, LIP);
    }
}
