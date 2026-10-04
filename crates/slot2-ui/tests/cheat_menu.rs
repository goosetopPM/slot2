//! Contract for the cheats submenu: a window over the session's list, two words for the state
//! of a row, and words for the empty list. No core is needed — the list is handed in at draw
//! time, which is the whole point of the menu not owning it.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_store::Cheat;
use slot2_ui::cheat_menu::{
    fit, CheatMenu, BOX_H, BOX_W, DIM, MAX_ROWS, MORE_H, MORE_W, PAD, ROW_H, STATE_GAP,
};
use slot2_ui::splash::{INK, INK_DIM};
use slot2_ui::{face, UiCtx, PX_BODY, PX_HINT, SAFE_H, SAFE_W};

/// The three panel shapes the UI is laid out for: 720x480, 640x480 and 720x720.
const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

const KEY_ENABLED: &str = "cheat-enabled";
const KEY_DISABLED: &str = "cheat-disabled";
const KEY_EMPTY: &str = "cheat-empty";
const KEY_TOGGLE_HINT: &str = "hint-cheat-toggle";
const KEY_BACK_HINT: &str = "hint-back";

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn cheat(description: &str, enabled: bool) -> Cheat {
    Cheat {
        description: description.into(),
        code: "7E007C9A".into(),
        enabled,
    }
}

/// A list of `n` cheats, every other one on.
fn cheats(n: usize) -> Vec<Cheat> {
    (0..n)
        .map(|i| cheat(&format!("Cheat {i}"), i % 2 == 0))
        .collect()
}

