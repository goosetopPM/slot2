//! What each shelf's cartridge looks like.
//!
//! One row per platform: which drawing, how big it is in its own units, where a label sits
//! on it, and what colour the shell is when nothing is known about the game. Sizes are the
//! artwork's `viewBox` and the test reads them back out of the files, so a typo here fails
//! the build rather than drawing a shelf of wrong-shaped carts.
//!
//! There is no drawing for the slot a cartridge goes into. DESIGN §7 anticipates one per
//! platform; until somebody draws it the slot is chrome built from rectangles, the way the
//! original built it, and `shelf_view` owns that. (`socket.svg` is not it — that is the IC
//! socket on the core picker's board.)
//!
//! There is a drawing for the front of the machine as well, one per platform (`port`), but it is
//! trim on the plastic rather than the slot itself: `shelf_view` still builds the bay, the
//! opening and the band out of rectangles, because that is what decides which part of a
//! cartridge is hidden as it goes in. A port's own middle is transparent, so it dresses the
//! mouth without covering it. (`socket.svg` is a different thing — the IC socket on the core
//! picker's board, not a cartridge slot.)
//!
//! All seven shelves have drawings of their own: the Game Boy Advance and Game Boy ones ported
//! from the original project (MIT), and the NES, Super Nintendo, Mega Drive and Master System
//! ones drawn for SLOT2. `borrowed` is what says a shelf is wearing another platform's artwork;
//! it is false for every row today, and it is the flag a shelf would raise on the day one ships
//! without art of its own.
//!
//! Each row also carries the two travel curves a cartridge moves on — `insert` and `eject`.
//! They are *normalized profiles*, not timings: the animation is the same length on every
//! platform, and the core load, the seated hold and the clip's own cue stay in the App and
//! `insert` contracts. What a platform owns here is the rhythm — when the foot meets the
//! connector, how long the cartridge is held against it, and how far it creeps.
//!
//! `sfx_in` and `sfx_out` are the same idea for the slot's two noises. There is one recording
//! per direction on disk (see `assets/sfx/PROVENANCE.md`) and every shelf plays it; what a
//! platform owns is the rate it is read at and how loud it comes out, which is what makes a
//! GBA cart click and a NES cart land.

use slot2_store::Platform;

use crate::insert::Curve;
use crate::shelf::{MOUTH_EXTRA, MOUTH_H};

/// The trim around one platform's mouth: the mouth itself — a cart's width plus
/// [`MOUTH_EXTRA`] — with 20px either side of it where the trim meets the band. Every port
/// drawing uses this, so the trim lines up with the slot on all seven shelves.
const fn port_size(cart_w: f32) -> (f32, f32) {
    (cart_w + MOUTH_EXTRA + 40.0, MOUTH_H)
}

/// A rectangle in artwork units, which are fractional — `sticker.svg` is 205.762 wide.
/// `slot2_gfx::Rect` is whole screen pixels and is the wrong tool here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// How a cartridge shell is finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    Solid,
    /// Clear plastic: the colour lightens toward the rim where light catches the edge.
    Translucent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shell {
    pub colour: [u8; 3],
    pub finish: Finish,
}

/// How one shelf plays one of the slot's own recordings.
///
/// Two files, seven machines, and the noises are not the same: a pak drops down a long rail
/// where a GBA cart clicks into a connector. The recording is what every shelf has to work
/// with, so what a platform owns is the rate it is read at — `speed` above 1 is the clip read
/// faster, which is shorter and higher, a lighter and quicker cartridge — and how loud it comes
/// out.
///
/// Small and copyable on purpose: `slot2-ui` does not depend on `slot2-audio`, so these two
/// numbers are the whole of what the drawing side hands the audio crate, which decides what the
/// sound actually is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundProfile {
    speed: f32,
    gain: f32,
}

