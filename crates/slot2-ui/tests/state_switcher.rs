//! Contract for the state switcher: the numbered save states as polaroid cards over the
//! paused game.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use slot2_gfx::{Color, Op, RecordingCanvas, TexId};
use slot2_i18n::Span;
use slot2_platform::by_target;
use slot2_store::{StateKind, StateSlot};
use slot2_ui::splash::{INK, INK_DIM};
use slot2_ui::state_switcher::{
    BORDER, CACHE_MAX, CARD_H, CARD_W, DIM, HINT_Y, LABEL_INK, NO_ART, THUMB_H, THUMB_W,
};
use slot2_ui::{StateSwitcher, UiCtx, PX_HINT};

static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
const TARGETS: [&str; 3] = ["rgsp", "rg35xxsp", "rgcubexx"];

fn ctx(target: &str, lang: &str) -> UiCtx {
    let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
    UiCtx::new(by_target(target).unwrap(), lang, vec![fonts], None)
}

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("slot2-switcher-{tag}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// A PNG of one flat colour, `w` x `h`.
fn write_png(path: &Path, w: u32, h: u32) {
    let f = fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(&vec![0x80u8; (w * h * 4) as usize])
        .unwrap();
}

fn slot(number: u32, thumb: Option<PathBuf>, mtime: u64) -> StateSlot {
    StateSlot {
        kind: StateKind::Numbered(number),
        path: PathBuf::from(format!("{number}.state")),
        thumb,
        modified: SystemTime::UNIX_EPOCH + Duration::from_secs(mtime),
    }
}

fn resume(mtime: u64) -> StateSlot {
    StateSlot {
        kind: StateKind::Resume,
        path: PathBuf::from("resume.state"),
        thumb: None,
        modified: SystemTime::UNIX_EPOCH + Duration::from_secs(mtime),
    }
}

fn frame(switcher: &mut StateSwitcher, ctx: &mut UiCtx, target: &str) -> RecordingCanvas {
    let (w, h) = by_target(target).unwrap().geometry.size();
    let mut c = RecordingCanvas::new(w, h);
    switcher.draw(&mut c, ctx, false);
    c
}

fn upload_id(ops: &[Op]) -> Option<TexId> {
    ops.iter().find_map(|o| match o {
        Op::UploadRgba8 { id, .. } => Some(*id),
        _ => None,
    })
}

fn uploads(ops: &[Op]) -> Vec<TexId> {
    ops.iter()
        .filter_map(|o| match o {
            Op::UploadRgba8 { id, .. } => Some(*id),
            _ => None,
        })
        .collect()
}

/// The button caps on screen: one per hint, so counting them counts the hints.
fn caps(ops: &[Op]) -> usize {
    let cap = INK_DIM.with_alpha(0.25);
    ops.iter()
        .filter(|o| matches!(o, Op::Rect { color, .. } if *color == cap))
        .count()
}

/// The widths of the marks drawn on the hint line, sorted: for every hint, the word and the
/// label inside its button cap. That says *which* hints are there and not just how many.
fn hint_marks(c: &RecordingCanvas, ctx: &UiCtx) -> Vec<f32> {
    let top = ctx.safe.py(HINT_Y);
    let mut widths: Vec<f32> = c
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image { y, w, .. } if *y >= top - 0.5 && *y < top + 24.0 => Some(*w),
            _ => None,
        })
        .collect();
    widths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    widths
}

/// What those widths should be, from the messages themselves.
fn hint_marks_for(ctx: &mut UiCtx, keys: &[&str]) -> Vec<f32> {
    let mut widths: Vec<f32> = keys
        .iter()
        .flat_map(|key| {
            ctx.i18n
                .spans(key, &[])
                .into_iter()
                .map(|span| match span {
                    Span::Text(text) => slot2_ui::face::measure(ctx, &text, PX_HINT),
                    // The cap draws the button's own label inside it.
                    Span::Btn(button) => slot2_ui::face::measure(ctx, button.label(), PX_HINT),
                })
                .collect::<Vec<f32>>()
        })
        .collect();
    widths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    widths
}

fn image_rect(ops: &[Op], tex: TexId) -> Option<(f32, f32, f32, f32)> {
    ops.iter().find_map(|o| match o {
        Op::Image {
            tex: t, x, y, w, h, ..
        } if *t == tex => Some((*x, *y, *w, *h)),
        _ => None,
    })
}

