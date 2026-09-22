//! Contract for the power menu. Task 05 makes these pass without editing this file.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_ui::power_menu::{BOX_H, BOX_W, DIM, ITEMS, PAD, ROW_H};
use slot2_ui::{face, PowerChoice, PowerMenu, UiCtx, PX_TITLE};

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

#[test]
fn navigation_wraps_and_choice_follows() {
    let mut m = PowerMenu::default();
    assert_eq!(m.choice(), PowerChoice::Resume);
    m.up();
    assert_eq!(m.choice(), PowerChoice::PowerOff);
    m.down();
    m.down();
    assert_eq!(m.choice(), PowerChoice::Restart);
    m.down();
    m.down();
    assert_eq!(m.choice(), PowerChoice::Resume);
    assert_eq!(ITEMS.len(), 3);
}

#[test]
fn draws_dim_box_three_labels_and_a_highlight_inside_the_safe_area() {
    for target in ["rgsp", "rg35xxsp", "rgcubexx"] {
        let mut ctx = ctx(target, "ko");
        let (w, h) = ctx.profile.geometry.size();
        let mut c = RecordingCanvas::new(w, h);
        let mut m = PowerMenu::default();
        m.down(); // Restart selected
        m.draw(&mut c, &mut ctx);
        let ops = &c.ops;
        // No clear: it overlays.
        assert!(!ops.iter().any(|o| matches!(o, Op::Clear(_))), "{target}");
        // First draw is the full-panel dim.
        let first_rect = ops.iter().find(|o| matches!(o, Op::Rect { .. })).unwrap();
        match first_rect {
            Op::Rect {
                x,
                y,
                w: rw,
                h: rh,
                color,
            } => {
                assert_eq!(
                    (*x, *y, *rw, *rh),
                    (0.0, 0.0, w as f32, h as f32),
                    "{target}: dim covers panel"
                );
                assert_eq!(*color, DIM);
            }
            _ => unreachable!(),
        }
        let (bx, by) = PowerMenu::box_origin(&ctx);
        assert!(
            ctx.safe.contains(bx, by, BOX_W, BOX_H),
            "{target}: box in safe area"
        );
        // Highlight on row 1.
        let row1 = PowerMenu::row_y(&ctx, 1);
        assert!(
            ops.iter().any(|o| matches!(o, Op::Rect { x, y, w: rw, h: rh, .. }
                if (*x - (bx + PAD)).abs() < 0.5 && (*y - row1).abs() < 0.5 && (*rw - (BOX_W - 2.0 * PAD)).abs() < 0.5 && (*rh - ROW_H).abs() < 0.5)),
            "{target}: highlight rect on the selected row"
        );
        // Three item labels, each centred in the box, in row order.
        let labels: Vec<f32> = ITEMS
            .iter()
            .map(|c| {
                let text = ctx.i18n.t(c.key());
                face::measure(&mut ctx, &text, PX_TITLE)
            })
            .collect();
        let imgs: Vec<(f32, f32, f32)> = ops
            .iter()
            .filter_map(|o| match o {
                Op::Image { x, y, w, .. } => Some((*x, *y, *w)),
                _ => None,
            })
            .collect();
        for (i, lw) in labels.iter().enumerate() {
            let expect_x = bx + (BOX_W - lw) / 2.0;
            let row = PowerMenu::row_y(&ctx, i);
            assert!(
                imgs.iter().any(|(x, y, w)| (x - expect_x).abs() < 0.5 && (w - lw).abs() < 0.5 && *y >= row && *y < row + ROW_H),
                "{target}: label {i} centred in its row (expect x {expect_x}, row {row}); got {imgs:?}"
            );
        }
        // Hint caps: at least two rects besides dim + highlight (A and B caps).
        let rects = ops.iter().filter(|o| matches!(o, Op::Rect { .. })).count();
        assert!(rects >= 4, "{target}: dim, highlight, two caps: {rects}");
    }
}

#[test]
fn second_draw_uploads_nothing() {
    let mut ctx = ctx("rgsp", "en");
    let mut c = RecordingCanvas::new(720, 480);
    let m = PowerMenu::default();
    m.draw(&mut c, &mut ctx);
    let n = c.ops.len();
    m.draw(&mut c, &mut ctx);
    assert!(!c.ops[n..]
        .iter()
        .any(|o| matches!(o, Op::UploadAlpha8 { .. })));
}
