//! Contract for the About sticker: the build it was handed, where the notices are, and nothing
//! it made up for itself.

use std::path::PathBuf;

use slot2_gfx::{Color, Op, RecordingCanvas};
use slot2_i18n::Arg;
use slot2_platform::by_target;
use slot2_ui::about_sticker::{
    BOX_H, BOX_W, DIM, HINT_KEY, HINT_Y, LICENSE_KEY, LICENSE_Y, NOTICES_KEY, NOTICES_Y, PAD,
    TARGET_KEY, TARGET_Y, TITLE_KEY, TITLE_Y, VERSION_KEY, VERSION_Y, WORDMARK_KEY, WORDMARK_PX,
    WORDMARK_Y,
};
use slot2_ui::layout::SafeArea;
use slot2_ui::splash::{BACKDROP, INK, INK_DIM};
use slot2_ui::{face, AboutInfo, AboutSticker, UiCtx, PX_BODY, PX_HINT};

const TARGETS: [&str; 3] = ["rg35xxsp", "rgsp", "rgcubexx"];

const VERSION: &str = "0.1.0";
const DEVICE: &str = "rgsp";
/// Long enough to be a real worry: twenty-one digits is what a version with a revision and a
/// build stamp looks like, and a target id nobody has shipped yet.
const LONG_VERSION: &str = "1234567890.1234567890";
const LONG_TARGET: &str = "unknown-long-target";

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

/// The sticker drawn onto a fresh canvas of the panel's own size.
fn frame(target: &str, lang: &str, info: AboutInfo<'_>) -> RecordingCanvas {
    let mut ctx = ctx(target, lang);
    let (w, h) = by_target(target).unwrap().geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    AboutSticker::new().draw(&mut c, &mut ctx, info);
    c
}

/// Every filled rect in a frame, as `(x, y, w, h, color)`.
fn rects(ops: &[Op]) -> Vec<(f32, f32, f32, f32, Color)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
            _ => None,
        })
        .collect()
}

/// Every text quad in a frame, as `(x, y, w, h, tint)`.
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

/// Whether a rect exactly there is drawn.
fn has_rect(ops: &[Op], x: f32, y: f32, w: f32, h: f32, color: Color) -> bool {
    rects(ops).iter().any(|(rx, ry, rw, rh, rc)| {
        (*rx - x).abs() < 0.5
            && (*ry - y).abs() < 0.5
            && (*rw - w).abs() < 0.5
            && (*rh - h).abs() < 0.5
            && *rc == color
    })
}

/// The width a message draws at, arguments included.
fn width(ctx: &mut UiCtx, key: &str, args: &[(&str, Arg)], px: f32) -> f32 {
    let spans = ctx.i18n.spans(key, args);
    face::spans_width(ctx, &spans, px)
}

/// How many quads a line of the sticker puts on screen.
///
/// A one-run message is a single text quad, and the numbers check it is that message with that
/// argument at that argument's width — a line glued together in Rust would not measure the same.
/// The hint is a button cap, its label and the word after it, so it is counted as the three quads
/// it is rather than by a width nothing draws at.
fn count_line(ops: &[Op], ctx: &mut UiCtx, line: &Line, slot: (f32, f32)) -> usize {
    if line.key == HINT_KEY {
        return quads_in(ops, slot).len();
    }
    let (y, h) = slot;
    let want = width(ctx, line.key, &line.args, line.px);
    images(ops)
        .iter()
        .filter(|(_, iy, iw, _, ic)| {
            (*iw - want).abs() < 0.5 && *ic == line.tint && *iy >= y && *iy < y + h
        })
        .count()
}

/// Every quad whose top is inside the slot, as `(top, bottom)` rows.
fn quads_in(ops: &[Op], slot: (f32, f32)) -> Vec<(f32, f32)> {
    let (y, h) = slot;
    ops.iter()
        .filter_map(|o| match o {
            Op::Rect { y, h, .. } | Op::Image { y, h, .. } => Some((*y, *y + *h)),
            _ => None,
        })
        .filter(|(top, _)| *top >= y && *top < y + h)
        .collect()
}

/// The vertical extent of everything drawn in the slot.
fn extent(ops: &[Op], slot: (f32, f32)) -> Option<(f32, f32)> {
    quads_in(ops, slot)
        .into_iter()
        .reduce(|(a, b), (c, d)| (a.min(c), b.max(d)))
}

fn uploads(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count()
}