impl SoundProfile {
    /// The profile with this playback rate and gain.
    ///
    /// The bounds are checked where the table is built, so a mistyped platform fails the build
    /// rather than playing a cartridge at a speed or a level nobody asked for. A NaN fails every
    /// comparison, so it is refused here with everything else out of range. The two halves of a
    /// range are written as two tests because this has to be callable in a `const` table, and
    /// `RangeInclusive::contains` is not const yet.
    pub const fn new(speed: f32, gain: f32) -> SoundProfile {
        assert!(0.80 <= speed, "a clip is read at 0.80x or more");
        assert!(speed <= 1.20, "a clip is read at 1.20x at most");
        assert!(
            0.70 <= gain,
            "a styled clip may only be quieter than the recording"
        );
        assert!(
            gain <= 1.00,
            "a styled clip cannot be louder than the recording"
        );
        SoundProfile { speed, gain }
    }

    /// The rate the recording is read at. 1 is the recording itself.
    pub const fn speed(self) -> f32 {
        self.speed
    }

    /// The amplitude multiplier, at most 1.
    pub const fn gain(self) -> f32 {
        self.gain
    }
}

/// Everything the shelf needs to draw one platform's cartridge and the slot it goes into.
pub struct PlatformSkin {
    pub platform: Platform,
    /// The shell outline. SVG source, white on transparency.
    pub cart: &'static str,
    /// Moulded ribs and recesses, drawn over the shell on the same grid. May be empty.
    pub cart_detail: &'static str,
    /// The artwork's own size, from its viewBox.
    pub cart_size: (f32, f32),
    /// Where a label sits on the shell, in artwork units.
    pub label: Rect,
    /// What colour to draw the shell when nothing is known about the game.
    pub shell: Shell,
    /// The front trim around the mouth, drawn over the machine's face. SVG source, white on
    /// transparency, with its middle left clear so the cartridge in the slot stays visible.
    pub port: &'static str,
    /// The trim's own size, from its viewBox. Its height is always the mouth's.
    pub port_size: (f32, f32),
    /// How this shelf's cartridges go in: a travel profile in fractions of the animation, so
    /// the shelves differ in rhythm and not in duration. See [`Curve`].
    pub insert: Curve,
    /// The way back out. Its own profile rather than the insert reversed: a cartridge is
    /// pushed against the connector on the way in and falls out of it on the way out.
    pub eject: Curve,
    /// How this shelf plays the insert recording: the cart's weight down its own rails.
    pub sfx_in: SoundProfile,
    /// And the eject recording. Its own profile rather than the insert reversed, because a
    /// cartridge is pushed into the connector and falls back out of it.
    pub sfx_out: SoundProfile,
    /// True when this platform has no drawing of its own and is using another's.
    pub borrowed: bool,
}

const CART_GBA: &str = include_str!("../../../assets/skins/cart.svg");
const DETAIL_GBA: &str = include_str!("../../../assets/skins/cart_detail.svg");
const SIZE_GBA: (f32, f32) = (240.0, 135.0);
const LABEL_GBA: Rect = Rect {
    x: 17.119,
    y: 28.89,
    w: 205.762,
    h: 71.116,
};
const SHELL_GBA: Shell = Shell {
    colour: [0x35, 0x35, 0x3a],
    finish: Finish::Solid,
};
const PORT_GBA: &str = include_str!("../../../assets/skins/gba_port.svg");

const CART_GB: &str = include_str!("../../../assets/skins/gb_cart.svg");
const DETAIL_GB: &str = include_str!("../../../assets/skins/gb_cart_detail.svg");
/// Both Game Boy paks were moulded in the same shell, so one size and one label serve the
/// grey pak and the Colour one alike; only the plastic differs.
const SIZE_GB: (f32, f32) = (240.0, 253.0);
const LABEL_GB: Rect = Rect {
    x: 17.119,
    y: 80.0,
    w: 205.762,
    h: 71.116,
};
const SHELL_GB: Shell = Shell {
    colour: [0x9a, 0x97, 0x8f],
    finish: Finish::Solid,
};
const PORT_GB: &str = include_str!("../../../assets/skins/gb_port.svg");

const CART_GBC: &str = include_str!("../../../assets/skins/gbc_cart.svg");
const DETAIL_GBC: &str = include_str!("../../../assets/skins/gbc_cart_detail.svg");
const SHELL_GBC: Shell = Shell {
    colour: [0x7c, 0x7a, 0x8a],
    finish: Finish::Translucent,
};
const PORT_GBC: &str = include_str!("../../../assets/skins/gbc_port.svg");

