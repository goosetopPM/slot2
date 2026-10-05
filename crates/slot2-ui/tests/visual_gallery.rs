//! Task 119: a host-rendered visual gallery of the menus and feedback surfaces Task117/118 did
//! not cover — the in-game menu and its children, the state switcher, the shelf settings and
//! its dialogs, disabled rows, and an error toast.
//!
//! Every scenario is drawn once in English and once in Korean through the real components, into
//! `target/ui-gallery-task119/<slug>-<locale>.png`, plus a local `index.html` that puts the two
//! languages side by side. Nothing here is a device acceptance test and nothing here is a
//! substitute for looking at the frames: the assertions only prove that a real GL surface was
//! used, that each frame differs from its background, and that the files are the PNGs they claim
//! to be. Pixel hashes are identity for the report, never a hard-coded correctness oracle.
//!
//! Needs a display and a GL driver, so it only runs with `SLOT2_GFX_TEST=1`; without it the test
//! prints a skip line and passes without creating evidence, which is what keeps CI green on a
//! runner with no GPU.
//!
//! One test, not fifteen: winit allows one event loop per process and a second one fails at run
//! time.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use slot2_gfx::{Canvas, Color, GlCanvas, HostSurface, Image};
use slot2_i18n::Arg;
use slot2_platform::by_target;
use slot2_retro::{CoreId, Platform as RetroPlatform};
use slot2_store::{Cheat, ScaleMode, ShaderPreset, StateKind, StateSlot};
use slot2_ui::about_sticker::AboutInfo;
use slot2_ui::toast::Toast;
use slot2_ui::{
    AboutSticker, CheatMenu, CorePicker, DeviceMenu, DisplayMenu, InGameMenu, LanguageOption,
    LanguagePicker, OverlayMenu, OverscanMenu, PowerMenu, ShaderMenu, ShelfAvailability, ShelfMenu,
    StateSwitcher, TimezoneMenu, UiCtx,
};

/// The deterministic dark start of every frame. The pixel-difference check is against exactly
/// this colour, so nothing here may come from the card, the clock or the environment.
const BACKGROUND: Color = Color::rgba(
    0x12 as f32 / 255.0,
    0x14 as f32 / 255.0,
    0x18 as f32 / 255.0,
    1.0,
);
const BACKGROUND_RGB: [u8; 3] = [0x12, 0x14, 0x18];
/// The panel every scenario is captured at. The `rgsp` profile is this and only this.
const PANEL: (u32, u32) = (720, 480);
const LOCALES: [&str; 2] = ["en", "ko"];
/// A clearly named subdirectory for the one synthetic fixture the state switcher needs. It is
/// the only thing in the output directory that is not a gallery PNG or `index.html`.
const FIXTURE_DIR: &str = "fixtures";
const FIXTURE_PNG: &str = "state-thumbnail.png";

/// One gallery row: the file stem, a human title for `index.html`, and whether a synthetic game
/// frame is drawn underneath it (the overlay screens, whose dim needs something to dim).
struct Scene {
    slug: &'static str,
    title: &'static str,
    frame: bool,
}

const SCENES: [Scene; 15] = [
    Scene {
        slug: "ingame-menu",
        title: "In-game menu",
        frame: true,
    },
    Scene {
        slug: "cheat-menu",
        title: "Cheat menu",
        frame: true,
    },
    Scene {
        slug: "display-menu",
        title: "Display menu",
        frame: true,
    },
    Scene {
        slug: "shader-menu",
        title: "Shader menu",
        frame: true,
    },
    Scene {
        slug: "overscan-menu",
        title: "Overscan menu",
        frame: true,
    },
    Scene {
        slug: "overlay-menu",
        title: "Overlay menu",
        frame: true,
    },
    Scene {
        slug: "core-picker",
        title: "Core picker",
        frame: false,
    },
    Scene {
        slug: "device-menu-disabled",
        title: "Device menu (unavailable rows)",
        frame: false,
    },
    Scene {
        slug: "state-switcher",
        title: "State switcher",
        frame: true,
    },
    Scene {
        slug: "shelf-menu-disabled",
        title: "Shelf settings (unavailable rows)",
        frame: false,
    },
    Scene {
        slug: "language-picker",
        title: "Language picker",
        frame: false,
    },
    Scene {
        slug: "timezone-menu",
        title: "Time zone",
        frame: false,
    },
    Scene {
        slug: "about-sticker",
        title: "About sticker",
        frame: false,
    },
    Scene {
        slug: "power-menu",
        title: "Power menu",
        frame: false,
    },
    Scene {
        slug: "toast-error",
        title: "Error toast",
        frame: true,
    },
];

