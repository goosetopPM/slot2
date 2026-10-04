//! Contract for the platform's core picker: the registry's candidates, one highlighted, the
//! running core badged, and no clearing.

use std::path::PathBuf;

use slot2_gfx::{Op, RecordingCanvas};
use slot2_i18n::Span;
use slot2_platform::by_target;
use slot2_retro::{supported_cores, CoreId, Platform};
use slot2_ui::core_picker::{
    BADGE_INSET, BOX_H, BOX_W, CURRENT_KEY, DIM, EMPTY_KEY, MAX_ROWS, PAD, RESTART_KEY, ROW_H,
    TITLE_KEY,
};
use slot2_ui::splash::{BACKDROP, INK};
use slot2_ui::{face, CorePicker, UiCtx, PX_BODY, PX_HINT, PX_TITLE};

const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

/// Every core this frontend ships, and the name a player is meant to read for it. The names
/// are proper nouns: both packs spell them the same way.
const NAMES: [(CoreId, &str); 6] = [
    (CoreId::Mgba, "mGBA"),
    (CoreId::Gambatte, "Gambatte"),
    (CoreId::Gpsp, "gpSP"),
    (CoreId::Fceumm, "FCEUmm"),
    (CoreId::Snes9x, "Snes9x"),
    (CoreId::GenesisPlusGx, "Genesis Plus GX"),
];

const PLATFORMS: [Platform; 7] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Nes,
    Platform::Snes,
    Platform::Md,
    Platform::Sms,
];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn frame(picker: &CorePicker, ctx: &mut UiCtx, target: &str) -> RecordingCanvas {
    let (w, h) = by_target(target).unwrap().geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    picker.draw(&mut c, ctx);
    c
}

/// Every text or image quad in a frame, as `(x, y, w)`.
fn images(ops: &[Op]) -> Vec<(f32, f32, f32)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Image { x, y, w, .. } => Some((*x, *y, *w)),
            _ => None,
        })
        .collect()
}

/// The width a line of text draws at, which is how a drawn quad is identified without
/// depending on texture ids.
fn width(ctx: &mut UiCtx, key: &str, px: f32) -> f32 {
    let spans = ctx.i18n.spans(key, &[]);
    face::spans_width(ctx, &spans, px)
}

/// Whether the highlighted row's backing rect is at `row_y`.
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

/// Whether the `Current` badge is drawn on the row at `row_y`: a quad at the badge's own x,
/// which is what makes it tellable from a row label.
fn badge(ops: &[Op], ctx: &mut UiCtx, bx: f32, row_y: f32) -> bool {
    let w = width(ctx, CURRENT_KEY, PX_BODY);
    let x = bx + BOX_W - PAD - BADGE_INSET - w;
    images(ops).iter().any(|(ix, iy, iw)| {
        (*ix - x).abs() < 0.5 && (*iw - w).abs() < 0.5 && *iy >= row_y && *iy < row_y + ROW_H
    })
}

/// Whether a line of `key` is drawn somewhere in the band `y .. y + h`.
///
/// A `BTN` placeholder is a cap plus its own label, so a hint made of several spans is several
/// quads: what has to be in the band is every text quad of the message.
fn line_in(ops: &[Op], ctx: &mut UiCtx, key: &str, px: f32, y: f32, h: f32) -> bool {
    let widths: Vec<f32> = ctx
        .i18n
        .spans(key, &[])
        .iter()
        .filter_map(|span| match span {
            Span::Text(text) => Some(face::spans_width(ctx, &[Span::Text(text.clone())], px)),
            Span::Btn(_) => None,
        })
        .collect();
    !widths.is_empty()
        && widths.iter().all(|want| {
            images(ops)
                .iter()
                .any(|(_, iy, iw)| (*iw - *want).abs() < 0.5 && *iy >= y && *iy < y + h)
        })
}

// ------------------------------------------------------------------ candidates