fn plates(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o, Op::Rect { color, .. } if *color == NO_ART))
        .count()
}

// ------------------------------------------------------------------ what it holds

#[test]
fn resume_is_dropped_unordered_numbers_sort_and_the_greatest_is_selected() {
    // Deliberately out of order, with the newest mtime on the lowest number: the order the
    // slots arrive in and their mtimes both point away from the answer.
    let slots = vec![
        slot(3, None, 10),
        slot(1, None, 9_000),
        resume(9_500),
        slot(7, None, 20),
        slot(2, None, 30),
    ];
    let mut s = StateSwitcher::new(slots);

    assert_eq!(s.len(), 4, "the resume state is not a numbered slot");
    assert!(!s.is_empty());
    assert_eq!(s.selected_kind(), Some(StateKind::Numbered(7)));

    // Numeric order, read through the navigation: from 7 the way down is 3, 2, 1, and then
    // round to 7 again.
    for want in [3, 2, 1, 7] {
        s.left();
        assert_eq!(s.selected_kind(), Some(StateKind::Numbered(want)));
    }
}

#[test]
fn navigation_wraps_and_the_empty_and_single_cases_are_safe() {
    let mut none = StateSwitcher::new(vec![resume(1)]);
    assert!(none.is_empty());
    assert_eq!(none.len(), 0);
    assert_eq!(none.selected_kind(), None);
    none.left();
    none.right();
    assert_eq!(
        none.selected_kind(),
        None,
        "the empty case has nothing to select"
    );

    let mut one = StateSwitcher::new(vec![slot(4, None, 1)]);
    assert_eq!(one.len(), 1);
    for _ in 0..3 {
        one.left();
        one.right();
        assert_eq!(one.selected_kind(), Some(StateKind::Numbered(4)));
    }

    let mut three = StateSwitcher::new(vec![slot(1, None, 1), slot(2, None, 2), slot(3, None, 3)]);
    assert_eq!(three.selected_kind(), Some(StateKind::Numbered(3)));
    three.right();
    assert_eq!(
        three.selected_kind(),
        Some(StateKind::Numbered(1)),
        "right wraps to the first"
    );
    three.left();
    assert_eq!(
        three.selected_kind(),
        Some(StateKind::Numbered(3)),
        "left wraps to the last"
    );
    for want in [2, 1, 3] {
        three.left();
        assert_eq!(three.selected_kind(), Some(StateKind::Numbered(want)));
    }
}

// ------------------------------------------------------------------ what it draws