fn draw(menu: &CheatMenu, ctx: &mut UiCtx, target: &str, list: &[Cheat]) -> RecordingCanvas {
    let (w, h) = by_target(target).unwrap().geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    menu.draw(&mut c, ctx, list);
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

/// Whether the "rows are hidden this way" bar is drawn at `y`.
fn bar(ops: &[Op], y: f32, bx: f32) -> bool {
    ops.iter().any(|o| {
        matches!(o, Op::Rect { x, y: oy, w, h, color }
        if (*x - (bx + (BOX_W - MORE_W) / 2.0)).abs() < 0.5
            && (*oy - y).abs() < 0.5
            && (*w - MORE_W).abs() < 0.5
            && (*h - MORE_H).abs() < 0.5
            && *color == INK_DIM)
    })
}

/// The leftmost and rightmost x of anything drawn on the hint line.
fn hint_extent(ops: &[Op], y: f32) -> Option<(f32, f32)> {
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    for op in ops {
        let (x, oy, w, _) = match op {
            Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
            _ => continue,
        };
        // By the mark's top: the full-panel dim starts at 0 and covers the panel, and it is
        // not on the hint line.
        if !(y - 6.0..=y + PX_HINT + 2.0).contains(&oy) {
            continue;
        }
        lo = lo.min(x);
        hi = hi.max(x + w);
    }
    (lo <= hi).then_some((lo, hi))
}

/// How many text faces are drawn inside the row area.
fn row_faces(ops: &[Op], ctx: &UiCtx) -> usize {
    let top = CheatMenu::row_y(ctx, 0);
    let bottom = CheatMenu::row_y(ctx, MAX_ROWS - 1) + ROW_H;
    ops.iter()
        .filter(|o| matches!(o, Op::Image { y, .. } if *y >= top && *y < bottom))
        .count()
}

#[test]
fn the_row_window_is_five_to_seven_rows_and_the_box_fits_a_panel() {
    assert!(
        (5..=7).contains(&MAX_ROWS),
        "MAX_ROWS = {MAX_ROWS} is not the 5..7 the safe area holds"
    );
    assert!(
        BOX_W <= SAFE_W as f32,
        "the box is wider than the safe area"
    );
    assert!(
        BOX_H + 2.0 * PAD <= SAFE_H as f32,
        "the box leaves no room in the safe area"
    );
}

#[test]
fn an_empty_list_has_nothing_to_highlight_and_only_the_way_out() {
    let mut menu = CheatMenu::new(0);
    assert_eq!(menu.selected_index(), None);
    let before = menu;
    menu.up();
    menu.down();
    assert_eq!(menu, before, "an empty menu moved");
    assert!(!menu.more_above(0) && !menu.more_below(0));

    let mut ctx = ctx("rgsp", "en");
    let bx = CheatMenu::box_origin(&ctx).0;
    let c = draw(&menu, &mut ctx, "rgsp", &[]);
    let ops = c.frame();
    assert!(
        !ops.iter().any(|o| matches!(o, Op::Clear(_))),
        "the empty list cleared the game frame"
    );
    assert!(
        !ops.iter()
            .any(|o| matches!(o, Op::Rect { color, .. } if *color == INK.with_alpha(0.15))),
        "the empty list highlighted a row"
    );
    let above = CheatMenu::more_above_y(&mut ctx);
    let below = CheatMenu::more_below_y(&ctx);
    assert!(
        !bar(ops, above, bx) && !bar(ops, below, bx),
        "an empty list pointed at hidden rows"
    );

    // The empty words, centred in the middle of the row area.
    let spans = ctx.i18n.spans(KEY_EMPTY, &[]);
    let w = face::spans_width(&mut ctx, &spans, PX_BODY);
    let y = CheatMenu::empty_y(&mut ctx);
    assert!(
        ops.iter().any(|o| matches!(o, Op::Image { x, y: oy, .. }
            if (*x - (bx + (BOX_W - w) / 2.0)).abs() < 0.5 && (*oy - y).abs() < 0.5)),
        "the empty line is not centred where the layout says: {ops:?}"
    );

    // Only the way out: the toggle hint is not on the line at all.
    let back = ctx.i18n.spans(KEY_BACK_HINT, &[]);
    let w_back = face::spans_width(&mut ctx, &back, PX_HINT);
    let x_back = bx + (BOX_W - w_back) / 2.0;
    let (lo, hi) = hint_extent(ops, CheatMenu::hint_y(&ctx)).expect("no hint at all");
    assert!(
        (hi - (x_back + w_back)).abs() < 0.5,
        "the hint line is not the back hint alone: {lo}..{hi}"
    );
    assert!(
        lo >= x_back - 0.5,
        "something was drawn left of the back hint: {lo}"
    );
}

#[test]
fn a_list_with_rows_offers_the_toggle_and_the_way_out() {
    let mut ctx = ctx("rgsp", "en");
    let list = vec![cheat("only", true)];
    let bx = CheatMenu::box_origin(&ctx).0;
    let c = draw(&CheatMenu::new(list.len()), &mut ctx, "rgsp", &list);

    let toggle = ctx.i18n.spans(KEY_TOGGLE_HINT, &[]);
    let w_toggle = face::spans_width(&mut ctx, &toggle, PX_HINT);
    let back = ctx.i18n.spans(KEY_BACK_HINT, &[]);
    let w_back = face::spans_width(&mut ctx, &back, PX_HINT);
    let total = w_toggle + PX_HINT + w_back;

    let (lo, hi) = hint_extent(c.frame(), CheatMenu::hint_y(&ctx)).expect("no hint at all");
    assert!(
        (hi - (bx + (BOX_W - total) / 2.0 + total)).abs() < 0.5,
        "the hint line does not end where two hints end: {lo}..{hi}"
    );
    assert!(
        lo < bx + (BOX_W - w_back) / 2.0,
        "the toggle hint is not on the line: {lo}"
    );
}

#[test]
fn an_empty_list_with_a_long_list_behind_it_still_draws_the_empty_view() {
    // The menu was opened for three cheats and the caller hands over none: the row area has
    // the empty words in it and nothing is read past the end of the slice.
    let mut ctx = ctx("rgsp", "en");
    let menu = CheatMenu::new(3);
    let c = draw(&menu, &mut ctx, "rgsp", &[]);
    assert_eq!(row_faces(c.frame(), &ctx), 1, "the empty view drew rows");

    let short = vec![cheat("only", true)];
    let c = draw(&menu, &mut ctx, "rgsp", &short);
    assert_eq!(
        row_faces(c.frame(), &ctx),
        2,
        "one entry did not draw one row"
    );

    let long = cheats(9);
    let c = draw(&menu, &mut ctx, "rgsp", &long);
    assert_eq!(
        row_faces(c.frame(), &ctx),
        2 * 3,
        "a longer list than the menu was opened for drew rows the menu never had"
    );
}

#[test]
fn a_list_of_one_stays_on_its_only_row() {
    let mut menu = CheatMenu::new(1);
    assert_eq!(menu.selected_index(), Some(0));
    menu.up();
    assert_eq!(menu.selected_index(), Some(0), "up left the only row");
    menu.down();
    assert_eq!(menu.selected_index(), Some(0));
    assert!(!menu.more_above(1) && !menu.more_below(1));
}

#[test]
fn navigation_wraps_and_keeps_the_highlight_inside_the_window() {
    let len = MAX_ROWS + 3;
    let mut menu = CheatMenu::new(len);
    assert_eq!(menu.selected_index(), Some(0));
    assert_eq!(menu.first_visible(), 0);
    assert!(!menu.more_above(len), "the top of the list hid rows above");
    assert!(menu.more_below(len), "a longer list hid nothing below");

    for step in 1..len {
        menu.down();
        let selected = menu.selected_index().unwrap();
        assert_eq!(selected, step);
        let first = menu.first_visible();
        assert!(
            first <= selected && selected < first + MAX_ROWS,
            "row {selected} is off screen (first {first})"
        );
        assert_eq!(
            menu.more_above(len),
            first > 0,
            "the top bar at row {selected}"
        );
        assert_eq!(
            menu.more_below(len),
            first + MAX_ROWS < len,
            "the bottom bar at row {selected}"
        );
    }

    // Wrapping down is the top of the list again, window and all.
    menu.down();
    assert_eq!(menu.selected_index(), Some(0));
    assert_eq!(menu.first_visible(), 0);

    // And up wraps to the last row, with the window following it there.
    menu.up();
    assert_eq!(menu.selected_index(), Some(len - 1));
    assert_eq!(menu.first_visible(), len - MAX_ROWS);
    assert!(menu.more_above(len) && !menu.more_below(len));
}

#[test]
fn the_window_bars_point_only_at_the_rows_that_are_hidden() {
    let len = MAX_ROWS + 2;
    let list = cheats(len);
    let mut ctx = ctx("rgsp", "en");
    let bx = CheatMenu::box_origin(&ctx).0;
    let above = CheatMenu::more_above_y(&mut ctx);
    let below = CheatMenu::more_below_y(&ctx);
    let mut menu = CheatMenu::new(len);

    let c = draw(&menu, &mut ctx, "rgsp", &list);
    assert!(
        bar(c.frame(), below, bx),
        "the top of a long list said nothing about the rows below"
    );
    assert!(
        !bar(c.frame(), above, bx),
        "the top of the list pointed above itself"
    );

    for _ in 0..len - 1 {
        menu.down();
    }
    assert_eq!(menu.selected_index(), Some(len - 1));
    let c = draw(&menu, &mut ctx, "rgsp", &list);
    assert!(
        bar(c.frame(), above, bx),
        "the bottom of the list said nothing about the rows above"
    );
    assert!(
        !bar(c.frame(), below, bx),
        "the bottom of the list pointed below itself"
    );
}

#[test]
fn the_upper_bar_sits_between_the_title_line_and_the_first_row() {
    // The upper bar used to be placed "a little above row 0", which is inside the title's own
    // line box: the bar was drawn across the words. It belongs in the empty band under the
    // title, and that band has to be real space for every language and every panel.
    let list = cheats(MAX_ROWS + 2);
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let case = format!("{target}/{lang}");
            let bx = CheatMenu::box_origin(&ctx).0;

            // Scrolled to the bottom, so the upper bar is the one on screen.
            let mut menu = CheatMenu::new(list.len());
            for _ in 0..list.len() - 1 {
                menu.down();
            }
            assert!(menu.more_above(list.len()));
            let c = draw(&menu, &mut ctx, target, &list);

            let title_bottom = CheatMenu::title_bottom(&mut ctx);
            let above = CheatMenu::more_above_y(&mut ctx);
            let below = CheatMenu::more_below_y(&ctx);
            let first_row = CheatMenu::row_y(&ctx, 0);
            let hint = CheatMenu::hint_y(&ctx);

            assert!(
                first_row - title_bottom > MORE_H,
                "{case}: the title line and the first row leave no band for the bar"
            );
            assert!(
                title_bottom < above,
                "{case}: the upper bar starts inside the title's line box ({title_bottom} vs {above})"
            );
            assert!(
                above + MORE_H < first_row,
                "{case}: the upper bar reaches the first row ({above} + {MORE_H} vs {first_row})"
            );
            assert!(
                bar(c.frame(), above, bx),
                "{case}: the upper bar is not drawn where the layout says it is"
            );

            // And the lower bar keeps its own clearances: rows above it, hints below it.
            assert!(
                below > CheatMenu::row_y(&ctx, MAX_ROWS - 1) + ROW_H,
                "{case}: the lower bar is inside the last row"
            );
            assert!(
                below + MORE_H < hint,
                "{case}: the lower bar reaches the hints ({below} + {MORE_H} vs {hint})"
            );
        }
    }
}