#[test]
fn the_candidates_are_the_registrys_order_without_the_rest() {
    // Whatever order, repeats and foreign cores a card hands over, the list is the registry's
    // for this platform: two cores for the Game Boy family and the GBA, one for the rest.
    let installed = [
        CoreId::GenesisPlusGx,
        CoreId::Mgba,
        CoreId::Mgba,
        CoreId::Gpsp,
        CoreId::Snes9x,
        CoreId::Fceumm,
        CoreId::Gambatte,
    ];
    for platform in PLATFORMS {
        let picker = CorePicker::new(platform, &installed, None);
        let want: Vec<CoreId> = supported_cores(platform)
            .iter()
            .copied()
            .filter(|core| installed.contains(core))
            .collect();
        assert_eq!(picker.rows(), want.as_slice(), "{platform:?}");
        assert_eq!(picker.len(), want.len());
        assert_eq!(picker.is_empty(), want.is_empty());
        for row in picker.rows() {
            assert!(
                supported_cores(platform).contains(row),
                "{platform:?}: {row:?} cannot run this console"
            );
            assert_eq!(
                picker.rows().iter().filter(|r| *r == row).count(),
                1,
                "{platform:?}: {row:?} is on the list twice"
            );
        }
    }

    // The order comes from the registry, not from the card: Gambatte first in the input still
    // draws mGBA first for a Game Boy.
    let picker = CorePicker::new(
        Platform::Gb,
        &[CoreId::Gambatte, CoreId::Mgba],
        Some(CoreId::Gambatte),
    );
    assert_eq!(picker.rows(), [CoreId::Mgba, CoreId::Gambatte]);
    assert_eq!(picker.highlighted(), Some(CoreId::Gambatte));

    // A single-core platform keeps its one row however much is installed.
    let picker = CorePicker::new(
        Platform::Nes,
        &[CoreId::Fceumm, CoreId::Mgba, CoreId::Gpsp],
        Some(CoreId::Fceumm),
    );
    assert_eq!(picker.rows(), [CoreId::Fceumm]);

    // Nothing installed for this platform is an empty picker, not a picker of the wrong cores.
    let picker = CorePicker::new(Platform::Gba, &[CoreId::Snes9x], Some(CoreId::Snes9x));
    assert!(picker.is_empty());
    assert_eq!(picker.rows(), []);
    assert_eq!(picker.highlighted(), None);
}

// ------------------------------------------------------------------ the current core

#[test]
fn the_current_core_starts_highlighted_and_stays_put_when_the_highlight_moves() {
    let mut picker = CorePicker::new(
        Platform::Gba,
        &[CoreId::Mgba, CoreId::Gpsp],
        Some(CoreId::Gpsp),
    );
    assert_eq!(picker.highlighted(), Some(CoreId::Gpsp));
    assert_eq!(picker.current(), Some(CoreId::Gpsp));

    picker.down();
    assert_eq!(picker.highlighted(), Some(CoreId::Mgba));
    assert_eq!(
        picker.current(),
        Some(CoreId::Gpsp),
        "the running core moved with the highlight"
    );
    picker.up();
    assert_eq!(picker.highlighted(), Some(CoreId::Gpsp));

    // And the frame agrees: the badge is on the running core's row before and after the walk.
    for _ in 0..2 {
        for target in TARGETS {
            for lang in ["en", "ko"] {
                let mut ctx = ctx(target, lang);
                let c = frame(&picker, &mut ctx, target);
                let (bx, _) = CorePicker::box_origin(&ctx);
                let rows = picker.rows().to_vec();
                for (i, row) in rows.iter().enumerate() {
                    let row_y = CorePicker::row_y(&ctx, i);
                    assert_eq!(
                        badge(c.frame(), &mut ctx, bx, row_y),
                        *row == CoreId::Gpsp,
                        "{target}/{lang}: the badge is not on the running core's row ({row:?})"
                    );
                }
            }
        }
        picker.down();
    }
}