fn enabled() -> bool {
    std::env::var("SLOT2_GFX_TEST")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn fonts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts")
}

/// The one directory this test may recreate. Derived from the manifest, and refused unless its
/// last two components are the ones the contract names, so a caller cannot point the deletion at
/// anything else.
fn output_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-gallery-task119");
    let tail: Vec<String> = dir
        .components()
        .rev()
        .take(2)
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        tail,
        vec!["ui-gallery-task119".to_string(), "target".to_string()],
        "refusing to recreate {dir:?}"
    );
    dir
}

/// Every gallery PNG's file name, in the order the test writes them.
fn expected_names() -> Vec<String> {
    SCENES
        .iter()
        .flat_map(|s| LOCALES.iter().map(move |l| format!("{}-{l}.png", s.slug)))
        .collect()
}

fn write_rgba_png(path: &Path, w: u32, h: u32, rgba: &[u8]) {
    let f = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(rgba).unwrap();
}

fn write_png(path: &Path, img: &Image) {
    write_rgba_png(path, img.width, img.height, &img.rgba);
}

/// Read a written frame back: PNG signature, decodable, and exactly the panel size. Returns the
/// byte length so the caller can report it. The file is not modified.
fn check_png_file(path: &Path) -> u64 {
    let bytes = std::fs::read(path).unwrap();
    assert!(
        bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
        "{}: not a PNG",
        path.display()
    );
    let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!(
        (info.width, info.height),
        PANEL,
        "{}: wrong size",
        path.display()
    );
    bytes.len() as u64
}

/// How many pixels differ from the deterministic background. A flat clear is zero, which is the
/// only failure this is meant to catch: a file on disk would not say the draw never happened.
fn ink_count(img: &Image) -> usize {
    img.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| [p[0], p[1], p[2]] != BACKGROUND_RGB)
        .count()
}

/// A synthetic game frame: flat coloured bands and blocks, drawn from three fixed colours so the
/// dim layer above them has something to move. No ROM screenshot, no copyrighted art.
fn draw_game_frame(canvas: &mut dyn Canvas) {
    let (w, h) = (PANEL.0 as f32, PANEL.1 as f32);
    canvas.rect(0.0, 0.0, w, h * 0.6, Color::from_rgb8(0x2A, 0x3D, 0x6B));
    canvas.rect(0.0, h * 0.6, w, h * 0.4, Color::from_rgb8(0x3C, 0x6B, 0x3A));
    canvas.rect(
        w * 0.62,
        h * 0.10,
        64.0,
        64.0,
        Color::from_rgb8(0xE8, 0xC8, 0x5A),
    );
    canvas.rect(
        w * 0.16,
        h * 0.50,
        96.0,
        48.0,
        Color::from_rgb8(0x8A, 0x4A, 0x2A),
    );
    canvas.rect(
        w * 0.42,
        h * 0.42,
        48.0,
        72.0,
        Color::from_rgb8(0x5A, 0x2A, 0x6A),
    );
}

