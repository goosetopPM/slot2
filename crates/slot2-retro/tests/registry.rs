//! The registry's contract. Task 09 makes these pass without editing this file.

use slot2_retro::{
    def, joypad_bit, mask_for, options_for, JoypadMask, LogicalButton as B, Platform, Tuning,
    PLATFORMS,
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
    assert_eq!(get(Platform::Gba, "mgba_color_correction").as_deref(), Some("GBA"));

    // A Game Boy Color distorted colour in a way games were drawn around.
    assert_eq!(get(Platform::Gbc, "mgba_color_correction").as_deref(), Some("GBC"));

    // A DMG has no colour to correct — it has a palette, and the green one is the point.
    assert_eq!(get(Platform::Gb, "mgba_color_correction").as_deref(), Some("OFF"));
    assert_eq!(get(Platform::Gb, "mgba_gb_colors").as_deref(), Some("DMG Green"));
    assert_eq!(get(Platform::Gbc, "mgba_gb_colors"), None, "a Colour game brings its own");

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
