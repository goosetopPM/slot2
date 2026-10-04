//! The game's overlay choice, `overlay` in `System/games/<PLAT>/<stem>.ini`.
//!
//! One PNG per platform and geometry is D-11's contract, so what a card stores is a yes or a no
//! and never a file name. The key has three states rather than two: absent (inherit the
//! platform's default, which today is no overlay anywhere), `on` (this game wants the picture)
//! and `off` (this game does not). A typo must not become any of them — and nothing here looks
//! at whether a picture exists, because a missing PNG is the loader's silence to keep rather
//! than a reason to throw away what the player asked for.

use std::fs;
use std::path::PathBuf;

use slot2_store::{Card, Cart, Error, GameSettings, Ini, Platform, ScaleMode, ShaderPreset};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-store-overlay-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn cart(card: &Card, p: Platform, stem: &str) -> Cart {
    let rom = card.games_dir(p).join(format!("{stem}.gba"));
    fs::create_dir_all(rom.parent().unwrap()).unwrap();
    fs::write(&rom, b"rom").unwrap();
    Cart {
        platform: p,
        stem: stem.into(),
        title: stem.into(),
        rom,
    }
}

/// A card with a game on it, and the path its settings live at.
fn fixture(name: &str) -> (Card, Cart, PathBuf) {
    let d = tempdir(name);
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);
    (card, cart, path)
}

fn write(card: &Card, cart: &Cart, overlay: Option<bool>) {
    let mut s = card.read_settings(cart);
    s.overlay = overlay;
    card.write_settings(cart, &s).unwrap();
}

// ------------------------------------------------------------------ the type

#[test]
fn the_default_game_has_no_overlay_choice() {
    // A game nobody has touched says nothing about overlays, and that is not the same as having
    // said no: a platform default added later must reach it.
    assert_eq!(GameSettings::default().overlay, None);
    assert!(GameSettings::default().is_default());
    for choice in [Some(true), Some(false)] {
        assert!(
            !GameSettings {
                overlay: choice,
                ..Default::default()
            }
            .is_default(),
            "{choice:?} was mistaken for the default"
        );
    }
}