/// One line of the sticker: the message it is, its arguments, its size, its ink, and how many
/// quads it is made of.
struct Line {
    key: &'static str,
    args: Vec<(&'static str, Arg)>,
    px: f32,
    tint: Color,
    quads: usize,
}

/// The lines in draw order. The slots they own run from one line's top to the next line's, so two
/// lines can be told apart and neither can be checked in a neighbour's space.
fn lines(info: AboutInfo<'_>) -> Vec<Line> {
    vec![
        Line {
            key: TITLE_KEY,
            args: vec![],
            px: PX_BODY,
            tint: INK_DIM,
            quads: 1,
        },
        Line {
            key: WORDMARK_KEY,
            args: vec![],
            px: WORDMARK_PX,
            tint: INK,
            quads: 1,
        },
        Line {
            key: VERSION_KEY,
            args: vec![("version", Arg::from(info.version))],
            px: PX_BODY,
            tint: INK,
            quads: 1,
        },
        Line {
            key: TARGET_KEY,
            args: vec![("target", Arg::from(info.target))],
            px: PX_BODY,
            tint: INK,
            quads: 1,
        },
        Line {
            key: LICENSE_KEY,
            args: vec![],
            px: PX_BODY,
            tint: INK,
            quads: 1,
        },
        Line {
            key: NOTICES_KEY,
            args: vec![],
            px: PX_BODY,
            tint: INK_DIM,
            quads: 1,
        },
        Line {
            key: HINT_KEY,
            args: vec![],
            px: PX_HINT,
            tint: INK_DIM,
            quads: 3,
        },
    ]
}

/// The slot each line owns: its own top to the next line's, with the last one running to the
/// bottom of the box.
fn slots() -> Vec<(f32, f32)> {
    let tops = [
        TITLE_Y, WORDMARK_Y, VERSION_Y, TARGET_Y, LICENSE_Y, NOTICES_Y, HINT_Y, BOX_H,
    ];
    tops.windows(2).map(|w| (w[0], w[1] - w[0])).collect()
}

/// The panel offset a slot is drawn at, in panel coordinates.
fn at(by: f32, slot: (f32, f32)) -> (f32, f32) {
    (by + slot.0, slot.1)
}

// ---------------------------------------------------------------- what is on it

#[test]
fn the_version_and_target_are_the_ones_the_caller_handed_over() {
    for lang in ["en", "ko"] {
        let info = AboutInfo {
            version: VERSION,
            target: DEVICE,
        };
        let mut ctx = ctx("rgsp", lang);
        let ops = frame("rgsp", lang, info).ops.clone();
        let case = lang.to_string();

        // The line on screen is the pack's own sentence with this caller's argument, at the width
        // that argument measures: a version glued together in Rust could not match it.
        let by = AboutSticker::box_origin(&ctx).1;
        for (line, slot) in lines(info).iter().zip(slots()) {
            if line.key != VERSION_KEY && line.key != TARGET_KEY {
                continue;
            }
            assert_eq!(
                count_line(&ops, &mut ctx, line, at(by, slot)),
                1,
                "{case}: no {} line",
                line.key
            );
        }

        // And it is the argument that was handed over rather than any other value: a different
        // version measures differently, so the quad on screen cannot be it.
        let version_slot = at(by, slots()[2]);
        let target_slot = at(by, slots()[3]);
        for (key, args, slot) in [
            (
                VERSION_KEY,
                vec![("version", Arg::from("9999.9999.9999"))],
                version_slot,
            ),
            (
                TARGET_KEY,
                vec![("target", Arg::from("rg-other"))],
                target_slot,
            ),
        ] {
            let want = width(&mut ctx, key, &args, PX_BODY);
            let found = images(&ops)
                .iter()
                .filter(|(_, iy, iw, _, _)| {
                    (*iw - want).abs() < 0.5 && *iy >= slot.0 && *iy < slot.0 + slot.1
                })
                .count();
            assert_eq!(
                found, 0,
                "{case}: {key} is not the value that was passed in"
            );
        }
    }
}

#[test]
fn every_line_of_the_sticker_is_there_once_and_stays_in_its_own_row() {
    let info = AboutInfo {
        version: VERSION,
        target: DEVICE,
    };
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let ops = frame(target, lang, info).ops.clone();
            let by = AboutSticker::box_origin(&ctx).1;
            let case = format!("{target}/{lang}");

            for (line, slot) in lines(info).iter().zip(slots()) {
                let slot = at(by, slot);
                assert_eq!(
                    count_line(&ops, &mut ctx, line, slot),
                    line.quads,
                    "{case}: {} is not on screen once",
                    line.key
                );

                // The line's own ink stays in its own row: nothing is squashed into the line
                // above or below it, which is what makes a sticker readable.
                let (top, bottom) = extent(&ops, slot)
                    .unwrap_or_else(|| panic!("{case}: {} drew nothing", line.key));
                assert!(
                    top >= slot.0 - 0.01 && bottom <= slot.0 + slot.1 + 0.01,
                    "{case}: {} is drawn {}..{} and its row is {}..{}",
                    line.key,
                    top,
                    bottom,
                    slot.0,
                    slot.0 + slot.1
                );

                // And the line fits the box it is drawn in, at its own measured width.
                let wide = width(&mut ctx, line.key, &line.args, line.px);
                assert!(
                    wide <= BOX_W - 2.0 * PAD,
                    "{case}: {} is {wide} wide and the box's inside is {}",
                    line.key,
                    BOX_W - 2.0 * PAD
                );
            }
        }
    }
}

