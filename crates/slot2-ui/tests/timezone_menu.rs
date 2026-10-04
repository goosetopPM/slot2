//! Contract for the time zone screen: a fixed offset in minutes, moved a quarter-hour or an
//! hour at a time, drawn from the packs and never from a string glued together in code.

use std::path::PathBuf;

use slot2_gfx::{Color, Op, RecordingCanvas};
use slot2_i18n::{Arg, Span};
use slot2_platform::by_target;
use slot2_store::{UTC_OFFSET_MINUTES_MAX, UTC_OFFSET_MINUTES_MIN};
use slot2_ui::splash::{BACKDROP, INK, INK_DIM};
use slot2_ui::timezone_menu::{
    format_offset, ADJUST_Y, BOX_H, BOX_W, DEFAULT_MINUTES, DIM, HINTS_Y, HINT_ADJUST_KEY,
    HINT_APPLY_KEY, HINT_CANCEL_KEY, MAX_MINUTES, MIN_MINUTES, NOTE_KEY, NOTE_Y, PAD, STEP_HOURS,
    STEP_MINUTES, TITLE_KEY, TITLE_Y, VALUE_KEY, VALUE_Y,
};
use slot2_ui::{face, TimezoneMenu, UiCtx, PX_BODY, PX_HINT, PX_TITLE};

