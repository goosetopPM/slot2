//! Contract for the language picker: the packs that loaded, as a list a person can walk.
//!
//! Nothing here touches the card, the environment or `I18n::load`: the caller hands over the
//! languages it already managed to speak, and the picker is only the list, the highlight and the
//! words around them.

use std::path::PathBuf;

use slot2_gfx::{Color, Op, RecordingCanvas};
use slot2_i18n::Arg;
use slot2_platform::by_target;
use slot2_ui::language_picker::{
    BOX_H, BOX_W, CODE_CURRENT_KEY, CODE_KEY, CODE_W, DIM, EMPTY_KEY, HINT_LINE_H, INSET,
    MAX_VISIBLE_ROWS, NAME_W, PAD, POSITION_KEY, ROW_H,
};
use slot2_ui::splash::{BACKDROP, INK};
use slot2_ui::{face, LanguageOption, LanguagePicker, UiCtx, PX_BODY, PX_HINT};

const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

fn ui_ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn frame(picker: &LanguagePicker, ctx: &mut UiCtx, target: &str) -> RecordingCanvas {
    let (w, h) = by_target(target).unwrap().geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    picker.draw(&mut c, ctx);
    c
}

fn option(code: &str, name: &str) -> LanguageOption {
    LanguageOption::new(code, name)
}

/// The three languages the rest of the file walks around.
fn three() -> Vec<LanguageOption> {
    vec![
        option("en", "English"),
        option("ko", "한국어"),
        option("pt-BR", "Português (Brasil)"),
    ]
}

/// A list longer than the window, in the caller's own order.
fn long() -> Vec<LanguageOption> {
    (0..MAX_VISIBLE_ROWS + 3)
        .map(|i| option(&format!("c{i:02}"), &format!("Language {i:02}")))
        .collect()
}

fn rects(ops: &[Op]) -> Vec<(f32, f32, f32, f32, Color)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
            _ => None,
        })
        .collect()
}

fn images(ops: &[Op]) -> Vec<(f32, f32, f32, f32, Color)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Image {
                x, y, w, h, tint, ..
            } => Some((*x, *y, *w, *h, *tint)),
            _ => None,
        })
        .collect()
}

/// Every quad whose top is inside `y .. y + h`, as `(x, y, w, h, tint)`.
fn quads_in(ops: &[Op], y: f32, h: f32) -> Vec<(f32, f32, f32, f32, Color)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, color }
            | Op::Image {
                x,
                y,
                w,
                h,
                tint: color,
                ..
            } => Some((*x, *y, *w, *h, *color)),
            _ => None,
        })
        .filter(|(_, top, _, _, _)| *top >= y && *top < y + h)
        .collect()
}

/// Whether a text run is drawn, whole, in the band — at the width it measures.
fn text_in(ops: &[Op], ctx: &mut UiCtx, text: &str, px: f32, band: (f32, f32)) -> bool {
    let want = face::measure(ctx, text, px);
    images(ops)
        .iter()
        .any(|(_, y, w, _, _)| (*w - want).abs() < 0.5 && *y >= band.0 && *y < band.0 + band.1)
}

/// Whether a message with its arguments is drawn, whole, in the band.
fn message_in(
    ops: &[Op],
    ctx: &mut UiCtx,
    key: &str,
    args: &[(&str, Arg)],
    px: f32,
    band: (f32, f32),
) -> bool {
    let text = ctx.i18n.t_args(key, args);
    text_in(ops, ctx, &text, px, band)
}

fn uploads(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count()
}

/// The row band of visible slot `i`.
fn row_band(ctx: &UiCtx, i: usize) -> (f32, f32) {
    (LanguagePicker::row_y(ctx, i), ROW_H)
}

// ---------------------------------------------------------------- the model

