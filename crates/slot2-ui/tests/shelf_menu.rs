//! Contract for the shelf settings menu: six rows in a settled order, the openable one
//! highlighted, and the rest drawn as what they are.

use std::path::PathBuf;

use slot2_gfx::{Canvas, Color, Op, RecordingCanvas};
use slot2_platform::by_target;
use slot2_ui::layout::SafeArea;
use slot2_ui::shelf_menu::{
    BOX_H, BOX_W, DIM, HIGHLIGHT_ALPHA, HINT_BACK_KEY, HINT_SELECT_KEY, INSET, PAD, ROW_H,
    SHELF_CHOICES, TITLE_KEY, UNAVAILABLE_KEY,
};
use slot2_ui::splash::{BACKDROP, INK, INK_DIM};
use slot2_ui::{
    face, ShelfAvailability, ShelfChoice, ShelfMenu, UiCtx, PX_BODY, PX_HINT, PX_TITLE,
};

/// The two panels this frontend ships, and the wider one the menu has to survive: no target has
/// a 1280x720 panel, so that one is a synthesised panel with the same 640x480 safe area every
/// geometry places at its centre.
const PANELS: [&str; 3] = ["rg35xxsp", "rgsp", "1280x720"];

fn all_available() -> ShelfAvailability {
    ShelfAvailability {
        language: true,
        display_defaults: true,
        boot_logo: true,
        sync: true,
        time_zone: true,
        about: true,
    }
}

