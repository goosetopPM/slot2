//! The contract for slot2-text. Task 02 makes these pass without editing this file.
//! Uses the real fonts under assets/fonts (Open Sans = Latin UI font, Noto Sans KR = CJK).

use std::path::PathBuf;
use std::time::Instant;

use slot2_text::{FontChain, FontId, TOFU};

fn fonts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts")
}
fn open_sans() -> PathBuf {
    fonts_dir().join("OpenSans-Regular.ttf")
}
fn noto_kr() -> PathBuf {
    fonts_dir().join("NotoSansKR-Regular.otf")
}

/// UI font eager, CJK lazy: the production shape of the chain.
fn chain() -> FontChain {
    let mut c = FontChain::new();
    c.push_file(&open_sans()).unwrap();
    c.push_lazy_file(&noto_kr());
    c
}

#[test]
fn fonts_load_and_are_numbered_in_order() {
    let mut c = FontChain::new();
    assert!(c.is_empty());
    let a = c.push_file(&open_sans()).unwrap();
    let b = c.push_lazy_file(&noto_kr());
    assert_eq!((a, b), (FontId(0), FontId(1)));
    assert_eq!(c.len(), 2);
    assert!(c.is_loaded(a));
    assert!(!c.is_loaded(b));
    let bytes = std::fs::read(open_sans()).unwrap();
    assert_eq!(c.push_bytes("open-sans-again", bytes).unwrap(), FontId(2));
}

#[test]
fn bad_fonts_are_errors_not_panics() {
    let mut c = FontChain::new();
    let err = c.push_file(&fonts_dir().join("nope.ttf")).unwrap_err();
    assert!(err.to_string().contains("nope.ttf"), "{err}");
    let err = c
        .push_bytes("junk", b"not a font at all".to_vec())
        .unwrap_err();
    assert!(err.to_string().contains("junk"), "{err}");
    assert!(c.is_empty());
}

#[test]
fn resolve_walks_the_chain_and_loads_lazily() {
    let mut c = chain();
    assert_eq!(c.resolve('A'), Some(FontId(0)));
    assert!(!c.is_loaded(FontId(1)), "Latin must not wake the CJK font");
    let t = Instant::now();
    assert_eq!(c.resolve('가'), Some(FontId(1)));
    let took = t.elapsed();
    eprintln!(
        "font: lazy-loaded Noto Sans KR in {} ms (V-10, host)",
        took.as_millis()
    );
    assert!(c.is_loaded(FontId(1)));
    assert_eq!(
        c.resolve('漢'),
        Some(FontId(1)),
        "hanja comes from the CJK font"
    );
    assert_eq!(c.resolve('あ'), Some(FontId(1)), "kana too");
    assert_eq!(c.resolve('\u{1F600}'), None, "no emoji anywhere");
}

#[test]
fn a_lazy_font_that_fails_is_skipped_quietly() {
    let mut c = FontChain::new();
    c.push_file(&open_sans()).unwrap();
    c.push_lazy_file(&fonts_dir().join("missing.otf"));
    c.push_lazy_file(&noto_kr());
    assert_eq!(c.resolve('가'), Some(FontId(2)));
    assert!(!c.is_loaded(FontId(1)));
    assert_eq!(c.resolve('나'), Some(FontId(2)));
}

#[test]
fn a_failed_first_slot_keeps_the_line_box_of_the_fonts_behind_it() {
    // The shape a card pack can produce: a preferred font the frontend cannot parse, with the
    // UI font behind it. Glyph resolution already falls through to the next slot; the line box
    // has to follow it, or a line would have zero height and draw nothing at all.
    let broken = std::env::temp_dir().join(format!("slot2-text-broken-{}.ttf", std::process::id()));
    std::fs::write(&broken, b"not a font at all").unwrap();

    let mut alone = FontChain::new();
    alone.push_file(&open_sans()).unwrap();
    let want = alone.measure("Tetris", 16.0);

    let mut c = FontChain::new();
    c.push_lazy_file(&broken);
    c.push_file(&open_sans()).unwrap();

    let got = c.measure("Tetris", 16.0);
    assert!(got.ascent > 0.0 && got.descent > 0.0, "{got:?}");
    assert_eq!(
        (got.ascent, got.descent, got.line_height),
        (want.ascent, want.descent, want.line_height),
        "the line box is not the healthy font's"
    );
    assert_eq!(
        got.width, want.width,
        "advances changed with the slot order"
    );

    let b = c.rasterize("Tetris", 16.0);
    assert_eq!(b.height, want.line_height);
    assert_eq!(b.data.len(), (b.width * b.height) as usize);
    assert!(b.ink() > 0, "the text behind the broken slot was not drawn");

    // The failure sticks: the next measurements do not try the file again, and the result is
    // the same one every time.
    assert!(!c.is_loaded(FontId(0)), "the broken file was retried");
    assert_eq!(c.resolve('T'), Some(FontId(1)));
    let again = c.rasterize("Tetris", 16.0);
    assert_eq!(
        (again.width, again.height, again.ink()),
        (b.width, b.height, b.ink())
    );
    let empty = c.measure("", 16.0);
    assert_eq!(empty.line_height, want.line_height);

    let _ = std::fs::remove_file(&broken);
}