#[test]
fn options_keep_their_spelling_and_the_callers_order() {
    let options = vec![
        option("pt-BR", "Português (Brasil)"),
        option("한국어", "한국어"),
        option("en", "English"),
        option("xx-E🚀", "🚀 Rocket"),
        option("ja_custom", "  日本語  "),
    ];
    let picker = LanguagePicker::new(options.clone(), "en");
    assert_eq!(picker.options(), options.as_slice());
    assert_eq!(picker.len(), 5);
    assert!(!picker.is_empty());

    // Spelling, case and Unicode come back exactly as they went in, on both fields.
    for (got, want) in picker.options().iter().zip(options.iter()) {
        assert_eq!(got.code(), want.code());
        assert_eq!(got.name(), want.name());
    }
    assert_eq!(picker.options()[0].code(), "pt-BR");
    assert_eq!(picker.options()[3].code(), "xx-E🚀");
    // A name's own padding is the pack's business and is not trimmed here.
    assert_eq!(picker.options()[4].name(), "  日本語  ");
    // The option itself answers the same way.
    assert_eq!(option("ko", "한국어").code(), "ko");
    assert_eq!(option("ko", "한국어").name(), "한국어");
}

#[test]
fn an_exact_repeat_code_is_one_row_and_a_shared_name_is_two() {
    let picker = LanguagePicker::new(
        vec![
            option("en", "English"),
            option("EN", "English"),
            option("en", "English again"),
            option("en", "and again"),
        ],
        "en",
    );
    assert_eq!(picker.len(), 2, "an exact repeat code is one language");
    // The first entry is the caller's own answer for that code.
    assert_eq!(picker.options()[0].code(), "en");
    assert_eq!(picker.options()[0].name(), "English");
    assert_eq!(picker.options()[1].code(), "EN", "case is part of a code");

    // Two codes that share a display name are two languages, not one.
    let picker = LanguagePicker::new(vec![option("aa", "Same"), option("bb", "Same")], "aa");
    assert_eq!(picker.len(), 2);
    assert_eq!(picker.options()[1].code(), "bb");
}

#[test]
fn the_badge_holds_still_while_the_highlight_walks() {
    let mut picker = LanguagePicker::new(three(), "ko");
    assert_eq!(picker.highlighted().unwrap().code(), "ko");
    assert_eq!(picker.current().unwrap().code(), "ko");
    assert!(!picker.changed(), "the language in use is not a change");

    picker.down();
    assert_eq!(picker.highlighted().unwrap().code(), "pt-BR");
    assert_eq!(
        picker.current().unwrap().code(),
        "ko",
        "the badge moved with the highlight"
    );
    assert!(picker.changed());

    picker.down();
    assert_eq!(
        picker.highlighted().unwrap().code(),
        "en",
        "down did not wrap"
    );
    assert_eq!(picker.current().unwrap().code(), "ko");
    assert!(picker.changed());

    picker.down();
    assert_eq!(picker.highlighted().unwrap().code(), "ko");
    assert!(!picker.changed(), "coming back is not a change");

    picker.up();
    assert_eq!(
        picker.highlighted().unwrap().code(),
        "en",
        "up did not wrap past the first row"
    );
    assert!(picker.changed());
    assert_eq!(picker.current().unwrap().code(), "ko");
}

#[test]
fn a_language_that_is_not_on_the_list_starts_at_the_top_with_no_badge() {
    let picker = LanguagePicker::new(three(), "xx-unknown");
    assert_eq!(picker.highlighted().unwrap().code(), "en");
    assert!(picker.current().is_none());
    assert!(
        picker.changed(),
        "a language nobody can load is still a change to make"
    );

    // And nothing at all: every accessor answers, and the keys do nothing.
    let empty = LanguagePicker::new(Vec::new(), "en");
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert!(empty.highlighted().is_none());
    assert!(empty.current().is_none());
    assert!(!empty.changed());
    assert_eq!(empty.first_visible(), 0);
    assert_eq!(empty.options(), &[] as &[LanguageOption]);

    let mut walked = empty.clone();
    for _ in 0..3 {
        walked.up();
        walked.down();
    }
    assert_eq!(walked, empty, "an empty picker moved");
}

