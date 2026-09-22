//! Platform → core, and the per-platform facts the frontend needs to run one.
//!
//! This is the table D-05 and DESIGN §6 describe, trimmed to what M1 needs: which core file
//! to look for, which options to preset, and how the platform's buttons map. Scaling,
//! shaders, rewind budgets and alternative cores join it in M2.

use crate::JoypadMask;

/// What `slot2_store::Platform` names, repeated here so this crate does not depend on the
/// store. The frontend converts between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Platform {
    Gb,
    Gbc,
    Gba,
    Nes,
    Snes,
    Md,
    Sms,
}

/// A libretro core this frontend ships.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Core {
    Mgba,
    Gpsp,
    Fceumm,
    Snes9x,
    GenesisPlusGx,
}

impl Core {
    /// The library's base name, without extension: `mgba_libretro`.
    pub const fn base_name(self) -> &'static str {
        match self {
            Core::Mgba => "mgba_libretro",
            Core::Gpsp => "gpsp_libretro",
            Core::Fceumm => "fceumm_libretro",
            Core::Snes9x => "snes9x_libretro",
            Core::GenesisPlusGx => "genesis_plus_gx_libretro",
        }
    }

    /// The platform's file name for this host: `.dll`, `.dylib` or `.so`.
    pub fn file_name(self) -> String {
        let ext = if cfg!(windows) {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        format!("{}.{ext}", self.base_name())
    }
}

/// One platform's entry.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformDef {
    pub platform: Platform,
    pub default_core: Core,
    /// Options to hand the core before the game loads.
    pub options: &'static [(&'static str, &'static str)],
    /// Optional BIOS files this platform can use, looked for in `BIOS/`.
    pub bios: &'static [&'static str],
}

/// mGBA plays the three Game Boy platforms; the rest have one core each (M2 adds gpSP as a
/// GBA alternative). Options are the minimum that makes a core behave: skip the BIOS splash
/// when there is no BIOS to show, and let the frontend own frame timing.
pub const PLATFORMS: &[PlatformDef] = &[
    PlatformDef {
        platform: Platform::Gb,
        default_core: Core::Mgba,
        options: &[("mgba_skip_bios", "ON")],
        bios: &["gb_bios.bin"],
    },
    PlatformDef {
        platform: Platform::Gbc,
        default_core: Core::Mgba,
        options: &[("mgba_skip_bios", "ON")],
        bios: &["gbc_bios.bin"],
    },
    PlatformDef {
        platform: Platform::Gba,
        default_core: Core::Mgba,
        options: &[("mgba_skip_bios", "ON")],
        bios: &["gba_bios.bin"],
    },
    PlatformDef {
        platform: Platform::Nes,
        default_core: Core::Fceumm,
        options: &[],
        bios: &[],
    },
    PlatformDef {
        platform: Platform::Snes,
        default_core: Core::Snes9x,
        options: &[],
        bios: &[],
    },
    PlatformDef {
        platform: Platform::Md,
        default_core: Core::GenesisPlusGx,
        options: &[],
        bios: &[],
    },
    PlatformDef {
        platform: Platform::Sms,
        default_core: Core::GenesisPlusGx,
        options: &[],
        bios: &["bios_U.sms", "bios_E.sms", "bios_J.sms"],
    },
];

pub fn def(platform: Platform) -> &'static PlatformDef {
    PLATFORMS
        .iter()
        .find(|d| d.platform == platform)
        .expect("every platform has an entry")
}

/// The libretro joypad bit for one of the frontend's logical buttons on this platform.
/// `None` when the platform has no such button (a Game Boy has no X or Y).
///
/// The frontend's face buttons are laid out like a modern pad (A right, B below, X top,
/// Y left); libretro's ids follow the SNES layout. For the Game Boy family only A and B
/// exist; for the Mega Drive, the frontend's Y/X/A row maps to the core's A/B/C.
pub fn joypad_bit(platform: Platform, button: LogicalButton) -> Option<u16> {
    let _ = (platform, button);
    todo!("task 09")
}

/// The buttons the frontend can send to a core. Mirrors `slot2_input::Button`'s game
/// buttons, without the ones that only mean something to the UI (Menu, Power, volume).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LogicalButton {
    A,
    B,
    X,
    Y,
    L1,
    R1,
    L2,
    R2,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

/// Build a mask from whatever the player is holding.
pub fn mask_for(platform: Platform, held: impl IntoIterator<Item = LogicalButton>) -> JoypadMask {
    let mut m = JoypadMask::default();
    for b in held {
        if let Some(bit) = joypad_bit(platform, b) {
            m = m.with(bit);
        }
    }
    m
}
