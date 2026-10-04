//! The card's file-name contract: what a scanned name becomes, and the order it is shown in.
//!
//! Two things M5 asked for are fixed here, and both are about the names a real card holds —
//! Korean, English, kana and hanja mixed in one folder, with brackets, spaces and punctuation
//! in between:
//!
//! 1. **Order.** `Cart::title` sorts by Rust `str` order, which is Unicode scalar value order
//!    (UTF-8 byte order preserves it). No locale collation, no natural-number ordering, no
//!    case folding: `Alpha < alpha`, and within Hangul syllables `가 < 각 < 나`. The order does
//!    not depend on the order the files were created in.
//! 2. **Names.** Exactly one trailing accepted extension is removed, and nothing else is
//!    touched: spaces, dots, parentheses, brackets, `'`, `+`, `&`, `!`, `#`, `%`, `@`, `_`, `-`
//!    and the script itself all survive into `stem`/`title`, and that exact stem is the one
//!    every sidecar path uses (`Labels/`, `Saves/`, `States/`, `System/games/`,
//!    `System/cheats/`).
//!
//! The limits are part of the contract too. A name that starts with `.` or `._`, a directory,
//! and a file whose *last* extension the platform does not accept (`name.gba.bak`) are not
//! carts. Only the extension is matched case-insensitively; the stem's own spelling is kept.
//! Two allowed extensions of one stem (`Twin.sfc` and `Twin.smc`) are two carts that share
//! every sidecar path — the legacy layout has room for one save per stem, and this is where
//! that is written down. Nothing is normalised, renamed or hashed: a name this frontend cannot
//! read as UTF-8 is left alone rather than mangled into a name that collides with another file,
//! and a name the card or the host could not hold in the first place is not this layer's to
//! invent a spelling for.
//!
//! One thing the tests have to work around: Windows file names are case-insensitive, so
//! `Alpha.gba` and `alpha.gba` cannot both exist in one folder and the `Alpha < alpha` rule is
//! asserted with two names that differ in something else as well. Nothing here depends on
//! `read_dir` order or on the order the files were created in.

use std::fs;
use std::path::{Component, Path, PathBuf};

use slot2_store::{Card, Cart, Platform, StateKind};