/// A context and a canvas for a panel. The context's safe area is the layout input the menu
/// reads, so the synthesised panel is a real one as far as the drawing is concerned.
fn ctx_and_canvas(panel: &str, lang: &str) -> (UiCtx, RecordingCanvas) {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    if panel == "1280x720" {
        let mut ctx = UiCtx::new(by_target("rgsp").unwrap(), lang, vec![fonts], None);
        ctx.safe = SafeArea {
            x: 320,
            y: 120,
            panel_w: 1280,
            panel_h: 720,
        };
        (ctx, RecordingCanvas::new(1280, 720))
    } else {
        let profile = by_target(panel).unwrap();
        let (w, h) = profile.geometry.size();
        (
            UiCtx::new(profile, lang, vec![fonts], None),
            RecordingCanvas::new(w, h),
        )
    }
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

/// Everything the frame draws, in order, without the uploads: a screen that repeats itself
/// leaves this list unchanged, and one that draws a marker for a state does not.
fn drawn_quads(ops: &[Op]) -> Vec<(f32, f32, f32, f32, Color)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
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

/// The width a message draws at.
fn width(ctx: &mut UiCtx, key: &str, px: f32) -> f32 {
    let spans = ctx.i18n.spans(key, &[]);
    face::spans_width(ctx, &spans, px)
}

/// Whether a one-run message is drawn, whole, with `tint`, in the band `y .. y + h`.
fn drawn(ops: &[Op], ctx: &mut UiCtx, key: &str, px: f32, tint: Color, band: (f32, f32)) -> bool {
    let (y, h) = band;
    let want = width(ctx, key, px);
    images(ops).iter().any(|(_, iy, iw, _, ic)| {
        (*iw - want).abs() < 0.5 && *ic == tint && *iy >= y && *iy < y + h
    })
}

/// The hint line's quads, left to right, as `(x, w)`: one button cap and one word per hint.
/// Counting them and comparing where the line starts is what tells one hint from two — two
/// words of the same width (two two-glyph Korean words) would pass a width-only test.
fn hint_line_quads(ops: &[Op], ctx: &UiCtx) -> Vec<(f32, f32)> {
    let (y, h) = hint_band(ctx);
    let mut v: Vec<(f32, f32)> = rects(ops)
        .iter()
        .filter(|(_, ry, _, _, _)| *ry >= y && *ry < y + h)
        .map(|(x, _, w, _, _)| (*x, *w))
        .collect();
    v.extend(
        images(ops)
            .iter()
            .filter(|(_, iy, _, _, _)| *iy >= y && *iy < y + h)
            .map(|(x, _, w, _, _)| (*x, *w)),
    );
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite hint positions"));
    v
}

/// Every button cap in the hint line, as `(x, w, h)`.
fn hint_caps(ops: &[Op], ctx: &UiCtx) -> Vec<(f32, f32, f32)> {
    let (y, h) = hint_band(ctx);
    rects(ops)
        .iter()
        .filter(|(_, ry, _, _, _)| *ry >= y && *ry < y + h)
        .map(|(x, _, w, rh, _)| (*x, *w, *rh))
        .collect()
}

fn uploads(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count()
}

/// The band the hint line occupies, measured the way the menu measures it.
fn hint_band(ctx: &UiCtx) -> (f32, f32) {
    let (_, by) = ShelfMenu::box_origin(ctx);
    (by + BOX_H - PAD - PX_HINT, 24.0)
}

// ---------------------------------------------------------------- the list and the selection

#[test]
fn the_rows_are_the_six_choices_in_the_settled_order() {
    assert_eq!(
        SHELF_CHOICES,
        [
            ShelfChoice::Language,
            ShelfChoice::DisplayDefaults,
            ShelfChoice::BootLogo,
            ShelfChoice::Sync,
            ShelfChoice::TimeZone,
            ShelfChoice::About,
        ]
    );
    // Six rows, six labels, no row twice: a duplicate would silently drop another entry off the
    // menu, and the count is what says every variant is on the list.
    for (i, a) in SHELF_CHOICES.iter().enumerate() {
        for b in &SHELF_CHOICES[i + 1..] {
            assert_ne!(a, b, "{a:?} is on the menu twice");
        }
    }
    for (i, a) in SHELF_CHOICES.iter().enumerate() {
        for b in &SHELF_CHOICES[i + 1..] {
            assert_ne!(a.key(), b.key(), "{a:?} and {b:?} share a label");
        }
    }
    // The time zone row is the time zone screen's own title, so the two cannot drift apart.
    assert_eq!(
        ShelfChoice::TimeZone.key(),
        slot2_ui::timezone_menu::TITLE_KEY
    );
}

#[test]
fn the_menu_opens_on_the_first_openable_row() {
    for (available, want) in [
        (all_available(), ShelfChoice::Language),
        (ShelfAvailability::timezone_only(), ShelfChoice::TimeZone),
        (
            ShelfAvailability {
                display_defaults: true,
                ..Default::default()
            },
            ShelfChoice::DisplayDefaults,
        ),
        (
            ShelfAvailability {
                sync: true,
                about: true,
                ..Default::default()
            },
            ShelfChoice::Sync,
        ),
        (
            ShelfAvailability {
                about: true,
                ..Default::default()
            },
            ShelfChoice::About,
        ),
    ] {
        let menu = ShelfMenu::new(available);
        assert_eq!(menu.selected(), Some(want), "{available:?}");
        assert!(menu.is_available(want));
    }
}

#[test]
fn the_highlight_skips_what_cannot_be_opened_and_wraps_both_ways() {
    let available = ShelfAvailability {
        language: true,
        time_zone: true,
        ..Default::default()
    };
    let mut menu = ShelfMenu::new(available);
    assert_eq!(menu.selected(), Some(ShelfChoice::Language));

    menu.down();
    assert_eq!(
        menu.selected(),
        Some(ShelfChoice::TimeZone),
        "down did not skip the four rows between the openable two"
    );
    menu.down();
    assert_eq!(
        menu.selected(),
        Some(ShelfChoice::Language),
        "down did not wrap past the last row"
    );
    menu.up();
    assert_eq!(
        menu.selected(),
        Some(ShelfChoice::TimeZone),
        "up did not wrap back over the first row"
    );
    menu.up();
    assert_eq!(menu.selected(), Some(ShelfChoice::Language));

    // Wherever the keys leave it, the highlight is on a row that can be opened.
    for _ in 0..12 {
        menu.down();
        let step = menu.selected().expect("there is an openable row");
        assert!(
            menu.is_available(step),
            "{step:?} was highlighted with nothing behind it"
        );
        assert_ne!(step, ShelfChoice::BootLogo);
    }
}

#[test]
fn one_openable_row_and_none_at_all_are_both_safe() {
    let mut one = ShelfMenu::new(ShelfAvailability::timezone_only());
    for _ in 0..4 {
        one.up();
        assert_eq!(one.selected(), Some(ShelfChoice::TimeZone));
        one.down();
        assert_eq!(one.selected(), Some(ShelfChoice::TimeZone));
    }

    let mut none = ShelfMenu::new(ShelfAvailability::default());
    assert_eq!(none.selected(), None);
    for choice in SHELF_CHOICES {
        assert!(!none.is_available(choice), "{choice:?} came from nowhere");
    }
    for _ in 0..4 {
        none.up();
        none.down();
        assert_eq!(
            none.selected(),
            None,
            "a row was highlighted with none openable"
        );
    }
}

#[test]
fn timezone_only_opens_the_time_zone_and_nothing_else() {
    let available = ShelfAvailability::timezone_only();
    for choice in SHELF_CHOICES {
        let want = choice == ShelfChoice::TimeZone;
        assert_eq!(available.is_available(choice), want, "{choice:?}");
    }
    let menu = ShelfMenu::new(available);
    assert_eq!(menu.selected(), Some(ShelfChoice::TimeZone));
    for choice in SHELF_CHOICES {
        assert_eq!(menu.is_available(choice), choice == ShelfChoice::TimeZone);
    }
}

// ---------------------------------------------------------------- drawing

#[test]
fn drawing_dims_then_panels_and_keeps_every_thing_in_the_safe_area() {
    for panel in PANELS {
        for lang in ["en", "ko"] {
            let (mut ctx, mut c) = ctx_and_canvas(panel, lang);
            // Everything openable, so the menu is as full as it gets: six labels at their
            // widest and both hints.
            let menu = ShelfMenu::new(all_available());
            menu.draw(&mut c, &mut ctx);
            let ops = c.frame();
            let case = format!("{panel}/{lang}");
            let (pw, ph) = (c.size().0 as f32, c.size().1 as f32);

            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{case}: the menu cleared the shelf underneath"
            );
            match ops
                .iter()
                .find(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
            {
                Some(Op::Rect { x, y, w, h, color }) => {
                    assert_eq!(
                        (*x, *y, *w, *h),
                        (0.0, 0.0, pw, ph),
                        "{case}: the dim is not the whole panel"
                    );
                    assert_eq!(*color, DIM, "{case}: the panel was not dimmed first");
                }
                other => panic!("{case}: the first thing drawn was not the dim: {other:?}"),
            }

            let (bx, by) = ShelfMenu::box_origin(&ctx);
            assert!(
                ctx.safe.contains(bx, by, BOX_W, BOX_H),
                "{case}: the box left the safe area"
            );
            assert!(
                has_rect(ops, bx, by, BOX_W, BOX_H, BACKDROP),
                "{case}: the panel is not where the layout says"
            );
            // The page and every word on it is inside the safe area. Only the deliberate
            // full-panel dim is anywhere else.
            for op in ops {
                let (x, y, w, h) = match op {
                    Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                    _ => continue,
                };
                if (x, y, w, h) == (0.0, 0.0, pw, ph) {
                    continue;
                }
                assert!(ctx.safe.contains(x, y, w, h), "{case}: {op:?} left it");
            }

            assert!(
                drawn(ops, &mut ctx, TITLE_KEY, PX_BODY, INK_DIM, (by + PAD, 28.0)),
                "{case}: no title"
            );
            for (i, choice) in SHELF_CHOICES.iter().enumerate() {
                let row_y = ShelfMenu::row_y(&ctx, i);
                assert!(
                    drawn(ops, &mut ctx, choice.key(), PX_TITLE, INK, (row_y, ROW_H)),
                    "{case}: no label on row {i}"
                );
                assert!(
                    !drawn(
                        ops,
                        &mut ctx,
                        UNAVAILABLE_KEY,
                        PX_BODY,
                        INK_DIM,
                        (row_y, ROW_H)
                    ),
                    "{case}: an openable row {i} says it is unavailable"
                );
            }

            // Exactly one highlight, on the openable row the menu says is selected.
            let selected = menu.selected().expect("every row is openable here");
            let index = SHELF_CHOICES
                .iter()
                .position(|choice| *choice == selected)
                .unwrap();
            let backing = INK.with_alpha(HIGHLIGHT_ALPHA);
            let highlights = rects(ops)
                .iter()
                .filter(|(_, _, w, h, color)| {
                    (*w - (BOX_W - 2.0 * PAD)).abs() < 0.5
                        && (*h - ROW_H).abs() < 0.5
                        && *color == backing
                })
                .count();
            assert_eq!(highlights, 1, "{case}: not exactly one highlighted row");
            assert!(
                has_rect(
                    ops,
                    bx + PAD,
                    ShelfMenu::row_y(&ctx, index),
                    BOX_W - 2.0 * PAD,
                    ROW_H,
                    backing
                ),
                "{case}: the highlight is not on the selected row"
            );

            // Both hints on one line, centred, with the menu's own cap convention and a width
            // that fits inside the panel.
            let gap = PX_HINT * 0.3;
            let total_hints = width(&mut ctx, HINT_SELECT_KEY, PX_HINT)
                + PX_HINT
                + width(&mut ctx, HINT_BACK_KEY, PX_HINT);
            // Six quads: two button caps, their two labels, and the two words between them —
            // one hint is three, so this is what "both hints are on the line" looks like.
            let line = hint_line_quads(ops, &ctx);
            assert_eq!(line.len(), 6, "{case}: the hint line is not two hints");
            assert!(
                (line[0].0 - (bx + (BOX_W - total_hints) / 2.0 + gap)).abs() < 0.5,
                "{case}: the hint line is not centred where the layout says"
            );
            let line_h = ctx.fonts.measure("", PX_HINT).line_height as f32;
            let caps = hint_caps(ops, &ctx);
            assert_eq!(caps.len(), 2, "{case}: not two button caps");
            assert!(
                caps.iter().all(|(_, _, h)| (*h - line_h).abs() < 0.5),
                "{case}: a button cap is not the standard height"
            );
            assert!(
                total_hints <= BOX_W - 2.0 * PAD,
                "{case}: the hints are {total_hints} wide and the panel's inside is {}",
                BOX_W - 2.0 * PAD
            );
            // And every row's label fits beside the word, however long the translation is.
            for choice in SHELF_CHOICES {
                let label = width(&mut ctx, choice.key(), PX_TITLE);
                assert!(
                    label <= BOX_W - 2.0 * (PAD + INSET),
                    "{case}: {choice:?} is {label} wide"
                );
            }
        }
    }
}

#[test]
fn a_row_with_nothing_behind_it_is_dim_and_says_so() {
    let (mut ctx, mut c) = ctx_and_canvas("rgsp", "en");
    let menu = ShelfMenu::new(ShelfAvailability::timezone_only());
    menu.draw(&mut c, &mut ctx);
    let ops = c.frame().to_vec();
    let (bx, _) = ShelfMenu::box_origin(&ctx);

    for (i, choice) in SHELF_CHOICES.iter().enumerate() {
        let row_y = ShelfMenu::row_y(&ctx, i);
        let openable = menu.is_available(*choice);
        assert!(
            drawn(
                &ops,
                &mut ctx,
                choice.key(),
                PX_TITLE,
                if openable { INK } else { INK_DIM },
                (row_y, ROW_H)
            ),
            "row {i} is not drawn in the colour its state calls for"
        );

        let want_right = bx + BOX_W - PAD - INSET;
        let word_w = width(&mut ctx, UNAVAILABLE_KEY, PX_BODY);
        let says_so = images(&ops).iter().any(|(x, y, w, _, tint)| {
            (*x - (want_right - word_w)).abs() < 0.5
                && (*w - word_w).abs() < 0.5
                && *y >= row_y
                && *y < row_y + ROW_H
                && *tint == INK_DIM
        });
        assert_eq!(
            says_so, !openable,
            "row {i} ({choice:?}) says the wrong thing about being unavailable"
        );
    }
}

#[test]
fn a_menu_with_nothing_to_open_offers_no_choice() {
    let (mut ctx, mut c) = ctx_and_canvas("rg35xxsp", "ko");
    let menu = ShelfMenu::new(ShelfAvailability::default());
    menu.draw(&mut c, &mut ctx);
    let ops = c.frame().to_vec();
    let (bx, _) = ShelfMenu::box_origin(&ctx);

    // Nothing is highlighted: the backing is the promise that A does something, and there is
    // nothing here for it to do.
    let backing = INK.with_alpha(HIGHLIGHT_ALPHA);
    assert!(
        !rects(&ops)
            .iter()
            .any(|(_, _, _, _, color)| *color == backing),
        "a row was highlighted with nothing behind it"
    );
    // Every row is dim and says what it is.
    for (i, choice) in SHELF_CHOICES.iter().enumerate() {
        let row_y = ShelfMenu::row_y(&ctx, i);
        assert!(drawn(
            &ops,
            &mut ctx,
            choice.key(),
            PX_TITLE,
            INK_DIM,
            (row_y, ROW_H)
        ));
        assert!(
            drawn(
                &ops,
                &mut ctx,
                UNAVAILABLE_KEY,
                PX_BODY,
                INK_DIM,
                (row_y, ROW_H)
            ),
            "row {i} ({choice:?}) does not say it is unavailable"
        );
    }
    // The select hint goes with the highlight; the way out stays, and stays centred on its
    // own — a line carrying both hints would start further left and be four quads wide.
    let gap = PX_HINT * 0.3;
    let back_w = width(&mut ctx, HINT_BACK_KEY, PX_HINT);
    let hint_line = hint_line_quads(&ops, &ctx);
    assert_eq!(
        hint_line.len(),
        3,
        "the back hint is not alone on its line: three quads are a cap, its label and its word"
    );
    assert_eq!(
        hint_caps(&ops, &ctx).len(),
        1,
        "a second button cap appeared"
    );
    assert!(
        (hint_line[0].0 - (bx + (BOX_W - back_w) / 2.0 + gap)).abs() < 0.5,
        "the back hint is not where one hint on its own is centred"
    );
}

#[test]
fn the_same_menu_draws_the_same_screen_twice() {
    let (mut ctx, mut c) = ctx_and_canvas("rg35xxsp", "ko");
    let mut menu = ShelfMenu::new(ShelfAvailability::timezone_only());
    menu.draw(&mut c, &mut ctx);
    let first = c.frame().to_vec();
    assert!(
        uploads(&first) > 0,
        "the first frame uploaded nothing at all"
    );

    let before = c.ops.len();
    menu.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "a warm redraw uploaded something"
    );
    assert_eq!(
        drawn_quads(&c.ops[before..]),
        drawn_quads(&first),
        "the second frame drew something else"
    );

    // Moving and coming back is the same screen again: with one openable row the keys have
    // nowhere to go, and nothing about the frame may change either way.
    let before = c.ops.len();
    menu.down();
    menu.up();
    menu.draw(&mut c, &mut ctx);
    assert_eq!(
        drawn_quads(&c.ops[before..]),
        drawn_quads(&first),
        "the keys changed the screen without moving the highlight"
    );
}