#[test]
fn drawing_never_clears_and_nothing_but_the_dim_leaves_the_safe_area() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            let mut s =
                StateSwitcher::new(vec![slot(1, None, 1), slot(2, None, 2), slot(3, None, 3)]);
            let mut c = RecordingCanvas::new(pw, ph);
            s.draw(&mut c, &mut ctx, false);
            let ops = c.frame();

            assert!(
                !ops.iter().any(|o| matches!(o, Op::Clear(_))),
                "{target}/{lang}: the switcher cleared the game frame"
            );

            // The dim over the whole physical panel is the one thing allowed outside the
            // safe area; a card, a label, a title, a hint and a placeholder all stay in it.
            let mut dims = 0;
            for op in ops {
                let (x, y, w, h, color) = match op {
                    Op::Rect { x, y, w, h, color } => (*x, *y, *w, *h, Some(*color)),
                    Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h, None),
                    _ => continue,
                };
                if (x, y, w, h) == (0.0, 0.0, pw as f32, ph as f32) {
                    assert_eq!(color, Some(DIM), "{target}/{lang}: the panel dim");
                    dims += 1;
                    continue;
                }
                assert!(
                    ctx.safe.contains(x, y, w, h),
                    "{target}/{lang}: {op:?} left the safe area"
                );
            }
            assert_eq!(dims, 1, "{target}/{lang}: the panel was not dimmed once");

            // The selected card is centred in the safe area, its neighbours fit beside it,
            // and all three are on the strip with a picture area and a plate of their own.
            let (cx, cy, cw, ch) = StateSwitcher::card_rect(&ctx, 0);
            assert_eq!((cw, ch), (CARD_W, CARD_H));
            assert!(
                (cx + CARD_W / 2.0 - ctx.safe.px(320.0)).abs() < 0.5,
                "{target}/{lang}: the selected card is not centred"
            );
            for (offset, ink) in [(0isize, INK), (-1, INK_DIM), (1, INK_DIM)] {
                let (x, y, w, h) = StateSwitcher::card_rect(&ctx, offset);
                assert!(
                    ops.iter().any(
                        |o| matches!(o, Op::Rect { x: ox, y: oy, w: ow, h: oh, color }
                        if (*ox - x).abs() < 0.5 && (*oy - y).abs() < 0.5
                            && (*ow - w).abs() < 0.5 && (*oh - h).abs() < 0.5 && *color == ink)
                    ),
                    "{target}/{lang}: no card at offset {offset}"
                );
                let (tx, ty, tw, th) = StateSwitcher::thumb_rect(&ctx, offset);
                assert_eq!((tw, th), (THUMB_W, THUMB_H));
                assert!(ctx.safe.contains(tx, ty, tw, th));
                assert!(
                    ops.iter()
                        .any(|o| matches!(o, Op::Rect { x: ox, y: oy, color, .. }
                        if (*ox - tx - 16.0).abs() < 0.5 && (*oy - ty - 16.0).abs() < 0.5
                            && *color == NO_ART)),
                    "{target}/{lang}: no placeholder in the card at offset {offset}"
                );
            }
            assert_eq!(plates(ops), 3, "{target}/{lang}: one plate per card");
            assert_eq!(cx, StateSwitcher::card_rect(&ctx, 0).0);
            assert!((cy + CARD_H / 2.0 - ctx.safe.py(250.0)).abs() < 0.5);
        }
    }
}

// ------------------------------------------------------------------ the number on the frame

/// The WCAG contrast ratio between two colours: 1.0 when they are the same, 4.5 or more for
/// text that can be read. Used on the slot number against the frame it is written on.
fn contrast(a: Color, b: Color) -> f32 {
    fn linear(v: f32) -> f32 {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    fn luminance(c: Color) -> f32 {
        0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
    }
    let (a, b) = (luminance(a), luminance(b));
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };
    (hi + 0.05) / (lo + 0.05)
}

#[test]
fn the_slot_number_is_dark_against_the_frame_it_is_written_on() {
    // The defect this closes: the frame was filled with the ink the number was then drawn in,
    // so the one thing a card exists to say — which slot it is — was invisible on the
    // selected card. Position alone does not catch that, so this asks about the tint.
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let mut s =
                StateSwitcher::new(vec![slot(1, None, 1), slot(2, None, 2), slot(3, None, 3)]);
            let c = frame(&mut s, &mut ctx, target);

            for (offset, frame_ink) in [(0isize, INK), (-1, INK_DIM), (1, INK_DIM)] {
                let (x, y, ..) = StateSwitcher::card_rect(&ctx, offset);
                let band = (y + BORDER + THUMB_H, y + CARD_H);
                let glyphs: Vec<Color> = c
                    .frame()
                    .iter()
                    .filter_map(|o| match o {
                        Op::Image {
                            x: ix, y: iy, tint, ..
                        } if *ix >= x && *ix < x + CARD_W && *iy >= band.0 && *iy < band.1 => {
                            Some(*tint)
                        }
                        _ => None,
                    })
                    .collect();

                assert!(
                    !glyphs.is_empty(),
                    "{target}/{lang}: no slot number in the band at offset {offset}"
                );
                for tint in glyphs {
                    assert_eq!(
                        tint, LABEL_INK,
                        "{target}/{lang}: the number is not drawn in the label ink"
                    );
                    assert_ne!(
                        tint, frame_ink,
                        "{target}/{lang}: the number is drawn in the frame's own colour"
                    );
                    let ratio = contrast(tint, frame_ink);
                    assert!(
                        ratio >= 4.5,
                        "{target}/{lang}: slot number contrast {ratio:.1}:1 at offset {offset}"
                    );
                }
            }
        }
    }
}

// ------------------------------------------------------------------ deleting and undoing

