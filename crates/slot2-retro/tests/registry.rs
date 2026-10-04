//! The registry's contract. Task 09 makes these pass without editing this file.

use std::path::Path;

use slot2_retro::{
    def, joypad_bit, mask_for, options_for, options_for_core, supported_cores, CoreId, JoypadMask,
    LogicalButton as B, Platform, PlatformShader, Tuning, PLATFORMS,
};

/// The device SLOT2 is developed against: a 720x480 H700 handheld.
const SP: Tuning = Tuning::handheld((720, 480));

#[test]
fn every_platform_has_an_entry_and_a_core_file_name() {
    assert_eq!(PLATFORMS.len(), 7);
    for p in [
        Platform::Gb,
        Platform::Gbc,
        Platform::Gba,
        Platform::Nes,
        Platform::Snes,
        Platform::Md,
        Platform::Sms,
    ] {
        let d = def(p);
        assert_eq!(d.platform, p);
        let name = d.default_core.file_name();
        assert!(name.starts_with(d.default_core.base_name()), "{name}");
        assert!(name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".dylib"));
    }
    assert_eq!(def(Platform::Gba).default_core.base_name(), "mgba_libretro");
    assert_eq!(
        def(Platform::Md).default_core,
        def(Platform::Sms).default_core,
        "GPGX covers both"
    );
    // The skip lives in `options_for` now, not in the static table: whether the intro is
    // skipped depends on whether a BIOS is on the card, which the table cannot know.
    assert!(options_for(Platform::Gba, false, SP)
        .iter()
        .any(|(k, _)| k == "mgba_skip_bios"));
    assert!(def(Platform::Gba).bios.contains(&"gba_bios.bin"));
    assert!(def(Platform::Nes).bios.is_empty());
}

#[test]
fn a_handheld_console_defaults_to_its_own_lcd_grid() {
    // A Game Boy, a Game Boy Color and a Game Boy Advance were played on a reflective LCD:
    // the grid between the pixels is what the picture looked like, so it is the default. The
    // platform's own screen is the whole of the reason — which core runs it and which panel
    // it lands on are not part of it.
    for p in [Platform::Gb, Platform::Gbc, Platform::Gba] {
        assert_eq!(def(p).shader_default, PlatformShader::Lcd3x, "{p:?}");
    }
}

#[test]
fn a_console_that_drove_a_television_defaults_to_its_scanlines() {
    // These four were played on a CRT in a living room, and what that looked like is what
    // the default puts back.
    for p in [Platform::Nes, Platform::Snes, Platform::Md, Platform::Sms] {
        assert_eq!(def(p).shader_default, PlatformShader::ZfastCrt, "{p:?}");
    }

    // Two families, no gaps and no stragglers: every platform in the table has a default and
    // the same console does not appear on both sides of it.
    let handhelds = [Platform::Gb, Platform::Gbc, Platform::Gba];
    for d in PLATFORMS {
        let expected = if handhelds.contains(&d.platform) {
            PlatformShader::Lcd3x
        } else {
            PlatformShader::ZfastCrt
        };
        assert_eq!(d.shader_default, expected, "{:?}", d.platform);
    }
}

#[test]
fn game_boy_has_only_two_face_buttons() {
    for p in [Platform::Gb, Platform::Gbc] {
        assert_eq!(joypad_bit(p, B::A), Some(JoypadMask::A));
        assert_eq!(joypad_bit(p, B::B), Some(JoypadMask::B));
        assert_eq!(joypad_bit(p, B::X), None);
        assert_eq!(joypad_bit(p, B::Y), None);
        assert_eq!(joypad_bit(p, B::L1), None, "no shoulders on a Game Boy");
        assert_eq!(joypad_bit(p, B::R1), None);
        assert_eq!(joypad_bit(p, B::Start), Some(JoypadMask::START));
        assert_eq!(joypad_bit(p, B::Up), Some(JoypadMask::UP));
    }
}

#[test]
fn gba_adds_shoulders_but_still_has_no_x_or_y() {
    assert_eq!(joypad_bit(Platform::Gba, B::L1), Some(JoypadMask::L));
    assert_eq!(joypad_bit(Platform::Gba, B::R1), Some(JoypadMask::R));
    assert_eq!(joypad_bit(Platform::Gba, B::X), None);
    assert_eq!(joypad_bit(Platform::Gba, B::Y), None);
}