/// The synthetic cheats the cheat menu is shown with: mixed enabled and disabled, more than one
/// screenful so the "rows below" bar draws too.
fn synthetic_cheats() -> Vec<Cheat> {
    [
        ("Infinite health", true),
        ("Unlimited coins", true),
        ("Walk through walls", false),
        ("Max experience", true),
        ("No random battles", false),
        ("Always rare drops", true),
        ("Debug map", false),
        ("Infinite items", false),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (description, enabled))| Cheat {
        description: description.to_string(),
        code: format!("{:04X}-{:04X}", 0x0100 + i as u32, 0x9000 + i as u32),
        enabled,
    })
    .collect()
}

fn state_slot(number: u32, thumb: Option<PathBuf>) -> StateSlot {
    StateSlot {
        kind: StateKind::Numbered(number),
        path: PathBuf::from(format!("{number}.state")),
        thumb,
        modified: SystemTime::UNIX_EPOCH + Duration::from_secs(u64::from(number)),
    }
}

/// The one scenario body. Every one of these uses the component's own constructor and draw, and
/// a fresh menu object each call, so no selection state survives from another frame.
fn draw_scene(slug: &str, canvas: &mut dyn Canvas, ctx: &mut UiCtx, out: &Path) {
    match slug {
        "ingame-menu" => {
            // A middle row (row 3 of 7) selected, so the full seven-row overlay is visible with
            // the highlight off the ends.
            let mut menu = InGameMenu::default();
            for _ in 0..3 {
                menu.down();
            }
            menu.draw(canvas, ctx);
        }
        "cheat-menu" => {
            let cheats = synthetic_cheats();
            let mut menu = CheatMenu::new(cheats.len());
            menu.down();
            menu.down();
            menu.draw(canvas, ctx, &cheats);
        }
        "display-menu" => {
            // Overscan available, so the seven-row cropping set draws, on a non-default scale.
            DisplayMenu::new(Some(ScaleMode::AspectFit), true).draw(canvas, ctx);
        }
        "shader-menu" => {
            ShaderMenu::new(Some(ShaderPreset::ZfastCrt)).draw(canvas, ctx);
        }
        "overscan-menu" => {
            OverscanMenu::new(Some(false)).draw(canvas, ctx);
        }
        "overlay-menu" => {
            OverlayMenu::new(Some(false)).draw(canvas, ctx);
        }
        "core-picker" => {
            // GBA with both shipped cores installed and mGBA running: the highlight walks to the
            // other row so the `Current` badge and the highlight are visibly two different rows.
            let mut picker = CorePicker::new(
                RetroPlatform::Gba,
                &[CoreId::Mgba, CoreId::Gpsp],
                Some(CoreId::Mgba),
            );
            picker.down();
            picker.draw(canvas, ctx);
        }
        "device-menu-disabled" => {
            // Volume is ours and always available; brightness and blue light are not on this
            // machine, so both rows draw dimmed and unreachable.
            DeviceMenu::new(60, false, None, None).draw(canvas, ctx);
        }
        "state-switcher" => {
            let thumb = out.join(FIXTURE_DIR).join(FIXTURE_PNG);
            let slots = vec![
                state_slot(1, None),
                state_slot(2, Some(thumb.clone())),
                state_slot(3, Some(thumb)),
            ];
            let mut switcher = StateSwitcher::new(slots);
            // Undo is live: the hint line then carries the undo word as well.
            switcher.draw(canvas, ctx, true);
        }
        "shelf-menu-disabled" => {
            // Production-like availability: language, time zone and About open; the rest keep
            // their rows and draw as unavailable.
            ShelfMenu::new(ShelfAvailability {
                language: true,
                display_defaults: false,
                boot_logo: false,
                sync: false,
                time_zone: true,
                about: true,
            })
            .draw(canvas, ctx);
        }
        "language-picker" => {
            let mut picker = LanguagePicker::new(
                vec![
                    LanguageOption::new("en", "English"),
                    LanguageOption::new("ko", "한국어"),
                ],
                "en",
            );
            // The non-first row is highlighted while the `Current` badge stays on English.
            picker.down();
            picker.draw(canvas, ctx);
        }
        "timezone-menu" => {
            TimezoneMenu::new(540).draw(canvas, ctx);
        }
        "about-sticker" => {
            // Synthetic build information: not read from this repository or this machine, and no
            // claim that either string describes the release artifact.
            AboutSticker::new().draw(
                canvas,
                ctx,
                AboutInfo {
                    version: "0.0.0-host-evidence",
                    target: "synthetic-target",
                },
            );
        }
        "power-menu" => {
            let mut menu = PowerMenu::default();
            menu.down();
            menu.down();
            menu.draw(canvas, ctx);
        }
        "toast-error" => {
            // A built-in error message key with a synthetic title argument, ticked past the fade
            // in so it is at full alpha.
            let safe = ctx.safe;
            let mut toast = Toast::new(
                "cheat-toggle-failed",
                vec![("title".to_string(), Arg::from("Sample Game"))],
            );
            toast.tick(1.0);
            toast.draw(canvas, ctx, &safe);
        }
        other => panic!("unknown scene {other}"),
    }
}