/// A switcher holding numbered slots and nothing else; the files are nobody's business here.
fn with_numbers(numbers: &[u32]) -> StateSwitcher {
    StateSwitcher::new(
        numbers
            .iter()
            .map(|n| slot(*n, None, u64::from(*n)))
            .collect(),
    )
}

#[test]
fn removing_a_slot_leaves_the_selection_on_the_card_that_moved_up() {
    // The middle one: the card above it takes its place.
    let mut s = with_numbers(&[1, 2, 3, 4]);
    s.refresh(vec![slot(1, None, 1), slot(3, None, 3), slot(4, None, 4)]);
    s.select_after_removing(2);
    assert_eq!(s.selected_kind(), Some(StateKind::Numbered(3)));

    // The greatest one: there is nothing above, so the greatest below it.
    let mut s = with_numbers(&[1, 2, 3]);
    s.refresh(vec![slot(1, None, 1), slot(2, None, 2)]);
    s.select_after_removing(3);
    assert_eq!(s.selected_kind(), Some(StateKind::Numbered(2)));

    // The last one: nothing is left, and nothing is selected.
    let mut s = with_numbers(&[7]);
    s.refresh(Vec::new());
    s.select_after_removing(7);
    assert!(s.is_empty());
    assert_eq!(s.selected_kind(), None);

    // And a restore puts the selection back on the number it restored.
    let mut s = with_numbers(&[1, 3]);
    assert!(s.select_number(1));
    assert_eq!(s.selected_kind(), Some(StateKind::Numbered(1)));
    assert!(!s.select_number(9), "a slot the card does not hold");
    assert_eq!(
        s.selected_kind(),
        Some(StateKind::Numbered(1)),
        "a refused selection moved anyway"
    );
}

#[test]
fn a_deleted_picture_is_let_go_and_a_reused_number_is_drawn_fresh() {
    let dir = temp_dir("evict");
    let one = dir.join("1.png");
    write_png(&one, 8, 6);
    let two = dir.join("2.png");
    write_png(&two, 8, 6);

    let mut ctx = ctx("rgsp", "en");
    let mut c = RecordingCanvas::new(720, 480);
    let mut s = StateSwitcher::new(vec![slot(1, Some(one.clone()), 1)]);
    s.draw(&mut c, &mut ctx, false);
    let first = upload_id(c.frame()).expect("the first picture was never uploaded");

    // A second state arrives: both pictures are on screen, and the one already decoded is not
    // decoded again.
    s.refresh(vec![
        slot(1, Some(one.clone()), 1),
        slot(2, Some(two.clone()), 2),
    ]);
    let warm = c.ops.len();
    s.draw(&mut c, &mut ctx, false);
    let fresh = uploads(&c.ops[warm..]);
    assert_eq!(
        fresh.len(),
        1,
        "a kept picture was uploaded again: {fresh:?}"
    );
    let second = fresh[0];
    assert!(image_rect(&c.ops[warm..], first).is_some());

    // Slot 2 is deleted. Its texture is given back on the next draw, and nothing draws it.
    s.refresh(vec![slot(1, Some(one.clone()), 1)]);
    assert_eq!(s.selected_kind(), Some(StateKind::Numbered(1)));
    let warm = c.ops.len();
    s.draw(&mut c, &mut ctx, false);
    let freed: Vec<TexId> = c.ops[warm..]
        .iter()
        .filter_map(|o| match o {
            Op::Free(tex) => Some(*tex),
            _ => None,
        })
        .collect();
    assert_eq!(
        freed,
        vec![second],
        "the deleted picture's texture came back"
    );
    assert!(
        !c.ops[warm..]
            .iter()
            .any(|o| matches!(o, Op::Image { tex, .. } if *tex == second)),
        "a deleted picture was drawn"
    );
    assert!(
        !c.ops[warm..]
            .iter()
            .any(|o| matches!(o, Op::UploadRgba8 { .. })),
        "the kept picture was decoded again"
    );

    // The number is handed out again to a different state: the new picture is drawn, not the
    // old one out of the cache.
    write_png(&two, 8, 4);
    s.refresh(vec![slot(1, Some(one), 1), slot(2, Some(two), 2)]);
    let warm = c.ops.len();
    s.draw(&mut c, &mut ctx, false);
    let again = uploads(&c.ops[warm..]);
    assert_eq!(again.len(), 1, "a reused number reused a stale picture");
    assert_ne!(again[0], second, "a reused number reused a stale texture");
    assert!(image_rect(&c.ops[warm..], first).is_some());

    // Nothing was leaked along the way: every texture is given back exactly once.
    let uploaded = uploads(c.ops.as_slice()).len();
    s.clear(&mut c);
    let frees = c.ops.iter().filter(|o| matches!(o, Op::Free(_))).count();
    assert_eq!(frees, uploaded, "a texture was leaked or freed twice");
}

