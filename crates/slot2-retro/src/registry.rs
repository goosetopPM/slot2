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

/// The shape a console's picture is meant to have, which is not always the shape of the
/// pixel buffer it hands over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aspect {
    /// Square pixels: the frame's own proportions are already right.
    Square,
    /// Each source pixel is `num:den` wide. A NES pixel is 8:7 — 256×240 was meant to look
    /// slightly wider than it measures.
    Pixel { num: u32, den: u32 },
    /// The console always filled a fixed shape whatever the frame size. A Mega Drive
    /// switches between 256 and 320 pixels across and both filled the same 4:3 screen, so
    /// its pixel ratio is not a constant and a fixed display ratio is the honest model.
    Display { num: u32, den: u32 },
}

impl Aspect {
    /// The ratio this frame should be displayed at. Not reduced — a ratio is all the
    /// caller needs, and reducing it would only lose precision.
    pub fn display(self, frame: (u32, u32)) -> (u32, u32) {
        match self {
            Aspect::Square => frame,
            Aspect::Pixel { num, den } => (frame.0 * num, frame.1 * den),
            Aspect::Display { num, den } => (num, den),
        }
    }
}

/// Source rows and columns a television never showed. Cropping them is optional: some games
/// draw to the edge and some leave garbage there, so this is a default, not a rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Overscan {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Overscan {
    pub const NONE: Overscan = Overscan {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    /// The same amount off the top and the bottom.
    pub const fn rows(n: u32) -> Overscan {
        Overscan {
            left: 0,
            top: n,
            right: 0,
            bottom: n,
        }
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
    /// The frame size this console usually produces. Cores report their own size and may
    /// change it mid-game, so this is for laying out a shelf before a core has run, never
    /// a substitute for `av_info()`.
    pub native: (u32, u32),
    pub aspect: Aspect,
    /// What to crop by default. Only the NES asks for any.
    pub overscan: Overscan,
}

/// mGBA plays the three Game Boy platforms; the rest have one core each (M2 adds gpSP as a
/// GBA alternative). Options are the minimum that makes a core behave: skip the BIOS splash
/// when there is no BIOS to show, and let the frontend own frame timing.
pub const PLATFORMS: &[PlatformDef] = &[
    PlatformDef {
        platform: Platform::Gb,
        default_core: Core::Mgba,
        options: &[],
        bios: &["gb_bios.bin"],
        native: (160, 144),
        aspect: Aspect::Square,
        overscan: Overscan::NONE,
    },
    PlatformDef {
        platform: Platform::Gbc,
        default_core: Core::Mgba,
        options: &[],
        bios: &["gbc_bios.bin"],
        native: (160, 144),
        aspect: Aspect::Square,
        overscan: Overscan::NONE,
    },
    PlatformDef {
        platform: Platform::Gba,
        default_core: Core::Mgba,
        options: &[],
        bios: &["gba_bios.bin"],
        native: (240, 160),
        aspect: Aspect::Square,
        overscan: Overscan::NONE,
    },
    PlatformDef {
        platform: Platform::Nes,
        default_core: Core::Fceumm,
        // FCEUmm crops eight rows top and bottom by default and would hand over 256x224.
        // Turning that off and cropping in the display layer keeps one mechanism instead of
        // two: the same policy for every core, toggled instantly rather than through a core
        // option, and the same answer for cores that offer no such knob. Sixteen rows of
        // texture is not a cost worth splitting the logic over.
        options: &[
            ("fceumm_overscan_v_top", "0"),
            ("fceumm_overscan_v_bottom", "0"),
        ],
        bios: &[],
        native: (256, 240),
        aspect: Aspect::Pixel { num: 8, den: 7 },
        overscan: Overscan::rows(8),
    },
    PlatformDef {
        platform: Platform::Snes,
        default_core: Core::Snes9x,
        options: &[],
        bios: &[],
        native: (256, 224),
        aspect: Aspect::Pixel { num: 8, den: 7 },
        overscan: Overscan::NONE,
    },
    PlatformDef {
        platform: Platform::Md,
        default_core: Core::GenesisPlusGx,
        options: &[],
        bios: &["bios_MD.bin"],
        native: (320, 224),
        aspect: Aspect::Display { num: 4, den: 3 },
        overscan: Overscan::NONE,
    },
    PlatformDef {
        platform: Platform::Sms,
        default_core: Core::GenesisPlusGx,
        options: &[],
        bios: &["bios_U.sms", "bios_E.sms", "bios_J.sms"],
        native: (256, 192),
        aspect: Aspect::Pixel { num: 8, den: 7 },
        overscan: Overscan::NONE,
    },
];

/// The options to launch this platform with, given whether one of its BIOS files was found
/// on the card.
///
/// A BIOS is optional for every platform here — the cores all emulate one well enough to
/// play without — so the question is not whether the game runs but whether the player sees
/// the real boot sequence. Someone who went to the trouble of putting `gb_bios.bin` on the
/// card wants the Game Boy logo to scroll down; someone who did not should not sit through
/// a core's imitation of it. So the presence of the file *is* the setting, and there is
/// nothing to configure until a player wants to disagree with it.
pub fn options_for(platform: Platform, bios_present: bool) -> Vec<(String, String)> {
    let def = def(platform);
    let mut out: Vec<(String, String)> = def
        .options
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let mut set = |k: &str, v: &str| out.push((k.to_string(), v.to_string()));
    match def.default_core {
        // mGBA looks for a BIOS on its own (`mgba_use_bios` is ON by default) and falls
        // back to its own high-level boot when there is none. Skipping the intro is about
        // the intro, not about the BIOS.
        Core::Mgba | Core::Gpsp => {
            set("mgba_skip_bios", if bios_present { "OFF" } else { "ON" });
        }
        // Genesis Plus GX will not touch a boot ROM unless told to, and says so plainly.
        Core::GenesisPlusGx => {
            set(
                "genesis_plus_gx_bios",
                if bios_present { "enabled" } else { "disabled" },
            );
        }
        Core::Fceumm | Core::Snes9x => {}
    }
    out
}

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
/// Y left), which is also the SNES layout libretro's ids are named after, so the SNES maps
/// straight through. For the Game Boy family only A and B exist. The Mega Drive is the one
/// that needs thinking about; see the arm below.
pub fn joypad_bit(platform: Platform, button: LogicalButton) -> Option<u16> {
    use JoypadMask as M;
    use LogicalButton as B;

    match (platform, button) {
        (_, B::Up) => Some(M::UP),
        (_, B::Down) => Some(M::DOWN),
        (_, B::Left) => Some(M::LEFT),
        (_, B::Right) => Some(M::RIGHT),
        (_, B::Start) => Some(M::START),

        (
            Platform::Gb | Platform::Gbc | Platform::Gba | Platform::Nes | Platform::Snes,
            B::Select,
        ) => Some(M::SELECT),

        (
            Platform::Gb
            | Platform::Gbc
            | Platform::Gba
            | Platform::Nes
            | Platform::Snes
            | Platform::Sms,
            B::A,
        ) => Some(M::A),
        (
            Platform::Gb
            | Platform::Gbc
            | Platform::Gba
            | Platform::Nes
            | Platform::Snes
            | Platform::Sms,
            B::B,
        ) => Some(M::B),

        (Platform::Snes, B::X) => Some(M::X),
        (Platform::Snes, B::Y) => Some(M::Y),

        (Platform::Gba | Platform::Snes, B::L1) => Some(M::L),
        (Platform::Gba | Platform::Snes, B::R1) => Some(M::R),

        // A Mega Drive pad, all six buttons of it. Genesis Plus GX reads the bottom row
        // A/B/C off libretro's Y/B/A and the top row X/Y/Z off L/X/R, with Select as Mode
        // (libretro.c, `_polled_input`). Laid over this handheld's diamond that puts MD
        // A/B/C on the left, bottom and right buttons — the arc a thumb sweeps — and MD
        // X/Y/Z on the shoulder, top, shoulder above them.
        //
        // The extra three are always mapped and cost nothing when unused: the core is
        // handed a plain RETRO_DEVICE_JOYPAD, which makes it read the cartridge's own I/O
        // support field and give a 3-button game a 3-button pad. Forcing six on a game
        // that predates them is how a 3-button game ends up not reading its pad at all.
        (Platform::Md, B::Y) => Some(M::Y),           // MD A
        (Platform::Md, B::B) => Some(M::B),           // MD B
        (Platform::Md, B::A) => Some(M::A),           // MD C
        (Platform::Md, B::L1) => Some(M::L),          // MD X
        (Platform::Md, B::X) => Some(M::X),           // MD Y
        (Platform::Md, B::R1) => Some(M::R),          // MD Z
        (Platform::Md, B::Select) => Some(M::SELECT), // Mode

        _ => None,
    }
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