// The NES and Super Nintendo carts are SLOT2's own redraws (see `assets/skins/PROVENANCE.md`):
// tall and narrow for the NES, wide and low for the Super Nintendo, which is what tells the two
// shelves apart before the label is even read. Their moulding is a single white shape per file,
// not the light/shadow pair the ported Game Boy details use.
const CART_NES: &str = include_str!("../../../assets/skins/nes_cart.svg");
const DETAIL_NES: &str = include_str!("../../../assets/skins/nes_cart_detail.svg");
const SIZE_NES: (f32, f32) = (210.0, 270.0);
const LABEL_NES: Rect = Rect {
    x: 20.0,
    y: 28.0,
    w: 170.0,
    h: 142.0,
};
const SHELL_NES: Shell = Shell {
    colour: [0x68, 0x69, 0x6e],
    finish: Finish::Solid,
};
const PORT_NES: &str = include_str!("../../../assets/skins/nes_port.svg");

const CART_SNES: &str = include_str!("../../../assets/skins/snes_cart.svg");
const DETAIL_SNES: &str = include_str!("../../../assets/skins/snes_cart_detail.svg");
const SIZE_SNES: (f32, f32) = (240.0, 190.0);
const LABEL_SNES: Rect = Rect {
    x: 25.0,
    y: 34.0,
    w: 190.0,
    h: 92.0,
};
const SHELL_SNES: Shell = Shell {
    colour: [0x96, 0x94, 0x9b],
    finish: Finish::Solid,
};
const PORT_SNES: &str = include_str!("../../../assets/skins/snes_port.svg");

// The Mega Drive and Master System carts are SLOT2's own redraws too, and they are the pair the
// shelf shows side by side with the NES and Super Nintendo ones: wide and low for the Mega
// Drive, tall with a stepped connector foot for the Master System.
const CART_MD: &str = include_str!("../../../assets/skins/md_cart.svg");
const DETAIL_MD: &str = include_str!("../../../assets/skins/md_cart_detail.svg");
const SIZE_MD: (f32, f32) = (250.0, 180.0);
const LABEL_MD: Rect = Rect {
    x: 32.0,
    y: 34.0,
    w: 186.0,
    h: 84.0,
};
const SHELL_MD: Shell = Shell {
    colour: [0x3d, 0x3e, 0x43],
    finish: Finish::Solid,
};
const PORT_MD: &str = include_str!("../../../assets/skins/md_port.svg");

const CART_SMS: &str = include_str!("../../../assets/skins/sms_cart.svg");
const DETAIL_SMS: &str = include_str!("../../../assets/skins/sms_cart_detail.svg");
const SIZE_SMS: (f32, f32) = (200.0, 230.0);
const LABEL_SMS: Rect = Rect {
    x: 24.0,
    y: 32.0,
    w: 152.0,
    h: 118.0,
};
const SHELL_SMS: Shell = Shell {
    colour: [0x70, 0x72, 0x78],
    finish: Finish::Solid,
};
const PORT_SMS: &str = include_str!("../../../assets/skins/sms_port.svg");

pub fn skin(platform: Platform) -> &'static PlatformSkin {
    match platform {
        Platform::Gb => &SKIN_GB,
        Platform::Gbc => &SKIN_GBC,
        Platform::Gba => &SKIN_GBA,
        Platform::Nes => &SKIN_NES,
        Platform::Snes => &SKIN_SNES,
        Platform::Md => &SKIN_MD,
        Platform::Sms => &SKIN_SMS,
    }
}

static SKIN_GB: PlatformSkin = PlatformSkin {
    platform: Platform::Gb,
    cart: CART_GB,
    cart_detail: DETAIL_GB,
    cart_size: SIZE_GB,
    port: PORT_GB,
    port_size: port_size(SIZE_GB.0),
    insert: Curve::new(0.400, 0.640, 0.025),
    eject: Curve::new(0.360, 0.580, 0.025),
    // A light pak, but a long rail to drag it down: slower and not much quieter.
    sfx_in: SoundProfile::new(0.86, 0.92),
    sfx_out: SoundProfile::new(0.89, 0.88),
    label: LABEL_GB,
    shell: SHELL_GB,
    borrowed: false,
};