#[test]
fn every_hint_row_is_the_right_one_and_fits_the_safe_area() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            // One context per panel and language: building one per case would load the Korean
            // font a dozen times over for the same handful of frames.
            let mut ctx = ctx(target, lang);
            for numbers in [&[][..], &[1, 2, 3][..]] {
                for undo in [false, true] {
                    let (pw, ph) = ctx.profile.geometry.size();
                    let mut s = with_numbers(numbers);
                    let mut c = RecordingCanvas::new(pw, ph);
                    s.draw(&mut c, &mut ctx, undo);
                    let ops = c.frame();
                    let case = format!("{target}/{lang}/{} slots/undo {undo}", numbers.len());

                    assert!(!ops.iter().any(|o| matches!(o, Op::Clear(_))), "{case}");
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
                        assert!(ctx.safe.contains(x, y, w, h), "{case}: {op:?}");
                    }

                    // The row says what the buttons do and nothing else: with no card to do
                    // it to there is no load and no delete, and undo only while there is one.
                    let keys: &[&str] = match (numbers.is_empty(), undo) {
                        (false, false) => &["hint-load", "hint-delete", "hint-back"],
                        (false, true) => &["hint-load", "hint-delete", "hint-back", "hint-undo"],
                        (true, false) => &["hint-back"],
                        (true, true) => &["hint-back", "hint-undo"],
                    };
                    assert_eq!(caps(ops), keys.len(), "{case}: the number of hints");
                    let drawn = hint_marks(&c, &ctx);
                    let want = hint_marks_for(&mut ctx, keys);
                    assert_eq!(drawn.len(), want.len(), "{case}: {drawn:?} vs {want:?}");
                    for (got, expected) in drawn.iter().zip(&want) {
                        assert!(
                            (got - expected).abs() < 0.5,
                            "{case}: {drawn:?} is not the row {keys:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_empty_case_draws_a_message_and_nothing_to_select() {
    for target in TARGETS {
        for lang in ["en", "ko"] {
            let mut ctx = ctx(target, lang);
            let (pw, ph) = ctx.profile.geometry.size();
            let mut s = StateSwitcher::new(vec![resume(5)]);
            assert_eq!(s.selected_kind(), None);

            let mut c = RecordingCanvas::new(pw, ph);
            s.draw(&mut c, &mut ctx, false);
            let ops = c.frame();
            assert!(!ops.iter().any(|o| matches!(o, Op::Clear(_))));
            assert_eq!(plates(ops), 0, "{target}/{lang}: a card was drawn empty");

            // The dim, the title, the message and the hints: every mark in the safe area.
            let mut inside = 0;
            for op in ops {
                let (x, y, w, h) = match op {
                    Op::Rect { x, y, w, h, .. } | Op::Image { x, y, w, h, .. } => (*x, *y, *w, *h),
                    _ => continue,
                };
                if (x, y, w, h) == (0.0, 0.0, pw as f32, ph as f32) {
                    continue;
                }
                assert!(ctx.safe.contains(x, y, w, h), "{target}/{lang}: {op:?}");
                inside += 1;
            }
            assert!(
                inside >= 3,
                "{target}/{lang}: only {inside} marks on screen"
            );
        }
    }
}

// ------------------------------------------------------------------ pictures

#[test]
fn a_thumbnail_is_uploaded_once_and_keeps_its_shape() {
    let dir = temp_dir("shape");
    let four_three = dir.join("4x3.png");
    write_png(&four_three, 8, 6);
    let wide = dir.join("2x1.png");
    write_png(&wide, 8, 4);

    for (path, ratio) in [(&four_three, 4.0 / 3.0), (&wide, 2.0)] {
        // A context of its own per case: texture ids are handed out per canvas and start
        // again at 1 for each one, so a warm face cache from the previous canvas would
        // collide with this one and point the lookup below at the wrong picture.
        let mut ctx = ctx("rgsp", "en");
        let mut s = StateSwitcher::new(vec![slot(1, Some(path.clone()), 1)]);
        let mut c = frame(&mut s, &mut ctx, "rgsp");
        let Some(tex) = upload_id(c.frame()) else {
            panic!("{path:?}: the thumbnail was never uploaded")
        };
        let (x, y, w, h) =
            image_rect(c.frame(), tex).unwrap_or_else(|| panic!("{path:?}: not drawn"));
        assert!(
            (w / h - ratio).abs() < 0.01,
            "{path:?}: drawn {w}x{h}, which is not {ratio}"
        );

        // Inside the picture area, and filling one of its axes rather than floating in it.
        let (tx, ty, tw, th) = StateSwitcher::thumb_rect(&ctx, 0);
        assert!(
            w <= tw + 0.5 && h <= th + 0.5,
            "{path:?}: {w}x{h} > {tw}x{th}"
        );
        assert!(
            (w - tw).abs() < 0.5 || (h - th).abs() < 0.5,
            "{path:?}: {w}x{h} leaves the area empty"
        );
        assert!(x >= tx - 0.5 && x + w <= tx + tw + 0.5 && y >= ty - 0.5 && y + h <= ty + th + 0.5);

        // The second frame uploads nothing and still draws the picture.
        let warm = c.ops.len();
        s.draw(&mut c, &mut ctx, false);
        assert!(
            !c.ops[warm..]
                .iter()
                .any(|o| matches!(o, Op::UploadRgba8 { .. } | Op::UploadAlpha8 { .. })),
            "{path:?}: the thumbnail was uploaded again"
        );
        assert!(image_rect(&c.ops[warm..], tex).is_some());
    }
}

#[test]
fn a_missing_or_corrupt_thumbnail_is_a_placeholder_that_is_not_read_again() {
    let dir = temp_dir("placeholder");
    let corrupt = dir.join("corrupt.png");
    fs::write(&corrupt, b"not a png at all").unwrap();
    let missing = dir.join("missing.png"); // never written
    let late = dir.join("late.png"); // written after the first frame

    let mut ctx = ctx("rgsp", "en");
    let mut s = StateSwitcher::new(vec![
        slot(1, Some(corrupt), 1),
        slot(2, Some(missing.clone()), 2),
        slot(3, Some(late.clone()), 3),
    ]);

    let mut c = frame(&mut s, &mut ctx, "rgsp");
    assert!(
        !c.frame()
            .iter()
            .any(|o| matches!(o, Op::UploadRgba8 { .. })),
        "a corrupt or missing thumbnail was uploaded"
    );
    assert_eq!(
        plates(c.frame()),
        3,
        "not every card fell back to the plate"
    );

    // The files become readable. A card that has already given up must not go back to the
    // disk every frame — and the placeholder has to stay put when it does not.
    write_png(&late, 8, 6);
    write_png(&missing, 8, 6);
    let warm = c.ops.len();
    s.draw(&mut c, &mut ctx, false);
    assert!(
        !c.ops[warm..]
            .iter()
            .any(|o| matches!(o, Op::UploadRgba8 { .. })),
        "the switcher went back for a thumbnail it had already given up on"
    );
    assert_eq!(
        plates(&c.ops[warm..]),
        3,
        "the placeholder stopped being drawn"
    );
}

#[test]
fn refreshing_takes_the_new_slots_in_place_and_keeps_the_pictures_it_has() {
    let dir = temp_dir("refresh");
    let one = dir.join("1.png");
    write_png(&one, 8, 6);
    let two = dir.join("2.png");
    write_png(&two, 8, 6);

    let mut ctx = ctx("rgsp", "en");
    let mut s = StateSwitcher::new(vec![slot(1, Some(one.clone()), 1), resume(9)]);
    let mut c = RecordingCanvas::new(720, 480);
    s.draw(&mut c, &mut ctx, false);
    assert_eq!(s.selected_kind(), Some(StateKind::Numbered(1)));
    let first = upload_id(c.frame()).expect("the first picture was never uploaded");

    // A later save: the slots are replaced in place, the same way `new` would have taken
    // them, and the picture already decoded is still the one on screen.
    s.refresh(vec![
        slot(2, Some(two), 2),
        slot(1, Some(one), 1),
        resume(9),
    ]);
    assert_eq!(
        s.selected_kind(),
        Some(StateKind::Numbered(2)),
        "the new greatest"
    );
    let warm = c.ops.len();
    s.draw(&mut c, &mut ctx, false);
    let fresh: Vec<TexId> = c.ops[warm..]
        .iter()
        .filter_map(|o| match o {
            Op::UploadRgba8 { id, .. } => Some(*id),
            _ => None,
        })
        .collect();
    assert_eq!(
        fresh.len(),
        1,
        "the refresh re-decoded a picture it already had: {fresh:?}"
    );
    assert!(
        image_rect(&c.ops[warm..], first).is_some(),
        "the picture from the first visit went missing"
    );

    // An empty list is a refresh too, and it leaves nothing to select.
    s.refresh(vec![resume(20)]);
    assert!(s.is_empty());
    assert_eq!(s.selected_kind(), None);
    s.refresh(Vec::new());
    assert_eq!(s.selected_kind(), None);
}

#[test]
fn the_thumbnail_cache_is_bounded_and_frees_what_it_holds() {
    let dir = temp_dir("bound");
    let mut slots = Vec::new();
    let mut paths = Vec::new();
    // More thumbnails than the cache holds, so a lap of the strip has to drop some.
    for n in 1..=20u32 {
        let p = dir.join(format!("{n}.png"));
        write_png(&p, 8, 6);
        slots.push(slot(n, Some(p.clone()), u64::from(n)));
        paths.push(p);
    }

    // One lap of a ring of twenty with a cache of `CACHE_MAX`: the pictures that fall off
    // the back are decoded again when the ring comes round to them, which is right. What
    // must not happen is a texture being held past the bound, or dropped without being
    // freed.
    let mut ctx = ctx("rgsp", "en");
    let mut s = StateSwitcher::new(slots);
    let mut c = RecordingCanvas::new(720, 480);

    for _ in 0..paths.len() {
        s.draw(&mut c, &mut ctx, false);
        s.right();
    }
    let uploads = c
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadRgba8 { .. }))
        .count();
    let dropped = c.ops.iter().filter(|o| matches!(o, Op::Free(_))).count();
    assert!(
        uploads >= paths.len(),
        "only {uploads} pictures were decoded for {} slots",
        paths.len()
    );
    assert_eq!(
        dropped,
        uploads - CACHE_MAX,
        "the cache did not stop growing at {CACHE_MAX} textures"
    );

    // And what is left is handed back, so `CACHE_MAX` is all that is ever alive.
    s.clear(&mut c);
    assert_eq!(
        c.ops.iter().filter(|o| matches!(o, Op::Free(_))).count(),
        uploads
    );
}