const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn frame(menu: &TimezoneMenu, ctx: &mut UiCtx, target: &str) -> RecordingCanvas {
    let (w, h) = by_target(target).unwrap().geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    menu.draw(&mut c, ctx);
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

/// Whether a one-run message is drawn, whole, with `tint`, in the band `y .. y + h`.
fn drawn(
    ops: &[Op],
    ctx: &mut UiCtx,
    key: &str,
    args: &[(&str, Arg)],
    px: f32,
    tint: Color,
    band: (f32, f32),
) -> bool {
    let (y, h) = band;
    let want = width(ctx, key, args, px);
    images(ops).iter().any(|(_, iy, iw, _, ic)| {
        (*iw - want).abs() < 0.5 && *ic == tint && *iy >= y && *iy < y + h
    })
}

/// Whether every part of a hint is drawn in the band `y .. y + h`: each text run as its own
/// quad, each button as its cap. A hint whose button id a pack misspells would be one text run
/// containing `[?up]`, and both widths would still be found — what this catches is a line drawn
/// outside the band it claims.
fn hint_in(ops: &[Op], ctx: &mut UiCtx, key: &str, band: (f32, f32)) -> bool {
    let (y, h) = band;
    let spans = ctx.i18n.spans(key, &[]);
    let line_h = ctx.fonts.measure("", PX_HINT).line_height as f32;
    let mut runs = 0;
    for span in &spans {
        match span {
            Span::Text(t) => {
                runs += 1;
                let w = face::measure(ctx, t, PX_HINT);
                if !images(ops)
                    .iter()
                    .any(|(_, iy, iw, _, _)| (*iw - w).abs() < 0.5 && *iy >= y && *iy < y + h)
                {
                    return false;
                }
            }
            Span::Btn(_) => {
                if !rects(ops)
                    .iter()
                    .any(|(_, ry, _, rh, _)| (*ry - y).abs() < 0.5 && (*rh - line_h).abs() < 0.5)
                {
                    return false;
                }
            }
        }
    }
    runs > 0
}

fn uploads(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count()
}

// ---------------------------------------------------------------- model

#[test]
fn the_constructor_takes_the_cards_own_value_exactly() {
    // 541 is the point: a card edited by hand holds whole minutes, and the screen shows what
    // the file says rather than rounding it to the quarter-hour its own keys step by.
    for minutes in [0, 1, -1, 345, 540, -480, 541, MIN_MINUTES, MAX_MINUTES] {
        let menu = TimezoneMenu::new(minutes);
        assert_eq!(menu.original(), minutes, "{minutes} did not round-trip");
        assert_eq!(menu.selected(), minutes, "{minutes} did not round-trip");
        assert!(!menu.changed(), "{minutes} opened already changed");
    }
    assert_eq!(
        MIN_MINUTES, UTC_OFFSET_MINUTES_MIN,
        "the screen's west end is not the card's"
    );
    assert_eq!(
        MAX_MINUTES, UTC_OFFSET_MINUTES_MAX,
        "the screen's east end is not the card's"
    );
    assert_eq!(DEFAULT_MINUTES, 0);
}

#[test]
fn a_value_the_card_cannot_hold_is_clamped_to_the_nearer_end() {
    for (given, want) in [
        (MIN_MINUTES - 1, MIN_MINUTES),
        (MAX_MINUTES + 1, MAX_MINUTES),
        (-100_000, MIN_MINUTES),
        (100_000, MAX_MINUTES),
        (i32::MIN, MIN_MINUTES),
        (i32::MAX, MAX_MINUTES),
    ] {
        let menu = TimezoneMenu::new(given);
        assert_eq!(menu.original(), want, "{given} was not clamped");
        assert_eq!(menu.selected(), want, "{given} was not clamped");
    }
}

#[test]
fn left_and_right_move_a_quarter_hour_each() {
    let mut menu = TimezoneMenu::new(0);
    menu.right();
    assert_eq!(menu.selected(), STEP_MINUTES);
    menu.right();
    assert_eq!(menu.selected(), 2 * STEP_MINUTES);
    menu.left();
    assert_eq!(menu.selected(), STEP_MINUTES);
    menu.left();
    menu.left();
    assert_eq!(menu.selected(), -STEP_MINUTES);

    // Navigation is not a save: what the card holds is where the screen opened.
    assert_eq!(menu.original(), 0);
    assert!(menu.changed());

    // And a value that comes back is not a change any more.
    menu.right();
    assert_eq!(menu.selected(), 0);
    assert!(!menu.changed());
}

#[test]
fn up_and_down_move_a_whole_hour_east_and_west() {
    let mut menu = TimezoneMenu::new(0);
    let before = menu.selected();
    menu.up();
    assert_eq!(menu.selected(), STEP_HOURS);
    assert!(menu.selected() > before, "up must be the larger value");
    menu.up();
    assert_eq!(menu.selected(), 2 * STEP_HOURS);
    menu.down();
    assert_eq!(menu.selected(), STEP_HOURS);
    menu.down();
    menu.down();
    assert_eq!(menu.selected(), -STEP_HOURS);
    assert_eq!(menu.original(), 0);

    // Nine presses up is KST, which is the whole point of the hour step.
    let mut kst = TimezoneMenu::new(0);
    for _ in 0..9 {
        kst.up();
    }
    assert_eq!(kst.selected(), 540);
}

#[test]
fn both_ends_stop_rather_than_wrap() {
    let mut east = TimezoneMenu::new(MAX_MINUTES);
    for _ in 0..20 {
        east.right();
        east.up();
    }
    assert_eq!(east.selected(), MAX_MINUTES, "the east end wrapped");
    let mut west = TimezoneMenu::new(MIN_MINUTES);
    for _ in 0..20 {
        west.left();
        west.down();
    }
    assert_eq!(west.selected(), MIN_MINUTES, "the west end wrapped");

    // Both ends are still reachable from inside, and leaving one works in either direction.
    let mut walk = TimezoneMenu::new(0);
    for _ in 0..100 {
        walk.up();
    }
    assert_eq!(walk.selected(), MAX_MINUTES);
    walk.left();
    assert_eq!(walk.selected(), MAX_MINUTES - STEP_MINUTES);
    walk.down();
    assert_eq!(walk.selected(), MAX_MINUTES - STEP_MINUTES - STEP_HOURS);
    for _ in 0..100 {
        walk.down();
    }
    assert_eq!(walk.selected(), MIN_MINUTES);
}

#[test]
fn an_offset_is_written_signed_with_two_digit_hours_and_minutes() {
    for (minutes, want) in [
        (0, "+00:00"),
        (1, "+00:01"),
        (-1, "-00:01"),
        (15, "+00:15"),
        (-45, "-00:45"),
        (60, "+01:00"),
        (-60, "-01:00"),
        (599, "+09:59"),
        (540, "+09:00"),
        (-480, "-08:00"),
        (345, "+05:45"),
        (-720, "-12:00"),
        (840, "+14:00"),
        (MIN_MINUTES, "-12:00"),
        (MAX_MINUTES, "+14:00"),
        // Out of range clamps the same way the model does, so a formatter call can never print
        // a time the machine will not use.
        (9999, "+14:00"),
        (-9999, "-12:00"),
        (i32::MIN, "-12:00"),
        (i32::MAX, "+14:00"),
    ] {
        let got = format_offset(minutes);
        assert_eq!(got, want, "{minutes} was written as {got}");
        assert_eq!(
            got.len(),
            6,
            "{minutes} is not sign + two digits + two digits"
        );
        // The three letters in front of the number belong to the message, not to this.
        assert!(!got.contains("UTC"), "{minutes} brought its own word");
    }
}

// ---------------------------------------------------------------- drawing

#[test]
fn drawing_dims_then_panels_and_keeps_every_thing_in_the_safe_area() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            // A value off the round numbers, so the drawn text is the four digits it says.
            let menu = TimezoneMenu::new(345);
            let mut c = RecordingCanvas::new(pw, ph);
            menu.draw(&mut c, &mut ctx);
            let ops = c.frame();
            let case = format!("{target}/{lang}");

            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{case}: the menu cleared the shelf underneath"
            );
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

            // The box is centred in the safe area — the region the rest of the UI is laid out
            // in — and not on the panel, so a 720x720 keeps it where the layout says.
            let (bx, by) = TimezoneMenu::box_origin(&ctx);
            assert_eq!(bx, ctx.safe.px((640.0 - BOX_W) / 2.0), "{case}: box x");
            assert_eq!(by, ctx.safe.py((480.0 - BOX_H) / 2.0), "{case}: box y");
            assert!(
                ctx.safe.contains(bx, by, BOX_W, BOX_H),
                "{case}: the box left the safe area"
            );
            assert!(
                has_rect(ops, bx, by, BOX_W, BOX_H, BACKDROP),
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

            // Title, the current value, the note, and the two hint lines.
            assert!(
                drawn(
                    ops,
                    &mut ctx,
                    TITLE_KEY,
                    &[],
                    PX_BODY,
                    INK_DIM,
                    (by + TITLE_Y, 28.0)
                ),
                "{case}: no title"
            );
            assert!(
                drawn(
                    ops,
                    &mut ctx,
                    VALUE_KEY,
                    &[("offset", Arg::from(format_offset(menu.selected())))],
                    PX_TITLE,
                    INK,
                    (by + VALUE_Y, 40.0)
                ),
                "{case}: no current value"
            );
            assert!(
                drawn(
                    ops,
                    &mut ctx,
                    NOTE_KEY,
                    &[],
                    PX_BODY,
                    INK_DIM,
                    (by + NOTE_Y, 28.0)
                ),
                "{case}: no note"
            );
            assert!(
                hint_in(ops, &mut ctx, HINT_ADJUST_KEY, (by + ADJUST_Y, 24.0)),
                "{case}: no adjust hint"
            );
            assert!(
                hint_in(ops, &mut ctx, HINT_APPLY_KEY, (by + HINTS_Y, 24.0)),
                "{case}: no apply hint"
            );
            assert!(
                hint_in(ops, &mut ctx, HINT_CANCEL_KEY, (by + HINTS_Y, 24.0)),
                "{case}: no cancel hint"
            );

            // Every line fits the panel it is drawn in. A translation that outgrew the box has
            // to fail here rather than on the device.
            let inside = BOX_W - 2.0 * PAD;
            for (key, px) in [
                (TITLE_KEY, PX_BODY),
                (NOTE_KEY, PX_BODY),
                (HINT_ADJUST_KEY, PX_HINT),
            ] {
                let w = width(&mut ctx, key, &[], px);
                assert!(
                    w <= inside,
                    "{case}: {key} is {w} wide and the panel's inside is {inside}"
                );
            }
            let w_value = width(
                &mut ctx,
                VALUE_KEY,
                &[("offset", Arg::from(format_offset(menu.selected())))],
                PX_TITLE,
            );
            assert!(w_value <= inside, "{case}: the value is {w_value} wide");
            let hints = width(&mut ctx, HINT_APPLY_KEY, &[], PX_HINT)
                + width(&mut ctx, HINT_CANCEL_KEY, &[], PX_HINT)
                + PX_HINT;
            assert!(hints <= inside, "{case}: the two choices are {hints} wide");
        }
    }
}