/// A card in a folder of this test's own: the process id and the test's own name, so two tests
/// running at once never share a directory. Removed when the test ends.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(test: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("slot2-filename-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let scratch = Scratch { root };
        scratch.card().ensure_layout();
        scratch
    }

    fn card(&self) -> Card {
        Card::new(&self.root)
    }

    /// A ROM whose name is written exactly as the caller wrote it.
    fn rom(&self, p: Platform, name: &str) -> PathBuf {
        let dir = self.card().games_dir(p);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, b"rom").unwrap();
        path
    }

    fn titles(&self, p: Platform) -> Vec<String> {
        self.card()
            .scan(p)
            .iter()
            .map(|c| c.title.clone())
            .collect()
    }

    /// The scanned cart with this stem — the one the shelf would hand the rest of the frontend.
    fn cart(&self, p: Platform, stem: &str) -> Cart {
        self.card()
            .scan(p)
            .into_iter()
            .find(|c| c.stem == stem)
            .unwrap_or_else(|| panic!("no cart named {stem:?} was scanned"))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The five destinations one cart's stem points at. Built here rather than asked of
/// `Card::label`, which answers only when the picture is really on the card.
fn sidecars(card: &Card, cart: &Cart) -> Vec<PathBuf> {
    vec![
        card.root()
            .join("Labels")
            .join(cart.platform.folder())
            .join(format!("{}.png", cart.stem)),
        card.save_path(cart),
        card.states_dir(cart),
        card.game_settings_path(cart),
        card.cheat_path(cart),
    ]
}

fn has_no_parent_dir(p: &Path) -> bool {
    !p.components().any(|c| c == Component::ParentDir)
}

// ---------------------------------------------------------------- order

#[test]
fn mixed_script_names_sort_by_scalar_value() {
    let scratch = Scratch::new("order");
    // Created in an order that is not the shelf's order, on purpose: the scan's answer may not
    // depend on the order the files were written in.
    for name in [
        "나무.gba",
        "漢字.gba",
        "alpha.gba",
        "각도.gba",
        "Zed.gba",
        "あいう.gba",
        "가방.gba",
    ] {
        scratch.rom(Platform::Gba, name);
    }

    // `Zed` before `alpha` is the whole of it: sorts are by Unicode scalar value, not by
    // dictionary order, where `apple` would come before `Zed`.
    assert_eq!(
        scratch.titles(Platform::Gba),
        vec!["Zed", "alpha", "あいう", "漢字", "가방", "각도", "나무"],
        "ASCII upper, then lower, then kana, hanja and Hangul syllables by code point"
    );

    let titles = scratch.titles(Platform::Gba);
    let at = |t: &str| titles.iter().position(|x| x == t).unwrap();
    assert!(at("Zed") < at("alpha"), "upper case is not before lower");
    assert!(
        at("가방") < at("각도") && at("각도") < at("나무"),
        "Hangul syllables are not in 가나다 order"
    );

    // The same rule, with the two titles spelled apart only by case. Two such ROMs cannot
    // share one folder on a case-insensitive filesystem, so they are two extensions of one
    // stem: the file names differ, and the order asserted is still the titles'.
    let twins = Scratch::new("order-case");
    twins.rom(Platform::Snes, "Alpha.sfc");
    twins.rom(Platform::Snes, "alpha.smc");
    assert_eq!(
        twins.titles(Platform::Snes),
        vec!["Alpha", "alpha"],
        "`Alpha` is not before `alpha`"
    );

    // The same files, written in another order: the same shelf.
    let other = Scratch::new("order-reversed");
    for name in [
        "가방.gba",
        "Zed.gba",
        "각도.gba",
        "alpha.gba",
        "나무.gba",
        "あいう.gba",
        "漢字.gba",
    ] {
        other.rom(Platform::Gba, name);
    }
    assert_eq!(other.titles(Platform::Gba), titles);
}

// ---------------------------------------------------------------- names

/// The file name without its last extension, spelled out here so the test does not borrow the
/// implementation's own answer.
fn stem_of(name: &str) -> String {
    let (stem, _) = name
        .rsplit_once('.')
        .expect("the test names have extensions");
    stem.to_string()
}

#[test]
fn only_the_last_extension_is_removed_and_the_rest_is_kept() {
    let scratch = Scratch::new("special");
    let spaced = "10-in-1 [한글] + 日本語 (Rev A)!.gba";
    let punctuation = "Alpha.beta's & more #50%@home.GBA";
    let a_rom = scratch.rom(Platform::Gba, spaced);
    let b_rom = scratch.rom(Platform::Gba, punctuation);

    let carts = scratch.card().scan(Platform::Gba);
    assert_eq!(carts.len(), 2, "a special name was dropped: {carts:?}");

    let a = scratch.cart(Platform::Gba, &stem_of(spaced));
    assert_eq!(a.title, a.stem, "title is the stem as scanned");
    assert_eq!(a.stem, stem_of(spaced));
    assert_eq!(a.rom, a_rom, "the ROM path was rewritten");

    // The dots inside the name are not extensions, and the last one is removed case-blind.
    let b = scratch.cart(Platform::Gba, &stem_of(punctuation));
    assert_eq!(b.title, b.stem);
    assert_eq!(b.stem, stem_of(punctuation));
    assert_eq!(b.rom, b_rom);
}

#[test]
fn a_special_stem_reaches_every_sidecar_path() {
    let scratch = Scratch::new("sidecars");
    let stem = "10-in-1 [한글] + 日本語 (Rev A)!";
    scratch.rom(Platform::Gba, &format!("{stem}.gba"));
    let card = scratch.card();
    let cart = scratch.cart(Platform::Gba, stem);

    let label = card
        .root()
        .join("Labels")
        .join("GBA")
        .join(format!("{stem}.png"));
    assert_eq!(card.label(&cart), None, "a label nobody wrote was found");
    fs::create_dir_all(label.parent().unwrap()).unwrap();
    fs::write(&label, b"png").unwrap();
    assert_eq!(card.label(&cart), Some(label.clone()));

    let want = [
        label.clone(),
        card.root()
            .join("Saves")
            .join("GBA")
            .join(format!("{stem}.sav")),
        card.root().join("States").join("GBA").join(stem),
        card.root()
            .join("System")
            .join("games")
            .join("GBA")
            .join(format!("{stem}.ini")),
        card.root()
            .join("System")
            .join("cheats")
            .join("GBA")
            .join(format!("{stem}.cht")),
    ];
    assert_eq!(sidecars(&card, &cart), want.to_vec());

    // The save round-trips through that path, and a state lands in that directory.
    assert_eq!(card.read_save(&cart), None);
    card.write_save(&cart, b"save bytes").unwrap();
    assert_eq!(card.read_save(&cart), Some(b"save bytes".to_vec()));
    card.write_state(&cart, StateKind::Numbered(1), b"state", None)
        .unwrap();
    assert_eq!(
        card.read_state(&cart, StateKind::Numbered(1)),
        Some(b"state".to_vec())
    );

    // None of them climbed out of the card, and each one is named after the stem.
    let files = [
        label.clone(),
        card.save_path(&cart),
        card.game_settings_path(&cart),
        card.cheat_path(&cart),
    ];
    for path in &files {
        assert!(path.starts_with(card.root()), "{path:?} left the card");
        assert!(has_no_parent_dir(path), "{path:?} has a `..` in it");
        let expected = format!(
            "{stem}.{}",
            path.extension().and_then(|e| e.to_str()).unwrap()
        );
        assert_eq!(
            path.file_name().and_then(|n| n.to_str()),
            Some(expected.as_str()),
            "the file is not named after the stem"
        );
    }
    let states = card.states_dir(&cart);
    assert!(states.starts_with(card.root().join("States").join("GBA")));
    assert!(has_no_parent_dir(&states));
    assert_eq!(states.file_name().and_then(|n| n.to_str()), Some(stem));
}

#[test]
fn skipped_names_and_extension_case() {
    let scratch = Scratch::new("skips");
    let dir = scratch.card().games_dir(Platform::Gba);
    fs::create_dir_all(&dir).unwrap();
    for name in [
        ".hidden.gba",  // hidden
        "._Zelda.gba",  // AppleDouble metadata
        "name.gba.bak", // the last extension is not one this platform plays
        "packed.zip",   // not a ROM at all
        "readme",       // no extension at all
        "Case.GBA",     // the extension is matched case-blind...
        "Case2.GbA",    //
    ] {
        fs::write(dir.join(name), b"x").unwrap();
    }
    fs::create_dir_all(dir.join("subdir.gba")).unwrap();

    let carts = scratch.card().scan(Platform::Gba);
    assert_eq!(
        scratch.titles(Platform::Gba),
        vec!["Case", "Case2"],
        "only the two real ROMs are carts: {carts:?}"
    );
    assert_eq!(
        carts.iter().map(|c| c.rom.clone()).collect::<Vec<_>>(),
        vec![dir.join("Case.GBA"), dir.join("Case2.GbA")],
        "the scanned path is the file that is there"
    );
    // ...while the stem keeps the spelling it had: the extension is not part of it, and the
    // rest of the name is not touched.
    assert!(carts.iter().all(|c| c.stem == c.title));
    assert!(carts.iter().all(|c| c.rom.is_file()));
}

// ---------------------------------------------------------------- one stem, several files

#[test]
fn one_stem_with_two_snes_extensions_is_two_carts_that_share_everything_else() {
    let scratch = Scratch::new("twin");
    let sfc = scratch.rom(Platform::Snes, "Twin.sfc");
    let smc = scratch.rom(Platform::Snes, "Twin.smc");
    let card = scratch.card();
    let carts = card.scan(Platform::Snes);
    assert_eq!(carts.len(), 2, "one of the two files is missing: {carts:?}");
    // The order is the sort's second key: extension bytes. On a filesystem that hands names
    // back in name order (Windows, NTFS) that coincides with the directory's own order, so what
    // is pinned here is the answer the shelf shows rather than the mechanism behind it; on a
    // filesystem with a hashed order it is the tie-break that decides.
    assert_eq!(
        carts.iter().map(|c| c.rom.clone()).collect::<Vec<_>>(),
        vec![sfc, smc],
        "equal stems are ordered by extension (byte order), not by read_dir order"
    );
    assert_eq!(carts[0].stem, carts[1].stem);
    assert_ne!(carts[0].rom, carts[1].rom);

    // One save, one set of states, one label, one settings file, one cheat file: the legacy
    // layout has room for one of each per stem, and both carts use the same ones. This is the
    // price of a card that came with two spellings of one game's extension.
    let a = sidecars(&card, &carts[0]);
    let b = sidecars(&card, &carts[1]);
    assert_eq!(a, b, "one stem, two sidecar sets");
    assert!(a.iter().all(|p| p.to_string_lossy().contains("Twin")));

    // And the other extensions of this frontend are not this platform's: the same stem with a
    // Mega Drive extension is not a SNES cart.
    scratch.rom(Platform::Snes, "Twin.gen");
    assert_eq!(
        card.scan(Platform::Snes).len(),
        2,
        "an extension SNES does not play was scanned"
    );
}

#[test]
fn the_same_stem_on_two_platforms_shares_nothing() {
    let scratch = Scratch::new("platforms");
    scratch.rom(Platform::Gb, "Tetris.gb");
    scratch.rom(Platform::Gba, "Tetris.gba");
    let card = scratch.card();
    let gb = scratch.cart(Platform::Gb, "Tetris");
    let gba = scratch.cart(Platform::Gba, "Tetris");
    assert_eq!(gb.stem, gba.stem);

    for (a, b) in sidecars(&card, &gb)
        .iter()
        .zip(sidecars(&card, &gba).iter())
    {
        assert_ne!(a, b, "a sidecar path is shared between platforms");
    }
    assert!(
        card.save_path(&gb)
            .starts_with(card.root().join("Saves").join("GB")),
        "the Game Boy save is not in the Game Boy folder"
    );
}