#[test]
fn the_window_follows_the_highlight_and_wraps_with_it() {
    // One row is stable under either key.
    let mut one = LanguagePicker::new(vec![option("en", "English")], "en");
    for _ in 0..3 {
        one.up();
        assert_eq!(one.highlighted().unwrap().code(), "en");
        one.down();
        assert_eq!(one.highlighted().unwrap().code(), "en");
        assert_eq!(one.first_visible(), 0);
    }

    let len = long().len();
    let mut picker = LanguagePicker::new(long(), "c00");
    assert_eq!(
        picker.first_visible(),
        0,
        "a fresh window starts at the top"
    );

    // Walking the whole list once: the highlight is always on screen, and the window never runs
    // past the end of the list.
    for step in 1..=len {
        picker.down();
        let want = step % len;
        let code = picker.highlighted().unwrap().code().to_owned();
        assert_eq!(code, format!("c{want:02}"), "down landed on the wrong row");
        let first = picker.first_visible();
        assert!(
            first < len && first <= want && want < first + MAX_VISIBLE_ROWS,
            "row {want} is not inside the window {first}..{}",
            first + MAX_VISIBLE_ROWS
        );
        assert!(
            first + MAX_VISIBLE_ROWS <= len,
            "the window runs past the last row"
        );
    }

    // Up from the first row wraps to the last, and the window shows the tail rather than
    // leaving the highlight below it.
    let mut picker = LanguagePicker::new(long(), "c00");
    picker.up();
    assert_eq!(
        picker.highlighted().unwrap().code(),
        format!("c{:02}", len - 1)
    );
    assert_eq!(picker.first_visible(), len - MAX_VISIBLE_ROWS);

    // And down from the last wraps to the first, with the window back at the top.
    picker.down();
    assert_eq!(picker.highlighted().unwrap().code(), "c00");
    assert_eq!(picker.first_visible(), 0);
}

// ---------------------------------------------------------------- the drawing

#[test]
fn both_packs_say_every_line_the_picker_draws() {
    for (lang, current) in [("en", "en · Current"), ("ko", "en · 사용 중")] {
        let mut ctx = ui_ctx("rgsp", lang);
        let picker = LanguagePicker::new(three(), "en");
        let c = frame(&picker, &mut ctx, "rgsp");
        let ops = c.ops.clone();
        let case = lang.to_string();

        // The title is the settings row's own name, and the hints are the other menus'.
        let title_band = (LanguagePicker::box_origin(&ctx).1 + PAD, 40.0);
        assert!(
            message_in(&ops, &mut ctx, "shelf-language", &[], PX_BODY, title_band),
            "{case}: no title"
        );

        // Every row: its own name on the left, its code on the right — and the code of the row
        // that is running carries the pack's own `· Current` sentence.
        for (i, want) in ["en", "ko", "pt-BR"].iter().enumerate() {
            let band = row_band(&ctx, i);
            let name = picker.options()[i].name();
            assert!(
                text_in(&ops, &mut ctx, name, PX_BODY, band),
                "{case}: row {i} does not show {name:?}"
            );
            let key = if *want == "en" {
                CODE_CURRENT_KEY
            } else {
                CODE_KEY
            };
            let text = ctx.i18n.t_args(key, &[("code", Arg::from(*want))]);
            assert!(
                text_in(&ops, &mut ctx, &text, PX_HINT, band),
                "{case}: row {i} does not show {text:?}"
            );
        }

        // The English pack says exactly that sentence, the Korean one its own.
        assert_eq!(
            ctx.i18n
                .t_args(CODE_CURRENT_KEY, &[("code", Arg::from("en"))]),
            current,
            "{case}: the current sentence is not the pack's"
        );
        assert_eq!(
            ctx.i18n.t_args(CODE_KEY, &[("code", Arg::from("ko"))]),
            "ko"
        );
        assert_ne!(ctx.i18n.t(EMPTY_KEY), format!("[{EMPTY_KEY}]"));
        assert_eq!(
            ctx.i18n.t_args(
                POSITION_KEY,
                &[("current", Arg::from(2i64)), ("total", Arg::from(3i64))]
            ),
            "2 / 3"
        );

        // The position and both hints are on screen.
        let band = (LanguagePicker::position_y(&ctx), HINT_LINE_H + 6.0);
        assert!(
            message_in(
                &ops,
                &mut ctx,
                POSITION_KEY,
                &[("current", Arg::from(1i64)), ("total", Arg::from(3i64))],
                PX_HINT,
                band
            ),
            "{case}: no position line"
        );
        let band = (LanguagePicker::hint_y(&ctx), HINT_LINE_H + 6.0);
        for key in ["hint-select", "hint-back"] {
            assert_ne!(
                ctx.i18n.t(key),
                format!("[{key}]"),
                "{case}: {key} is missing from the pack"
            );
            // A hint is a button cap, its label and the word after it: several quads, all in the
            // hint band.
            assert!(
                quads_in(&ops, band.0, band.1).len() >= 4,
                "{case}: {key} is not drawn"
            );
        }
    }

    // A Korean pack on an English screen still names itself in Korean.
    let mut ctx = ui_ctx("rgsp", "en");
    let picker = LanguagePicker::new(vec![option("ko", "한국어")], "en");
    let c = frame(&picker, &mut ctx, "rgsp");
    let band = row_band(&ctx, 0);
    assert!(
        text_in(&c.ops, &mut ctx, "한국어", PX_BODY, band),
        "a language does not name itself"
    );
}

