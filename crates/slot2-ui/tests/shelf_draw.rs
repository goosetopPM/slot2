//! The contract for drawing the shelf. Task 12 makes these pass without editing this file.

use slot2_gfx::{Color, Op, RecordingCanvas, TexId};
use slot2_platform::{detect, Geometry};
use slot2_store::Platform;
use slot2_ui::layout::SafeArea;
use slot2_ui::shelf_view::ShelfView;
use slot2_ui::{skin, UiCtx};

const GEOMETRIES: [Geometry; 3] = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];

fn ctx() -> UiCtx {
    UiCtx::new(detect().profile, "en", Vec::new(), None)
}

fn images(canvas: &RecordingCanvas) -> Vec<(TexId, f32, f32, f32, f32, Color)> {
    canvas
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image {
                tex,
                x,
                y,
                w,
                h,
                tint,
                ..
            } => Some((*tex, *x, *y, *w, *h, *tint)),
            _ => None,
        })
        .collect()
}

fn titles(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("Game {i}")).collect()
}

fn draw(
    view: &mut ShelfView,
    g: Geometry,
    platform: Platform,
    n: usize,
) -> (RecordingCanvas, SafeArea) {
    let safe = SafeArea::for_geometry(g);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    let t = titles(n);
    let refs: Vec<&str> = t.iter().map(|s| s.as_str()).collect();
    view.shelf.set_len(n);
    view.draw(&mut canvas, &mut c, &safe, platform, &refs);
    (canvas, safe)
}

#[test]
fn an_empty_shelf_still_draws_its_slot() {
    // A shelf with no games is not a blank screen: the slot is the thing that says this is
    // a shelf and that something is meant to go in it. It is chrome rather than artwork —
    // there is no drawing of a cartridge slot — so what to look for is rectangles.
    let mut v = ShelfView::default();
    let (canvas, safe) = draw(&mut v, Geometry::W720H480, Platform::Gba, 0);
    let rects: Vec<&Op> = canvas
        .frame()
        .iter()
        .filter(|o| matches!(o, Op::Rect { .. }))
        .collect();
    assert!(
        rects.len() >= 3,
        "an empty shelf drew {} rects",
        rects.len()
    );

    // And it is at the foot of the panel, where a slot is, not floating in the middle.
    let lowest = canvas
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Rect { y, h, .. } => Some(y + h),
            _ => None,
        })
        .fold(0.0f32, f32::max);
    assert!(
        (lowest - safe.panel_h as f32).abs() < 1.0,
        "the slot ends at {lowest} on a {}-tall panel",
        safe.panel_h
    );
}

#[test]
fn every_cart_on_the_row_is_drawn() {
    let mut v = ShelfView::default();
    let (canvas, _) = draw(&mut v, Geometry::W720H480, Platform::Gba, 5);
    let n = images(&canvas).len();
    // Three carts on screen, each a shell and a detail pass, plus the slot. The exact
    // number is the implementation's business; that several things were drawn is not.
    assert!(n >= 4, "a row of five drew only {n} things");
}

#[test]
fn the_shell_is_tinted_with_the_platforms_colour() {
    // The artwork is a white silhouette; the colour is not in the file. If the tint is
    // dropped every cartridge comes out white.
    let mut v = ShelfView::default();
    let (canvas, _) = draw(&mut v, Geometry::W720H480, Platform::Gb, 3);
    let want = skin::skin(Platform::Gb).shell.colour;
    let hit = images(&canvas).iter().any(|(_, _, _, _, _, tint)| {
        let [r, g, b, _] = tint.to_u8();
        [r, g, b] == want
    });
    assert!(hit, "no cart was drawn in the Game Boy's shell colour");
}

#[test]
fn switching_platform_changes_the_artwork_and_the_colour() {
    // L1/R1 moves between shelves, and a Game Boy shelf has to look like one. This is M3's
    // acceptance criterion: "GB 선반에서 GB 카트·GB 스킨으로 바뀜".
    //
    // Both shelves are drawn into *one* canvas. Texture ids are handed out per canvas and
    // start again at 1 for each, so comparing ids across two canvases compares nothing —
    // an earlier version of this test passed on that coincidence.
    let mut v = ShelfView::default();
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.shelf.set_len(3);
    let t = ["A", "B", "C"];

    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
    let split = canvas.ops.len();
    v.draw(&mut canvas, &mut c, &safe, Platform::Gb, &t);

    // The biggest thing on a shelf is the selected cartridge. Asking whether *anything*
    // drawn was taller than wide catches a title glyph instead — "A" is.
    let shown = |ops: &[Op]| -> (std::collections::BTreeSet<u32>, bool) {
        let mut tex = std::collections::BTreeSet::new();
        let mut biggest = (0.0f32, 0.0f32);
        for o in ops {
            if let Op::Image { tex: t, w, h, .. } = o {
                tex.insert(t.0);
                if w * h > biggest.0 * biggest.1 {
                    biggest = (*w, *h);
                }
            }
        }
        (tex, biggest.1 > biggest.0)
    };
    let (gba_tex, gba_tall) = shown(&canvas.ops[..split]);
    let (gb_tex, gb_tall) = shown(&canvas.ops[split..]);

    assert!(
        gba_tex.intersection(&gb_tex).count() < gba_tex.len(),
        "both shelves drew the same artwork: {gba_tex:?} and {gb_tex:?}"
    );

    // A Game Boy pak is taller than it is wide; a GBA cart is wider than tall. Whatever
    // else changed, the shape must have.
    assert!(gb_tall, "the Game Boy shelf drew nothing pak-shaped");
    assert!(!gba_tall, "the GBA shelf drew something pak-shaped");
}

