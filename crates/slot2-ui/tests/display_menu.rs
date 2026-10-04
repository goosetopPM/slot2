//! Contract for the Display submenu: four scale rows, the row that opens the shader menu, the
//! row that opens the overlay menu, and — on a platform whose registry entry crops something —
//! the row that opens the overscan menu. One highlighted, no clearing.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_store::ScaleMode;
use slot2_ui::display_menu::{BOX_H, BOX_H_CROPPING, BOX_W, CROPPING_ROWS, DIM, PAD, ROWS, ROW_H};
use slot2_ui::splash::INK;
use slot2_ui::{face, DisplayChoice, DisplayMenu, UiCtx, PX_TITLE};

const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

/// The six rows of a platform with nothing to crop, in draw order, and the message each row
/// uses.
const ORDER: [(DisplayChoice, &str); 6] = [
    (DisplayChoice::Scale(None), "display-platform-default"),
    (
        DisplayChoice::Scale(Some(ScaleMode::Integer)),
        "display-integer",
    ),
    (
        DisplayChoice::Scale(Some(ScaleMode::AspectFit)),
        "display-aspect-fit",
    ),
    (DisplayChoice::Scale(Some(ScaleMode::Fill)), "display-fill"),
    // The submenu rows are ways into another screen, not settings, so each wears that screen's
    // own word rather than a second spelling of it.
    (DisplayChoice::Shader, "shader-title"),
    (DisplayChoice::Overlay, "overlay-title"),
];

/// The same six with the overscan row before the overlay one, for a platform that has something
/// to crop: the overlay row is last in both sets, because a row a platform gained does not push
/// the rows a player already knows.
const ORDER_CROPPING: [(DisplayChoice, &str); 7] = [
    ORDER[0],
    ORDER[1],
    ORDER[2],
    ORDER[3],
    ORDER[4],
    (DisplayChoice::Overscan, "overscan-title"),
    ORDER[5],
];

/// The four settings a game can be drawn with, which are what `new` can be handed.
const SCALES: [Option<ScaleMode>; 4] = [
    None,
    Some(ScaleMode::Integer),
    Some(ScaleMode::AspectFit),
    Some(ScaleMode::Fill),
];

