//! Contract for the Device submenu: three rows, one highlighted and usable, and bars that say
//! what the machine is actually doing.

use std::path::PathBuf;

use slot2_gfx::{Color, Op, RecordingCanvas};
use slot2_i18n::{Arg, Span};
use slot2_platform::by_target;
use slot2_ui::device_menu::{
    fill_color, track_color, BAR_H, BOX_H, BOX_W, DIM, HIGHLIGHT_ALPHA, HINT_ADJUST_KEY,
    HINT_BACK_KEY, HINT_MUTE_KEY, MUTED_KEY, PAD, PERCENT_KEY, ROWS, ROW_H, UNAVAILABLE_KEY,
};
use slot2_ui::splash::{BACKDROP, INK, INK_DIM};
use slot2_ui::{face, DeviceMenu, DeviceSetting, UiCtx, PX_BODY, PX_HINT, PX_TITLE};

const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn frame(menu: &DeviceMenu, ctx: &mut UiCtx, target: &str) -> RecordingCanvas {
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

/// Every text quad in a frame, as `(x, y, w, tint)`.
fn images(ops: &[Op]) -> Vec<(f32, f32, f32, Color)> {
    ops.iter()
        .filter_map(|o| match o {
            Op::Image { x, y, w, tint, .. } => Some((*x, *y, *w, *tint)),
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

/// The width of the fill drawn at the bar's own line, or `None` when no fill is there. The
/// track shares the line, so only the fill's ink counts.
fn fill_at(ops: &[Op], x: f32, y: f32, h: f32) -> Option<f32> {
    rects(ops).iter().find_map(|(rx, ry, rw, rh, rc)| {
        ((*rx - x).abs() < 0.5
            && (*ry - y).abs() < 0.5
            && (*rh - h).abs() < 0.5
            && (*rc == fill_color(false) || *rc == fill_color(true)))
        .then_some(*rw)
    })
}

/// The width a message draws at, arguments included.
fn width(ctx: &mut UiCtx, key: &str, args: &[(&str, Arg)], px: f32) -> f32 {
    let spans = ctx.i18n.spans(key, args);
    face::spans_width(ctx, &spans, px)
}

/// Whether a message is drawn with `tint` somewhere in the band `y .. y + h`.
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
    images(ops)
        .iter()
        .any(|(_, iy, iw, ic)| (*iw - want).abs() < 0.5 && *ic == tint && *iy >= y && *iy < y + h)
}

/// Whether every text quad of a message is drawn in the band `y .. y + h`. A `BTN` placeholder
/// is a cap plus its own label, so a hint made of several spans is several quads.
fn line_in(ops: &[Op], ctx: &mut UiCtx, key: &str, y: f32, h: f32) -> bool {
    let widths: Vec<f32> = ctx
        .i18n
        .spans(key, &[])
        .iter()
        .filter_map(|span| match span {
            Span::Text(text) => Some(face::spans_width(ctx, &[Span::Text(text.clone())], PX_HINT)),
            Span::Btn(_) => None,
        })
        .collect();
    !widths.is_empty()
        && widths.iter().all(|want| {
            images(ops)
                .iter()
                .any(|(_, iy, iw, _)| (*iw - *want).abs() < 0.5 && *iy >= y && *iy < y + h)
        })
}

/// Whether the highlighted row's backing is at `row_y`.
fn highlighted(ops: &[Op], row_y: f32, bx: f32) -> bool {
    ops.iter().any(|o| {
        matches!(o, Op::Rect { x, y, w, h, color }
        if (*x - (bx + PAD)).abs() < 0.5
            && (*y - row_y).abs() < 0.5
            && (*w - (BOX_W - 2.0 * PAD)).abs() < 0.5
            && (*h - ROW_H).abs() < 0.5
            && *color == INK.with_alpha(HIGHLIGHT_ALPHA))
    })
}

// ------------------------------------------------------------------ the model

#[test]
fn the_three_rows_are_the_three_settings_in_order() {
    assert_eq!(
        ROWS,
        [
            DeviceSetting::Volume,
            DeviceSetting::Brightness,
            DeviceSetting::BlueLight
        ]
    );
    let menu = DeviceMenu::new(70, false, Some(50), Some(30));
    assert_eq!(
        menu.selected(),
        DeviceSetting::Volume,
        "the menu did not open on its first row"
    );

    // Each row has words of its own: a shared key would put the same label on two rows.
    let mut keys: Vec<&str> = ROWS.iter().map(|row| row.key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), ROWS.len(), "{keys:?}");
}

#[test]
fn levels_are_clamped_and_a_setting_this_machine_lacks_stays_lacking() {
    let menu = DeviceMenu::new(120, false, Some(200), None);
    assert_eq!(menu.value(DeviceSetting::Volume), Some(100));
    assert_eq!(menu.value(DeviceSetting::Brightness), Some(100));
    assert_eq!(
        menu.value(DeviceSetting::BlueLight),
        None,
        "an unavailable setting was given a level"
    );
    assert!(!menu.is_available(DeviceSetting::BlueLight));
    assert!(
        menu.is_available(DeviceSetting::Volume),
        "the volume is always the frontend's to offer"
    );
    assert!(menu.is_available(DeviceSetting::Brightness));
}

#[test]
fn navigation_walks_only_the_available_rows() {
    // Everything is there: the walk is the row order, wrapping both ways.
    let mut menu = DeviceMenu::new(70, false, Some(50), Some(30));
    for want in [
        DeviceSetting::Brightness,
        DeviceSetting::BlueLight,
        DeviceSetting::Volume,
    ] {
        menu.down();
        assert_eq!(menu.selected(), want);
    }
    for want in [
        DeviceSetting::BlueLight,
        DeviceSetting::Brightness,
        DeviceSetting::Volume,
    ] {
        menu.up();
        assert_eq!(menu.selected(), want);
    }

    // Brightness is not on this machine: up and down go straight past it.
    let mut menu = DeviceMenu::new(70, false, None, Some(30));
    menu.down();
    assert_eq!(
        menu.selected(),
        DeviceSetting::BlueLight,
        "down landed on the row that is not there"
    );
    menu.down();
    assert_eq!(menu.selected(), DeviceSetting::Volume);
    let mut menu = DeviceMenu::new(70, false, None, Some(30));
    menu.up();
    assert_eq!(
        menu.selected(),
        DeviceSetting::BlueLight,
        "up landed on the row that is not there"
    );

    // Nothing but the volume: the walk has nowhere to go, and stays where the player can act.
    let mut menu = DeviceMenu::new(70, false, None, None);
    for _ in 0..3 {
        menu.up();
        assert_eq!(menu.selected(), DeviceSetting::Volume);
        menu.down();
        assert_eq!(menu.selected(), DeviceSetting::Volume);
    }
}

#[test]
fn the_setters_clamp_and_never_revive_a_row() {
    let mut menu = DeviceMenu::new(70, false, Some(50), None);
    menu.set_value(DeviceSetting::Volume, u8::MAX);
    assert_eq!(menu.value(DeviceSetting::Volume), Some(100));
    menu.set_value(DeviceSetting::Brightness, 120);
    assert_eq!(menu.value(DeviceSetting::Brightness), Some(100));

    // A value for a setting that is not there is a caller's mistake, and it stays one: the row
    // must not become a usable control behind the player's back.
    menu.set_value(DeviceSetting::BlueLight, 40);
    assert_eq!(menu.value(DeviceSetting::BlueLight), None);
    assert!(!menu.is_available(DeviceSetting::BlueLight));

    // Muting belongs to the volume and to nothing else, and it does not touch the level.
    menu.set_muted(true);
    assert!(menu.volume_muted());
    assert_eq!(menu.value(DeviceSetting::Volume), Some(100));
    menu.toggle_muted();
    assert!(!menu.volume_muted());
    assert_eq!(menu.value(DeviceSetting::Volume), Some(100));
}

// ------------------------------------------------------------------ the bars

#[test]
fn a_bar_is_as_long_as_its_level_and_never_leaves_its_track() {
    let mut ctx = ctx("rgsp", "en");
    let (bar_x, bar_y, track, bar_h) = DeviceMenu::bar_rect(&ctx, 0);
    assert_eq!(bar_h, BAR_H);
    assert!(track > 0.0, "a bar with no length is not a bar");

    // Zero is no fill at all, and a hundred is the whole track.
    let menu = DeviceMenu::new(0, false, None, None);
    let c = frame(&menu, &mut ctx, "rgsp");
    assert!(
        has_rect(c.frame(), bar_x, bar_y, track, BAR_H, track_color(true)),
        "the track is not drawn"
    );
    assert_eq!(fill_at(c.frame(), bar_x, bar_y, BAR_H), None);

    let menu = DeviceMenu::new(100, false, None, None);
    let c = frame(&menu, &mut ctx, "rgsp");
    assert_eq!(fill_at(c.frame(), bar_x, bar_y, BAR_H), Some(track));

    // Everything in between, including values past the end: the fill is the level and never
    // more than the track.
    for level in [1u8, 25, 50, 99, 101, u8::MAX] {
        let want = DeviceMenu::fill_width(&ctx, level);
        assert!(
            want > 0.0 && want <= track,
            "level {level} fills {want} of a {track} track"
        );
        let menu = DeviceMenu::new(level, false, None, None);
        let c = frame(&menu, &mut ctx, "rgsp");
        assert_eq!(
            fill_at(c.frame(), bar_x, bar_y, BAR_H),
            Some(want),
            "level {level} is not drawn {want} wide"
        );
    }
}

#[test]
fn a_muted_volume_keeps_its_bar_and_says_so() {
    let mut ctx = ctx("rgsp", "en");
    let menu = DeviceMenu::new(70, true, None, None);
    let (bar_x, bar_y, track, _) = DeviceMenu::bar_rect(&ctx, 0);
    let row_y = DeviceMenu::row_y(&ctx, 0);
    let c = frame(&menu, &mut ctx, "rgsp");

    // The remembered level is still the length of the fill, drawn dimmed: what is coming out of
    // the speaker is nothing, and the bar has to say so the same way the text does.
    let want = DeviceMenu::fill_width(&ctx, 70);
    assert_eq!(fill_at(c.frame(), bar_x, bar_y, BAR_H), Some(want));
    assert_ne!(fill_color(true), fill_color(false));
    assert!(has_rect(
        c.frame(),
        bar_x,
        bar_y,
        want,
        BAR_H,
        fill_color(true)
    ));
    assert!(
        !has_rect(c.frame(), bar_x, bar_y, want, BAR_H, fill_color(false)),
        "a muted bar was drawn as if it were playing"
    );

    // The text is the word, not a percentage that would read as sound.
    assert!(drawn(
        c.frame(),
        &mut ctx,
        MUTED_KEY,
        &[],
        PX_BODY,
        INK_DIM,
        (row_y, ROW_H)
    ));
    assert!(!drawn(
        c.frame(),
        &mut ctx,
        PERCENT_KEY,
        &[("value", Arg::from(70i64))],
        PX_BODY,
        INK,
        (row_y, ROW_H)
    ));
    assert!(track > 0.0);
}

#[test]
fn an_unavailable_row_is_dim_and_never_highlighted() {
    // Korean, so the words on screen are the pack's own rather than a fallback's.
    let mut ctx = ctx("rgsp", "ko");
    let mut menu = DeviceMenu::new(70, false, None, Some(30));
    let (bx, _) = DeviceMenu::box_origin(&ctx);
    let c = frame(&menu, &mut ctx, "rgsp");
    let ops = c.frame();

    // The row that cannot be used: a quieter track, no fill, the label and the word in dim ink.
    let (bar_x, bar_y, track, _) = DeviceMenu::bar_rect(&ctx, 1);
    let dim_row = DeviceMenu::row_y(&ctx, 1);
    assert!(
        has_rect(ops, bar_x, bar_y, track, BAR_H, track_color(false)),
        "the unavailable track is not drawn"
    );
    assert_ne!(track_color(false), track_color(true));
    assert_eq!(
        fill_at(ops, bar_x, bar_y, BAR_H),
        None,
        "an unavailable row drew a level"
    );
    assert!(drawn(
        ops,
        &mut ctx,
        UNAVAILABLE_KEY,
        &[],
        PX_BODY,
        INK_DIM,
        (dim_row, ROW_H)
    ));
    assert!(drawn(
        ops,
        &mut ctx,
        DeviceSetting::Brightness.key(),
        &[],
        PX_TITLE,
        INK_DIM,
        (dim_row, ROW_H)
    ));

    // Exactly one highlight, and it is never on that row.
    assert!(highlighted(ops, DeviceMenu::row_y(&ctx, 0), bx));
    assert!(!highlighted(ops, DeviceMenu::row_y(&ctx, 1), bx));
    assert!(!highlighted(ops, DeviceMenu::row_y(&ctx, 2), bx));

    // And walking the menu cannot land on it either.
    for _ in 0..4 {
        menu.down();
        assert_ne!(menu.selected(), DeviceSetting::Brightness);
        assert!(menu.is_available(menu.selected()));
    }
}

// ------------------------------------------------------------------ the hints

#[test]
fn the_mute_hint_is_only_on_the_volume_row() {
    let mut ctx = ctx("rgsp", "en");
    let (_, by) = DeviceMenu::box_origin(&ctx);
    let hint_y = by + BOX_H - PAD - PX_HINT;
    let band = PX_HINT * 2.0;

    let mut menu = DeviceMenu::new(70, false, None, Some(30));
    let c = frame(&menu, &mut ctx, "rgsp");
    assert!(
        line_in(c.frame(), &mut ctx, HINT_ADJUST_KEY, hint_y, band),
        "no adjust hint"
    );
    assert!(
        line_in(c.frame(), &mut ctx, HINT_BACK_KEY, hint_y, band),
        "no back hint"
    );
    assert!(
        line_in(c.frame(), &mut ctx, HINT_MUTE_KEY, hint_y, band),
        "the volume row does not say what A does"
    );

    // Away from the volume, A means nothing here: the hint for it is gone, and the two that are
    // still true stay.
    menu.down();
    assert_eq!(menu.selected(), DeviceSetting::BlueLight);
    let c = frame(&menu, &mut ctx, "rgsp");
    assert!(line_in(c.frame(), &mut ctx, HINT_ADJUST_KEY, hint_y, band));
    assert!(line_in(c.frame(), &mut ctx, HINT_BACK_KEY, hint_y, band));
    assert!(
        !line_in(c.frame(), &mut ctx, HINT_MUTE_KEY, hint_y, band),
        "a mute hint on a row that cannot be muted"
    );
}

// ------------------------------------------------------------------ drawing

#[test]
fn drawing_dims_then_panels_and_keeps_every_thing_in_the_safe_area() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            // Everything available, so every row has a label, a value, a bar and a hint is
            // drawn: the fullest this screen can be.
            let menu = DeviceMenu::new(70, false, Some(50), Some(30));
            let mut c = RecordingCanvas::new(pw, ph);
            menu.draw(&mut c, &mut ctx);
            let ops = c.frame();
            let case = format!("{target}/{lang}");

            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{case}: the menu cleared the game frame"
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
            // dim stays in the safe area: title, rows, labels, values, bars and hints.
            let (bx, by) = DeviceMenu::box_origin(&ctx);
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

            // The title, then each row's label, its value and its bar.
            assert!(
                drawn(
                    ops,
                    &mut ctx,
                    "ingame-device",
                    &[],
                    PX_BODY,
                    INK_DIM,
                    (by + PAD, 28.0)
                ),
                "{case}: no title"
            );
            for (i, setting) in ROWS.iter().enumerate() {
                let row_y = DeviceMenu::row_y(&ctx, i);
                assert!(
                    drawn(
                        ops,
                        &mut ctx,
                        setting.key(),
                        &[],
                        PX_TITLE,
                        INK,
                        (row_y, ROW_H)
                    ),
                    "{case}: no label on row {i}"
                );
                let level = menu.value(*setting).expect("every row is available here");
                assert!(
                    drawn(
                        ops,
                        &mut ctx,
                        PERCENT_KEY,
                        &[("value", Arg::from(i64::from(level)))],
                        PX_BODY,
                        INK,
                        (row_y, ROW_H)
                    ),
                    "{case}: no value on row {i}"
                );
                let (bar_x, bar_y, track, _) = DeviceMenu::bar_rect(&ctx, i);
                assert!(
                    has_rect(ops, bar_x, bar_y, track, BAR_H, track_color(true)),
                    "{case}: no bar on row {i}"
                );
            }

            // The hint line fits the panel it is drawn in: a longer translation has to fail
            // here rather than on the device.
            let hint_w = width(&mut ctx, HINT_ADJUST_KEY, &[], PX_HINT)
                + width(&mut ctx, HINT_MUTE_KEY, &[], PX_HINT)
                + width(&mut ctx, HINT_BACK_KEY, &[], PX_HINT)
                + 2.0 * PX_HINT;
            assert!(
                hint_w <= BOX_W - 2.0 * PAD,
                "{case}: the hints are {hint_w} wide and the panel's inside is {}",
                BOX_W - 2.0 * PAD
            );
        }
    }
}

