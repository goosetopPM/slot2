//! The hint under the shelf row: what A will do for the cart on show.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_store::Platform;
use slot2_ui::shelf::MOUTH_H;
use slot2_ui::shelf_view::ShelfView;
use slot2_ui::{skin, UiCtx, PX_HINT};

const PANELS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn draw_on(view: &mut ShelfView, ctx: &mut UiCtx, platform: Platform) -> RecordingCanvas {
    let safe = ctx.safe;
    let mut c = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    view.draw(&mut c, ctx, &safe, platform, &["A", "B"]);
    c
}

fn draw(view: &mut ShelfView, ctx: &mut UiCtx) -> RecordingCanvas {
    draw_on(view, ctx, Platform::Gba)
}

/// What the hint line drew, in the order it drew it: the word and the label inside each cap.
/// The hints are the only things on the shelf written in the dim ink.
fn hint_marks(c: &RecordingCanvas) -> Vec<(f32, f32, f32)> {
    let ink = slot2_ui::splash::INK_DIM;
    let cap = ink.with_alpha(0.25);
    c.frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image { x, y, w, tint, .. } if *tint == ink => Some((*x, *y, *w)),
            Op::Rect { x, y, w, color, .. } if *color == cap => Some((*x, *y, *w)),
            _ => None,
        })
        .collect()
}

/// The widths of the words and cap labels the hint line drew: those are the images, and the
/// caps themselves are rects of the same ink a quarter lit.
fn hint_words(c: &RecordingCanvas) -> Vec<f32> {
    let ink = slot2_ui::splash::INK_DIM;
    c.frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image { w, tint, .. } if *tint == ink => Some(*w),
            _ => None,
        })
        .collect()
}

/// The widths those marks should have, from the messages themselves.
fn expected_widths(ctx: &mut UiCtx, keys: &[&str]) -> Vec<f32> {
    let mut widths: Vec<f32> = keys
        .iter()
        .flat_map(|key| {
            ctx.i18n
                .spans(key, &[])
                .into_iter()
                .map(|span| match span {
                    slot2_i18n::Span::Text(text) => slot2_ui::face::measure(ctx, &text, PX_HINT),
                    slot2_i18n::Span::Btn(button) => {
                        slot2_ui::face::measure(ctx, button.label(), PX_HINT)
                    }
                })
                .collect::<Vec<f32>>()
        })
        .collect();
    widths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    widths
}

#[test]
fn the_hint_follows_the_cart_under_the_cursor() {
    let mut ctx = ctx("rgsp", "en");
    let mut view = ShelfView::default();
    view.shelf.set_len(2);
    view.set_resume_available(vec![false, true]);

    let c = draw(&mut view, &mut ctx);
    let mut drawn = hint_words(&c);
    drawn.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(drawn, expected_widths(&mut ctx, &["hint-play"]));

    // The cart with a state to go back to says both things it can do.
    view.shelf.right();
    let c = draw(&mut view, &mut ctx);
    let mut drawn = hint_words(&c);
    drawn.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut want = expected_widths(&mut ctx, &["hint-resume", "hint-new-game"]);
    want.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(drawn, want, "the resume hint is not the row it should be");
}

#[test]
fn an_empty_shelf_does_not_draw_a_hint() {
    let mut ctx = ctx("rgsp", "en");
    let mut view = ShelfView::default();
    view.shelf.set_len(0);
    let c = draw(&mut view, &mut ctx);
    assert!(
        hint_marks(&c).is_empty(),
        "an empty shelf promised a game: {:?}",
        hint_marks(&c)
    );
}

/// Everything the hint line painted, as one rectangle: the words and the cap labels are a
/// single line of text, and the band they cover is what a cartridge can hide.
fn hint_rects(c: &RecordingCanvas) -> Vec<(f32, f32, f32, f32)> {
    let ink = slot2_ui::splash::INK_DIM;
    let cap = ink.with_alpha(0.25);
    c.frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image {
                x, y, w, h, tint, ..
            } if *tint == ink => Some((*x, *y, *w, *h)),
            Op::Rect {
                x, y, w, h, color, ..
            } if *color == cap => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .collect()
}

fn hint_band(c: &RecordingCanvas) -> Option<(f32, f32, f32, f32)> {
    let marks = hint_rects(c);
    if marks.is_empty() {
        return None;
    }
    let min = |pick: fn(&(f32, f32, f32, f32)) -> f32| {
        marks.iter().map(pick).fold(f32::INFINITY, f32::min)
    };
    let max = |pick: fn(&(f32, f32, f32, f32)) -> f32| {
        marks.iter().map(pick).fold(f32::NEG_INFINITY, f32::max)
    };
    let (x0, y0) = (min(|m| m.0), min(|m| m.1));
    let (x1, y1) = (max(|m| m.0 + m.2), max(|m| m.1 + m.3));
    Some((x0, y0, x1 - x0, y1 - y0))
}

