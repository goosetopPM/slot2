//! DESIGN.md §5's scaling table, as something that fails when it stops being true.
//!
//! The table lists a default multiplier per platform per panel geometry. None of those
//! numbers is stored anywhere: they are what `ScalePolicy::Integer` works out from the
//! platform's native frame size and the panel. Writing the table down here keeps the design
//! and the code from drifting apart without either one noticing — if a native size is ever
//! mistyped, or `place` changes how it rounds, this says so in the language of the design
//! document rather than in pixels.

use slot2_gfx::{place, ScalePolicy};
use slot2_retro::{def, Aspect, Platform};

/// The three panels SLOT2 targets, in the order DESIGN's columns run.
const PANELS: [(u32, u32); 3] = [(640, 480), (720, 480), (720, 720)];

/// Platform, and the multiplier the table claims for each panel.
const TABLE: &[(Platform, [i32; 3])] = &[
    (Platform::Gba, [2, 3, 3]),
    (Platform::Gb, [3, 3, 4]),
    (Platform::Gbc, [3, 3, 4]),
    (Platform::Nes, [2, 2, 2]),
    (Platform::Snes, [2, 2, 2]),
    (Platform::Md, [2, 2, 2]),
    (Platform::Sms, [2, 2, 2]),
];

#[test]
fn the_design_tables_default_multipliers_are_what_integer_scaling_produces() {
    for (platform, wanted) in TABLE {
        let native = def(*platform).native;
        for (panel, want) in PANELS.iter().zip(wanted) {
            let r = place(ScalePolicy::Integer, native, (1, 1), *panel);
            let got_x = r.w / native.0 as i32;
            let got_y = r.h / native.1 as i32;
            assert_eq!(
                (got_x, got_y),
                (*want, *want),
                "{platform:?} {}x{} on {}x{}: table says {want}x, got {}x{} ({}x{} px)",
                native.0,
                native.1,
                panel.0,
                panel.1,
                got_x,
                got_y,
                r.w,
                r.h
            );
            assert!(
                r.w <= panel.0 as i32 && r.h <= panel.1 as i32,
                "{platform:?} overflows {panel:?}: {r:?}"
            );
        }
    }
}

#[test]
fn every_platform_fits_its_panel_and_sits_in_the_middle() {
    for (platform, _) in TABLE {
        let native = def(*platform).native;
        for panel in PANELS {
            for policy in [
                ScalePolicy::Integer,
                ScalePolicy::AspectFit,
                ScalePolicy::Fill,
            ] {
                let aspect = def(*platform).aspect.display(native);
                let r = place(policy, native, aspect, panel);
                assert!(r.w > 0 && r.h > 0, "{platform:?} {policy:?}: {r:?}");
                assert!(
                    r.w <= panel.0 as i32 && r.h <= panel.1 as i32,
                    "{platform:?} {policy:?} overflows {panel:?}: {r:?}"
                );
                // Centred to within the odd pixel that an odd leftover leaves over.
                let slack_x = panel.0 as i32 - r.w - 2 * r.x;
                let slack_y = panel.1 as i32 - r.h - 2 * r.y;
                assert!(
                    (0..=1).contains(&slack_x) && (0..=1).contains(&slack_y),
                    "{platform:?} {policy:?} is off centre on {panel:?}: {r:?}"
                );
            }
        }
    }
}

#[test]
fn a_mega_drive_keeps_its_place_when_the_width_changes() {
    // The acceptance criterion "MD 폭 전환에서 화면 깨짐 없음", at the geometry level: the
    // core switches between 256 and 320 pixels across mid-game, and under aspect correction
    // the picture must stay exactly where it was rather than jumping wider.
    let md = def(Platform::Md);
    assert!(matches!(md.aspect, Aspect::Display { .. }));
    for panel in PANELS {
        let narrow = place(
            ScalePolicy::AspectFit,
            (256, 224),
            md.aspect.display((256, 224)),
            panel,
        );
        let wide = place(
            ScalePolicy::AspectFit,
            (320, 224),
            md.aspect.display((320, 224)),
            panel,
        );
        assert_eq!(narrow, wide, "the picture moved on {panel:?}");
    }
}

#[test]
fn snes_hi_res_still_fits() {
    // 512x448 is twice the SNES's usual frame in both directions, and it has to land on a
    // 720x480 panel without spilling. Integer can only manage 1x there, which is correct:
    // the alternative is cropping half the screen away.
    let r = place(ScalePolicy::Integer, (512, 448), (8, 7), (720, 480));
    assert_eq!((r.w, r.h), (512, 448));
    assert!(r.x >= 0 && r.y >= 0, "{r:?}");

    // Aspect correction gives it the full height, as it does for the low-res frame, so
    // switching between the two does not resize the picture.
    let snes = def(Platform::Snes);
    let lo = place(
        ScalePolicy::AspectFit,
        (256, 224),
        snes.aspect.display((256, 224)),
        (720, 480),
    );
    let hi = place(
        ScalePolicy::AspectFit,
        (512, 448),
        snes.aspect.display((512, 448)),
        (720, 480),
    );
    assert_eq!(lo, hi, "hi-res moved the picture");
}

#[test]
fn only_the_nes_crops_by_default() {
    for (platform, _) in TABLE {
        let o = def(*platform).overscan;
        let cropped = o != slot2_retro::Overscan::NONE;
        assert_eq!(
            cropped,
            *platform == Platform::Nes,
            "{platform:?} overscan {o:?}"
        );
    }
    // 240 lines out, about 224 of them ever seen.
    let nes = def(Platform::Nes);
    assert_eq!(
        slot2_gfx::cropped_size(
            nes.native,
            nes.overscan.left,
            nes.overscan.top,
            nes.overscan.right,
            nes.overscan.bottom
        ),
        (256, 224)
    );
}