#[test]
fn a_warm_menu_uploads_nothing_while_the_screen_says_the_same_thing() {
    // Korean, so the CJK face is part of what has to settle and not only the Latin one.
    let mut ctx = ctx("rgsp", "ko");
    let (pw, ph) = ctx.profile.geometry.size();
    let mut c = RecordingCanvas::new(pw, ph);
    let mut menu = DeviceMenu::new(70, false, Some(50), None);

    let uploads = |ops: &[Op]| {
        ops.iter()
            .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
            .count()
    };

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

    // Walking the rows is not new text: the labels are the same words, and the bar and the
    // highlight are geometry.
    let before = c.ops.len();
    for _ in 0..3 {
        menu.down();
        menu.draw(&mut c, &mut ctx);
    }
    for _ in 0..3 {
        menu.up();
        menu.draw(&mut c, &mut ctx);
    }
    assert_eq!(
        uploads(&c.ops[before..]),
        0,
        "moving the highlight uploaded something"
    );

    // A word the screen has never said costs the one texture that word needs — and nothing
    // else, because the bar, the highlight and the hints are geometry and cached text.
    let before = c.ops.len();
    menu.set_muted(true);
    menu.draw(&mut c, &mut ctx);
    let muted = uploads(&c.ops[before..]);
    assert!(muted <= 1, "one word cost {muted} textures");

    // Both of those words have now been on screen, and so have the levels 70 and 50: from here
    // the setters are free whatever they do to the state.
    let before = c.ops.len();
    for _ in 0..3 {
        menu.toggle_muted();
        menu.draw(&mut c, &mut ctx);
    }
    menu.set_value(DeviceSetting::Volume, 50);
    menu.draw(&mut c, &mut ctx);
    menu.set_value(DeviceSetting::Volume, 100);
    menu.draw(&mut c, &mut ctx);
    assert_eq!(
        uploads(&c.ops[before..]),
        1,
        "moving the highlight, muting and unmuting should cost nothing; only the level 100 \
         these frames showed for the first time may cost its own text"
    );
}
