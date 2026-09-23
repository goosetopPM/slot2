//! The contract for slot2-store. Task 07 makes these pass without editing this file.

use std::fs;
use std::path::PathBuf;

use slot2_store::{
    atomic_write, Card, Cart, GameSettings, Ini, Platform, ScaleMode, StateKind, Thumb,
};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-store-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn cart(card: &Card, p: Platform, stem: &str, ext: &str) -> Cart {
    let rom = card.games_dir(p).join(format!("{stem}.{ext}"));
    fs::create_dir_all(rom.parent().unwrap()).unwrap();
    fs::write(&rom, b"rom").unwrap();
    Cart {
        platform: p,
        stem: stem.into(),
        title: stem.into(),
        rom,
    }
}

// ---------- atomic ----------

#[test]
fn atomic_write_creates_parents_and_leaves_no_temp() {
    let d = tempdir("atomic");
    let p = d.join("a/b/c.bin");
    assert!(atomic_write(&p, b"hello").unwrap());
    assert_eq!(fs::read(&p).unwrap(), b"hello");
    assert!(
        !atomic_write(&p, b"hello").unwrap(),
        "identical bytes are not rewritten"
    );
    assert!(atomic_write(&p, b"hello!").unwrap());
    assert_eq!(fs::read(&p).unwrap(), b"hello!");
    let leftovers: Vec<_> = fs::read_dir(p.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(leftovers, vec!["c.bin"], "{leftovers:?}");
}

#[test]
fn atomic_write_reports_io_errors_with_the_path() {
    let d = tempdir("atomic-err");
    let dir_as_file = d.join("dir");
    fs::create_dir_all(&dir_as_file).unwrap();
    let e = atomic_write(&dir_as_file, b"x").unwrap_err();
    assert!(e.to_string().contains("dir"), "{e}");
}

// ---------- layout + scan ----------

#[test]
fn layout_is_created_once() {
    let d = tempdir("layout");
    let card = Card::new(&d);
    let n = card.ensure_layout();
    assert!(n >= 20, "{n}");
    assert!(d.join("Games/GBA").is_dir());
    assert!(d.join("Labels/SMS").is_dir());
    assert!(d.join("System/Lang").is_dir());
    assert_eq!(card.ensure_layout(), 0, "second call creates nothing");
    assert_eq!(card.settings_path(), d.join("System").join("slot2.ini"));
}

#[test]
fn scan_filters_by_extension_and_sorts_by_title() {
    let d = tempdir("scan");
    let card = Card::new(&d);
    let g = card.games_dir(Platform::Gba);
    fs::create_dir_all(&g).unwrap();
    for name in [
        "Zelda.gba",
        "apotris.GBA",
        "Metroid Fusion (USA).gba",
        "포켓몬스터 루비.gba",
        "ファイアーエムブレム.gba",
        "notes.txt",
        "packed.zip",
        "._Zelda.gba",
        ".hidden.gba",
    ] {
        fs::write(g.join(name), b"x").unwrap();
    }
    fs::create_dir_all(g.join("subdir.gba")).unwrap();
    let carts = card.scan(Platform::Gba);
    let titles: Vec<&str> = carts.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(
        titles,
        vec![
            "Metroid Fusion (USA)",
            "Zelda",
            "apotris",
            "ファイアーエムブレム",
            "포켓몬스터 루비"
        ],
        "code-point order: ASCII upper, ASCII lower, kana, hangul"
    );
    let z = carts.iter().find(|c| c.stem == "Zelda").unwrap();
    assert_eq!(z.platform, Platform::Gba);
    assert_eq!(z.rom, g.join("Zelda.gba"));
    assert!(
        card.scan(Platform::Snes).is_empty(),
        "missing folder is an empty shelf"
    );
    // SNES accepts two extensions; both stems appear.
    let s = card.games_dir(Platform::Snes);
    fs::create_dir_all(&s).unwrap();
    fs::write(s.join("a.sfc"), b"x").unwrap();
    fs::write(s.join("b.smc"), b"x").unwrap();
    fs::write(s.join("c.sfc.bak"), b"x").unwrap();
    assert_eq!(card.scan(Platform::Snes).len(), 2);
}

#[test]
fn label_is_found_by_stem() {
    let d = tempdir("label");
    let card = Card::new(&d);
    let c = cart(&card, Platform::Gb, "Tetris", "gb");
    assert_eq!(card.label(&c), None);
    let l = d.join("Labels/GB/Tetris.png");
    fs::create_dir_all(l.parent().unwrap()).unwrap();
    fs::write(&l, b"png").unwrap();
    assert_eq!(card.label(&c), Some(l));
}

// ---------- saves ----------

#[test]
fn saves_round_trip_per_platform() {
    let d = tempdir("saves");
    let card = Card::new(&d);
    let gba = cart(&card, Platform::Gba, "Same Name", "gba");
    let gb = cart(&card, Platform::Gb, "Same Name", "gb");
    assert_eq!(card.read_save(&gba), None);
    assert!(card.write_save(&gba, &[1, 2, 3]).unwrap());
    assert!(!card.write_save(&gba, &[1, 2, 3]).unwrap());
    assert_eq!(card.read_save(&gba), Some(vec![1, 2, 3]));
    assert_eq!(card.read_save(&gb), None, "platforms do not share saves");
    assert_eq!(card.save_path(&gba), d.join("Saves/GBA/Same Name.sav"));
}

// ---------- states ----------

#[test]
fn states_list_in_order_with_thumbnails() {
    let d = tempdir("states");
    let card = Card::new(&d);
    let c = cart(&card, Platform::Nes, "Mario", "nes");
    assert!(card.list_states(&c).is_empty());
    assert_eq!(card.next_state_number(&c), 1);

    let thumb_rgba = vec![200u8; 4 * 4 * 4];
    let thumb = Thumb {
        width: 4,
        height: 4,
        rgba: &thumb_rgba,
    };
    card.write_state(&c, StateKind::Numbered(2), b"two", Some(thumb.clone()))
        .unwrap();
    card.write_state(&c, StateKind::Resume, b"resume", None)
        .unwrap();
    card.write_state(&c, StateKind::Numbered(1), b"one", Some(thumb))
        .unwrap();
    assert_eq!(card.next_state_number(&c), 3);

    let list = card.list_states(&c);
    let kinds: Vec<StateKind> = list.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        vec![
            StateKind::Resume,
            StateKind::Numbered(1),
            StateKind::Numbered(2)
        ]
    );
    assert!(list[0].thumb.is_none());
    let t1 = list[1].thumb.clone().expect("thumbnail beside state 1");
    assert_eq!(t1, d.join("States/NES/Mario/1.png"));
    // A real PNG, 4x4 RGBA.
    let dec = png::Decoder::new(fs::File::open(&t1).unwrap());
    let mut reader = dec.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!((info.width, info.height), (4, 4));
    assert_eq!(&buf[..4], &[200, 200, 200, 200]);

    assert_eq!(card.read_state(&c, StateKind::Numbered(2)).unwrap(), b"two");
    assert_eq!(card.read_state(&c, StateKind::Numbered(9)), None);
    card.delete_state(&c, StateKind::Numbered(1)).unwrap();
    assert!(!d.join("States/NES/Mario/1.state").exists());
    assert!(!d.join("States/NES/Mario/1.png").exists());
    assert_eq!(card.list_states(&c).len(), 2);
    assert_eq!(card.next_state_number(&c), 3, "numbers are never reused");
    let bad = Thumb {
        width: 4,
        height: 4,
        rgba: &[0; 3],
    };
    assert!(card
        .write_state(&c, StateKind::Numbered(3), b"x", Some(bad))
        .is_err());
}