/// The local index: two 720-pixel frames side by side per scenario, inline CSS only, no external
/// resource of any kind.
fn build_index() -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<title>SLOT2 host UI visual gallery (Task 119)</title>\n");
    html.push_str(
        "<style>\n\
         body{background:#101216;color:#e6e6e6;font-family:sans-serif;margin:24px}\n\
         h1{font-size:20px}\n\
         .warning{color:#ffcc66;max-width:1500px}\n\
         section{margin-bottom:32px}\n\
         .row{display:flex;gap:16px}\n\
         figure{margin:0}\n\
         img{width:720px;image-rendering:pixelated;border:1px solid #333333}\n\
         figcaption{font-size:13px;color:#aaaaaa}\n\
         </style>\n</head>\n<body>\n",
    );
    html.push_str("<h1>SLOT2 host UI visual gallery</h1>\n");
    html.push_str(
        "<p class=\"warning\">Host evidence only. These frames were rendered from the real UI \
         components on a host GL surface. They are not physical-device acceptance, and Codex and \
         user review of the images is still required.</p>\n",
    );
    for scene in &SCENES {
        html.push_str(&format!(
            "<section><h2>{}</h2><div class=\"row\">\n",
            scene.title
        ));
        for locale in LOCALES {
            let caption = if locale == "en" {
                "English"
            } else {
                "한국어"
            };
            html.push_str(&format!(
                "<figure><img src=\"{slug}-{locale}.png\" alt=\"{locale}\">\
                 <figcaption>{caption}</figcaption></figure>\n",
                slug = scene.slug,
            ));
        }
        html.push_str("</div></section>\n");
    }
    html.push_str("</body>\n</html>\n");
    html
}

/// Assert the written index is exactly what the contract allows: UTF-8, one relative reference
/// per PNG and nothing else to fetch.
fn validate_index(html: &str, out: &Path, names: &[String]) {
    assert!(
        html.contains("<meta charset=\"utf-8\">"),
        "no charset declared"
    );
    assert!(html.contains("<title>"), "no title");
    assert!(!html.contains("<script"), "the index carries script");
    assert!(!html.contains("data:"), "the index embeds image data");
    assert!(
        !html.contains("http://") && !html.contains("https://"),
        "remote URL"
    );
    assert!(!html.contains("file://"), "file URL");
    assert!(!html.contains(":\\"), "a Windows absolute path");
    assert!(
        !html.contains("/home/") && !html.contains("/Users/"),
        "an absolute path"
    );

    let mut refs = 0;
    for chunk in html.split("src=\"").skip(1) {
        let name = chunk.split('"').next().unwrap();
        assert!(
            !name.contains('/') && !name.contains('\\') && !name.contains(':'),
            "src {name} is not a bare relative file name"
        );
        assert!(names.iter().any(|n| n == name), "unexpected src {name}");
        refs += 1;
    }
    assert_eq!(refs, names.len(), "one reference per PNG");
    for name in names {
        assert_eq!(html.matches(name).count(), 1, "{name} referenced twice");
    }

    let bytes = std::fs::read(out.join("index.html")).unwrap();
    assert!(
        std::str::from_utf8(&bytes).is_ok(),
        "index.html is not UTF-8"
    );
}