#[test]
fn the_selected_row_is_the_only_one_highlighted() {
    let list = cheats(3);
    for selected in 0..3 {
        let mut menu = CheatMenu::new(list.len());
        for _ in 0..selected {
            menu.down();
        }
        let mut ctx = ctx("rgsp", "en");
        let bx = CheatMenu::box_origin(&ctx).0;
        let c = draw(&menu, &mut ctx, "rgsp", &list);
        let mut highlighted = 0;
        for row in 0..list.len() {
            if highlight(c.frame(), CheatMenu::row_y(&ctx, row), bx) {
                highlighted += 1;
                assert_eq!(row, selected, "the highlight is on row {row}");
            }
        }
        assert_eq!(highlighted, 1, "{highlighted} rows are highlighted");
    }
}

#[test]
fn each_row_shows_its_description_on_the_left_and_its_state_on_the_right() {
    let list = vec![cheat("Infinite lives", true), cheat("Max hearts", false)];
    let rows = [
        ("Infinite lives", KEY_ENABLED),
        ("Max hearts", KEY_DISABLED),
    ];

    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let case = format!("{target}/{lang}");
            let c = draw(&CheatMenu::new(list.len()), &mut ctx, target, &list);
            let ops = c.frame();

            for (row, (description, key)) in rows.iter().enumerate() {
                let row_y = CheatMenu::row_y(&ctx, row);
                let desc_w = face::measure(&mut ctx, description, PX_BODY);
                assert!(
                    ops.iter().any(|o| matches!(o, Op::Image { x, y, w, .. }
                        if (*x - CheatMenu::row_text_x(&ctx)).abs() < 0.5
                            && *y >= row_y && *y < row_y + ROW_H
                            && (*w - desc_w).abs() < 0.5)),
                    "{case}: row {row} has no description on the left"
                );

                let spans = ctx.i18n.spans(key, &[]);
                let state_w = face::spans_width(&mut ctx, &spans, PX_BODY);
                let state_x = CheatMenu::state_x(&ctx, state_w);
                assert!(
                    ops.iter().any(|o| matches!(o, Op::Image { x, y, w, .. }
                        if (*x - state_x).abs() < 0.5
                            && *y >= row_y && *y < row_y + ROW_H
                            && (*w - state_w).abs() < 0.5)),
                    "{case}: row {row} ({key}) has no state word on the right"
                );
                assert!(
                    state_x + state_w <= CheatMenu::box_origin(&ctx).0 + BOX_W
                        && state_x > CheatMenu::row_text_x(&ctx),
                    "{case}: the state column is not inside the row"
                );
            }
        }
    }
}