#[test]
fn the_three_overlay_states_round_trip_through_the_card() {
    let (card, cart, path) = fixture("round-trip");

    // No file at all: the key is absent.
    assert_eq!(card.read_settings(&cart).overlay, None);
    assert!(!path.exists(), "reading created a settings file");

    for want in [Some(true), Some(false), None] {
        write(&card, &cart, want);
        assert_eq!(
            card.read_settings(&cart).overlay,
            want,
            "{want:?} did not come back"
        );
        if want.is_none() {
            assert!(
                !path.exists(),
                "a file was left holding nothing but the absence of a choice"
            );
        }
    }

    // The three are three: an explicit on and an explicit off are not the read of the other, and
    // neither of them is the absence of the key that a future platform default would fill in.
    let states = [None, Some(true), Some(false)];
    for (i, a) in states.iter().enumerate() {
        for b in &states[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn every_spelling_a_person_might_write_is_read_and_saved_canonically() {
    let (card, cart, path) = fixture("spellings");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    for (text, want) in [
        ("on", true),
        ("true", true),
        ("yes", true),
        ("1", true),
        ("ON", true),
        ("True", true),
        ("YES", true),
        ("off", false),
        ("false", false),
        ("no", false),
        ("0", false),
        ("OFF", false),
        ("False", false),
        ("NO", false),
    ] {
        // Padding, case and a trailing comment-free newline are all the same decision.
        fs::write(&path, format!("overlay =  {text}  \n")).unwrap();
        assert_eq!(
            card.read_settings(&cart).overlay,
            Some(want),
            "{text:?} was not read"
        );

        // And the next save writes the one spelling the file format owns.
        write(&card, &cart, Some(want));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("overlay = {}\n", if want { "on" } else { "off" }),
            "{text:?} was not rewritten canonically"
        );
    }
}

#[test]
fn an_empty_or_unknown_value_is_no_choice_and_a_read_leaves_the_file_alone() {
    let (card, cart, path) = fixture("typo");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // Values a person gets wrong, plus a key that only looks like this one.
    for text in [
        "overlay = \n",
        "overlay =\n",
        "overlay = maybe\n",
        "overlay = onn\n",
        "overlay = overlay\n",
        "overlay = y\n",
        "overlay = 2\n",
        "overlays = on\n",
    ] {
        fs::write(&path, text).unwrap();
        assert_eq!(
            card.read_settings(&cart).overlay,
            None,
            "{text:?} was not the absence of a choice"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            text,
            "a read rewrote {text:?}"
        );
    }

    // A file that holds other settings and a typo for this one reads as inheritance for the
    // overlay and keeps everything else.
    fs::write(&path, "scale = fill\noverlay = onn\n").unwrap();
    let s = card.read_settings(&cart);
    assert_eq!(s.overlay, None);
    assert_eq!(s.scale, Some(ScaleMode::Fill));
}

#[test]
fn the_store_never_asks_whether_the_picture_exists() {
    // `on` is a request for `assets/overlays/<platform>/<geometry>.png` or its card override,
    // and neither is on this card. The choice is still the player's, and reading it creates
    // nothing: no overlay folder, no second file.
    let (card, cart, path) = fixture("no-picture");
    write(&card, &cart, Some(true));
    assert_eq!(card.read_settings(&cart).overlay, Some(true));
    assert!(
        !card.root().join("System").join("Overlays").exists(),
        "reading a choice created an overlay folder"
    );
    assert_eq!(
        fs::read_dir(path.parent().unwrap()).unwrap().count(),
        1,
        "the settings folder holds something besides the game's own file"
    );

    // And a picture appearing or going away later cannot change what is stored.
    fs::write(&path, "overlay = on\n").unwrap();
    assert_eq!(card.read_settings(&cart).overlay, Some(true));
}

// ------------------------------------------------------------------ writing

#[test]
fn clearing_the_choice_removes_only_the_overlay_key_and_then_the_file() {
    let (card, cart, path) = fixture("cleared");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // Somebody else's key keeps the file, and is left exactly as it was.
    fs::write(&path, "future_filter = keep\noverlay = on\n").unwrap();
    assert_eq!(card.read_settings(&cart).overlay, Some(true));
    write(&card, &cart, None);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "future_filter = keep\n",
        "the overlay key did not go cleanly"
    );
    assert_eq!(card.read_settings(&cart).overlay, None);

    // The choice was the only decision in the file: the file goes with it.
    fs::write(&path, "overlay = off\n").unwrap();
    write(&card, &cart, None);
    assert!(
        !path.exists(),
        "an all-default game left a file behind after the overlay went"
    );
    assert_eq!(card.read_settings(&cart), GameSettings::default());

    // And with no file at all, clearing is a no-op success rather than an error.
    card.write_settings(&cart, &GameSettings::default())
        .unwrap();
    assert!(!path.exists(), "a default write created a file");
}

#[test]
fn an_overlay_change_keeps_the_other_settings_and_the_unknown_keys() {
    let (card, cart, path) = fixture("overlay-preserves");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "future_filter = keep\n").unwrap();

    card.write_settings(
        &cart,
        &GameSettings {
            core: Some("gambatte_libretro".into()),
            scale: Some(ScaleMode::AspectFit),
            overscan: Some(false),
            rewind: Some(false),
            shader: Some(ShaderPreset::Lcd3x),
            overlay: None,
        },
    )
    .unwrap();

    write(&card, &cart, Some(true));
    let back = Ini::load(&path).unwrap();
    assert_eq!(back.get(GameSettings::KEY_OVERLAY), Some("on"));
    assert_eq!(back.get("core"), Some("gambatte_libretro"));
    assert_eq!(back.get("scale"), Some("aspect"));
    assert_eq!(back.get("overscan"), Some("off"));
    assert_eq!(back.get("rewind"), Some("off"));
    assert_eq!(back.get("shader"), Some("lcd3x"));
    assert_eq!(back.get("future_filter"), Some("keep"));

    write(&card, &cart, Some(false));
    let back = Ini::load(&path).unwrap();
    assert_eq!(back.get(GameSettings::KEY_OVERLAY), Some("off"));
    assert_eq!(back.get("core"), Some("gambatte_libretro"));
    assert_eq!(back.get("future_filter"), Some("keep"));

    // And the whole object still reads back as what it was written as.
    let s = card.read_settings(&cart);
    assert_eq!(s.overlay, Some(false));
    assert_eq!(s.core.as_deref(), Some("gambatte_libretro"));
    assert_eq!(s.scale, Some(ScaleMode::AspectFit));
    assert_eq!(s.overscan, Some(false));
    assert_eq!(s.rewind, Some(false));
    assert_eq!(s.shader, Some(ShaderPreset::Lcd3x));
}

