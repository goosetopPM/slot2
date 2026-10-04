//! The state switcher: the numbered save states as polaroid cards over the paused game.
//!
//! Draw order: full-panel dim (BLACK at alpha 0.6), the title `state-switcher-title` at
//! PX_TITLE in INK centred near the top of the safe area, then the strip, then the existing
//! `hint-select` and `hint-back` spans at PX_HINT in INK_DIM on one centred line near the
//! bottom. In the strip the cards either side of the selection are drawn first, right to
//! left, and the selected one last so it is on top. The strip is a ring, like the
//! navigation: the card after the last is the first. A card that would hang off the panel is
//! not drawn at all.
//!
//! A card is a light frame (INK when selected, INK_DIM when not), a 4:3 picture area, and the
//! slot's number underneath in `LABEL_INK`, dark against either frame. The picture is decoded
//! on the first draw and kept. A slot with no thumbnail file, or one that will not decode,
//! draws a neutral plate instead — and the failure is remembered, so a card that cannot show
//! a picture does not go back to the card every frame.
//!
//! The hint line says what the buttons do and nothing else: A load and X delete when there is
//! a card to do it to, B back always, and Y undo only while the caller says an undo is live.
//!
//! draw overlays the last game frame and does not clear: the paused game is what the switcher
//! is for. Nothing here reads a state file, owns a `Card`, or looks at an mtime.

use std::collections::HashMap;
use std::path::PathBuf;

use slot2_gfx::{Canvas, Color, TexId};
use slot2_i18n::Arg;
use slot2_store::{StateKind, StateSlot};

use crate::{face, UiCtx, PX_BODY, PX_HINT, PX_TITLE};

/// Card width, in safe-area pixels.
pub const CARD_W: f32 = 180.0;
/// The frame around the picture, on the sides and the top.
pub const BORDER: f32 = 6.0;
/// The picture area is 4:3, which is what a GBA frame and a 4:3 television both are.
pub const THUMB_W: f32 = CARD_W - 2.0 * BORDER;
pub const THUMB_H: f32 = THUMB_W * 3.0 / 4.0;
/// The band under the picture, where the slot number is written.
pub const LABEL_H: f32 = 32.0;
pub const CARD_H: f32 = BORDER + THUMB_H + LABEL_H + BORDER;
/// Space between cards, so the frame of one does not touch the next.
pub const GAP: f32 = 16.0;

/// Where the middle of the strip sits in the safe area, and where the title's line box
/// starts. The hints sit at the foot, the way they do on every other menu.
pub const STRIP_CY: f32 = 250.0;
pub const TITLE_Y: f32 = 44.0;
pub const HINT_Y: f32 = 436.0;

pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);
/// The ink the slot number is written in, on the frame rather than in the frame's colour.
///
/// The frame under the number is the lightest thing on the panel, and the number drawn in
/// the frame's own ink is a number nobody can read — including the selected card, where both
/// used to be `INK`. This is dark enough for either frame: about 16:1 against `INK` and 6:1
/// against `INK_DIM`, so a card's number reads whether or not it is the selected one.
pub const LABEL_INK: Color = Color::from_rgb8(0x14, 0x16, 0x1A);
/// The plate a card with no picture shows: a neutral grey, inset from the picture area, so
/// it reads as an empty frame rather than a broken one.
pub const NO_ART: Color = Color::from_rgb8(0x4A, 0x4E, 0x55);
const NO_ART_INSET: f32 = 16.0;
/// How lit a neighbour's picture is. The selected card is the one being looked at.
const NEIGHBOUR_ALPHA: f32 = 0.55;
/// Between one hint and the next on the one line they share.
const HINT_GAP: f32 = PX_HINT;
/// A thumbnail is kept at twice the size it is drawn, like the label art.
const DECODE: (u32, u32) = ((THUMB_W as u32) * 2, (THUMB_H as u32) * 2);
/// How many thumbnail textures are kept at once. At most three cards are on screen, so this
/// is generous; it is a bound on the cache, not a working set. The least recently used one is
/// dropped when a new picture arrives.
pub const CACHE_MAX: usize = 12;

/// One numbered slot, reduced to what the strip draws. The store's `StateSlot` carries a path
/// and an mtime; neither belongs on screen, and carrying them would invite reading them.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Slot {
    number: u32,
    /// The thumbnail PNG, when the card has one. Having none is normal, not an error.
    thumb: Option<PathBuf>,
}

/// A thumbnail's texture and the size it was decoded at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Thumb {
    tex: TexId,
    w: u32,
    h: u32,
}

