//! Contract for the overlay submenu: three rows — the platform's own default, an explicit on
//! and an explicit off — one highlighted, no clearing.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_ui::overlay_menu::{BOX_H, BOX_W, DIM, PAD, ROWS, ROW_H};
use slot2_ui::splash::INK;
use slot2_ui::{face, OverlayMenu, UiCtx, PX_BODY, PX_HINT, PX_TITLE};

const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

/// The three choices, in draw order, and the message each row uses. Written out rather than
/// read off `ROWS`, so a row that moved or changed meaning fails here.
const ORDER: [(Option<bool>, &str); 3] = [
    (None, "display-platform-default"),
    (Some(true), "overlay-on"),
    (Some(false), "overlay-off"),
];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn frame(menu: &OverlayMenu, ctx: &mut UiCtx, target: &str) -> RecordingCanvas {
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
fn each_setting_selects_its_own_row() {
    for (i, (want, key)) in ORDER.iter().enumerate() {
        let menu = OverlayMenu::new(*want);
        assert_eq!(menu.selected(), *want, "{key} did not select its own row");

        // And the frame says the same thing: the highlight is on that row, and only there.
        let mut ctx = ctx("rgsp", "en");
        let c = frame(&menu, &mut ctx, "rgsp");
        let (bx, _) = OverlayMenu::box_origin(&ctx);
        for j in 0..ORDER.len() {
            let row = OverlayMenu::row_y(&ctx, j);
            assert_eq!(
                highlight(c.frame(), row, bx),
                j == i,
                "{key}: the highlight is on row {j}"
            );
        }
    }

    // The platform default and an explicit off are different settings even while both draw no
    // overlay today: `None` follows a future platform default, `Some(false)` is a decision this
    // game made and must not start drawing one when the default changes. The constructor must
    // not land them on one row.
    assert_ne!(
        OverlayMenu::new(None).selected(),
        OverlayMenu::new(Some(false)).selected(),
        "the platform default and an explicit off are one row"
    );
    assert_ne!(
        OverlayMenu::new(None).selected(),
        OverlayMenu::new(Some(true)).selected(),
        "the platform default and an explicit on are one row"
    );
}

#[test]
fn navigation_follows_the_order_and_wraps_at_both_ends() {
    let mut menu = OverlayMenu::new(None);
    for (want, _) in ORDER.iter().skip(1) {
        menu.down();
        assert_eq!(menu.selected(), *want);
    }
    menu.down();
    assert_eq!(
        menu.selected(),
        ORDER[0].0,
        "down from the last row did not wrap"
    );

    menu.up();
    assert_eq!(
        menu.selected(),
        ORDER[ORDER.len() - 1].0,
        "up from the first row did not wrap"
    );

    // One lap in either direction comes back to where it started.
    let start = menu.selected();
    for _ in 0..ORDER.len() {
        menu.down();
    }
    assert_eq!(menu.selected(), start);
    for _ in 0..ORDER.len() {
        menu.up();
    }
    assert_eq!(menu.selected(), start);
}

#[test]
fn drawing_dims_then_panels_and_highlights_exactly_one_row() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            let menu = OverlayMenu::new(Some(false));
            let mut c = RecordingCanvas::new(pw, ph);
            menu.draw(&mut c, &mut ctx);
            let ops = c.frame();
            let case = format!("{target}/{lang}");

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
            let (bx, by) = OverlayMenu::box_origin(&ctx);
            let mut highlighted = 0;
            for i in 0..ORDER.len() {
                if highlight(ops, OverlayMenu::row_y(&ctx, i), bx) {
                    highlighted += 1;
                    assert_eq!(i, 2, "{case}: the menu is on Off, not on row {i}");
                }
            }
            assert_eq!(highlighted, 1, "{case}: {highlighted} rows are highlighted");

            // The panel is where the layout says it is, and everything but the deliberate
            // full-panel dim stays in the safe area: title, rows, highlight and hints.
            assert_eq!((bx, by), OverlayMenu::box_origin(&ctx));
            assert!(
                ctx.safe.contains(bx, by, BOX_W, BOX_H),
                "{case}: the box left the safe area"
            );
            let hint_y = by + BOX_H - PAD - PX_HINT;
            let mut hint = false;
            for op in ops {
                let (x, y, w, h) = match op {
                    Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                    _ => continue,
                };
                if (x, y, w, h) == (0.0, 0.0, pw as f32, ph as f32) {
                    continue;
                }
                assert!(ctx.safe.contains(x, y, w, h), "{case}: {op:?} left it");
                hint |= (y - hint_y).abs() < 0.5;
            }
            assert!(
                hint,
                "{case}: nothing was drawn on the hint line at {hint_y}"
            );

            // And the title sits at the top of the box, centred in it.
            let title = ctx.i18n.t("overlay-title");
            let want = face::measure(&mut ctx, &title, PX_BODY);
            assert!(
                ops.iter().any(|o| matches!(o, Op::Image { x, y, w, .. }
                    if (*x - (bx + (BOX_W - want) / 2.0)).abs() < 0.5
                        && (*w - want).abs() < 0.5
                        && (*y - (by + PAD)).abs() < 0.5)),
                "{case}: the title is not at the top of the box"
            );
        }
    }
}