#[test]
fn another_setting_s_change_keeps_the_overlay_and_the_unknown_keys() {
    let (card, cart, path) = fixture("other-preserves");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "overlay = on\nfuture_filter = keep\n").unwrap();

    let mut s = card.read_settings(&cart);
    assert_eq!(s.overlay, Some(true));
    s.core = Some("gpsp".into());
    card.write_settings(&cart, &s).unwrap();
    let back = Ini::load(&path).unwrap();
    assert_eq!(
        back.get(GameSettings::KEY_OVERLAY),
        Some("on"),
        "core rewrite"
    );
    assert_eq!(back.get("core"), Some("gpsp"));
    assert_eq!(back.get("future_filter"), Some("keep"));

    let mut s = card.read_settings(&cart);
    s.scale = Some(ScaleMode::Fill);
    card.write_settings(&cart, &s).unwrap();
    let back = Ini::load(&path).unwrap();
    assert_eq!(
        back.get(GameSettings::KEY_OVERLAY),
        Some("on"),
        "scale rewrite"
    );
    assert_eq!(back.get("scale"), Some("fill"));
    assert_eq!(back.get("future_filter"), Some("keep"));

    let mut s = card.read_settings(&cart);
    s.shader = Some(ShaderPreset::Scanline);
    card.write_settings(&cart, &s).unwrap();
    let back = Ini::load(&path).unwrap();
    assert_eq!(
        back.get(GameSettings::KEY_OVERLAY),
        Some("on"),
        "shader rewrite"
    );
    assert_eq!(back.get("shader"), Some("scanline"));
    assert_eq!(back.get("future_filter"), Some("keep"));

    // A hand-edited spelling survives another setting's save as the decision it is.
    fs::write(&path, "core = mgba\noverlay = YES\n").unwrap();
    let mut s = card.read_settings(&cart);
    assert_eq!(s.overlay, Some(true));
    s.rewind = Some(false);
    card.write_settings(&cart, &s).unwrap();
    assert_eq!(card.read_settings(&cart).overlay, Some(true));
}

// ------------------------------------------------------------------ failures

#[test]
fn an_unreadable_file_is_never_written_and_keeps_its_bytes() {
    let (card, cart, path) = fixture("invalid-utf8");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let damaged = b"overlay = on\n\xff\xfe not utf-8\n".to_vec();
    fs::write(&path, &damaged).unwrap();

    // Reading stays forgiving: a damaged file is no choice, not a refusal to show a shelf.
    assert_eq!(card.read_settings(&cart).overlay, None);

    let set = GameSettings {
        overlay: Some(true),
        ..Default::default()
    };
    let clear = GameSettings::default();
    for want in [set, clear] {
        match card.write_settings(&cart, &want) {
            Err(Error::Io(p, _)) => assert_eq!(p, path, "{want:?} failed elsewhere"),
            other => panic!("{want:?} was not refused: {other:?}"),
        }
        assert_eq!(
            fs::read(&path).unwrap(),
            damaged,
            "{want:?} rewrote a file it could not read"
        );
    }
    assert!(
        !path.with_extension("ini.tmp").exists(),
        "a refused overlay save left a temp file beside the settings"
    );
}

#[test]
fn a_directory_where_the_settings_file_belongs_is_never_removed() {
    let (card, cart, path) = fixture("directory");
    fs::create_dir_all(&path).unwrap();

    let set = GameSettings {
        overlay: Some(false),
        ..Default::default()
    };
    let clear = GameSettings::default();
    for want in [set, clear] {
        match card.write_settings(&cart, &want) {
            Err(Error::Io(p, _)) => assert_eq!(p, path, "{want:?} failed elsewhere"),
            other => panic!("{want:?} was not refused: {other:?}"),
        }
        assert!(path.is_dir(), "{want:?} removed the directory");
    }
    assert_eq!(card.read_settings(&cart).overlay, None);
}