#[test]
fn a_current_core_that_is_not_a_candidate_badges_nothing() {
    // A session on an external library reports no core at all.
    let external = CorePicker::new(Platform::Gba, &[CoreId::Mgba, CoreId::Gpsp], None);
    assert_eq!(external.highlighted(), Some(CoreId::Mgba));
    assert_eq!(external.current(), None);

    // A core that cannot run this console, and one whose library is not on the card: neither
    // has a row, so neither is pointed at. A wrong badge is worse than no badge.
    let foreign = CorePicker::new(
        Platform::Gba,
        &[CoreId::Mgba, CoreId::Gpsp],
        Some(CoreId::Gambatte),
    );
    assert_eq!(foreign.highlighted(), Some(CoreId::Mgba));
    assert_eq!(foreign.current(), None);

    let not_installed = CorePicker::new(Platform::Gba, &[CoreId::Mgba], Some(CoreId::Gpsp));
    assert_eq!(not_installed.highlighted(), Some(CoreId::Mgba));
    assert_eq!(not_installed.current(), None);

    // The highlight falls back to the first row, and no row wears the badge.
    for picker in [external, foreign, not_installed] {
        assert_eq!(picker.highlighted(), picker.rows().first().copied());
        for target in TARGETS {
            for lang in ["en", "ko"] {
                let mut ctx = ctx(target, lang);
                let c = frame(&picker, &mut ctx, target);
                let (bx, _) = CorePicker::box_origin(&ctx);
                for i in 0..picker.len() {
                    let row_y = CorePicker::row_y(&ctx, i);
                    assert!(
                        !badge(c.frame(), &mut ctx, bx, row_y),
                        "{target}/{lang}: row {i} wore a badge it had no core for"
                    );
                }
            }
        }
    }
}

// ------------------------------------------------------------------ navigation

#[test]
fn navigation_wraps_and_an_empty_or_single_row_list_is_safe() {
    // Nothing to walk: the highlight stays where there is no row to be on.
    let mut empty = CorePicker::new(Platform::Gba, &[], None);
    assert!(empty.is_empty());
    for _ in 0..4 {
        empty.up();
        assert_eq!(empty.highlighted(), None);
        empty.down();
        assert_eq!(empty.highlighted(), None);
    }
    assert_eq!(empty.len(), 0);

    // One row stays that row, whichever way it is walked.
    let mut single = CorePicker::new(Platform::Nes, &[CoreId::Fceumm], Some(CoreId::Fceumm));
    for _ in 0..3 {
        single.up();
        assert_eq!(single.highlighted(), Some(CoreId::Fceumm));
        single.down();
        assert_eq!(single.highlighted(), Some(CoreId::Fceumm));
    }

    // Two rows wrap at both ends and a lap comes back to where it started.
    let mut two = CorePicker::new(Platform::Gba, &[CoreId::Mgba, CoreId::Gpsp], None);
    assert_eq!(two.highlighted(), Some(CoreId::Mgba));
    two.up();
    assert_eq!(two.highlighted(), Some(CoreId::Gpsp), "up did not wrap");
    two.down();
    assert_eq!(two.highlighted(), Some(CoreId::Mgba));
    two.down();
    assert_eq!(two.highlighted(), Some(CoreId::Gpsp));
    two.down();
    assert_eq!(two.highlighted(), Some(CoreId::Mgba), "down did not wrap");
}

// ------------------------------------------------------------------ the words

#[test]
fn every_core_is_named_and_no_library_file_name_is_drawn() {
    let mut keys: Vec<&str> = NAMES
        .iter()
        .map(|(core, _)| CorePicker::key(*core))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(
        keys.len(),
        NAMES.len(),
        "two cores share one name: {keys:?}"
    );

    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            for (core, name) in NAMES {
                assert_eq!(
                    ctx.i18n.t(CorePicker::key(core)),
                    name,
                    "{core:?} in {lang}"
                );
                assert!(
                    !ctx.i18n.t(CorePicker::key(core)).contains("libretro"),
                    "{core:?}: the player was shown a file name"
                );
            }

            for platform in PLATFORMS {
                let picker =
                    CorePicker::new(platform, &CoreId::ALL, Some(supported_cores(platform)[0]));
                let c = frame(&picker, &mut ctx, target);
                let drawn = images(c.frame());
                let case = format!("{target}/{lang}/{platform:?}");

                for core in supported_cores(platform) {
                    let want = width(&mut ctx, CorePicker::key(*core), PX_TITLE);
                    assert!(
                        drawn.iter().any(|(_, _, w)| (*w - want).abs() < 0.5),
                        "{case}: {:?} has no row of its own: {drawn:?}",
                        core
                    );
                }
                for core in CoreId::ALL {
                    let library = face::spans_width(
                        &mut ctx,
                        &[Span::Text(core.base_name().to_string())],
                        PX_TITLE,
                    );
                    assert!(
                        !drawn.iter().any(|(_, _, w)| (*w - library).abs() < 0.5),
                        "{case}: {:?} was drawn as a file name",
                        core
                    );
                }
            }
        }
    }
}