// ---------- ini ----------

#[test]
fn ini_parses_loosely_and_writes_sorted() {
    let text = "# comment\nlang = ko\n\n  scale=Integer  \nbroken line\nlang = en\ntitle = 포켓몬 = 루비\n";
    let ini = Ini::parse(text);
    assert_eq!(ini.get("lang"), Some("en"), "later wins");
    assert_eq!(ini.get("scale"), Some("Integer"));
    assert_eq!(
        ini.get("title"),
        Some("포켓몬 = 루비"),
        "split at the first ="
    );
    assert_eq!(ini.get("broken line"), None);
    assert_eq!(ini.len(), 3);
    assert_eq!(
        ini.to_string(),
        "lang = en\nscale = Integer\ntitle = 포켓몬 = 루비\n"
    );
}

#[test]
fn ini_load_and_save() {
    let d = tempdir("ini");
    let p = d.join("System/slot2.ini");
    assert!(Ini::load(&p).unwrap().is_empty(), "missing file is empty");
    let mut ini = Ini::default();
    ini.set("lang", "ko");
    ini.set("brightness", "70");
    ini.save(&p).unwrap();
    let back = Ini::load(&p).unwrap();
    assert_eq!(back, ini);
    assert_eq!(
        fs::read_to_string(&p).unwrap(),
        "brightness = 70\nlang = ko\n"
    );
}