#[test]
fn nes_has_two_buttons_and_snes_has_six() {
    assert_eq!(joypad_bit(Platform::Nes, B::A), Some(JoypadMask::A));
    assert_eq!(joypad_bit(Platform::Nes, B::B), Some(JoypadMask::B));
    assert_eq!(joypad_bit(Platform::Nes, B::X), None);
    assert_eq!(joypad_bit(Platform::Nes, B::L1), None);
    for b in [B::A, B::B, B::X, B::Y, B::L1, B::R1] {
        assert!(joypad_bit(Platform::Snes, b).is_some(), "SNES {b:?}");
    }
    assert_eq!(
        joypad_bit(Platform::Snes, B::L2),
        None,
        "no triggers on a SNES pad"
    );
}

#[test]
fn a_mega_drive_pad_has_all_six_buttons() {
    // Genesis Plus GX reads the bottom row A/B/C off libretro's Y/B/A and the top row
    // X/Y/Z off L/X/R. On this handheld's diamond that is left/bottom/right for A/B/C —
    // the arc a thumb sweeps — and L1/top/R1 for X/Y/Z.
    assert_eq!(joypad_bit(Platform::Md, B::Y), Some(JoypadMask::Y));
    assert_eq!(joypad_bit(Platform::Md, B::B), Some(JoypadMask::B));
    assert_eq!(joypad_bit(Platform::Md, B::A), Some(JoypadMask::A));
    assert_eq!(joypad_bit(Platform::Md, B::L1), Some(JoypadMask::L));
    assert_eq!(joypad_bit(Platform::Md, B::X), Some(JoypadMask::X));
    assert_eq!(joypad_bit(Platform::Md, B::R1), Some(JoypadMask::R));
    assert_eq!(
        joypad_bit(Platform::Md, B::Select),
        Some(JoypadMask::SELECT)
    );
    assert_eq!(joypad_bit(Platform::Md, B::Start), Some(JoypadMask::START));

    // Every one of the eight is a different bit: a mapping that doubles up silently makes
    // two buttons one.
    let mut seen = std::collections::HashSet::new();
    for b in [B::Y, B::B, B::A, B::L1, B::X, B::R1, B::Select, B::Start] {
        let bit = joypad_bit(Platform::Md, b).expect("mapped");
        assert!(seen.insert(bit), "{b:?} shares a bit with something else");
    }

    // The Master System had two, and its Pause is on the console, which the core reads as
    // Start.
    assert_eq!(joypad_bit(Platform::Sms, B::A), Some(JoypadMask::A));
    assert_eq!(joypad_bit(Platform::Sms, B::B), Some(JoypadMask::B));
    assert_eq!(joypad_bit(Platform::Sms, B::Start), Some(JoypadMask::START));
    assert_eq!(joypad_bit(Platform::Sms, B::X), None);
}

#[test]
fn nothing_on_the_face_of_the_handheld_goes_nowhere_on_a_mega_drive() {
    // The bug this replaces: the bottom button, the one a thumb rests on, was mapped to
    // nothing at all while the top button stood in for it.
    for b in [B::A, B::B, B::X, B::Y] {
        assert!(
            joypad_bit(Platform::Md, b).is_some(),
            "{b:?} does nothing on a Mega Drive"
        );
    }
}

#[test]
fn masks_combine_and_ignore_unmapped_buttons() {
    let m = mask_for(Platform::Gba, [B::A, B::Right, B::L1]);
    assert_eq!(m.0, JoypadMask::A | JoypadMask::RIGHT | JoypadMask::L);
    let m = mask_for(Platform::Gb, [B::A, B::X, B::L1]);
    assert_eq!(m.0, JoypadMask::A, "X and L1 do not exist on a Game Boy");
    assert_eq!(mask_for(Platform::Nes, []).0, 0);
}

