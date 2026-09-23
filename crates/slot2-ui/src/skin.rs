//! What each shelf's cartridge looks like.
//!
//! One row per platform: which drawing, how big it is in its own units, where a label sits
//! on it, and what colour the shell is when nothing is known about the game. Sizes are the
//! artwork's `viewBox` and the test reads them back out of the files, so a typo here fails
//! the build rather than drawing a shelf of wrong-shaped carts.
//!
//! Three shelves have drawings and four do not. The four say so (`borrowed`) instead of
//! quietly wearing the Game Boy Advance's, because a silent fallback leaves nobody able to
//! tell which shelves still need art.

use slot2_store::Platform;

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
    pub port: &'static str,
    pub port_size: (f32, f32),
    /// True when this platform has no drawing of its own and is using another's.
    pub borrowed: bool,
}

const SOCKET: &str = include_str!("../../../assets/skins/socket.svg");
const PORT_SIZE: (f32, f32) = (41.0, 29.8);

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

const CART_GBC: &str = include_str!("../../../assets/skins/gbc_cart.svg");
const DETAIL_GBC: &str = include_str!("../../../assets/skins/gbc_cart_detail.svg");
const SHELL_GBC: Shell = Shell {
    colour: [0x7c, 0x7a, 0x8a],
    finish: Finish::Translucent,
};

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
    label: LABEL_GB,
    shell: SHELL_GB,
    port: SOCKET,
    port_size: PORT_SIZE,
    borrowed: false,
};

static SKIN_GBC: PlatformSkin = PlatformSkin {
    platform: Platform::Gbc,
    cart: CART_GBC,
    cart_detail: DETAIL_GBC,
    cart_size: SIZE_GB,
    label: LABEL_GB,
    shell: SHELL_GBC,
    port: SOCKET,
    port_size: PORT_SIZE,
    borrowed: false,
};

static SKIN_GBA: PlatformSkin = PlatformSkin {
    platform: Platform::Gba,
    cart: CART_GBA,
    cart_detail: DETAIL_GBA,
    cart_size: SIZE_GBA,
    label: LABEL_GBA,
    shell: SHELL_GBA,
    port: SOCKET,
    port_size: PORT_SIZE,
    borrowed: false,
};

/// A shelf with no drawing of its own wears the Game Boy Advance's, and admits it.
const fn borrowing(platform: Platform) -> PlatformSkin {
    PlatformSkin {
        platform,
        cart: CART_GBA,
        cart_detail: DETAIL_GBA,
        cart_size: SIZE_GBA,
        label: LABEL_GBA,
        shell: SHELL_GBA,
        port: SOCKET,
        port_size: PORT_SIZE,
        borrowed: true,
    }
}

static SKIN_NES: PlatformSkin = borrowing(Platform::Nes);
static SKIN_SNES: PlatformSkin = borrowing(Platform::Snes);
static SKIN_MD: PlatformSkin = borrowing(Platform::Md);
static SKIN_SMS: PlatformSkin = borrowing(Platform::Sms);