// ---------------------------------------------------------------- drawing

#[test]
fn drawing_dims_the_panel_then_puts_the_sticker_on_it() {
    let info = AboutInfo {
        version: VERSION,
        target: DEVICE,
    };
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            let mut c = RecordingCanvas::new(pw, ph);
            AboutSticker::new().draw(&mut c, &mut ctx, info);
            let ops = c.frame();
            let case = format!("{target}/{lang}");

            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{case}: the sticker cleared the screen under it"
            );
            // The first mark is the whole panel, dimmed; the second is the sticker itself.
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
            let (bx, by) = AboutSticker::box_origin(&ctx);
            assert!(
                has_rect(ops, bx, by, BOX_W, BOX_H, BACKDROP),
                "{case}: the sticker is not where the layout says"
            );
            match ops.get(1) {
                Some(Op::Rect { x, y, .. }) => assert_eq!(
                    (*x, *y),
                    (bx, by),
                    "{case}: something was drawn between the dim and the panel"
                ),
                other => panic!("{case}: the panel is not the second mark: {other:?}"),
            }
            assert!(
                ctx.safe.contains(bx, by, BOX_W, BOX_H),
                "{case}: the box left the safe area"
            );

            // Everything but that deliberate full-panel dim is inside the safe area.
            for op in ops {
                let (x, y, qw, qh) = match op {
                    Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                    _ => continue,
                };
                if (x, y, qw, qh) == (0.0, 0.0, pw as f32, ph as f32) {
                    continue;
                }
                assert!(ctx.safe.contains(x, y, qw, qh), "{case}: {op:?} left it");
            }
        }
    }
}

#[test]
fn the_sticker_is_centred_in_the_safe_area_and_not_on_the_panel() {
    for target in TARGETS {
        let ctx = ctx(target, "en");
        let (bx, by) = AboutSticker::box_origin(&ctx);
        assert_eq!(bx, ctx.safe.px((640.0 - BOX_W) / 2.0), "{target}: box x");
        assert_eq!(by, ctx.safe.py((480.0 - BOX_H) / 2.0), "{target}: box y");
        assert!(ctx.safe.contains(bx, by, BOX_W, BOX_H));
        // The box's own centre is the safe area's centre, whatever the panel around it is.
        assert_eq!(bx + BOX_W / 2.0, ctx.safe.px(640.0 / 2.0), "{target}");
        assert_eq!(by + BOX_H / 2.0, ctx.safe.py(480.0 / 2.0), "{target}");
    }

    // No shipped panel has an off-centre safe area, so the two answers agree on all three of
    // them; a safe area that is not centred is what tells the two rules apart, and this is that
    // panel.
    let mut ctx = ctx("rgcubexx", "en");
    ctx.safe = SafeArea {
        x: 0,
        y: 0,
        panel_w: 720,
        panel_h: 720,
    };
    let (bx, by) = AboutSticker::box_origin(&ctx);
    assert_eq!(
        (bx, by),
        (100.0, 110.0),
        "the box was placed against the panel rather than in the safe area"
    );
    let mut c = RecordingCanvas::new(720, 720);
    AboutSticker::new().draw(
        &mut c,
        &mut ctx,
        AboutInfo {
            version: VERSION,
            target: DEVICE,
        },
    );
    assert!(
        has_rect(c.ops.as_slice(), bx, by, BOX_W, BOX_H, BACKDROP),
        "the drawn panel is not where the safe area puts it"
    );
}

