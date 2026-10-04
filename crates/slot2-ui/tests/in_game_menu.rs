//! Contract for the in-game menu. Task 33 completes these tests without editing them.
//!
//! Unlike the power menu this one is an overlay: it is drawn on top of a running game frame,
//! so it must never clear.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_ui::in_game_menu::{BOX_H, BOX_W, DIM, INGAME_ITEMS, PAD, ROW_H};
use slot2_ui::{face, InGameChoice, InGameMenu, UiCtx, PX_TITLE};

/// The rows in draw order, with the dedicated message key each one must use.
const ORDER: [(InGameChoice, &str); 7] = [
    (InGameChoice::Continue, "ingame-continue"),
    (InGameChoice::SaveState, "ingame-save-state"),
    (InGameChoice::Cheats, "ingame-cheats"),
    (InGameChoice::Display, "ingame-display"),
    (InGameChoice::Core, "ingame-core"),
    (InGameChoice::Device, "ingame-device"),
    (InGameChoice::Eject, "ingame-eject"),
];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn asset(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/lang")
        .join(name);
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn rows_keep_their_order_and_dedicated_keys() {
    assert_eq!(INGAME_ITEMS.len(), ORDER.len());
    for (i, (choice, key)) in ORDER.iter().enumerate() {
        assert_eq!(INGAME_ITEMS[i], *choice);
        assert_eq!(choice.key(), *key);
    }
    // Dedicated means no two rows can render the same words.
    let mut keys: Vec<&str> = ORDER.iter().map(|(c, _)| c.key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), ORDER.len(), "{keys:?}");
}

#[test]
fn default_is_continue_and_navigation_wraps() {
    let mut m = InGameMenu::default();
    assert_eq!(m.choice(), InGameChoice::Continue);
    assert_eq!(m.selected_index(), 0);

    m.up(); // wraps backwards past the first row
    assert_eq!(m.choice(), InGameChoice::Eject);
    m.down();
    assert_eq!(m.choice(), InGameChoice::Continue);

    // Walking off the end comes back to the top; the index never leaves the array.
    for _ in 0..INGAME_ITEMS.len() {
        m.down();
        assert!(m.selected_index() < INGAME_ITEMS.len());
    }
    assert_eq!(m.choice(), InGameChoice::Continue);
}

#[test]
fn every_dedicated_key_has_english_and_korean_text() {
    for lang in ["en", "ko"] {
        let ctx = ctx("rgsp", lang);
        let title = ctx.i18n.t("ingame-menu-title");
        assert!(!title.starts_with('['), "{lang}: {title}");
        for (_, key) in ORDER {
            let text = ctx.i18n.t(key);
            assert!(!text.starts_with('['), "{lang}: missing {key}");
        }
    }
    // A ko lookup falls back to en, so check the ko pack itself defines the keys.
    let ko = asset("ko.ftl");
    for key in ["ingame-menu-title"]
        .into_iter()
        .chain(ORDER.iter().map(|(_, k)| *k))
    {
        let defined = ko
            .lines()
            .any(|l| l.trim_start().starts_with(&format!("{key} =")));
        assert!(defined, "ko.ftl does not define {key}");
    }
}

#[test]
fn overlays_the_frame_and_keeps_every_row_inside_the_safe_area() {
    for target in ["rgsp", "rg35xxsp", "rgcubexx"] {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (w, h) = ctx.profile.geometry.size();
            let mut c = RecordingCanvas::new(w, h);
            let mut m = InGameMenu::default();
            m.down();
            m.down(); // Cheats selected
            m.draw(&mut c, &mut ctx);
            let ops = &c.ops;

            // It is an overlay: nothing may clear the game frame underneath.
            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{target}/{lang}: cleared"
            );

            // The first draw is the whole-panel dim.
            match ops.iter().find(|o| matches!(o, Op::Rect { .. })).unwrap() {
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
                        "{target}/{lang}: dim covers the panel"
                    );
                    assert_eq!(*color, DIM, "{target}/{lang}");
                }
                _ => unreachable!(),
            }

            let (bx, by) = InGameMenu::box_origin(&ctx);
            assert!(
                ctx.safe.contains(bx, by, BOX_W, BOX_H),
                "{target}/{lang}: box in the safe area"
            );

            // One highlight, on the selected row only, and every row rect fits the safe area.
            let selected = m.selected_index();
            for i in 0..INGAME_ITEMS.len() {
                let row = InGameMenu::row_y(&ctx, i);
                let highlighted = ops.iter().any(|o| {
                    matches!(o, Op::Rect { x, y, w: rw, h: rh, .. }
                    if (*x - (bx + PAD)).abs() < 0.5
                        && (*y - row).abs() < 0.5
                        && (*rw - (BOX_W - 2.0 * PAD)).abs() < 0.5
                        && (*rh - ROW_H).abs() < 0.5)
                });
                assert_eq!(
                    highlighted,
                    i == selected,
                    "{target}/{lang}: highlight on row {i}"
                );
                assert!(
                    ctx.safe.contains(bx + PAD, row, BOX_W - 2.0 * PAD, ROW_H),
                    "{target}/{lang}: row {i} rect in the safe area"
                );
            }

            // Every row's label is centred in the box, in row order.
            let imgs: Vec<(f32, f32, f32, f32)> = ops
                .iter()
                .filter_map(|o| match o {
                    Op::Image { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
                    _ => None,
                })
                .collect();
            for (i, (choice, _)) in ORDER.iter().enumerate() {
                let spans = ctx.i18n.spans(choice.key(), &[]);
                let lw = face::spans_width(&mut ctx, &spans, PX_TITLE);
                let expect_x = bx + (BOX_W - lw) / 2.0;
                let row = InGameMenu::row_y(&ctx, i);
                assert!(
                    imgs.iter().any(|(x, y, w, _)| (x - expect_x).abs() < 0.5
                        && (w - lw).abs() < 0.5
                        && *y >= row
                        && *y < row + ROW_H),
                    "{target}/{lang}: label {i} centred in its row; got {imgs:?}"
                );
            }

            // Title, labels and hints live inside the box, which is itself inside the safe
            // area, so nothing may spill outside the 640x480 safe rectangle.
            for (x, y, iw, ih) in imgs {
                assert!(
                    x >= bx && x + iw <= bx + BOX_W && y >= by && y + ih <= by + BOX_H,
                    "{target}/{lang}: {x},{y},{iw},{ih} inside the box {bx},{by}"
                );
                assert!(
                    ctx.safe.contains(x, y, iw, ih),
                    "{target}/{lang}: image outside the safe area"
                );
            }
        }
    }
}