#[test]
fn a_long_description_is_cut_on_a_character_boundary() {
    let mut ctx = ctx("rgsp", "ko");
    let max_w = 180.0;

    // Text that fits is handed back untouched.
    assert_eq!(
        fit(&mut ctx, "Infinite lives", PX_BODY, max_w),
        "Infinite lives"
    );
    assert_eq!(fit(&mut ctx, "치트", PX_BODY, max_w), "치트");

    let ascii = "A".repeat(400);
    let hangul = "이름이 아주 긴 치트".repeat(40);
    for text in [ascii.as_str(), hangul.as_str()] {
        let cut = fit(&mut ctx, text, PX_BODY, max_w);
        assert!(cut != text, "a 400 character description was not cut");
        assert!(cut.ends_with('…'), "{cut:?} does not end with an ellipsis");
        assert!(
            text.starts_with(cut.trim_end_matches('…')),
            "the cut is not a prefix of the text: {cut:?}"
        );
        assert!(
            face::measure(&mut ctx, &cut, PX_BODY) <= max_w + 0.5,
            "the cut is still wider than the room it was given: {cut:?}"
        );
        assert!(!cut.trim_end_matches('…').is_empty(), "nothing was kept");
    }
}

#[test]
fn a_very_long_description_is_cut_without_measuring_every_prefix() {
    // Tens of thousands of characters: the length a database line can hold, and the length
    // that made the old prefix-by-prefix search quadratic. The search is a bisection over
    // character boundaries now, so what this pins down is the result — the longest prefix
    // that fits, and an ellipsis after it.
    let max_w = 180.0;
    let ell_w = face::measure(&mut ctx("rgsp", "ko"), "…", PX_BODY);
    for (label, text) in [
        ("ascii", "A".repeat(50_000)),
        ("hangul", "치트".repeat(20_000)),
    ] {
        let mut ctx = ctx("rgsp", "ko");
        let cut = fit(&mut ctx, &text, PX_BODY, max_w);
        assert!(cut.ends_with('…'), "{label}: no ellipsis at all");
        let kept = cut.trim_end_matches('…');
        assert!(!kept.is_empty(), "{label}: nothing was kept");
        assert!(
            text.starts_with(kept),
            "{label}: the cut is not a prefix of the text"
        );
        assert!(
            face::measure(&mut ctx, kept, PX_BODY) + ell_w <= max_w + 0.5,
            "{label}: the cut does not fit the room it was given"
        );
        // One character more would not have fitted: the longest prefix, not merely a prefix
        // that happens to be inside the room.
        let next = text[kept.len()..]
            .chars()
            .next()
            .expect("the text does not end where the cut does");
        let wider = format!("{kept}{next}…");
        assert!(
            face::measure(&mut ctx, &wider, PX_BODY) > max_w,
            "{label}: a longer prefix would have fitted"
        );
    }
}