// ------------------------------------------------------------------ drawing

#[test]
fn drawing_dims_then_panels_and_keeps_every_thing_in_the_safe_area() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            let picker = CorePicker::new(
                Platform::Gba,
                &[CoreId::Mgba, CoreId::Gpsp],
                Some(CoreId::Mgba),
            );
            let mut c = RecordingCanvas::new(pw, ph);
            picker.draw(&mut c, &mut ctx);
            let ops = c.frame();
            let case = format!("{target}/{lang}");

            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{case}: the picker cleared the game frame"
            );

            // The first mark is the whole-panel dim, before the panel itself.
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

            // The panel is where the layout says, and everything but the deliberate full-panel
            // dim stays inside the safe area: title, rows, badge, notice and hints.
            let (bx, by) = CorePicker::box_origin(&ctx);
            assert!(
                ctx.safe.contains(bx, by, BOX_W, BOX_H),
                "{case}: the box left the safe area"
            );
            assert!(
                ops.iter()
                    .any(|o| matches!(o, Op::Rect { x, y, w, h, color }
                    if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5
                        && (*w - BOX_W).abs() < 0.5 && (*h - BOX_H).abs() < 0.5
                        && *color == BACKDROP)),
                "{case}: the panel is not where the layout says"
            );
            for op in ops {
                let (x, y, w, h) = match op {
                    Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                    _ => continue,
                };
                if (x, y, w, h) == (0.0, 0.0, pw as f32, ph as f32) {
                    continue;
                }
                assert!(ctx.safe.contains(x, y, w, h), "{case}: {op:?} left it");
            }

            // Each part of the panel is really there, in its own band.
            assert!(
                line_in(ops, &mut ctx, TITLE_KEY, PX_BODY, by + PAD, 28.0),
                "{case}: no title"
            );
            for (i, core) in picker.rows().iter().enumerate() {
                let row_y = CorePicker::row_y(&ctx, i);
                assert!(
                    line_in(
                        ops,
                        &mut ctx,
                        CorePicker::key(*core),
                        PX_TITLE,
                        row_y,
                        ROW_H
                    ),
                    "{case}: no row for {core:?}"
                );
            }
            let badge_row = CorePicker::row_y(&ctx, 0);
            assert!(
                badge(ops, &mut ctx, bx, badge_row),
                "{case}: the running core is not badged"
            );
            let notice_y = CorePicker::notice_y(&ctx);
            assert!(
                line_in(ops, &mut ctx, RESTART_KEY, PX_HINT, notice_y, PX_HINT * 2.0),
                "{case}: no restart notice"
            );
            let hint_y = by + BOX_H - PAD - PX_HINT;
            assert!(
                line_in(ops, &mut ctx, "hint-select", PX_HINT, hint_y, PX_HINT * 2.0)
                    && line_in(ops, &mut ctx, "hint-back", PX_HINT, hint_y, PX_HINT * 2.0),
                "{case}: no hints"
            );

            // The two bottom lines do not sit on top of each other, and both fit the panel's
            // own width: a longer translation has to fail here rather than on the device.
            let line_h = ctx.fonts.measure("", PX_HINT).line_height as f32;
            assert!(
                notice_y + line_h <= hint_y,
                "{case}: the notice and the hints overlap"
            );
            for key in [RESTART_KEY, "hint-select", "hint-back"] {
                let w = width(&mut ctx, key, PX_HINT);
                assert!(
                    w <= BOX_W - 2.0 * PAD,
                    "{case}: {key} is {w} wide and the panel's inside is {}",
                    BOX_W - 2.0 * PAD
                );
            }
        }
    }
}