fn overlaps(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

#[test]
fn the_stable_hint_is_readable_on_every_panel_platform_and_language() {
    // Three panels, seven shelves, two languages, both things A can mean. The line has to be
    // there, inside the safe area, and clear of the row and the machine's face — on the square
    // panel the tall Game Boy cartridge is what used to cover it.
    //
    // One view for the whole matrix on purpose: the artwork cache is keyed by drawing and
    // size, so a fresh view per case would re-rasterise every cart and port without asking a
    // single extra question.
    let mut view = ShelfView::default();
    for target in PANELS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let safe = ctx.safe;
            for platform in Platform::ALL {
                for flags in [vec![false], vec![true]] {
                    view.shelf.set_len(2);
                    view.set_resume_available(flags);
                    let c = draw_on(&mut view, &mut ctx, platform);

                    let band = hint_band(&c).unwrap_or_else(|| {
                        panic!("{target}/{lang}/{platform:?}: the row drew no hint at all")
                    });
                    assert!(
                        safe.contains(band.0, band.1, band.2, band.3),
                        "{target}/{lang}/{platform:?}: the hint at {band:?} left the safe area"
                    );

                    // The cart it belongs to, from the row's own placement, and the band of
                    // plastic across the foot of the panel the mouth is cut into.
                    let placements = view.shelf.placements(&safe, skin::skin(platform).cart_size);
                    let selected = placements
                        .iter()
                        .find(|p| p.index == view.shelf.selected())
                        .unwrap_or_else(|| {
                            panic!("{target}/{lang}/{platform:?}: the selection is off the panel")
                        });
                    let cart = (selected.x, selected.y, selected.w, selected.h);
                    assert!(
                        !overlaps(band, cart),
                        "{target}/{lang}/{platform:?}: the cart at {cart:?} covers the hint at {band:?}"
                    );
                    let face = (
                        0.0,
                        safe.panel_h as f32 - MOUTH_H,
                        safe.panel_w as f32,
                        MOUTH_H,
                    );
                    assert!(
                        !overlaps(band, face),
                        "{target}/{lang}/{platform:?}: the hint at {band:?} sits on the machine's face"
                    );
                }
            }
        }
    }
}

#[test]
fn a_tall_cart_puts_the_hint_above_the_row() {
    // The panel the defect was found on: square, wearing the tall Game Boy cartridge. Its foot
    // stands below the safe area, so there is no room under the row for the line at all — the
    // only place it can be read is above the carts.
    let mut ctx = ctx("rgcubexx", "en");
    let safe = ctx.safe;
    let mut view = ShelfView::default();
    view.shelf.set_len(2);
    let c = draw_on(&mut view, &mut ctx, Platform::Gb);

    let band = hint_band(&c).expect("the square panel drew no hint");
    let placements = view
        .shelf
        .placements(&safe, skin::skin(Platform::Gb).cart_size);
    let selected = placements
        .iter()
        .find(|p| p.index == view.shelf.selected())
        .expect("the selection is off the panel");
    assert!(
        band.1 + band.3 <= selected.y,
        "the hint at {band:?} is not above the tall cart whose top is at {}",
        selected.y
    );
}

#[test]
fn an_empty_shelf_draws_no_hint_on_any_panel() {
    let mut view = ShelfView::default();
    view.shelf.set_len(0);
    for target in PANELS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            for platform in Platform::ALL {
                let c = draw_on(&mut view, &mut ctx, platform);
                assert!(
                    hint_rects(&c).is_empty(),
                    "{target}/{lang}/{platform:?}: an empty shelf promised a game"
                );
            }
        }
    }
}

#[test]
fn every_hint_row_fits_the_safe_area_on_every_panel() {
    for target in PANELS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            for flags in [vec![false], vec![true]] {
                let mut view = ShelfView::default();
                view.shelf.set_len(1);
                view.set_resume_available(flags);
                let c = draw(&mut view, &mut ctx);
                let marks = hint_marks(&c);
                assert!(!marks.is_empty(), "{target}/{lang}: no hint at all");
                for (x, y, w) in marks {
                    // The line box the text was laid out in is as tall as the face at this
                    // size; the mark itself is what has to be on the panel.
                    assert!(
                        ctx.safe.contains(x, y, w, 24.0),
                        "{target}/{lang}: a hint at {x},{y} {w} wide left the safe area"
                    );
                }
            }
        }
    }
}