#[test]
fn a_room_narrower_than_the_ellipsis_gives_nothing_back() {
    let mut ctx = ctx("rgsp", "en");
    let ell_w = face::measure(&mut ctx, "…", PX_BODY);
    let one = face::measure(&mut ctx, "I", PX_BODY);

    for max_w in [0.0, 1.0, ell_w - 1.0] {
        let cut = fit(&mut ctx, "Infinite lives", PX_BODY, max_w);
        assert!(
            cut.is_empty(),
            "a {max_w} px room produced {cut:?} rather than nothing"
        );
        assert!(
            face::measure(&mut ctx, &cut, PX_BODY) <= max_w + 0.5,
            "the empty answer is wider than the {max_w} px it was given"
        );
    }

    // A room that holds the ellipsis and one character shows exactly those, and no more.
    let cut = fit(&mut ctx, "Infinite lives", PX_BODY, ell_w + one);
    assert_eq!(cut, "I…");
    assert!(face::measure(&mut ctx, &cut, PX_BODY) <= ell_w + one + 0.5);
}

#[test]
fn a_long_description_stays_on_its_side_of_the_row() {
    for lang in ["en", "ko"] {
        let long = if lang == "ko" {
            "아주 긴 치트 이름".repeat(40)
        } else {
            "a very long cheat description ".repeat(30)
        };
        let list = vec![cheat(&long, true)];
        let mut ctx = ctx("rgsp", lang);
        let c = draw(&CheatMenu::new(list.len()), &mut ctx, "rgsp", &list);

        let spans = ctx.i18n.spans(KEY_ENABLED, &[]);
        let state_w = face::spans_width(&mut ctx, &spans, PX_BODY);
        let state_x = CheatMenu::state_x(&ctx, state_w);
        let left = CheatMenu::row_text_x(&ctx);

        let (x, w) = c
            .frame()
            .iter()
            .find_map(|o| match o {
                Op::Image { x, y, w, .. }
                    if (*x - left).abs() < 0.5 && *y >= CheatMenu::row_y(&ctx, 0) =>
                {
                    Some((*x, *w))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{lang}: the description was not drawn"));
        assert!(
            x + w <= state_x - STATE_GAP + 0.5,
            "{lang}: the description ran into the state column: {x}+{w} vs {state_x}"
        );
    }
}

#[test]
fn every_geometry_keeps_the_panel_rows_and_hints_in_the_safe_area() {
    let list = cheats(MAX_ROWS + 2);
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            let case = format!("{target}/{lang}");
            let (bx, by) = CheatMenu::box_origin(&ctx);
            let c = draw(&CheatMenu::new(list.len()), &mut ctx, target, &list);
            let ops = c.frame();

            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{case}: the menu cleared the game frame"
            );
            match ops
                .iter()
                .find(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
            {
                Some(Op::Rect { x, y, w, h, color }) => {
                    assert_eq!((*x, *y, *w, *h), (0.0, 0.0, pw as f32, ph as f32));
                    assert_eq!(*color, DIM, "{case}: the panel was not dimmed first");
                }
                other => panic!("{case}: the first mark was not the dim: {other:?}"),
            }

            assert!(
                ctx.safe.contains(bx, by, BOX_W, BOX_H),
                "{case}: the box left the safe area"
            );
            for op in ops {
                let (x, y, w, h) = match op {
                    Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                    _ => continue,
                };
                if (x, y, w, h) == (0.0, 0.0, pw as f32, ph as f32) {
                    continue;
                }
                assert!(
                    ctx.safe.contains(x, y, w, h),
                    "{case}: {op:?} left the safe area"
                );
            }

            // Title, rows and hints stay in that order, all of them inside the box.
            let rows_top = CheatMenu::row_y(&ctx, 0);
            let hint = CheatMenu::hint_y(&ctx);
            assert!(
                by + PAD < rows_top && rows_top < hint && hint + PX_HINT <= by + BOX_H,
                "{case}: the box's lines are not in order"
            );
        }
    }
}

#[test]
fn a_warm_menu_uploads_nothing() {
    // Korean, so the CJK face is part of what has to settle and not only the Latin one.
    let list = cheats(MAX_ROWS + 1);
    let mut ctx = ctx("rgsp", "ko");
    let (pw, ph) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(pw, ph);
    let mut menu = CheatMenu::new(list.len());
    let uploads = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
            .count()
    };

    menu.draw(&mut c, &mut ctx, &list);
    assert!(
        uploads(&c.ops) > 0,
        "the first frame uploaded nothing at all"
    );

    let before = c.ops.len();
    for _ in 0..8 {
        menu.draw(&mut c, &mut ctx, &list);
    }
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "eight unchanged frames uploaded something"
    );

    // A lap down the list and back shows the same words again: the faces are already there,
    // so the only thing that changes between two frames is which row is highlighted.
    for _ in 0..list.len() {
        menu.down();
        menu.draw(&mut c, &mut ctx, &list);
    }
    let before = c.ops.len();
    menu.up();
    menu.draw(&mut c, &mut ctx, &list);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "moving the highlight uploaded something"
    );
}