// --- per-game settings -------------------------------------------------------------------

fn tmp_card(name: &str) -> (Card, PathBuf) {
    let d = tempdir(name);
    let card = Card::new(&d);
    card.ensure_layout();
    (card, d)
}

fn a_cart(card: &Card) -> Cart {
    cart(card, Platform::Nes, "Mappy", "nes")
}

#[test]
fn a_game_with_no_settings_file_has_no_settings() {
    let (card, _d) = tmp_card("nosettings");
    let cart = a_cart(&card);
    assert_eq!(card.read_settings(&cart), GameSettings::default());
    assert!(!card.game_settings_path(&cart).exists());
}

#[test]
fn settings_round_trip_and_land_where_the_design_says() {
    let (card, d) = tmp_card("settings");
    let cart = a_cart(&card);

    let want = GameSettings {
        core: Some("mgba_libretro".into()),
        scale: Some(ScaleMode::AspectFit),
        overscan: Some(false),
    };
    card.write_settings(&cart, &want).unwrap();

    assert_eq!(
        card.game_settings_path(&cart),
        d.join("System").join("games").join("NES").join("Mappy.ini")
    );
    assert_eq!(card.read_settings(&cart), want);
}

#[test]
fn going_back_to_defaults_leaves_no_file_behind() {
    let (card, _d) = tmp_card("cleared");
    let cart = a_cart(&card);
    card.write_settings(
        &cart,
        &GameSettings {
            scale: Some(ScaleMode::Fill),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(card.game_settings_path(&cart).exists());

    card.write_settings(&cart, &GameSettings::default())
        .unwrap();
    assert!(
        !card.game_settings_path(&cart).exists(),
        "an all-default game should not leave an empty file on the card"
    );
    assert_eq!(card.read_settings(&cart), GameSettings::default());
}

#[test]
fn a_key_this_version_does_not_know_survives_a_save() {
    // A card moved between SLOT2 versions must not lose the newer one's settings the first
    // time the older one writes to it.
    let (card, _d) = tmp_card("forward");
    let cart = a_cart(&card);
    let path = card.game_settings_path(&cart);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "shader=lcd3x\nscale=fill\n").unwrap();

    let mut s = card.read_settings(&cart);
    assert_eq!(s.scale, Some(ScaleMode::Fill));
    s.scale = Some(ScaleMode::Integer);
    card.write_settings(&cart, &s).unwrap();

    // Ini::save writes "key = value", sorted, so compare on the parsed file rather than
    // on its spelling.
    let back = Ini::load(&path).unwrap();
    assert_eq!(
        back.get("shader"),
        Some("lcd3x"),
        "lost an unknown key: {}",
        std::fs::read_to_string(&path).unwrap()
    );
    assert_eq!(back.get("scale"), Some("integer"));
}

#[test]
fn a_typo_falls_back_to_the_default_rather_than_refusing_to_start() {
    let (card, _d) = tmp_card("typo");
    let cart = a_cart(&card);
    let path = card.game_settings_path(&cart);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "scale=integar\noverscan=maybe\ncore=\n").unwrap();

    let s = card.read_settings(&cart);
    assert_eq!(s.scale, None);
    assert_eq!(s.overscan, None);
    assert_eq!(s.core, None);
}

#[test]
fn the_spellings_a_person_would_actually_write_are_accepted() {
    assert_eq!(ScaleMode::parse("Integer"), Some(ScaleMode::Integer));
    assert_eq!(ScaleMode::parse(" INT "), Some(ScaleMode::Integer));
    assert_eq!(ScaleMode::parse("aspect_fit"), Some(ScaleMode::AspectFit));
    assert_eq!(ScaleMode::parse("stretch"), Some(ScaleMode::Fill));
    assert_eq!(ScaleMode::parse("nonsense"), None);
    for m in [ScaleMode::Integer, ScaleMode::AspectFit, ScaleMode::Fill] {
        assert_eq!(
            ScaleMode::parse(m.as_str()),
            Some(m),
            "{m:?} must round-trip"
        );
    }
}