#[test]
fn the_value_is_the_localized_message_and_not_a_string_built_here() {
    // The offset is the message's argument and the pack owns the word in front of it. What can
    // be checked from outside is that the drawn run is exactly the message at that argument's
    // width and in the value's own ink — a screen that glued `UTC` to the number itself would
    // still have to agree with the pack here, which is the whole point of the argument.
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            for minutes in [0, 345, 540, -480] {
                let menu = TimezoneMenu::new(minutes);
                let (_, by) = TimezoneMenu::box_origin(&ctx);
                let mut c = frame(&menu, &mut ctx, target);
                let case = format!("{target}/{lang}/{minutes}");
                let want = width(
                    &mut ctx,
                    VALUE_KEY,
                    &[("offset", Arg::from(format_offset(minutes)))],
                    PX_TITLE,
                );
                let line = c
                    .ops
                    .iter()
                    .filter_map(|o| match o {
                        Op::Image { x, y, w, tint, .. } => Some((*x, *y, *w, *tint)),
                        _ => None,
                    })
                    .find(|(_, y, _, _)| (*y - (by + VALUE_Y)).abs() < 0.5);
                let (x, _, w, tint) = line.unwrap_or_else(|| panic!("{case}: no value line"));
                assert_eq!(w, want, "{case}: the value is not the message's own text");
                assert_eq!(tint, INK, "{case}: the value is not the line's bright ink");
                let (bx, _) = TimezoneMenu::box_origin(&ctx);
                assert_eq!(
                    x,
                    bx + (BOX_W - want) / 2.0,
                    "{case}: the value is not centred"
                );
                c.ops.clear();
            }
        }
    }
}