#[test]
fn a_thumbnail_that_works_is_kept_until_it_is_released() {
    let dir = temp_dir("release");
    let good = dir.join("good.png");
    write_png(&good, 8, 6);

    let mut ctx = ctx("rgsp", "en");
    let mut s = StateSwitcher::new(vec![slot(1, Some(good), 1)]);
    let mut c = frame(&mut s, &mut ctx, "rgsp");
    let Some(tex) = upload_id(c.frame()) else {
        panic!("the thumbnail was never uploaded")
    };
    assert_eq!(plates(c.frame()), 0, "a good thumbnail drew a placeholder");

    let warm = c.ops.len();
    s.draw(&mut c, &mut ctx, false);
    assert!(!c.ops[warm..]
        .iter()
        .any(|o| matches!(o, Op::UploadRgba8 { .. })));

    // And the texture is handed back when the canvas is about to go away.
    s.clear(&mut c);
    assert!(
        c.ops.iter().any(|o| matches!(o, Op::Free(t) if *t == tex)),
        "the texture was not released"
    );
    let warm = c.ops.len();
    s.draw(&mut c, &mut ctx, false);
    assert!(
        c.ops[warm..]
            .iter()
            .any(|o| matches!(o, Op::UploadRgba8 { .. })),
        "a released thumbnail was not uploaded again"
    );
}