#[test]
fn a_bios_on_the_card_turns_the_real_boot_sequence_on() {
    let find = |opts: &[(String, String)], key: &str| {
        opts.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("{key} not set: {opts:?}"))
    };

    // The Game Boys and the GBA run on mGBA, which boots high-level without a BIOS. What
    // the file changes is whether the player watches the logo scroll.
    for p in [Platform::Gb, Platform::Gbc, Platform::Gba] {
        assert_eq!(find(&options_for(p, true, SP), "mgba_skip_bios"), "OFF");
        assert_eq!(find(&options_for(p, false, SP), "mgba_skip_bios"), "ON");
    }

    // Genesis Plus GX will not touch a boot ROM unless it is told to.
    for p in [Platform::Md, Platform::Sms] {
        assert_eq!(
            find(&options_for(p, true, SP), "genesis_plus_gx_bios"),
            "enabled"
        );
        assert_eq!(
            find(&options_for(p, false, SP), "genesis_plus_gx_bios"),
            "disabled"
        );
    }

    // The NES and SNES cores have no boot ROM to offer, so nothing changes either way.
    for p in [Platform::Nes, Platform::Snes] {
        assert_eq!(options_for(p, true, SP), options_for(p, false, SP));
    }

    // Every platform that names a BIOS file gets asked about it, and every platform that
    // does not is left alone.
    for d in PLATFORMS {
        let differs = options_for(d.platform, true, SP) != options_for(d.platform, false, SP);
        assert_eq!(
            differs,
            !d.bios.is_empty(),
            "{:?} names {:?} but its options do not depend on it",
            d.platform,
            d.bios
        );
    }
}

#[test]
fn the_nes_hands_over_all_240_lines_so_the_display_can_crop_them() {
    // FCEUmm crops eight rows top and bottom by default. Cropping is a display decision
    // here, in one place for every core, so the core is told not to — and the platform's
    // native size is the uncropped one, with the eight rows recorded as overscan.
    let nes = def(Platform::Nes);
    assert_eq!(nes.native, (256, 240));
    assert_eq!(nes.overscan.top, 8);
    assert_eq!(nes.overscan.bottom, 8);
    for key in ["fceumm_overscan_v_top", "fceumm_overscan_v_bottom"] {
        assert_eq!(
            nes.options.iter().find(|(k, _)| *k == key).map(|(_, v)| *v),
            Some("0"),
            "{key} must be off so the crop is not applied twice"
        );
    }
}

#[test]
fn the_same_console_is_configured_differently_on_different_devices() {
    // SLOT2 runs on 640x480, 720x480 and 720x720 handhelds. Options that are really about
    // the screen have to follow the screen, or a 720x720 machine ends up configured like a
    // 640x480 one.
    let small = Tuning::handheld((640, 480));
    let tall = Tuning::handheld((720, 720));
    let find = |t: Tuning, p: Platform, key: &str| {
        options_for(p, false, t)
            .into_iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("{key} not set for {p:?}"))
    };

    // A Super Game Boy border replaces a 160x144 picture with a 256x224 one, so the game
    // loses an integer step of scale. Every panel SLOT2 ships for is too small for that to
    // be worth it; a television is not.
    for t in [small, tall, Tuning::handheld((720, 480))] {
        assert_eq!(
            find(t, Platform::Gb, "mgba_sgb_borders"),
            "OFF",
            "{:?} is not big enough for a border",
            t.geometry
        );
    }
    assert!(Tuning::handheld((1280, 720)).room_for_a_border());

    // Hi-res SNES fits on all three panels at 1x, so it is a question about cycles.
    assert_eq!(find(small, Platform::Snes, "snes9x_gfx_hires"), "disabled");
    assert_eq!(find(tall, Platform::Snes, "snes9x_gfx_hires"), "disabled");
    assert_eq!(
        find(Tuning::DESKTOP, Platform::Snes, "snes9x_gfx_hires"),
        "enabled"
    );

    // And a machine with cycles to spare gets what it can afford whatever its panel.
    assert_eq!(
        find(Tuning::DESKTOP, Platform::Nes, "fceumm_sndquality"),
        "High"
    );
    assert_eq!(find(small, Platform::Nes, "fceumm_sndquality"), "Low");
}

