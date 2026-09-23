//! The contract for what is on the front of a cartridge. Task 16 makes these pass without
//! editing this file.
//!
//! A card carries a scan of the real sticker for some games and none for most, so the
//! printed fallback is the normal case rather than an error state, and most of what is
//! checked here is that it looks deliberate.

use std::path::PathBuf;

use slot2_gfx::{Color, Op, RecordingCanvas};
use slot2_platform::{detect, Geometry};
use slot2_store::{Card, Platform};
use slot2_ui::label::{clean_title, printed_colour, LabelCache, CACHE_MAX};
use slot2_ui::layout::SafeArea;
use slot2_ui::shelf_view::ShelfView;
use slot2_ui::{skin, UiCtx};

static NEXT_CARD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn ctx() -> UiCtx {
    UiCtx::new(detect().profile, "en", Vec::new(), None)
}

/// A card with `n` GBA games, and label art for the ones named in `art`.
fn card(n: usize, art: &[usize], art_size: (u32, u32)) -> (Card, PathBuf) {
    let i = NEXT_CARD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("slot2-label-{}-{i}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let card = Card::new(&root);
    card.ensure_layout();
    for g in 0..n {
        let stem = format!("Game {g:02}");
        std::fs::write(card.games_dir(Platform::Gba).join(format!("{stem}.gba")), b"r").unwrap();
        if art.contains(&g) {
            write_png(
                &root.join("Labels/GBA").join(format!("{stem}.png")),
                art_size,
            );
        }
    }
    (card, root)
}

/// A solid magenta PNG, which is a colour nothing else here produces.
fn write_png(path: &std::path::Path, (w, h): (u32, u32)) {
    let f = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header().unwrap();
    let px: Vec<u8> = (0..w * h).flat_map(|_| [0xFF, 0x00, 0xFF, 0xFF]).collect();
    wr.write_image_data(&px).unwrap();
}

fn images(canvas: &RecordingCanvas) -> Vec<(f32, f32, f32, f32, Color)> {
    canvas
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Image {
                x, y, w, h, tint, ..
            } => Some((*x, *y, *w, *h, *tint)),
            _ => None,
        })
        .collect()
}

fn rects(canvas: &RecordingCanvas) -> Vec<(f32, f32, f32, f32, Color)> {
    canvas
        .frame()
        .iter()
        .filter_map(|o| match o {
            Op::Rect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
            _ => None,
        })
        .collect()
}

// ------------------------------------------------------------------ the title

#[test]
fn a_dumps_tags_are_not_part_of_the_games_name() {
    // A filename carries facts about the dump. They belong in the filename.
    assert_eq!(clean_title("Advance Wars (USA)"), "Advance Wars");
    assert_eq!(
        clean_title("Advance Wars (USA, Europe) (Rev 1)"),
        "Advance Wars"
    );
    assert_eq!(clean_title("Some Game [!]"), "Some Game");
    assert_eq!(clean_title("Game (USA) [b1]"), "Game");
}

#[test]
fn a_hyphen_inside_a_word_stays_there() {
    // `Spider-Man` is one word. `Zelda - A Link to the Past` is a title and a subtitle, and
    // the shelf has room for the title.
    assert_eq!(clean_title("Spider-Man"), "Spider-Man");
    assert_eq!(
        clean_title("Zelda - A Link to the Past"),
        "Zelda A Link to the Past"
    );
}

#[test]
fn a_title_that_is_all_tags_keeps_something_to_show() {
    // Cleaning a name down to nothing leaves a cart with no name at all, which on a shelf is
    // a blank where a game should be. Whatever comes back must not be empty.
    for stem in ["(USA)", "[!]", "   ", "()"] {
        assert!(
            !clean_title(stem).trim().is_empty(),
            "{stem:?} cleaned away to nothing"
        );
    }
}

#[test]
fn korean_and_japanese_titles_survive() {
    // The card is Korean before it is anything else.
    assert_eq!(clean_title("파이널 판타지 (Korea)"), "파이널 판타지");
    assert_eq!(clean_title("ポケモン (Japan)"), "ポケモン");
}

// ------------------------------------------------------------------ the colour

#[test]
fn a_printed_label_is_the_same_colour_every_time() {
    // It is derived, not assigned. A shelf whose colours moved between boots would teach a
    // player nothing.
    let a = printed_colour("Advance Wars");
    let b = printed_colour("Advance Wars");
    assert_eq!(a, b);
}