#[test]
fn exactly_one_row_is_highlighted_and_an_empty_picker_highlights_none() {
    // Two candidates, walked to both ends: the backing rect follows the highlight and there is
    // never a second one.
    let mut picker = CorePicker::new(
        Platform::Gba,
        &[CoreId::Mgba, CoreId::Gpsp],
        Some(CoreId::Gpsp),
    );
    for _ in 0..2 {
        for target in TARGETS {
            for lang in ["en", "ko"] {
                let mut ctx = ctx(target, lang);
                let c = frame(&picker, &mut ctx, target);
                let (bx, _) = CorePicker::box_origin(&ctx);
                let rows = picker.rows().to_vec();
                let lit: Vec<CoreId> = (0..rows.len())
                    .filter(|i| highlight(c.frame(), CorePicker::row_y(&ctx, *i), bx))
                    .map(|i| rows[i])
                    .collect();
                assert_eq!(
                    lit,
                    vec![picker.highlighted().unwrap()],
                    "{target}/{lang}: {lit:?} is highlighted"
                );
            }
        }
        picker.down();
    }

    // Nothing installed: the empty message in the first row's place, and no highlight at all.
    let picker = CorePicker::new(Platform::Gba, &[], None);
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let c = frame(&picker, &mut ctx, target);
            let (bx, _) = CorePicker::box_origin(&ctx);
            let case = format!("{target}/{lang}");
            for i in 0..MAX_ROWS {
                assert!(
                    !highlight(c.frame(), CorePicker::row_y(&ctx, i), bx),
                    "{case}: an empty picker highlighted row {i}"
                );
            }
            let empty_w = width(&mut ctx, EMPTY_KEY, PX_BODY);
            let row_y = CorePicker::row_y(&ctx, 0);
            assert!(
                images(c.frame())
                    .iter()
                    .any(
                        |(x, y, w)| (*x - (bx + (BOX_W - empty_w) / 2.0)).abs() < 0.5
                            && (*w - empty_w).abs() < 0.5
                            && *y >= row_y
                            && *y < row_y + ROW_H
                    ),
                "{case}: no empty message where the rows would be"
            );
        }
    }
}

#[test]
fn a_warm_picker_uploads_nothing() {
    // Korean, so the CJK face is part of what has to settle and not only the Latin one.
    let mut warm = ctx("rgsp", "ko");
    let (pw, ph) = warm.profile.geometry.size();
    let mut c = RecordingCanvas::new(pw, ph);
    let mut picker = CorePicker::new(
        Platform::Gba,
        &[CoreId::Mgba, CoreId::Gpsp],
        Some(CoreId::Mgba),
    );

    let uploads = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
            .count()
    };

    picker.draw(&mut c, &mut warm);
    assert!(
        uploads(&c.ops) > 0,
        "the first frame uploaded nothing at all"
    );

    let before = c.ops.len();
    for _ in 0..8 {
        picker.draw(&mut c, &mut warm);
    }
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "eight unchanged frames uploaded something"
    );

    // Walking is not new text: no row label changes with the highlight, and the badge belongs
    // to the running core rather than to the row the player is on.
    let before = c.ops.len();
    for _ in 0..2 {
        picker.down();
        picker.draw(&mut c, &mut warm);
        picker.up();
        picker.draw(&mut c, &mut warm);
    }
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "moving the highlight uploaded something"
    );

    // An empty picker has words of its own, and they settle the same way.
    let mut empty_ctx = ctx("rgsp", "ko");
    let mut empty_canvas = RecordingCanvas::new(pw, ph);
    let empty = CorePicker::new(Platform::Gba, &[], None);
    empty.draw(&mut empty_canvas, &mut empty_ctx);
    assert!(
        uploads(&empty_canvas.ops) > 0,
        "the empty frame uploaded nothing"
    );
    let before = empty_canvas.ops.len();
    for _ in 0..4 {
        empty.draw(&mut empty_canvas, &mut empty_ctx);
    }
    assert_eq!(uploads(&empty_canvas.ops[before..]), 0);
}

#[test]
fn the_panel_has_room_for_the_longest_candidate_list() {
    // The box is laid out for the longest list any platform has, so two rows fit with the
    // notice and hints below them and none of the three collide.
    assert_eq!(
        MAX_ROWS,
        PLATFORMS
            .iter()
            .map(|p| supported_cores(*p).len())
            .max()
            .unwrap()
    );
    let mut ctx = ctx("rgsp", "ko");
    let (_, by) = CorePicker::box_origin(&ctx);
    let last_row_bottom = CorePicker::row_y(&ctx, MAX_ROWS - 1) + ROW_H;
    let line_h = ctx.fonts.measure("", PX_HINT).line_height as f32;
    assert!(
        CorePicker::notice_y(&ctx) >= last_row_bottom,
        "the notice starts above the last row"
    );
    assert!(
        CorePicker::notice_y(&ctx) + line_h <= by + BOX_H - PAD - PX_HINT,
        "the notice reaches the hints"
    );
}
