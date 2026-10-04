//! The game's shader choice, `shader` in `System/games/<PLAT>/<stem>.ini`.
//!
//! The file is a list of decisions, and the shader key has three states rather than two: absent
//! (inherit the platform's default preset), a preset name, and `none` (the player turned
//! shaders off for this game). A typo must not become the third one, and every write here goes
//! through the same safe path the other game settings already use.

use std::fs;
use std::path::PathBuf;

use slot2_store::{Card, Cart, Error, GameSettings, Ini, Platform, ScaleMode, ShaderPreset};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-store-shader-{}-{name}", std::process::id()));
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

const ALL: [ShaderPreset; 5] = [
    ShaderPreset::Off,
    ShaderPreset::SharpBilinear,
    ShaderPreset::Lcd3x,
    ShaderPreset::ZfastCrt,
    ShaderPreset::Scanline,
];

// ------------------------------------------------------------------ the type

#[test]
fn the_canonical_spelling_of_each_preset_is_the_one_on_the_card() {
    let canonical = [
        (ShaderPreset::Off, "none"),
        (ShaderPreset::SharpBilinear, "sharp-bilinear"),
        (ShaderPreset::Lcd3x, "lcd3x"),
        (ShaderPreset::ZfastCrt, "zfast-crt"),
        (ShaderPreset::Scanline, "scanline"),
    ];
    for (preset, text) in canonical {
        assert_eq!(preset.as_str(), text, "{preset:?} has the wrong spelling");
        assert_eq!(
            ShaderPreset::parse(text),
            Some(preset),
            "{text} did not parse back"
        );
    }
}

#[test]
fn padding_and_capitalisation_are_read() {
    for (text, want) in [
        ("None", ShaderPreset::Off),
        (" NONE ", ShaderPreset::Off),
        ("Sharp-Bilinear", ShaderPreset::SharpBilinear),
        ("  sharp-bilinear\t", ShaderPreset::SharpBilinear),
        ("LCD3X", ShaderPreset::Lcd3x),
        ("ZFAST-CRT", ShaderPreset::ZfastCrt),
        ("Scanline", ShaderPreset::Scanline),
    ] {
        assert_eq!(
            ShaderPreset::parse(text),
            Some(want),
            "{text:?} was not read"
        );
    }
}

#[test]
fn the_spellings_a_person_would_actually_write_are_accepted() {
    for (text, want) in [
        ("off", ShaderPreset::Off),
        ("sharp_bilinear", ShaderPreset::SharpBilinear),
        ("sharpbilinear", ShaderPreset::SharpBilinear),
        ("lcd-3x", ShaderPreset::Lcd3x),
        ("zfast_crt", ShaderPreset::ZfastCrt),
        ("zfastcrt", ShaderPreset::ZfastCrt),
        ("scanlines", ShaderPreset::Scanline),
    ] {
        assert_eq!(
            ShaderPreset::parse(text),
            Some(want),
            "{text:?} was not read"
        );
    }

    // Nothing unrecognised, and nothing empty, is a preset — and a near miss is not "off".
    for text in ["", "  ", "nonee", "non", "shader", "off2", "crt", "lcd"] {
        assert_eq!(
            ShaderPreset::parse(text),
            None,
            "{text:?} was accepted as a preset"
        );
    }
}

// ------------------------------------------------------------------ reading

#[test]
fn a_game_with_no_shader_key_inherits_the_platform_default() {
    let d = tempdir("inherit");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);

    assert_eq!(GameSettings::default().shader, None);
    assert!(GameSettings::default().is_default());
    assert_eq!(card.read_settings(&cart).shader, None);
    assert!(
        !GameSettings {
            shader: Some(ShaderPreset::Off),
            ..Default::default()
        }
        .is_default(),
        "an explicit Off was mistaken for the default"
    );

    // A setting that is not the shader leaves the shader inheriting.
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "scale = fill\n").unwrap();
    assert_eq!(card.read_settings(&cart).shader, None);
    assert_eq!(card.read_settings(&cart).scale, Some(ScaleMode::Fill));
}

#[test]
fn an_explicit_off_is_not_the_same_as_saying_nothing() {
    let d = tempdir("none-vs-absent");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    fs::write(&path, "shader = none\n").unwrap();
    assert_eq!(
        card.read_settings(&cart).shader,
        Some(ShaderPreset::Off),
        "an explicit Off read back as inheritance"
    );

    fs::write(&path, "\n# nothing at all\n").unwrap();
    assert_eq!(
        card.read_settings(&cart).shader,
        None,
        "an absent key read back as an explicit Off"
    );
}

#[test]
fn an_empty_or_unknown_value_is_inheritance_and_a_read_leaves_the_file_alone() {
    let d = tempdir("typo");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // A misspelling, an empty value, and a value that only looks like a preset.
    for text in [
        "shader = \n",
        "shader =\n",
        "shader = nonsense\n",
        "shader = sharp-biliner\n",
        "shader = scanline2\n",
        "future_shader = lcd3x\n",
    ] {
        fs::write(&path, text).unwrap();
        assert_eq!(
            card.read_settings(&cart).shader,
            None,
            "{text:?} was not inheritance"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            text,
            "a read rewrote {text:?}"
        );
    }
}

// ------------------------------------------------------------------ writing