#[test]
fn different_games_get_different_colours() {
    // The whole point of the printed label: at a glance, a row of them is distinguishable
    // even when the words are too small to read.
    let titles = [
        "Advance Wars",
        "Metroid Fusion",
        "Golden Sun",
        "Mario Kart",
        "Fire Emblem",
        "Castlevania",
        "파이널 판타지",
        "Kirby",
    ];
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for t in titles {
        let c = printed_colour(t);
        for other in &seen {
            let d = (c[0] as i32 - other[0] as i32).abs()
                + (c[1] as i32 - other[1] as i32).abs()
                + (c[2] as i32 - other[2] as i32).abs();
            assert!(d > 24, "{t} came out at {c:?}, too near {other:?}");
        }
        seen.push(c);
    }
}

#[test]
fn a_printed_label_is_a_background_for_text() {
    // Dark ink goes on it. A fully saturated colour at handheld brightness is a warning
    // light, and a near-white one is a blank sticker.
    for t in ["A", "Advance Wars", "", "파이널 판타지", "zzzzzzzzzzzz"] {
        let [r, g, b] = printed_colour(t);
        let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        assert!(
            (70.0..=215.0).contains(&lum),
            "{t:?} came out at {r},{g},{b} — luminance {lum}"
        );
        let max = r.max(g).max(b) as i32;
        let min = r.min(g).min(b) as i32;
        assert!(
            max - min > 20,
            "{t:?} came out grey: {r},{g},{b}"
        );
    }
}

// ------------------------------------------------------------------ the card

#[test]
fn the_card_finds_the_art_it_has_and_admits_the_art_it_does_not() {
    let (card, _root) = card(3, &[1], (64, 64));
    assert!(card.label_path(Platform::Gba, "Game 00").is_none());
    let found = card
        .label_path(Platform::Gba, "Game 01")
        .expect("the card has art for Game 01 and did not find it");
    assert!(found.is_file(), "{} is not a file", found.display());
    assert!(card.label_path(Platform::Gba, "Game 02").is_none());
    // And a stem that is not on the card at all.
    assert!(card.label_path(Platform::Gba, "Nothing").is_none());
    // A different platform's folder is a different shelf.
    assert!(card.label_path(Platform::Snes, "Game 01").is_none());
}

// ------------------------------------------------------------------ the cache

#[test]
fn art_is_decoded_once_and_a_missing_file_is_opened_once() {
    let (card, _root) = card(2, &[0], (64, 64));
    let path = card.label_path(Platform::Gba, "Game 00").unwrap();
    let missing = card.games_dir(Platform::Gba).join("not-a-label.png");

    let mut canvas = RecordingCanvas::new(720, 480);
    let mut cache = LabelCache::default();

    let first = cache.art(&mut canvas, &path, (200, 130));
    assert!(first.is_some(), "the label did not decode");
    let uploads = |c: &RecordingCanvas| {
        c.ops
            .iter()
            .filter(|o| matches!(o, Op::UploadRgba8 { .. } | Op::UploadAlpha8 { .. }))
            .count()
    };
    let after_one = uploads(&canvas);

    for _ in 0..30 {
        let again = cache.art(&mut canvas, &path, (200, 130));
        assert_eq!(
            again.map(|a| a.tex),
            first.map(|a| a.tex),
            "the label was uploaded again"
        );
    }
    assert_eq!(uploads(&canvas), after_one, "a redraw re-uploaded the label");

    // A file that is not there is remembered as not there.
    for _ in 0..30 {
        assert!(cache.art(&mut canvas, &missing, (200, 130)).is_none());
    }
    assert_eq!(uploads(&canvas), after_one);
}

#[test]
fn a_big_scan_is_not_uploaded_at_scanner_resolution() {
    // A 1024-square label drawn two hundred pixels wide costs four megabytes to look no
    // better. On a Mali sharing system memory that is the difference between a shelf and a
    // shelf that stutters.
    let (card, _root) = card(1, &[0], (1024, 1024));
    let path = card.label_path(Platform::Gba, "Game 00").unwrap();
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut cache = LabelCache::default();

    let plate = (200u32, 130u32);
    let art = cache.art(&mut canvas, &path, plate).expect("did not decode");
    assert!(
        art.w <= plate.0 * 2 && art.h <= plate.1 * 2,
        "a 1024-square scan was kept at {}x{}",
        art.w,
        art.h
    );
    // And not scaled down so far that the label is mush.
    assert!(
        art.w >= plate.0 && art.h >= plate.1,
        "a 1024-square scan was reduced to {}x{}",
        art.w,
        art.h
    );
}

