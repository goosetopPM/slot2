//! The hint under the shelf row: what A will do for the cart on show.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_ui::shelf_view::ShelfView;
use slot2_ui::{UiCtx, PX_HINT};

const PANELS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn draw(view: &mut ShelfView, ctx: &mut UiCtx) -> RecordingCanvas {
    let safe = ctx.safe;
    let mut c = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    view.draw(&mut c, ctx, &safe, slot2_store::Platform::Gba, &["A", "B"]);
    c
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