/// The output directory holds the gallery and nothing else: the 30 PNGs, `index.html`, and the
/// one documented fixture subdirectory.
fn check_output_tree(out: &Path, names: &[String]) {
    let mut top: Vec<String> = std::fs::read_dir(out)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    top.sort();

    let mut want: Vec<String> = names.to_vec();
    want.push("index.html".to_string());
    want.push(FIXTURE_DIR.to_string());
    want.sort();
    assert_eq!(top, want, "unexpected entries in {}", out.display());

    let mut fixture: Vec<String> = std::fs::read_dir(out.join(FIXTURE_DIR))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    fixture.sort();
    assert_eq!(
        fixture,
        vec![FIXTURE_PNG.to_string()],
        "fixture dir contents"
    );
}

#[test]
fn the_visual_gallery_renders_for_real() {
    if !enabled() {
        eprintln!("SLOT2_GFX_TEST not set; skipping GL visual gallery");
        return;
    }

    let profile = by_target("rgsp").unwrap();
    assert_eq!(profile.geometry.size(), PANEL, "the rgsp panel is 720x480");

    let out = output_dir();
    if out.exists() {
        assert!(out.is_dir(), "{} is not a directory", out.display());
        std::fs::remove_dir_all(&out).unwrap();
    }
    std::fs::create_dir_all(out.join(FIXTURE_DIR)).unwrap();

    // The state switcher's only picture. Synthetic, written under the ignored output directory,
    // and the one non-gallery file the tree check allows.
    {
        let (w, h) = (96u32, 72u32);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let checker = ((x / 12) + (y / 12)) % 2 == 0;
                rgba.extend_from_slice(&[
                    (x * 255 / (w - 1)) as u8,
                    (y * 255 / (h - 1)) as u8,
                    if checker { 0xC0 } else { 0x40 },
                    0xFF,
                ]);
            }
        }
        write_rgba_png(&out.join(FIXTURE_DIR).join(FIXTURE_PNG), w, h, &rgba);
    }

    // One window for the whole gallery; each frame is drawn into the canvas's own offscreen
    // buffer and read back from there.
    let mut surface = HostSurface::open("slot2 ui gallery", PANEL, 1).unwrap();
    let mut canvas = GlCanvas::new(&mut surface, PANEL).unwrap();

    let names = expected_names();
    let min_ink = (PANEL.0 * PANEL.1 / 100) as usize;

    for scene in &SCENES {
        for locale in LOCALES {
            // A fresh context per scenario: no face texture, no font slot and no selection can
            // cross from one language or one screen into the next.
            let mut ctx = UiCtx::new(profile, locale, vec![fonts_dir()], None);
            canvas.clear(BACKGROUND);
            if scene.frame {
                draw_game_frame(&mut canvas);
            }
            draw_scene(scene.slug, &mut canvas, &mut ctx, &out);

            let img = canvas.read_back();
            assert_eq!(
                (img.width, img.height),
                PANEL,
                "{}: wrong panel",
                scene.slug
            );

            let ink = ink_count(&img);
            assert!(
                ink >= min_ink,
                "{} {locale}: only {ink} pixels differ from the background",
                scene.slug
            );

            let path = out.join(format!("{}-{locale}.png", scene.slug));
            write_png(&path, &img);
            let bytes = check_png_file(&path);
            eprintln!(
                "{locale} {}: {bytes} bytes, {ink} pixels of ink",
                path.display()
            );
        }
    }

    let html = build_index();
    std::fs::write(out.join("index.html"), html.as_bytes()).unwrap();
    validate_index(&html, &out, &names);
    check_output_tree(&out, &names);

    eprintln!(
        "wrote {} frames and index.html under {}",
        names.len(),
        out.display()
    );
}