#[test]
fn the_cache_does_not_grow_without_end() {
    // A card with two hundred games would otherwise hold two hundred textures. Seven carts
    // are ever on screen.
    let (card, _root) = card(40, &(0..40).collect::<Vec<_>>(), (32, 32));
    let mut canvas = RecordingCanvas::new(720, 480);
    let mut cache = LabelCache::default();

    for g in 0..40 {
        let p = card
            .label_path(Platform::Gba, &format!("Game {g:02}"))
            .unwrap();
        assert!(cache.art(&mut canvas, &p, (200, 130)).is_some());
        assert!(
            cache.len() <= CACHE_MAX,
            "the cache holds {} after {} labels",
            cache.len(),
            g + 1
        );
    }

    // What it drops it also frees, or the cap saves nothing.
    let freed = canvas
        .ops
        .iter()
        .filter(|o| matches!(o, Op::Free { .. }))
        .count();
    assert!(
        freed >= 40 - CACHE_MAX,
        "{freed} textures were freed after evicting {} labels",
        40 - CACHE_MAX
    );
}

// ------------------------------------------------------------------ the shelf

fn shelf_with(n: usize, labels: Vec<Option<PathBuf>>) -> (ShelfView, SafeArea) {
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut v = ShelfView::default();
    v.shelf.set_len(n);
    v.set_labels(labels);
    (v, safe)
}

#[test]
fn a_cart_with_no_art_gets_a_printed_label() {
    // Not a white blank. The colour is the whole of what tells one dark cartridge from
    // another at the edge of the row.
    let (mut v, safe) = shelf_with(3, vec![None, None, None]);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &["A", "B", "C"]);

    let want = printed_colour("A");
    let hit = rects(&canvas).iter().any(|(_, _, _, _, col)| {
        let [r, g, b, _] = col.to_u8();
        [r, g, b] == want
    });
    assert!(hit, "no label was printed in {want:?}");
}

#[test]
fn two_carts_on_one_row_do_not_get_one_colour() {
    let (mut v, safe) = shelf_with(3, vec![None, None, None]);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &["Advance Wars", "Metroid Fusion", "Golden Sun"],
    );

    let plates: std::collections::BTreeSet<[u8; 3]> = rects(&canvas)
        .iter()
        .map(|(_, _, _, _, col)| {
            let [r, g, b, _] = col.to_u8();
            [r, g, b]
        })
        .collect();
    for t in ["Advance Wars", "Metroid Fusion", "Golden Sun"] {
        assert!(
            plates.contains(&printed_colour(t)),
            "{t}'s label was not printed"
        );
    }
}

#[test]
fn a_cart_with_art_shows_the_art() {
    let (card, _root) = card(3, &[0, 1, 2], (128, 96));
    let labels: Vec<Option<PathBuf>> = (0..3)
        .map(|g| card.label_path(Platform::Gba, &format!("Game {g:02}")))
        .collect();
    assert!(labels.iter().all(|l| l.is_some()), "the card lost its art");

    let (mut v, safe) = shelf_with(3, labels);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &["A", "B", "C"]);

    // Three carts' worth of label art, drawn as images rather than printed as rects. Art is
    // a picture, so it goes on untinted — tinting it with the shell colour would stain a
    // scan of a real sticker.
    let white = images(&canvas)
        .iter()
        .filter(|(_, _, _, _, tint)| {
            let [r, g, b, _] = tint.to_u8();
            r == 0xFF && g == 0xFF && b == 0xFF
        })
        .count();
    assert!(white >= 3, "only {white} labels were drawn as art");
}

#[test]
fn a_shelf_of_both_kinds_draws_both() {
    // The ordinary card: art for the few games someone bothered with, printed for the rest.
    let (card, _root) = card(3, &[1], (128, 96));
    let labels: Vec<Option<PathBuf>> = (0..3)
        .map(|g| card.label_path(Platform::Gba, &format!("Game {g:02}")))
        .collect();
    let (mut v, safe) = shelf_with(3, labels);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &["Zero", "One", "Two"],
    );

    for t in ["Zero", "Two"] {
        let want = printed_colour(t);
        assert!(
            rects(&canvas).iter().any(|(_, _, _, _, col)| {
                let [r, g, b, _] = col.to_u8();
                [r, g, b] == want
            }),
            "{t} has no art and was not printed either"
        );
    }
    assert!(
        !rects(&canvas).iter().any(|(_, _, _, _, col)| {
            let [r, g, b, _] = col.to_u8();
            [r, g, b] == printed_colour("One")
        }),
        "One has art and was printed over anyway"
    );
}