#[test]
fn a_long_version_and_target_still_fit_the_panel() {
    let info = AboutInfo {
        version: LONG_VERSION,
        target: LONG_TARGET,
    };
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            let mut c = RecordingCanvas::new(pw, ph);
            AboutSticker::new().draw(&mut c, &mut ctx, info);
            let ops = c.ops.clone();
            let by = AboutSticker::box_origin(&ctx).1;
            let case = format!("{target}/{lang}");

            for (line, slot) in lines(info).iter().zip(slots()) {
                assert_eq!(
                    count_line(&ops, &mut ctx, line, at(by, slot)),
                    line.quads,
                    "{case}: {} is not on screen",
                    line.key
                );
                let wide = width(&mut ctx, line.key, &line.args, line.px);
                assert!(
                    wide <= BOX_W - 2.0 * PAD,
                    "{case}: {} is {wide} wide and the box's inside is {}",
                    line.key,
                    BOX_W - 2.0 * PAD
                );
            }
            for op in &ops {
                let (x, y, qw, qh) = match op {
                    Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                    _ => continue,
                };
                if (x, y, qw, qh) == (0.0, 0.0, pw as f32, ph as f32) {
                    continue;
                }
                assert!(ctx.safe.contains(x, y, qw, qh), "{case}: {op:?} left it");
            }
        }
    }
}

#[test]
fn an_empty_version_and_target_are_shown_as_nothing_rather_than_refused() {
    let info = AboutInfo {
        version: "",
        target: "",
    };
    for (target, lang) in [("rg35xxsp", "en"), ("rgcubexx", "ko")] {
        let mut ctx = ctx(target, lang);
        let (pw, ph) = ctx.profile.geometry.size();
        let mut c = RecordingCanvas::new(pw, ph);
        AboutSticker::new().draw(&mut c, &mut ctx, info);
        let ops = c.ops.clone();
        let by = AboutSticker::box_origin(&ctx).1;
        let case = format!("{target}/{lang}");

        // The rest of the sticker is unaffected: only the two arguments are empty.
        for (line, slot) in lines(info).iter().zip(slots()) {
            assert_eq!(
                count_line(&ops, &mut ctx, line, at(by, slot)),
                line.quads,
                "{case}: {} did not survive an empty argument",
                line.key
            );
        }
        for op in &ops {
            let (x, y, qw, qh) = match op {
                Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                _ => continue,
            };
            if (x, y, qw, qh) == (0.0, 0.0, pw as f32, ph as f32) {
                continue;
            }
            assert!(ctx.safe.contains(x, y, qw, qh), "{case}: {op:?} left it");
        }
    }
}

#[test]
fn a_warm_sticker_uploads_nothing_while_it_says_the_same_thing() {
    // Korean, so the CJK faces are part of what has to settle and not only the Latin ones.
    let mut ctx = ctx("rgsp", "ko");
    let (pw, ph) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(pw, ph);
    let info = AboutInfo {
        version: VERSION,
        target: DEVICE,
    };
    AboutSticker::new().draw(&mut c, &mut ctx, info);
    let first = c.ops.clone();
    assert!(
        uploads(&first) > 0,
        "the first frame uploaded nothing at all"
    );

    let before = c.ops.len();
    for _ in 0..8 {
        AboutSticker::new().draw(&mut c, &mut ctx, info);
    }
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "eight unchanged frames uploaded something"
    );

    // And the next frame draws exactly the same marks as the first one: a sticker with no state
    // has nothing to drift with.
    let before = c.ops.len();
    AboutSticker::new().draw(&mut c, &mut ctx, info);
    let marks = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
            .cloned()
            .collect::<Vec<Op>>()
    };
    assert_eq!(
        marks(&c.ops[before..]),
        marks(&first),
        "the sticker redrew differently"
    );
}

#[test]
fn both_packs_carry_all_six_keys_themselves() {
    // The i18n crate's own test pins the exact wording; this one is here so a key the drawing
    // asks for cannot quietly become the `[key]` marker on a real screen.
    let en = slot2_i18n::I18n::embedded("en").unwrap();
    let ko = slot2_i18n::I18n::embedded("ko").unwrap();
    for key in [
        TITLE_KEY,
        WORDMARK_KEY,
        VERSION_KEY,
        TARGET_KEY,
        LICENSE_KEY,
        NOTICES_KEY,
    ] {
        for pack in [&en, &ko] {
            assert_ne!(pack.t(key), format!("[{key}]"), "{key} is missing");
            // The back hint is the other menus' own message, and it is here too.
            assert!(pack.t("hint-back").contains("[B]"), "{key}: no back hint");
        }
    }
}