#[test]
fn a_moved_value_draws_nothing_the_opened_one_did_not() {
    // Whether the value differs from what the card holds is the caller's business, not the
    // screen's: no star, no colour, no extra word may appear with `changed()`.
    let mut ctx = ctx("rgsp", "en");
    let opened = frame(&TimezoneMenu::new(0), &mut ctx, "rgsp");
    let mut menu = TimezoneMenu::new(0);
    menu.up();
    assert!(menu.changed());
    let moved = frame(&menu, &mut ctx, "rgsp");
    // The shape of the screen: the quads, variant by variant, with the one-off texture uploads
    // left out — the first frame warms faces the second frame then finds. A marker for a moved
    // value would be one more quad and show up here whatever it was drawn in or coloured.
    let shape = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
            .map(std::mem::discriminant)
            .collect::<Vec<std::mem::Discriminant<Op>>>()
    };
    assert_eq!(
        shape(&opened.ops),
        shape(&moved.ops),
        "a moved value drew a different shape of screen"
    );

    // And back on the value the card holds, the state is un-changed again.
    menu.down();
    assert!(!menu.changed());
    assert_eq!(menu.selected(), menu.original());
}

#[test]
fn a_warm_menu_uploads_nothing_while_the_screen_says_the_same_thing() {
    // Korean, so the CJK face is part of what has to settle and not only the Latin one.
    let mut ctx = ctx("rgsp", "ko");
    let (pw, ph) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(pw, ph);
    let mut menu = TimezoneMenu::new(0);

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

    // A value the screen has never shown is a new string, so it costs its own face...
    let before = c.ops.len();
    menu.right();
    menu.draw(&mut c, &mut ctx);
    assert!(
        uploads(&c.ops[before..]) > 0,
        "a value the screen had never shown cost nothing"
    );

    // ...and coming back to one it has already drawn costs nothing at all.
    let before = c.ops.len();
    menu.left();
    menu.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "returning to a value already on screen uploaded something"
    );

    // Another step out and back: the first of the two values is new to the screen and costs
    // its face, and the one it comes back to is not drawn again.
    let before = c.ops.len();
    menu.up();
    menu.draw(&mut c, &mut ctx);
    let cost = uploads(&c.ops[before..]);
    let before = c.ops.len();
    menu.down();
    menu.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "coming back to a value already on screen uploaded something, after {cost} for the new one"
    );

    // The note and the hints never change with the value, so moving back onto a value the
    // screen has already drawn costs nothing either: +00:15 was on screen earlier.
    let before = c.ops.len();
    menu.right();
    menu.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "a value already drawn was uploaded again"
    );
}

#[test]
fn both_packs_carry_all_six_keys() {
    // The i18n crate's own test pins the exact wording; this one is here so a key the drawing
    // asks for cannot quietly become the `[key]` marker on a real screen.
    let en = slot2_i18n::I18n::embedded("en").unwrap();
    let ko = slot2_i18n::I18n::embedded("ko").unwrap();
    for key in [
        TITLE_KEY,
        VALUE_KEY,
        NOTE_KEY,
        HINT_ADJUST_KEY,
        HINT_APPLY_KEY,
        HINT_CANCEL_KEY,
    ] {
        for pack in [&en, &ko] {
            assert_ne!(pack.t(key), format!("[{key}]"), "{key} is missing");
        }
    }
    // Five of the six are translated; the value message is the same in both because it is the
    // offset itself with a word both packs spell the same way.
    for key in [
        TITLE_KEY,
        NOTE_KEY,
        HINT_ADJUST_KEY,
        HINT_APPLY_KEY,
        HINT_CANCEL_KEY,
    ] {
        assert_ne!(en.t(key), ko.t(key), "{key} is not translated");
    }
}