#[test]
fn a_row_with_no_labels_set_still_draws() {
    // `set_labels` has not been called, or the row grew since it was. Neither is a reason
    // for a cart to have no front.
    let safe = SafeArea::for_geometry(Geometry::W720H480);
    let mut v = ShelfView::default();
    v.shelf.set_len(4);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &["A", "B", "C", "D"]);
    assert!(!images(&canvas).is_empty(), "the shelf drew no carts");

    // And a labels list that is shorter than the row.
    v.shelf.set_len(6);
    v.set_labels(vec![None, None]);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    v.draw(
        &mut canvas,
        &mut c,
        &safe,
        Platform::Gba,
        &["A", "B", "C", "D", "E", "F"],
    );
    assert!(!images(&canvas).is_empty(), "a short labels list emptied the row");
}

#[test]
fn the_label_sits_on_the_plate_wherever_the_cart_is() {
    // Every platform's skin puts its label somewhere different, and a neighbour is drawn
    // smaller. The label has to follow the cart rather than the panel.
    let (card, _root) = card(3, &[0, 1, 2], (128, 96));
    let labels: Vec<Option<PathBuf>> = (0..3)
        .map(|g| card.label_path(Platform::Gba, &format!("Game {g:02}")))
        .collect();
    let (mut v, safe) = shelf_with(3, labels);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &["A", "B", "C"]);

    let skin = skin::skin(Platform::Gba);
    let placements = v.shelf.placements(&safe, skin.cart_size);
    for p in &placements {
        let sx = p.w / skin.cart_size.0;
        let sy = p.h / skin.cart_size.1;
        let (lx, ly) = (p.x + skin.label.x * sx, p.y + skin.label.y * sy);
        let (lw, lh) = (skin.label.w * sx, skin.label.h * sy);
        let on_plate = images(&canvas).iter().any(|(x, y, w, h, _)| {
            (x - lx).abs() < 1.0 && (y - ly).abs() < 1.0 && (w - lw).abs() < 1.0 && (h - lh).abs() < 1.0
        });
        assert!(
            on_plate,
            "the cart at {},{} {}x{} has no label on its plate at {lx},{ly} {lw}x{lh}",
            p.x, p.y, p.w, p.h
        );
    }
}

#[test]
fn scrolling_a_shelf_of_art_does_not_upload_every_frame() {
    // Sixty frames of a slide, with a cart's drawn size changing on every one of them. The
    // fault task 12 had, on a different texture.
    let (card, _root) = card(8, &(0..8).collect::<Vec<_>>(), (128, 96));
    let labels: Vec<Option<PathBuf>> = (0..8)
        .map(|g| card.label_path(Platform::Gba, &format!("Game {g:02}")))
        .collect();
    let (mut v, safe) = shelf_with(8, labels);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    let t = ["A", "B", "C", "D", "E", "F", "G", "H"];

    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
    let settled = canvas.ops.len();

    v.shelf.right();
    for _ in 0..60 {
        v.shelf.update(1.0 / 60.0);
        v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &t);
    }
    let uploads = canvas
        .ops
        .iter()
        .skip(settled)
        .filter(|o| matches!(o, Op::UploadRgba8 { .. } | Op::UploadAlpha8 { .. }))
        .count();
    assert!(
        uploads <= 4,
        "a one-second scroll uploaded {uploads} textures"
    );
}

#[test]
fn a_broken_png_is_a_printed_label_not_a_hole() {
    // Half a file, a JPEG renamed, a card pulled mid-copy.
    let (card, root) = card(1, &[], (0, 0));
    let bad = root.join("Labels/GBA/Game 00.png");
    std::fs::write(&bad, b"this is not a png").unwrap();
    let found = card
        .label_path(Platform::Gba, "Game 00")
        .expect("the file is there, whatever is in it");

    let (mut v, safe) = shelf_with(1, vec![Some(found)]);
    let mut canvas = RecordingCanvas::new(safe.panel_w, safe.panel_h);
    let mut c = ctx();
    v.draw(&mut canvas, &mut c, &safe, Platform::Gba, &["Broken"]);

    let want = printed_colour("Broken");
    assert!(
        rects(&canvas).iter().any(|(_, _, _, _, col)| {
            let [r, g, b, _] = col.to_u8();
            [r, g, b] == want
        }),
        "a label that would not decode left the cart with no front"
    );
}