#[test]
fn measure_is_positive_monotonic_and_scales() {
    let mut c = chain();
    let a = c.measure("a", 16.0);
    let ab = c.measure("ab", 16.0);
    let empty = c.measure("", 16.0);
    assert!(a.width > 0.0);
    assert!(ab.width > a.width);
    assert_eq!(empty.width, 0.0);
    assert!(empty.line_height > 0 && empty.line_height == a.line_height);
    assert!(a.ascent > 0.0 && a.descent > 0.0);
    assert_eq!(a.line_height, (a.ascent + a.descent).ceil() as u32);
    let big = c.measure("ab", 32.0);
    let ratio = big.width / ab.width;
    assert!((1.8..=2.2).contains(&ratio), "ratio {ratio}");
}

#[test]
fn line_height_comes_from_the_first_font_even_for_hangul() {
    let mut c = chain();
    let latin = c.measure("Tetris", 20.0);
    let hangul = c.measure("포켓몬", 20.0);
    assert_eq!(latin.line_height, hangul.line_height);
    assert_eq!(latin.ascent, hangul.ascent);
    assert!(hangul.width > 0.0);
}

#[test]
fn rasterize_matches_measure_and_has_ink() {
    let mut c = chain();
    let m = c.measure("Tetris", 24.0);
    let b = c.rasterize("Tetris", 24.0);
    assert_eq!(b.width, m.width.ceil() as u32);
    assert_eq!(b.height, m.line_height);
    assert_eq!(b.baseline, m.ascent.round() as u32);
    assert_eq!(b.data.len(), (b.width * b.height) as usize);
    assert!(b.ink() > 50, "ink {}", b.ink());
    assert!(
        b.data.contains(&255),
        "solid strokes should reach full coverage"
    );
    // Nothing above the top row or below the bottom row is lost: the tallest glyph 'T'
    // must touch the upper half, and nothing sits on the very last row for this string.
    let upper: usize = (0..b.height / 2)
        .map(|y| (0..b.width).filter(|&x| b.get(x, y) > 0).count())
        .sum();
    assert!(upper > 0);
}

#[test]
fn rasterize_mixed_scripts_on_one_line() {
    let mut c = chain();
    let ko = c.rasterize("포켓몬", 24.0);
    let mixed = c.rasterize("포켓몬 Ruby", 24.0);
    assert!(mixed.width > ko.width);
    assert_eq!(mixed.height, ko.height);
    assert!(mixed.ink() > ko.ink());
}

#[test]
fn empty_and_missing_glyphs_do_not_panic() {
    let mut c = chain();
    let e = c.rasterize("", 16.0);
    assert_eq!(e.width, 1);
    assert_eq!(e.ink(), 0);
    let m = c.measure("\u{1F600}", 16.0);
    assert!(
        m.width > 0.0,
        "a missing glyph still advances (tofu or half em)"
    );
    let b = c.rasterize("\u{1F600}", 16.0);
    assert_eq!(b.width, m.width.ceil() as u32);
    // Some font in the chain has WHITE SQUARE (Noto does; Open Sans does not), so the
    // tofu box is drawn rather than a blank advance.
    assert!(c.resolve(TOFU).is_some());
    assert!(b.ink() > 0);
}

#[test]
fn glyph_cache_grows_once_per_new_glyph() {
    let mut c = chain();
    assert_eq!(c.cache_len(), 0);
    c.rasterize("aa", 16.0);
    assert_eq!(c.cache_len(), 1);
    c.rasterize("ab", 16.0);
    assert_eq!(c.cache_len(), 2);
    c.rasterize("ab", 16.0);
    assert_eq!(c.cache_len(), 2);
    c.rasterize("ab", 17.0);
    assert_eq!(c.cache_len(), 4, "size is part of the key");
    c.rasterize("가", 16.0);
    assert_eq!(c.cache_len(), 5);
}

#[test]
fn no_font_at_all_is_blank_not_a_crash() {
    let mut c = FontChain::new();
    let m = c.measure("abc", 16.0);
    assert_eq!(m.line_height, 0);
    assert!(m.width > 0.0);
    let b = c.rasterize("abc", 16.0);
    assert_eq!(b.height, 0);
    assert!(b.data.is_empty());
    assert_eq!(c.resolve('a'), None);
}