#[test]
fn every_preset_round_trips_canonically_through_the_card() {
    let d = tempdir("round-trip");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);

    for preset in ALL {
        card.write_settings(
            &cart,
            &GameSettings {
                shader: Some(preset),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("shader = {}\n", preset.as_str()),
            "{preset:?} was not written canonically"
        );
        assert_eq!(card.read_settings(&cart).shader, Some(preset));
    }
}

#[test]
fn an_explicit_off_is_stored_as_the_word_none() {
    let d = tempdir("off-spelling");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);

    card.write_settings(
        &cart,
        &GameSettings {
            shader: Some(ShaderPreset::Off),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "shader = none\n");
    assert_eq!(card.read_settings(&cart).shader, Some(ShaderPreset::Off));

    // A hand-edited spelling is read, and the next save rewrites it canonical.
    fs::write(&path, "shader = OFF\n").unwrap();
    let mut s = card.read_settings(&cart);
    assert_eq!(s.shader, Some(ShaderPreset::Off));
    s.shader = Some(ShaderPreset::ZfastCrt);
    card.write_settings(&cart, &s).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "shader = zfast-crt\n");
}

#[test]
fn inheriting_again_removes_only_the_shader_key_and_then_the_file() {
    let d = tempdir("back-to-default");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);

    // Somebody else's key keeps the file, and is left exactly as it was.
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "future_filter = keep\nshader = lcd3x\n").unwrap();
    let mut s = card.read_settings(&cart);
    assert_eq!(s.shader, Some(ShaderPreset::Lcd3x));
    s.shader = None;
    card.write_settings(&cart, &s).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "future_filter = keep\n",
        "the shader key did not go cleanly"
    );
    assert_eq!(card.read_settings(&cart).shader, None);

    // The shader was the last decision in the file: the file goes with it.
    fs::write(&path, "shader = scanline\n").unwrap();
    let mut s = card.read_settings(&cart);
    s.shader = None;
    card.write_settings(&cart, &s).unwrap();
    assert!(
        !path.exists(),
        "an all-default game left a file behind after the shader went"
    );
    assert_eq!(card.read_settings(&cart), GameSettings::default());

    // And with no file at all, that is a no-op success rather than an error.
    card.write_settings(&cart, &GameSettings::default())
        .unwrap();
    assert!(!path.exists(), "a default write created a file");
}

#[test]
fn a_shader_change_keeps_the_other_settings_and_the_unknown_keys() {
    let d = tempdir("shader-preserves");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);
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
    let mut s = card.read_settings(&cart);
    assert_eq!(s.shader, Some(ShaderPreset::Lcd3x));
    s.shader = Some(ShaderPreset::SharpBilinear);
    card.write_settings(&cart, &s).unwrap();

    let back = Ini::load(&path).unwrap();
    assert_eq!(back.get("shader"), Some("sharp-bilinear"));
    assert_eq!(back.get("core"), Some("gambatte_libretro"));
    assert_eq!(back.get("scale"), Some("aspect"));
    assert_eq!(back.get("overscan"), Some("off"));
    assert_eq!(back.get("rewind"), Some("off"));
    assert_eq!(back.get("future_filter"), Some("keep"));
}

#[test]
fn another_setting_s_change_keeps_the_shader_and_the_unknown_keys() {
    let d = tempdir("other-preserves");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "shader = zfast-crt\nfuture_filter = keep\n").unwrap();

    let mut s = card.read_settings(&cart);
    assert_eq!(s.shader, Some(ShaderPreset::ZfastCrt));
    s.core = Some("gpsp".into());
    card.write_settings(&cart, &s).unwrap();
    let back = Ini::load(&path).unwrap();
    assert_eq!(
        back.get("shader"),
        Some("zfast-crt"),
        "core rewrite lost it"
    );
    assert_eq!(back.get("core"), Some("gpsp"));
    assert_eq!(back.get("future_filter"), Some("keep"));

    let mut s = card.read_settings(&cart);
    s.scale = Some(ScaleMode::Fill);
    card.write_settings(&cart, &s).unwrap();
    let back = Ini::load(&path).unwrap();
    assert_eq!(
        back.get("shader"),
        Some("zfast-crt"),
        "scale rewrite lost it"
    );
    assert_eq!(back.get("scale"), Some("fill"));
    assert_eq!(back.get("future_filter"), Some("keep"));
    assert_eq!(
        card.read_settings(&cart).shader,
        Some(ShaderPreset::ZfastCrt),
        "an explicit shader was read back as inheritance"
    );
}

// ------------------------------------------------------------------ failures

#[test]
fn an_unreadable_file_is_never_written_and_keeps_its_bytes() {
    let d = tempdir("invalid-utf8");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let damaged = b"shader = lcd3x\n\xff\xfe not utf-8\n".to_vec();
    fs::write(&path, &damaged).unwrap();

    // Reading stays forgiving — a damaged file must not stop the shelf or the launch.
    assert_eq!(card.read_settings(&cart).shader, None);

    let set = GameSettings {
        shader: Some(ShaderPreset::Scanline),
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
        "a refused shader save left a temp file beside the settings"
    );
}

#[test]
fn a_directory_where_the_settings_file_belongs_is_never_removed() {
    let d = tempdir("directory");
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = cart(&card, Platform::Gba, "arm");
    let path = card.game_settings_path(&cart);
    fs::create_dir_all(&path).unwrap();

    let set = GameSettings {
        shader: Some(ShaderPreset::Lcd3x),
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
    assert_eq!(card.read_settings(&cart), GameSettings::default());
}