#[test]
fn no_core_is_left_to_correct_the_aspect_or_the_overscan_itself() {
    // This frontend decides both, from PlatformDef. A core told to do it as well would
    // apply the correction twice — which is exactly what happened with FCEUmm's default
    // eight-row crop before it was turned off.
    for d in PLATFORMS {
        for (k, _) in options_for(d.platform, false, SP) {
            let k = k.to_lowercase();
            let sets_aspect = k.ends_with("_aspect") || k.contains("aspect_ratio");
            assert!(
                !sets_aspect,
                "{:?} sets {k}, but the frontend owns the aspect",
                d.platform
            );
            // The one overscan option that is allowed is the one turning a core's own crop
            // off, so the display layer can do it instead.
            if k.contains("overscan") {
                let v = options_for(d.platform, false, SP)
                    .into_iter()
                    .find(|(kk, _)| kk.to_lowercase() == k)
                    .map(|(_, v)| v)
                    .unwrap_or_default();
                assert_eq!(
                    v, "0",
                    "{:?} sets {k}={v}; the frontend crops, so a core may only be told not to",
                    d.platform
                );
            }
        }
    }
}

#[test]
fn colour_correction_follows_the_console_not_the_core() {
    // One core runs all three Game Boy platforms, and the right answer differs for each,
    // so mGBA's own "Auto" is not it.
    let get = |p: Platform, key: &str| {
        options_for(p, false, SP)
            .into_iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    };

    // A GBA's panel over-saturated what it was given, and artists drew around that.
    // mGBA's correction is a mild panel model, not a heavy filter, and has no strength
    // dial — a lighter touch than this would have to be a grading pass of our own.
    assert_eq!(
        get(Platform::Gba, "mgba_color_correction").as_deref(),
        Some("GBA")
    );

    // A Game Boy Color distorted colour in a way games were drawn around.
    assert_eq!(
        get(Platform::Gbc, "mgba_color_correction").as_deref(),
        Some("GBC")
    );

    // A DMG has no colour to correct — it has a palette, and the green one is the point.
    assert_eq!(
        get(Platform::Gb, "mgba_color_correction").as_deref(),
        Some("OFF")
    );
    assert_eq!(
        get(Platform::Gb, "mgba_gb_colors").as_deref(),
        Some("DMG Green")
    );
    assert_eq!(
        get(Platform::Gbc, "mgba_gb_colors"),
        None,
        "a Colour game brings its own"
    );

    // Whatever is set, it is set once: a duplicate key means the last write silently wins.
    for p in [Platform::Gb, Platform::Gbc, Platform::Gba] {
        let opts = options_for(p, false, SP);
        let mut keys: Vec<&str> = opts.iter().map(|(k, _)| k.as_str()).collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(before, keys.len(), "{p:?} sets an option twice");
    }
}

// -------------------------------------------------------------- the library a card handed us

#[test]
fn a_library_path_names_its_core() {
    for core in CoreId::ALL {
        for ext in ["dll", "so", "dylib"] {
            let p = format!("System/cores/{}.{ext}", core.base_name());
            assert_eq!(
                CoreId::from_library_path(Path::new(&p)),
                Some(core),
                "{p} is not recognised"
            );
        }
        // The host's own file name comes back to the same core.
        assert_eq!(
            CoreId::from_library_path(Path::new(&core.file_name())),
            Some(core),
            "{}",
            core.file_name()
        );
    }

    // A Windows filesystem hands back whatever case the file was written with, so the name
    // is compared without case at all.
    assert_eq!(
        CoreId::from_library_path(Path::new("System/cores/MGBA_LIBRETRO.DLL")),
        Some(CoreId::Mgba)
    );
    assert_eq!(
        CoreId::from_library_path(Path::new("System/cores/sNeS9x_LIBretro.So")),
        Some(CoreId::Snes9x)
    );

    // The folder says nothing about which core it is, and a card written on one machine has
    // to be read on another, so all three extensions are recognised everywhere.
    assert_eq!(
        CoreId::from_library_path(Path::new("gpsp_libretro/snes9x_libretro.so")),
        Some(CoreId::Snes9x)
    );
    assert_eq!(
        CoreId::from_library_path(Path::new("C:/cores/genesis_plus_gx_libretro.dylib")),
        Some(CoreId::GenesisPlusGx)
    );
    // No extension at all is still the core's own name.
    assert_eq!(
        CoreId::from_library_path(Path::new("System/cores/fceumm_libretro")),
        Some(CoreId::Fceumm)
    );
}