/// The numbered save states, as cards. Built from the card's slots, then only navigated.
pub struct StateSwitcher {
    /// Ascending by number, so the strip reads left to right the way the slots count.
    slots: Vec<Slot>,
    /// Index into `slots`, always valid when `slots` is not empty.
    selected: usize,
    /// Thumbnails by path, with when each was last asked for. `None` means "asked for and
    /// there was nothing to draw", which is remembered so a missing or corrupt file is not
    /// read again on every frame. `CACHE_MAX` of them are kept; `clear` hands the textures
    /// back.
    thumbs: HashMap<PathBuf, (Option<Thumb>, u64)>,
    /// Bumped on every lookup, so the least recently used thumbnail is the one to drop.
    clock: u64,
    /// Pictures whose slot is gone, waiting for a canvas to give them back. A refresh is a
    /// state change rather than a frame, so it has none.
    pending: Vec<Thumb>,
}

impl StateSwitcher {
    /// The numbered slots of `slots`, in numeric order, with the greatest selected.
    ///
    /// Resume is dropped: it belongs to stopping and starting the frontend, not to the slots
    /// a player moves between. The order the slots arrive in and their mtimes decide nothing.
    pub fn new(slots: Vec<StateSlot>) -> Self {
        let slots = numbered(slots);
        let selected = slots.len().saturating_sub(1);
        StateSwitcher {
            slots,
            selected,
            thumbs: HashMap::new(),
            clock: 0,
            pending: Vec::new(),
        }
    }

    /// Replace the slots with what the card holds now, the way `new` would: Resume dropped,
    /// numbered slots in numeric order, the greatest selected, nothing selected when there
    /// are none.
    ///
    /// The thumbnail cache is kept. A slot that was on screen before is the same picture
    /// now, and building a fresh switcher instead would abandon texture ids the canvas is
    /// still holding.
    pub fn refresh(&mut self, slots: Vec<StateSlot>) {
        self.slots = numbered(slots);
        self.selected = self.slots.len().saturating_sub(1);
        self.evict_missing();
    }

    /// Drop cache entries for pictures whose slot is gone.
    ///
    /// A deleted slot takes its picture with it, and its number can be handed out again to a
    /// different state: a picture left under that path would be drawn for a state that never
    /// had one. The texture cannot be freed here — a refresh is a state change, not a frame,
    /// and there is no canvas — so it waits in `pending` for the next draw or `clear`.
    fn evict_missing(&mut self) {
        let gone: Vec<PathBuf> = self
            .thumbs
            .keys()
            .filter(|path| {
                !self
                    .slots
                    .iter()
                    .any(|slot| slot.thumb.as_deref() == Some(path.as_path()))
            })
            .cloned()
            .collect();
        for path in gone {
            if let Some((Some(art), _)) = self.thumbs.remove(&path) {
                self.pending.push(art);
            }
        }
    }

    /// Put the selection where removing numbered slot `number` leaves the player: the next one
    /// up, or the one it left behind when there is nothing above it, or nothing at all.
    ///
    /// The rule the player would apply by hand: the card moves up under the cursor rather than
    /// the strip jumping back to the newest state.
    pub fn select_after_removing(&mut self, number: u32) {
        self.selected = self
            .slots
            .iter()
            .position(|slot| slot.number > number)
            .unwrap_or_else(|| self.slots.len().saturating_sub(1));
    }

    /// Move the selection onto numbered slot `number`, when the card holds it. `false` when it
    /// does not, with the selection left where it was.
    pub fn select_number(&mut self, number: u32) -> bool {
        match self.slots.iter().position(|slot| slot.number == number) {
            Some(index) => {
                self.selected = index;
                true
            }
            None => false,
        }
    }

    /// How many numbered slots there are.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// The slot on show, or `None` when the card has no numbered states at all.
    pub fn selected_kind(&self) -> Option<StateKind> {
        self.slots
            .get(self.selected)
            .map(|slot| StateKind::Numbered(slot.number))
    }

    /// One slot left, wrapping. Nothing to do when there is nothing to move through.
    pub fn left(&mut self) {
        let n = self.slots.len();
        if n == 0 {
            return;
        }
        self.selected = (self.selected + n - 1) % n;
    }

    /// One slot right, wrapping.
    pub fn right(&mut self) {
        let n = self.slots.len();
        if n == 0 {
            return;
        }
        self.selected = (self.selected + 1) % n;
    }