#[test]
fn the_panel_keeps_every_thing_in_the_safe_area_on_every_geometry() {
    let mut relative: Option<(f32, f32)> = None;
    for target in TARGETS {
        let mut ctx = ui_ctx(target, "en");
        let picker = LanguagePicker::new(long(), "c00");
        let c = frame(&picker, &mut ctx, target);
        let ops = c.ops.clone();
        let case = target.to_string();
        let (pw, ph) = ctx.profile.geometry.size();

        assert!(
            !ops.iter().any(|o| matches!(o, Op::Clear(_))),
            "{case}: the picker cleared the panel"
        );
        // The dim, then the panel.
        match ops.first() {
            Some(Op::Rect { x, y, w, h, color }) => {
                assert_eq!(
                    (*x, *y, *w, *h),
                    (0.0, 0.0, pw as f32, ph as f32),
                    "{case}: the dim is not the whole panel"
                );
                assert_eq!(*color, DIM, "{case}: the panel was not dimmed first");
            }
            other => panic!("{case}: the first thing drawn was not the dim: {other:?}"),
        }
        let (bx, by) = LanguagePicker::box_origin(&ctx);
        assert!(
            rects(&ops).iter().any(|(x, y, w, h, color)| {
                (*x - bx).abs() < 0.5
                    && (*y - by).abs() < 0.5
                    && (*w - BOX_W).abs() < 0.5
                    && (*h - BOX_H).abs() < 0.5
                    && *color == BACKDROP
            }),
            "{case}: the panel is not where the layout says"
        );
        assert!(
            ctx.safe.contains(bx, by, BOX_W, BOX_H),
            "{case}: the box left the safe area"
        );

        // Everything but the deliberate full-panel dim is inside the safe area.
        for op in &ops {
            let (x, y, w, h) = match op {
                Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                _ => continue,
            };
            if (x, y, w, h) == (0.0, 0.0, pw as f32, ph as f32) {
                continue;
            }
            assert!(ctx.safe.contains(x, y, w, h), "{case}: {op:?} left it");
        }

        // The rows, the position line and the hints are all drawn.
        for i in 0..MAX_VISIBLE_ROWS {
            assert!(
                !quads_in(&ops, row_band(&ctx, i).0, ROW_H).is_empty(),
                "{case}: row {i} is empty"
            );
        }
        assert!(
            !quads_in(&ops, LanguagePicker::position_y(&ctx), HINT_LINE_H + 6.0).is_empty(),
            "{case}: no position line"
        );
        assert!(
            !quads_in(&ops, LanguagePicker::hint_y(&ctx), HINT_LINE_H + 6.0).is_empty(),
            "{case}: no hints"
        );

        // The same safe-area coordinates on every panel: what moves is where the safe area sits,
        // not where anything inside it goes.
        let here = (bx - ctx.safe.x as f32, by - ctx.safe.y as f32);
        match relative {
            None => relative = Some(here),
            Some(first) => assert_eq!(here, first, "{case}: the panel moved inside the safe area"),
        }
    }
}