#[test]
fn a_name_that_only_looks_like_a_core_is_not_one() {
    for p in [
        // A prefix the frontend never ships, and a versioned or backed-up name.
        "System/cores/libmgba_libretro.so",
        "System/cores/mgba_libretro.so.1",
        "System/cores/mgba_libretro.dll.bak",
        // A library, but not a core's: the name is only part of one.
        "System/cores/mgba.so",
        "System/cores/mgba_libretro.txt",
        // Not a core at all.
        "System/cores/nosuch_libretro.so",
        "System/cores/",
        "System/cores/.so",
        "snes9x_libretro_patched.so.dist",
    ] {
        assert_eq!(CoreId::from_library_path(Path::new(p)), None, "{p}");
    }
}

#[test]
fn every_core_names_the_consoles_it_runs() {
    // The same table again, written out here on purpose: a support list that agrees with
    // itself because it is the same list proves nothing.
    let supported: [(CoreId, &[Platform]); 6] = [
        (CoreId::Mgba, &[Platform::Gb, Platform::Gbc, Platform::Gba]),
        (CoreId::Gambatte, &[Platform::Gb, Platform::Gbc]),
        (CoreId::Gpsp, &[Platform::Gba]),
        (CoreId::Fceumm, &[Platform::Nes]),
        (CoreId::Snes9x, &[Platform::Snes]),
        (CoreId::GenesisPlusGx, &[Platform::Md, Platform::Sms]),
    ];
    for (core, platforms) in supported {
        for p in [
            Platform::Gb,
            Platform::Gbc,
            Platform::Gba,
            Platform::Nes,
            Platform::Snes,
            Platform::Md,
            Platform::Sms,
        ] {
            assert_eq!(
                core.supports_platform(p),
                platforms.contains(&p),
                "{core:?} on {p:?}"
            );
        }
    }

    // The trap this list exists to avoid: gpSP is a GBA alternative and never a default, so
    // answering "which core covers this platform" from `PLATFORMS` would miss it entirely.
    assert!(CoreId::Gpsp.supports_platform(Platform::Gba));
    assert!(
        !PLATFORMS.iter().any(|d| d.default_core == CoreId::Gpsp),
        "gpSP is a default now, so this test no longer proves the point"
    );

    // And the other direction: every platform's default core runs that platform.
    for d in PLATFORMS {
        assert!(
            d.default_core.supports_platform(d.platform),
            "{d:?} is not run by its own default core"
        );
    }
}

#[test]
fn each_platform_lists_its_cores_best_first() {
    let expected: [(Platform, &[CoreId]); 7] = [
        (Platform::Gb, &[CoreId::Mgba, CoreId::Gambatte]),
        (Platform::Gbc, &[CoreId::Mgba, CoreId::Gambatte]),
        (Platform::Gba, &[CoreId::Mgba, CoreId::Gpsp]),
        (Platform::Nes, &[CoreId::Fceumm]),
        (Platform::Snes, &[CoreId::Snes9x]),
        (Platform::Md, &[CoreId::GenesisPlusGx]),
        (Platform::Sms, &[CoreId::GenesisPlusGx]),
    ];
    for (platform, cores) in expected {
        let case = format!("{platform:?}");
        assert_eq!(supported_cores(platform), cores, "{case}");
        // The list and the default agree: what a launch picks is the first candidate.
        assert_eq!(cores[0], def(platform).default_core, "{case}");

        // Both ways against the predicate, so the two cannot drift apart: nothing is listed
        // that the core cannot run, and nothing a core can run is missing from its list.
        for core in CoreId::ALL {
            assert_eq!(
                cores.contains(&core),
                core.supports_platform(platform),
                "{core:?} on {platform:?}"
            );
        }
    }

    // The alternatives are alternatives: each covers the console it is for and nothing else.
    assert!(CoreId::Gambatte.supports_platform(Platform::Gb));
    assert!(CoreId::Gambatte.supports_platform(Platform::Gbc));
    assert!(!CoreId::Gambatte.supports_platform(Platform::Gba));
    assert!(!CoreId::Gambatte.supports_platform(Platform::Nes));
    assert!(CoreId::Gpsp.supports_platform(Platform::Gba));
    assert!(!CoreId::Gpsp.supports_platform(Platform::Gb));
    assert!(!CoreId::Gpsp.supports_platform(Platform::Gbc));
}