    /// Panel-space rect `(x, y, w, h)` of the card `offset` places from the selection. 0 is
    /// the selected card, which is centred in the safe area; positive is to its right.
    pub fn card_rect(ctx: &UiCtx, offset: isize) -> (f32, f32, f32, f32) {
        let centre_x = ctx.safe.px(crate::SAFE_W as f32 / 2.0) + offset as f32 * (CARD_W + GAP);
        let centre_y = ctx.safe.py(STRIP_CY);
        (
            centre_x - CARD_W / 2.0,
            centre_y - CARD_H / 2.0,
            CARD_W,
            CARD_H,
        )
    }

    /// Panel-space rect of the picture area inside the card `offset` places from the
    /// selection.
    pub fn thumb_rect(ctx: &UiCtx, offset: isize) -> (f32, f32, f32, f32) {
        let (x, y, ..) = Self::card_rect(ctx, offset);
        (x + BORDER, y + BORDER, THUMB_W, THUMB_H)
    }

    /// Drop every cached thumbnail. Call before the canvas goes away.
    pub fn clear(&mut self, canvas: &mut dyn Canvas) {
        for (art, _) in self.thumbs.values() {
            if let Some(art) = art {
                canvas.free(art.tex);
            }
        }
        self.thumbs.clear();
        for art in self.pending.drain(..) {
            canvas.free(art.tex);
        }
    }

    pub fn draw(&mut self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, undo_available: bool) {
        // Pictures a refresh dropped get their textures back here: a refresh has no canvas.
        for art in self.pending.drain(..) {
            canvas.free(art.tex);
        }

        let (pw, ph) = ctx.profile.geometry.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let title = ctx.i18n.spans("state-switcher-title", &[]);
        let title_w = face::spans_width(ctx, &title, PX_TITLE);
        crate::draw_spans(
            canvas,
            ctx,
            &title,
            PX_TITLE,
            ctx.safe.centre_x(title_w),
            ctx.safe.py(TITLE_Y),
            crate::splash::INK,
        );

        if self.slots.is_empty() {
            // No numbered states: the message sits where the strip would be. There is no
            // card, and nothing to select.
            let empty = ctx.i18n.spans("states-empty", &[]);
            let empty_w = face::spans_width(ctx, &empty, PX_BODY);
            let line_h = ctx.fonts.measure("", PX_BODY).line_height as f32;
            crate::draw_spans(
                canvas,
                ctx,
                &empty,
                PX_BODY,
                ctx.safe.centre_x(empty_w),
                ctx.safe.py(STRIP_CY - line_h / 2.0),
                crate::splash::INK_DIM,
            );
        } else {
            self.draw_strip(canvas, ctx);
        }

        self.draw_hints(canvas, ctx, undo_available);
    }

    /// The cards either side of the selection, then the selected one on top of them.
    ///
    /// The strip is a ring, like the navigation: the card after the last is the first, so
    /// what is drawn beside the selection is what left and right will move to. A card that
    /// would hang off the panel is not drawn at all, and a card that is already on screen is
    /// not drawn twice.
    fn draw_strip(&mut self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        let n = self.slots.len();
        let mut distance = 1isize;
        while distance < n as isize {
            let mut drew = false;
            let mut drawn_left = None;
            for side in [-1isize, 1] {
                let offset = side * distance;
                let index = (self.selected as isize + offset).rem_euclid(n as isize) as usize;
                // Two slots put the same card on both sides; one of it is enough.
                if side == 1 && drawn_left == Some(index) {
                    continue;
                }
                let rect = Self::card_rect(ctx, offset);
                if !ctx.safe.contains(rect.0, rect.1, rect.2, rect.3) {
                    continue;
                }
                self.draw_card(canvas, ctx, index, offset, false);
                drew = true;
                if side == -1 {
                    drawn_left = Some(index);
                }
            }
            if !drew {
                break;
            }
            distance += 1;
        }
        self.draw_card(canvas, ctx, self.selected, 0, true);
    }