#[test]
fn nothing_is_drawn_off_the_panel() {
    for g in GEOMETRIES {
        let mut v = ShelfView::default();
        let (canvas, safe) = draw(&mut v, g, Platform::Gba, 30);
        for (_, x, y, w, h, _) in images(&canvas) {
            assert!(
                x + w > 0.0 && x < safe.panel_w as f32 && y + h > 0.0 && y < safe.panel_h as f32,
                "{g:?}: something was drawn at {x},{y} {w}x{h} on a {}x{} panel",
                safe.panel_w,
                safe.panel_h
            );
        }
    }
}

#[test]
fn the_selected_carts_title_is_drawn() {
    let mut v = ShelfView::default();
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.shelf.set_len(3);
    v.draw(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &["Zero", "One", "Two"],
    );

    // Text is drawn through alpha masks, so what proves a title reached the screen is that
    // coverage was uploaded for it. A shelf that draws carts and no titles is unusable.
    let masks = canvas
        .ops
        .iter()
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. }))
        .count();
    assert!(masks > 0, "nothing text-shaped was rasterised");
}

#[test]
fn redrawing_does_not_re_upload() {
    // Sixty frames a second. If the view rasterises or uploads per frame it is unusable on
    // the device however good it looks here.
    let mut v = ShelfView::default();
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.shelf.set_len(4);
    let t = ["A", "B", "C", "D"];

    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
    let after_first = canvas.ops.len();
    let uploads = |c: &RecordingCanvas| {
        c.ops
            .iter()
            .skip(after_first)
            .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
            .count()
    };

    for _ in 0..5 {
        v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
    }
    assert_eq!(uploads(&canvas), 0, "the shelf re-uploaded on a redraw");
}

#[test]
fn it_draws_on_every_geometry() {
    for g in GEOMETRIES {
        for n in [0usize, 1, 2, 7] {
            let mut v = ShelfView::default();
            let (canvas, _) = draw(&mut v, g, Platform::Snes, n);
            for (_, x, y, w, h, _) in images(&canvas) {
                for val in [x, y, w, h] {
                    assert!(val.is_finite(), "{g:?} n={n}: {x},{y} {w}x{h}");
                }
                assert!(w > 0.0 && h > 0.0, "{g:?} n={n}: {w}x{h}");
            }
        }
    }
}

#[test]
fn scrolling_does_not_re_upload_either() {
    // The gap the first version of this contract left. A neighbour's size lerps
    // continuously as the row slides, so caching artwork at the size it is drawn at means
    // a fresh rasterise and a fresh texture on nearly every frame of every scroll — and the
    // cache never evicts, so they accumulate. Rasterise at a fixed size and scale the quad.
    let mut v = ShelfView::default();
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.shelf.set_len(8);
    let t: Vec<&str> = vec!["A", "B", "C", "D", "E", "F", "G", "H"];

    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
    let settled = canvas.ops.len();

    // A press, then a second of the row sliding.
    v.shelf.right();
    for _ in 0..60 {
        v.shelf.update(1.0 / 60.0);
        v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
    }

    let uploads = canvas
        .ops
        .iter()
        .skip(settled)
        .filter(|o| matches!(o, Op::UploadAlpha8 { .. } | Op::UploadRgba8 { .. }))
        .count();
    assert!(
        uploads <= 2,
        "a one-second scroll uploaded {uploads} textures"
    );
}

#[test]
fn the_title_stays_on_screen_while_the_row_slides() {
    // Drawing the title only when the last placement happens to be the selection means it
    // blinks out mid-scroll, which is when the player is most likely to be reading it.
    let mut v = ShelfView::default();
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut c = ctx();
    v.shelf.set_len(6);
    let t: Vec<&str> = vec!["A", "B", "C", "D", "E", "F"];

    v.shelf.right();
    for _ in 0..12 {
        v.shelf.update(1.0 / 60.0);
        let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
        v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
        let selected = v.shelf.selected();
        let drawn = v.shelf.placements(&safe, (240.0, 135.0)).len();
        assert!(
            images(&canvas).len() > drawn,
            "mid-scroll frame drew {drawn} carts and nothing else — no title for {selected}"
        );
    }
}