/// A row set as a list: the choice each row draws, with the message it uses.
type Rows = [(DisplayChoice, &'static str)];
/// One menu mode: whether the platform crops, the rows it offers, their messages, and the height
/// of its box.
type Mode = (bool, &'static [DisplayChoice], &'static Rows, f32);

/// The two row sets a menu can offer, with the height each one's box has.
const MODES: [Mode; 2] = [
    (false, &ROWS, &ORDER, BOX_H),
    (true, &CROPPING_ROWS, &ORDER_CROPPING, BOX_H_CROPPING),
];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn frame(menu: &DisplayMenu, ctx: &mut UiCtx, target: &str) -> RecordingCanvas {
    let (w, h) = by_target(target).unwrap().geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    menu.draw(&mut c, ctx);
    c
}

fn highlight(ops: &[Op], row_y: f32, bx: f32) -> bool {
    ops.iter().any(|o| {
        matches!(o, Op::Rect { x, y, w, h, color }
        if (*x - (bx + PAD)).abs() < 0.5
            && (*y - row_y).abs() < 0.5
            && (*w - (BOX_W - 2.0 * PAD)).abs() < 0.5
            && (*h - ROW_H).abs() < 0.5
            && *color == INK.with_alpha(0.15))
    })
}

fn uploads(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count()
}

#[test]
fn each_scale_setting_selects_its_own_row_in_either_row_set() {
    for (crops, rows, order, _) in MODES {
        for scale in SCALES {
            let want = DisplayChoice::Scale(scale);
            let menu = DisplayMenu::new(scale, crops);
            assert_eq!(
                menu.choice(),
                want,
                "crops={crops}: {scale:?} did not select its own row"
            );
            assert_eq!(menu.choices(), rows, "crops={crops}: another row set");

            // And the frame says the same thing: the highlight is on that row, and only there.
            let i = rows.iter().position(|row| *row == want).expect("a row");
            let mut ctx = ctx("rgsp", "en");
            let c = frame(&menu, &mut ctx, "rgsp");
            let (bx, _) = menu.box_origin(&ctx);
            for j in 0..order.len() {
                let row = menu.row_y(&ctx, j);
                assert_eq!(
                    highlight(c.frame(), row, bx),
                    j == i,
                    "crops={crops}: {scale:?}: the highlight is on row {j}"
                );
            }
        }
    }

    // A scale row is never reached by walking: the menu opens on the game's own scale, and the
    // submenu rows are what sit past them.
    for (crops, ..) in MODES {
        assert_eq!(
            DisplayMenu::new(Some(ScaleMode::Fill), crops).choice(),
            DisplayChoice::Scale(Some(ScaleMode::Fill)),
            "crops={crops}: the last scale row is not where the last scale setting lands"
        );
    }
}

#[test]
fn navigation_follows_the_order_and_wraps_at_both_ends() {
    for (crops, _, order, _) in MODES {
        let mut menu = DisplayMenu::new(None, crops);
        for (want, _) in order.iter().skip(1) {
            menu.down();
            assert_eq!(menu.choice(), *want, "crops={crops}");
        }
        menu.down();
        assert_eq!(
            menu.choice(),
            order[0].0,
            "crops={crops}: down from the last row did not wrap"
        );

        menu.up();
        assert_eq!(
            menu.choice(),
            order[order.len() - 1].0,
            "crops={crops}: up from the first row did not wrap"
        );

        // One lap in either direction comes back to where it started.
        let start = menu.choice();
        for _ in 0..order.len() {
            menu.down();
        }
        assert_eq!(menu.choice(), start, "crops={crops}");
        for _ in 0..order.len() {
            menu.up();
        }
        assert_eq!(menu.choice(), start, "crops={crops}");
    }

    // The last row of each set is the one every platform has: the overlay screen. The overscan
    // screen sits one before it, and only where there is something to crop.
    assert_eq!(ROWS[ROWS.len() - 1], DisplayChoice::Overlay);
    assert_eq!(
        CROPPING_ROWS[CROPPING_ROWS.len() - 1],
        DisplayChoice::Overlay
    );
    assert_eq!(
        CROPPING_ROWS[CROPPING_ROWS.len() - 2],
        DisplayChoice::Overscan,
        "the overscan row is not the one before the overlay row"
    );
    for set in [&ROWS[..], &CROPPING_ROWS[..]] {
        assert_eq!(
            set.iter()
                .filter(|row| **row == DisplayChoice::Overlay)
                .count(),
            1,
            "the overlay row is not on exactly one line: {set:?}"
        );
    }
}

#[test]
fn drawing_dims_then_panels_and_highlights_exactly_one_row() {
    for (crops, _, order, box_h) in MODES {
        for target in TARGETS {
            for lang in ["en", "ko"] {
                let mut ctx = ctx(target, lang);
                let (pw, ph) = ctx.profile.geometry.size();
                let menu = DisplayMenu::new(Some(ScaleMode::Fill), crops);
                let mut c = RecordingCanvas::new(pw, ph);
                menu.draw(&mut c, &mut ctx);
                let ops = c.frame();
                let case = format!("crops={crops}/{target}/{lang}");

                assert!(
                    !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                    "{case}: the menu cleared the game frame"
                );

                // The first mark is the whole-panel dim, before the menu panel itself.
                match ops
                    .iter()
                    .find(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
                {
                    Some(Op::Rect { x, y, w, h, color }) => {
                        assert_eq!((*x, *y, *w, *h), (0.0, 0.0, pw as f32, ph as f32));
                        assert_eq!(*color, DIM, "{case}: the panel was not dimmed first");
                    }
                    other => panic!("{case}: the first thing drawn was not the dim: {other:?}"),
                }

                // One highlight, on the row that is selected.
                let (bx, by) = menu.box_origin(&ctx);
                let mut highlighted = 0;
                for i in 0..order.len() {
                    if highlight(ops, menu.row_y(&ctx, i), bx) {
                        highlighted += 1;
                        assert_eq!(i, 3, "{case}: the menu is on Fill, not on row {i}");
                    }
                }
                assert_eq!(highlighted, 1, "{case}: {highlighted} rows are highlighted");

                // The panel is where the layout says it is, and everything but the deliberate
                // full-panel dim stays in the safe area: title, rows, highlight and hints.
                assert_eq!((bx, by), menu.box_origin(&ctx));
                assert_eq!(menu.box_h(), box_h, "{case}: another box height");
                assert_eq!(BOX_H, 316.0, "the six-row box moved");
                assert_eq!(
                    BOX_H_CROPPING, 352.0,
                    "the seven-row box is not one row taller"
                );
                assert!(
                    ctx.safe.contains(bx, by, BOX_W, box_h),
                    "{case}: the box left the safe area"
                );
                // And the box really is the height the menu reports, not the smaller one.
                assert!(
                    ops.iter()
                        .any(|o| matches!(o, Op::Rect { x, y, w, h, color }
                        if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5
                            && (*w - BOX_W).abs() < 0.5 && (*h - box_h).abs() < 0.5
                            && *color == slot2_ui::splash::BACKDROP)),
                    "{case}: the panel is not the height the menu says"
                );
                for op in ops {
                    let (x, y, w, h) = match op {
                        Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => {
                            (*x, *y, *w, *h)
                        }
                        _ => continue,
                    };
                    if (x, y, w, h) == (0.0, 0.0, pw as f32, ph as f32) {
                        continue;
                    }
                    assert!(ctx.safe.contains(x, y, w, h), "{case}: {op:?} left it");
                }
            }
        }
    }
}

#[test]
fn each_row_says_its_own_words_in_its_own_place() {
    for (crops, _, order, _) in MODES {
        for target in TARGETS {
            for lang in ["en", "ko"] {
                let mut ctx = ctx(target, lang);
                let menu = DisplayMenu::new(None, crops);
                let c = frame(&menu, &mut ctx, target);
                let (bx, _) = menu.box_origin(&ctx);
                let imgs: Vec<(f32, f32, f32)> = c
                    .frame()
                    .iter()
                    .filter_map(|o| match o {
                        Op::Image { x, y, w, .. } => Some((*x, *y, *w)),
                        _ => None,
                    })
                    .collect();
                let case = format!("crops={crops}/{target}/{lang}");

                for (i, (choice, key)) in order.iter().enumerate() {
                    let spans = ctx.i18n.spans(key, &[]);
                    let want = face::spans_width(&mut ctx, &spans, PX_TITLE);
                    let row = menu.row_y(&ctx, i);
                    assert!(
                        imgs.iter()
                            .any(|(x, y, w)| (*x - (bx + (BOX_W - want) / 2.0)).abs() < 0.5
                                && (*w - want).abs() < 0.5
                                && *y >= row
                                && *y < row + ROW_H),
                        "{case}: row {i} ({key}) is not on its own line; got {imgs:?}"
                    );
                    assert!(!ctx.i18n.t(key).is_empty(), "{case}: row {i} has no words");
                    assert_eq!(DisplayMenu::key(*choice), *key, "{case}: row {i}");
                }
            }
        }
    }
}

#[test]
fn a_warm_menu_uploads_nothing() {
    for (crops, _, order, _) in MODES {
        // Korean, so the CJK face is part of what has to settle and not only the Latin one.
        let mut ctx = ctx("rgsp", "ko");
        let (pw, ph) = ctx.profile.geometry.size();
        let mut c = RecordingCanvas::new(pw, ph);
        let mut menu = DisplayMenu::new(Some(ScaleMode::AspectFit), crops);

        menu.draw(&mut c, &mut ctx);
        assert!(
            uploads(&c.ops) > 0,
            "crops={crops}: the first frame uploaded nothing at all"
        );

        let before = c.ops.len();
        for _ in 0..8 {
            menu.draw(&mut c, &mut ctx);
        }
        assert_eq!(
            uploads(&c.ops[before..]),
            0,
            "crops={crops}: eight unchanged frames uploaded something"
        );

        // Moving the highlight is not new text: no row label changes with the selection, the
        // submenu rows included.
        let before = c.ops.len();
        for _ in 0..order.len() {
            menu.down();
            menu.draw(&mut c, &mut ctx);
        }
        assert_eq!(
            uploads(&c.ops[before..]),
            0,
            "crops={crops}: walking the rows uploaded something"
        );
    }
}

#[test]
fn the_rows_are_the_four_settings_and_the_submenus_in_order() {
    assert_eq!(ROWS.len(), ORDER.len());
    assert_eq!(CROPPING_ROWS.len(), ORDER_CROPPING.len());
    for (i, (want, key)) in ORDER.iter().enumerate() {
        assert_eq!(ROWS[i], *want);
        assert_eq!(DisplayMenu::key(*want), *key);
    }
    for (i, (want, key)) in ORDER_CROPPING.iter().enumerate() {
        assert_eq!(CROPPING_ROWS[i], *want);
        assert_eq!(DisplayMenu::key(*want), *key);
    }
    // Each row has words of its own: a shared key would put the same label on two rows.
    let mut keys: Vec<&str> = ORDER_CROPPING.iter().map(|(_, k)| *k).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), ORDER_CROPPING.len(), "{keys:?}");

    // The row set a cropping platform gets is the other one plus one row in the middle: the
    // rows a player already knows do not move because a platform gained a feature, so everything
    // before that row is shared and the overlay row stays last in both.
    assert_eq!(&CROPPING_ROWS[..ROWS.len() - 1], &ROWS[..ROWS.len() - 1]);
    assert_eq!(
        CROPPING_ROWS[ROWS.len() - 1],
        DisplayChoice::Overscan,
        "the cropping set is not the other one with the overscan row before the last"
    );

    // And nothing else: four scale settings, the shader, overscan and overlay rows.
    for row in CROPPING_ROWS {
        assert!(
            matches!(row, DisplayChoice::Scale(_))
                || matches!(
                    row,
                    DisplayChoice::Shader | DisplayChoice::Overscan | DisplayChoice::Overlay
                ),
            "{row:?} is not a row"
        );
        assert!(
            !CROPPING_ROWS[..CROPPING_ROWS.iter().position(|r| *r == row).unwrap()].contains(&row),
            "{row:?} is on two rows"
        );
    }

    // A menu built from any scale lands on a scale row, never on a submenu row.
    for (crops, ..) in MODES {
        for scale in SCALES {
            let choice = DisplayMenu::new(scale, crops).choice();
            assert!(matches!(choice, DisplayChoice::Scale(_)), "{choice:?}");
        }
    }
}

#[test]
fn the_overscan_row_is_offered_only_where_the_platform_crops() {
    // A platform with nothing to crop: the row is not in the set, so no amount of walking reaches
    // it and nothing draws it. The six-row set is the five-row set plus that one row.
    let mut menu = DisplayMenu::new(None, false);
    assert!(!menu.choices().contains(&DisplayChoice::Overscan));
    for _ in 0..ROWS.len() * 2 {
        menu.down();
        assert_ne!(menu.choice(), DisplayChoice::Overscan);
    }
    for _ in 0..ROWS.len() * 2 {
        menu.up();
        assert_ne!(menu.choice(), DisplayChoice::Overscan);
    }

    let mut menu = DisplayMenu::new(None, true);
    assert_eq!(
        menu.choices().len(),
        ROWS.len() + 1,
        "the cropping set is not the other one plus a row"
    );
    let mut seen = false;
    for _ in 0..CROPPING_ROWS.len() {
        if menu.choice() == DisplayChoice::Overscan {
            seen = true;
        }
        menu.down();
    }
    assert!(seen, "the overscan row cannot be reached by walking");

    // A frame drawn by the five-row menu has five labels in its row band, not six: the row a
    // cropping platform adds is not drawn by a platform that has nothing to put on it.
    for target in TARGETS {
        let mut ctx = ctx(target, "en");
        for (crops, _, order, _) in MODES {
            let menu = DisplayMenu::new(None, crops);
            let c = frame(&menu, &mut ctx, target);
            let top = menu.row_y(&ctx, 0);
            let bottom = top + order.len() as f32 * ROW_H;
            let labels = c
                .frame()
                .iter()
                .filter(|o| matches!(o, Op::Image { y, .. } if *y >= top && *y < bottom))
                .count();
            assert_eq!(
                labels,
                order.len(),
                "{target}: crops={crops}: {labels} labels for {} rows",
                order.len()
            );
        }
    }
}
