//! The registry's contract. Task 09 makes these pass without editing this file.

use slot2_retro::{def, joypad_bit, mask_for, JoypadMask, LogicalButton as B, Platform, PLATFORMS};

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
    assert!(def(Platform::Gba)
        .options
        .iter()
        .any(|(k, _)| *k == "mgba_skip_bios"));
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
    assert_eq!(joypad_bit(Platform::Md, B::Select), Some(JoypadMask::SELECT));
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