static SKIN_GBC: PlatformSkin = PlatformSkin {
    platform: Platform::Gbc,
    cart: CART_GBC,
    cart_detail: DETAIL_GBC,
    cart_size: SIZE_GB,
    port: PORT_GBC,
    port_size: port_size(SIZE_GB.0),
    insert: Curve::new(0.420, 0.650, 0.030),
    eject: Curve::new(0.380, 0.600, 0.030),
    // The same pak in clear plastic: a touch quicker, and a touch quieter still.
    sfx_in: SoundProfile::new(0.93, 0.88),
    sfx_out: SoundProfile::new(0.96, 0.84),
    label: LABEL_GB,
    shell: SHELL_GBC,
    borrowed: false,
};

static SKIN_GBA: PlatformSkin = PlatformSkin {
    platform: Platform::Gba,
    cart: CART_GBA,
    cart_detail: DETAIL_GBA,
    cart_size: SIZE_GBA,
    port: PORT_GBA,
    port_size: port_size(SIZE_GBA.0),
    insert: Curve::new(0.380, 0.580, 0.035),
    eject: Curve::new(0.340, 0.540, 0.035),
    // A short cart with little of it to travel: a quick click, and the quietest of the lot.
    sfx_in: SoundProfile::new(1.10, 0.86),
    sfx_out: SoundProfile::new(1.13, 0.82),
    label: LABEL_GBA,
    shell: SHELL_GBA,
    borrowed: false,
};

static SKIN_NES: PlatformSkin = PlatformSkin {
    platform: Platform::Nes,
    cart: CART_NES,
    cart_detail: DETAIL_NES,
    cart_size: SIZE_NES,
    port: PORT_NES,
    port_size: port_size(SIZE_NES.0),
    insert: Curve::new(0.460, 0.700, 0.020),
    eject: Curve::new(0.420, 0.680, 0.020),
    // The biggest cart of all, dropped in flat: slow, low and at the recording's own level.
    sfx_in: SoundProfile::new(0.82, 1.00),
    sfx_out: SoundProfile::new(0.85, 0.96),
    label: LABEL_NES,
    shell: SHELL_NES,
    borrowed: false,
};

static SKIN_SNES: PlatformSkin = PlatformSkin {
    platform: Platform::Snes,
    cart: CART_SNES,
    cart_detail: DETAIL_SNES,
    cart_size: SIZE_SNES,
    port: PORT_SNES,
    port_size: port_size(SIZE_SNES.0),
    insert: Curve::new(0.360, 0.560, 0.040),
    eject: Curve::new(0.320, 0.520, 0.040),
    // A wide shell, neutral at the recording's own speed.
    sfx_in: SoundProfile::new(1.00, 0.96),
    sfx_out: SoundProfile::new(1.03, 0.92),
    label: LABEL_SNES,
    shell: SHELL_SNES,
    borrowed: false,
};

static SKIN_MD: PlatformSkin = PlatformSkin {
    platform: Platform::Md,
    cart: CART_MD,
    cart_detail: DETAIL_MD,
    cart_size: SIZE_MD,
    port: PORT_MD,
    port_size: port_size(SIZE_MD.0),
    insert: Curve::new(0.340, 0.520, 0.045),
    eject: Curve::new(0.300, 0.500, 0.045),
    // The fastest connector of the seven, in and out.
    sfx_in: SoundProfile::new(1.16, 0.90),
    sfx_out: SoundProfile::new(1.19, 0.86),
    label: LABEL_MD,
    shell: SHELL_MD,
    borrowed: false,
};

static SKIN_SMS: PlatformSkin = PlatformSkin {
    platform: Platform::Sms,
    cart: CART_SMS,
    cart_detail: DETAIL_SMS,
    cart_size: SIZE_SMS,
    port: PORT_SMS,
    port_size: port_size(SIZE_SMS.0),
    insert: Curve::new(0.440, 0.670, 0.028),
    eject: Curve::new(0.400, 0.640, 0.028),
    // A tall cart with a little drag to it: just under the recording's speed, and the
    // quietest eject of the seven.
    sfx_in: SoundProfile::new(0.96, 0.82),
    sfx_out: SoundProfile::new(0.99, 0.78),
    label: LABEL_SMS,
    shell: SHELL_SMS,
    borrowed: false,
};