    /// One card: frame, picture, number.
    fn draw_card(
        &mut self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        index: usize,
        offset: isize,
        selected: bool,
    ) {
        let (x, y, w, h) = Self::card_rect(ctx, offset);
        let ink = if selected {
            crate::splash::INK
        } else {
            crate::splash::INK_DIM
        };
        canvas.rect(x, y, w, h, ink);

        let (tx, ty, tw, th) = Self::thumb_rect(ctx, offset);
        // The plate behind the picture, which is also what shows in the bars either side of
        // a picture that is not 4:3.
        canvas.rect(tx, ty, tw, th, crate::splash::BACKDROP);

        match self.thumbnail(canvas, index) {
            Some(art) => {
                let (iw, ih) = fit_inside(art.w as f32, art.h as f32, tw, th);
                let tint = if selected {
                    Color::WHITE
                } else {
                    Color::WHITE.with_alpha(NEIGHBOUR_ALPHA)
                };
                canvas.image(
                    art.tex,
                    tx + (tw - iw) / 2.0,
                    ty + (th - ih) / 2.0,
                    iw,
                    ih,
                    tint,
                );
            }
            None => {
                canvas.rect(
                    tx + NO_ART_INSET,
                    ty + NO_ART_INSET,
                    tw - 2.0 * NO_ART_INSET,
                    th - 2.0 * NO_ART_INSET,
                    NO_ART,
                );
            }
        }

        let label = ctx
            .i18n
            .spans("state-slot", &[("n", Arg::from(self.slots[index].number))]);
        let label_w = face::spans_width(ctx, &label, PX_BODY);
        let line_h = ctx.fonts.measure("", PX_BODY).line_height as f32;
        crate::draw_spans(
            canvas,
            ctx,
            &label,
            PX_BODY,
            x + (CARD_W - label_w) / 2.0,
            y + BORDER + THUMB_H + (LABEL_H - line_h) / 2.0,
            LABEL_INK,
        );
    }

    /// The texture for a slot's thumbnail, decoding and uploading on first ask. `None` when
    /// the slot has no file or the file will not decode — remembered either way, so a card
    /// that cannot show a picture does not go back to the card every frame.
    fn thumbnail(&mut self, canvas: &mut dyn Canvas, index: usize) -> Option<Thumb> {
        let path = self.slots[index].thumb.as_ref()?;
        if let Some((art, used)) = self.thumbs.get_mut(path) {
            *used = self.clock;
            self.clock += 1;
            return *art;
        }

        if self.thumbs.len() >= CACHE_MAX {
            let oldest = self
                .thumbs
                .iter()
                .min_by_key(|(_, (_, used))| *used)
                .map(|(p, _)| p.clone());
            if let Some(p) = oldest {
                if let Some((Some(art), _)) = self.thumbs.remove(&p) {
                    canvas.free(art.tex);
                }
            }
        }

        let art = crate::image::decode_png(path, DECODE).map(|decoded| {
            let tex = canvas.upload_rgba8(decoded.w, decoded.h, &decoded.rgba);
            Thumb {
                tex,
                w: decoded.w,
                h: decoded.h,
            }
        });
        self.thumbs.insert(path.clone(), (art, self.clock));
        self.clock += 1;
        art
    }

    /// The hint line: what the buttons do, and only what they do.
    ///
    /// With nothing selected there is nothing to load and nothing to delete, so the row says
    /// less; the undo hint is there only while the caller says an undo is live.
    fn draw_hints(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, undo_available: bool) {
        let keys: &[&str] = match (self.slots.is_empty(), undo_available) {
            (false, false) => &["hint-load", "hint-delete", "hint-back"],
            (false, true) => &["hint-load", "hint-delete", "hint-back", "hint-undo"],
            (true, false) => &["hint-back"],
            (true, true) => &["hint-back", "hint-undo"],
        };

        // Measured first so the line can be centred: how wide a word is belongs to the
        // language, not to the layout.
        let widths: Vec<f32> = keys
            .iter()
            .map(|key| {
                let spans = ctx.i18n.spans(key, &[]);
                face::spans_width(ctx, &spans, PX_HINT)
            })
            .collect();
        let total = widths.iter().sum::<f32>() + HINT_GAP * (keys.len() - 1) as f32;
        let mut x = ctx.safe.centre_x(total);
        let y = ctx.safe.py(HINT_Y);

        for (key, width) in keys.iter().zip(&widths) {
            let spans = ctx.i18n.spans(key, &[]);
            crate::draw_spans(canvas, ctx, &spans, PX_HINT, x, y, crate::splash::INK_DIM);
            x += width + HINT_GAP;
        }
    }
}

/// The largest `(w, h)` with the shape of `(w, h)` that fits inside `(max_w, max_h)`.
fn fit_inside(w: f32, h: f32, max_w: f32, max_h: f32) -> (f32, f32) {
    let scale = (max_w / w).min(max_h / h);
    (w * scale, h * scale)
}

/// The numbered slots of `slots`, in numeric order.
///
/// The one place Resume is dropped and the order is decided, so a construction and a refresh
/// cannot drift apart.
fn numbered(slots: Vec<StateSlot>) -> Vec<Slot> {
    let mut slots: Vec<Slot> = slots
        .into_iter()
        .filter_map(|slot| match slot.kind {
            StateKind::Numbered(number) => Some(Slot {
                number,
                thumb: slot.thumb,
            }),
            StateKind::Resume => None,
        })
        .collect();
    slots.sort_by_key(|slot| slot.number);
    slots
}