#[test]
fn no_text_is_trusted_to_fit_its_column() {
    let long_ascii = "a".repeat(400);
    let long_korean = "가".repeat(200);
    let emoji = "🚀".repeat(120);
    for (field, text) in [
        ("name", long_ascii.as_str()),
        ("name", long_korean.as_str()),
        ("name", emoji.as_str()),
        ("code", long_ascii.as_str()),
        ("code", emoji.as_str()),
    ] {
        let option = match field {
            "name" => option("ko", text),
            _ => option(text, "한국어"),
        };
        let mut ctx = ui_ctx("rg35xxsp", "en");
        let picker = LanguagePicker::new(vec![option], "ko");
        let c = frame(&picker, &mut ctx, "rg35xxsp");
        let ops = c.ops.clone();
        let (bx, _) = LanguagePicker::box_origin(&ctx);
        let case = format!("{field}/{text}");

        for op in &ops {
            let (x, y, w, h) = match op {
                Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                _ => continue,
            };
            assert!(
                ctx.safe.contains(x, y, w, h),
                "{case}: {op:?} left the safe area"
            );
        }
        // The two columns never meet: the name stops before the code column begins. Only the
        // rows are checked — the position line and the hints are centred across the whole panel.
        let name_right = bx + PAD + INSET + NAME_W;
        let code_left = bx + BOX_W - PAD - INSET - CODE_W;
        for (x, y, w, _, _) in images(&ops) {
            if !(y >= LanguagePicker::row_y(&ctx, 0)
                && y < LanguagePicker::row_y(&ctx, MAX_VISIBLE_ROWS))
            {
                continue;
            }
            assert!(
                x >= code_left - 0.5 || x + w <= name_right + 0.5,
                "{case}: a quad at {x} is between the columns"
            );
        }
    }

    // A long name is cut, and the cut keeps the character boundaries: the drawn run is what a
    // caller would get from `fit` at the same width.
    let mut ctx = ui_ctx("rg35xxsp", "en");
    let picker = LanguagePicker::new(vec![option("ko", &long_ascii)], "ko");
    let c = frame(&picker, &mut ctx, "rg35xxsp");
    let cut = slot2_ui::cheat_menu::fit(&mut ctx, &long_ascii, PX_BODY, NAME_W);
    assert!(cut.ends_with('…') && cut.len() < long_ascii.len());
    let band = row_band(&ctx, 0);
    assert!(
        text_in(&c.ops, &mut ctx, &cut, PX_BODY, band),
        "the name was not cut to its column"
    );

    // A name of nothing but spaces is no name: the code stands in for it, and the option itself
    // still holds what the caller passed.
    for blank in ["", "   ", "\t\n"] {
        let option = option("pt-BR", blank);
        assert_eq!(option.name(), blank, "the picker trimmed the caller's name");
        let picker = LanguagePicker::new(vec![option], "pt-BR");
        let mut ctx = ui_ctx("rgsp", "en");
        let c = frame(&picker, &mut ctx, "rgsp");
        let band = row_band(&ctx, 0);
        assert!(
            text_in(&c.ops, &mut ctx, "pt-BR", PX_BODY, band),
            "{blank:?}: the code did not stand in for the blank name"
        );
    }
}