#[test]
fn each_row_says_its_own_words_in_its_own_place() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let c = frame(&OverlayMenu::new(None), &mut ctx, target);
            let (bx, _) = OverlayMenu::box_origin(&ctx);
            let imgs: Vec<(f32, f32, f32)> = c
                .frame()
                .iter()
                .filter_map(|o| match o {
                    Op::Image { x, y, w, .. } => Some((*x, *y, *w)),
                    _ => None,
                })
                .collect();
            let case = format!("{target}/{lang}");

            for (i, (_, key)) in ORDER.iter().enumerate() {
                let spans = ctx.i18n.spans(key, &[]);
                let want = face::spans_width(&mut ctx, &spans, PX_TITLE);
                let row = OverlayMenu::row_y(&ctx, i);
                assert!(
                    imgs.iter()
                        .any(|(x, y, w)| (*x - (bx + (BOX_W - want) / 2.0)).abs() < 0.5
                            && (*w - want).abs() < 0.5
                            && *y >= row
                            && *y < row + ROW_H),
                    "{case}: row {i} ({key}) is not on its own line; got {imgs:?}"
                );
            }
        }
    }
}

#[test]
fn a_warm_menu_uploads_nothing() {
    // Korean, so the CJK face is part of what has to settle and not only the Latin one.
    let mut ctx = ctx("rgsp", "ko");
    let (pw, ph) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(pw, ph);
    let mut menu = OverlayMenu::new(Some(true));

    menu.draw(&mut c, &mut ctx);
    assert!(
        uploads(&c.ops) > 0,
        "the first frame uploaded nothing at all"
    );

    let before = c.ops.len();
    for _ in 0..8 {
        menu.draw(&mut c, &mut ctx);
    }
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "eight unchanged frames uploaded something"
    );

    // Moving the highlight is not new text: no row label changes with the selection.
    let before = c.ops.len();
    menu.down();
    menu.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "moving the highlight uploaded something"
    );
}

#[test]
fn the_rows_are_the_three_settings_in_order() {
    assert_eq!(ROWS.len(), ORDER.len());
    for (i, (want, key)) in ORDER.iter().enumerate() {
        assert_eq!(ROWS[i], *want);
        assert_eq!(OverlayMenu::key(*want), *key);
    }
    // Each row has words of its own: a shared key would put the same label on two rows.
    let mut keys: Vec<&str> = ORDER.iter().map(|(_, k)| *k).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), ORDER.len(), "{keys:?}");

    // Three rows, three different settings: a repeat would be a row the player can never reach.
    for (i, row) in ROWS.iter().enumerate() {
        assert!(
            !ROWS[..i].contains(row),
            "row {i} repeats {}",
            OverlayMenu::key(*row)
        );
    }
}

#[test]
fn both_packs_define_the_new_words_themselves() {
    // Exact strings on purpose: a key the ko pack lacks falls back to English, and a menu that
    // mixes the two reads as a half-finished translation rather than as a missing one.
    let cases = [
        ("overlay-title", "Overlay", "오버레이"),
        ("overlay-on", "On", "켜기"),
        ("overlay-off", "Off", "끄기"),
    ];
    let en = ctx("rgsp", "en");
    let ko = ctx("rgsp", "ko");
    for (key, english, korean) in cases {
        assert_eq!(en.i18n.t(key), english, "{key} in en");
        assert_eq!(ko.i18n.t(key), korean, "{key} in ko");
    }

    // The row the three menus share is the same word in both.
    assert_eq!(
        en.i18n.t("display-platform-default"),
        en.i18n.t(OverlayMenu::key(None))
    );
    assert_eq!(
        ko.i18n.t("display-platform-default"),
        ko.i18n.t(OverlayMenu::key(None))
    );
}