#[test]
fn an_alternative_core_is_not_handed_the_defaults_options() {
    // The wrapper still answers for the platform's own core, unchanged.
    assert_eq!(
        options_for(Platform::Gba, false, SP),
        options_for_core(CoreId::Mgba, Platform::Gba, false, SP).unwrap()
    );
    assert_eq!(
        options_for(Platform::Nes, true, SP),
        options_for_core(CoreId::Fceumm, Platform::Nes, true, SP).unwrap()
    );

    // The two alternatives have nothing of their own here yet, and above all they get none of
    // mGBA's keys: a guessed option is a setting that does nothing while looking like one.
    for (core, platform) in [
        (CoreId::Gpsp, Platform::Gba),
        (CoreId::Gambatte, Platform::Gb),
        (CoreId::Gambatte, Platform::Gbc),
    ] {
        let opts = options_for_core(core, platform, false, SP).expect("supported pair");
        assert!(opts.is_empty(), "{core:?} was handed {opts:?}");
    }

    // The default core on the same console keeps every one of them.
    let mgba = options_for_core(CoreId::Mgba, Platform::Gba, false, SP).unwrap();
    assert!(mgba.iter().any(|(k, _)| k == "mgba_skip_bios"), "{mgba:?}");
    assert_eq!(mgba, options_for(Platform::Gba, false, SP));

    // And a pair that cannot happen has no options at all rather than the wrong ones.
    for (core, platform) in [
        (CoreId::Gambatte, Platform::Gba),
        (CoreId::Gpsp, Platform::Gb),
        (CoreId::Gpsp, Platform::Gbc),
        (CoreId::Snes9x, Platform::Nes),
        (CoreId::Mgba, Platform::Md),
    ] {
        assert_eq!(options_for_core(core, platform, false, SP), None);
    }
}

// -------------------------------------------------------------- the card's core manifest

/// The repository root, from this crate's own directory.
fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_cores_a_card_ships_are_one_list() {
    // `cores/required.txt` is what build/cores.ps1 builds, build/dist-device.ps1 assembles and
    // CI checks; `Core::ALL` is what this frontend can open at runtime. Two lists that are
    // meant to be one drift the moment nobody compares them, so this compares them — the
    // manifest as it is in the repository against the runtime table, and neither PowerShell
    // nor the workflow is consulted for it.
    let manifest = std::fs::read_to_string(repo_root().join("cores/required.txt"))
        .expect("cores/required.txt");
    let names: Vec<&str> = manifest.lines().collect();

    // The order matters: it is the order a card is assembled in.
    assert_eq!(
        names,
        [
            "mgba",
            "gambatte",
            "gpsp",
            "fceumm",
            "snes9x",
            "genesis_plus_gx"
        ]
    );

    let runtime: Vec<&str> = CoreId::ALL
        .iter()
        .map(|core| {
            core.base_name()
                .strip_suffix("_libretro")
                .expect("a base name ends in _libretro")
        })
        .collect();
    assert_eq!(
        names, runtime,
        "the distribution manifest and Core::ALL are not the same list"
    );
}

#[test]
fn every_manifest_row_names_a_core_this_repository_can_build() {
    let root = repo_root();
    let manifest =
        std::fs::read_to_string(root.join("cores/required.txt")).expect("cores/required.txt");
    let mut seen = std::collections::HashSet::new();
    for name in manifest.lines() {
        assert!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "{name:?} is not a core name"
        );
        assert!(seen.insert(name), "{name} is in the manifest twice");

        let dir = root.join("cores").join(name);
        assert!(
            dir.join("build.sh").is_file(),
            "{} has no build.sh, so nothing can build it",
            dir.display()
        );
        let pin = std::fs::read_to_string(dir.join("commit")).expect("a commit file");
        let pin = pin.trim();
        assert_eq!(pin.len(), 40, "{name}: {pin:?} is not a full commit hash");
        assert!(
            pin.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "{name}: {pin:?} is not a lowercase hex commit hash"
        );
    }
    assert!(!seen.is_empty(), "the manifest names no cores");
}