#[test]
fn an_empty_menu_highlights_nothing_and_the_badge_is_not_the_highlight() {
    // Empty: no highlight, no position line, the empty message in the first row's place.
    let mut ctx = ui_ctx("rgsp", "en");
    let empty = LanguagePicker::new(Vec::new(), "en");
    let c = frame(&empty, &mut ctx, "rgsp");
    let ops = c.ops.clone();
    let backing = INK.with_alpha(0.15);
    assert!(
        !rects(&ops)
            .iter()
            .any(|(_, _, _, _, color)| *color == backing),
        "an empty picker highlighted a row"
    );
    let empty_text = ctx.i18n.t(EMPTY_KEY);
    let band = row_band(&ctx, 0);
    assert!(
        text_in(&ops, &mut ctx, &empty_text, PX_BODY, band),
        "no empty message"
    );
    let position = ctx.i18n.t_args(
        POSITION_KEY,
        &[("current", Arg::from(1i64)), ("total", Arg::from(0i64))],
    );
    let want = face::measure(&mut ctx, &position, PX_HINT);
    let band = (LanguagePicker::position_y(&ctx), HINT_LINE_H + 6.0);
    assert!(
        !images(&ops)
            .iter()
            .any(|(_, y, w, _, _)| (*w - want).abs() < 0.5 && *y >= band.0 && *y < band.0 + band.1),
        "an empty picker showed a position"
    );
    assert!(
        !quads_in(&ops, LanguagePicker::hint_y(&ctx), HINT_LINE_H + 6.0).is_empty(),
        "an empty picker has no way out"
    );

    // The badge is not the highlight: with the highlight moved off the running language, exactly
    // one row is highlighted and the badge is on the other one.
    let mut picker = LanguagePicker::new(three(), "ko");
    picker.down(); // ko → pt-BR
    assert_eq!(picker.highlighted().unwrap().code(), "pt-BR");
    let mut ctx = ui_ctx("rgsp", "en");
    let c = frame(&picker, &mut ctx, "rgsp");
    let ops = c.ops.clone();
    let (bx, _) = LanguagePicker::box_origin(&ctx);
    let highlights: Vec<f32> = rects(&ops)
        .iter()
        .filter(|(_, _, w, h, color)| {
            (*w - (BOX_W - 2.0 * PAD)).abs() < 0.5 && (*h - ROW_H).abs() < 0.5 && *color == backing
        })
        .map(|(x, y, _, _, _)| {
            assert!((*x - (bx + PAD)).abs() < 0.5);
            *y
        })
        .collect();
    assert_eq!(highlights.len(), 1, "not exactly one highlighted row");
    assert!(
        (highlights[0] - LanguagePicker::row_y(&ctx, 2)).abs() < 0.5,
        "the highlight is not on the row that is selected"
    );
    let badge_band = row_band(&ctx, 1);
    assert!(
        message_in(
            &ops,
            &mut ctx,
            CODE_CURRENT_KEY,
            &[("code", Arg::from("ko"))],
            PX_HINT,
            badge_band
        ),
        "the badge is not on the row that is running"
    );
    let selected_band = row_band(&ctx, 2);
    assert!(
        message_in(
            &ops,
            &mut ctx,
            CODE_KEY,
            &[("code", Arg::from("pt-BR"))],
            PX_HINT,
            selected_band
        ),
        "the selected row lost its code"
    );
}

#[test]
fn the_picker_reaches_for_nothing_but_the_list_it_was_given() {
    // The boundary is the whole point of this screen: which languages exist is the frontend's
    // answer, given once, and a picker that went looking for packs — or that could replace the
    // context it is drawn with — would be a second, worse answer to it. The claims are read off
    // the source itself, so a later edit that reached for a card, the file system or the packs
    // fails here rather than in review.
    const SRC: &str = include_str!("../src/language_picker.rs");
    for forbidden in [
        "std::fs",        // no file system, so no System/Lang
        "std::env",       // and no environment
        "env::var",       //
        "UiCtx::new",     // the context is drawn with, never replaced
        "slot2_store",    // no Card: the picker is not where a language is read or saved
        "write_language", //
        "available(",     // and no pack listing, filtering or loading
        "read_dir",       //
    ] {
        assert!(
            !SRC.contains(forbidden),
            "the picker reaches for {forbidden}"
        );
    }
    // What it does use: the canvas, the arguments of a message, and the context it is handed.
    for expected in ["Canvas", "Arg", "UiCtx", "cheat_menu::fit"] {
        assert!(SRC.contains(expected), "the picker has no {expected}");
    }
}

#[test]
fn a_warm_picker_uploads_nothing_while_it_says_the_same_thing() {
    // Korean, so the CJK faces are part of what has to settle and not only the Latin ones.
    let mut ctx = ui_ctx("rgsp", "ko");
    let (pw, ph) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(pw, ph);
    let picker = LanguagePicker::new(long(), "c02");

    picker.draw(&mut c, &mut ctx);
    let first = c.ops.clone();
    assert!(
        uploads(&first) > 0,
        "the first frame uploaded nothing at all"
    );

    let before = c.ops.len();
    picker.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "the second frame uploaded something"
    );
    let marks = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
            .cloned()
            .collect::<Vec<Op>>()
    };
    assert_eq!(
        marks(&c.ops[before..]),
        marks(&first),
        "the picker redrew differently"
    );

    // Moving the highlight inside the window costs the position line's new number and nothing
    // else: the names and codes it moves over were already on screen.
    let mut picker = picker.clone();
    picker.down();
    let before = c.ops.len();
    picker.draw(&mut c, &mut ctx);
    let cost = uploads(&c.ops[before..]);
    assert!(cost <= 1, "moving inside the window cost {cost} uploads");

    // And the number it has just written is on screen now, so a move back to it costs nothing.
    picker.up();
    let before = c.ops.len();
    picker.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "the position line was uploaded again"
    );
}
