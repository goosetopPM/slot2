//! Screens, platform skins, rasterised faces, span layout.
//!
//! Everything here draws through `slot2_gfx::Canvas` and knows nothing about windows, GL or
//! framebuffers, so every screen can be exercised against `RecordingCanvas` in a unit test
//! and rendered for real by the binary.
//!
//! - [`UiCtx`] bundles what every screen needs: the font chain, the language, the device
//!   profile, and the safe-area offset for the current panel (D-09).
//! - [`FaceCache`] turns text into textures ("faces") through the font chain and remembers
//!   them, so a label that does not change costs nothing per frame.
//! - [`draw_spans`] lays out an i18n message's spans left to right: text through the face
//!   cache, `BTN(..)` spans as a rounded button cap with its label inside.
//! - [`Splash`] is the first screen: wordmark, greeting, device line, a hint row.

pub mod art;
pub mod face;
pub mod hud;
pub mod image;
pub mod insert;
pub mod label;
pub mod layout;
pub mod power_menu;
pub mod refusal;
pub mod shelf;
pub mod shelf_view;
pub mod skin;
pub mod splash;
pub mod svg;
pub mod toast;
pub mod wallpaper;

pub use art::ArtCache;
pub use face::FaceCache;
pub use hud::Hud;
pub use layout::{SafeArea, SAFE_H, SAFE_W};
pub use power_menu::{PowerChoice, PowerMenu};
pub use shelf::{Placement, Shelf, SIDE_ALPHA, SIDE_SCALE};
pub use shelf_view::ShelfView;
pub use splash::Splash;

use std::path::{Path, PathBuf};

use slot2_gfx::{Canvas, Color};
use slot2_i18n::{I18n, Span};
use slot2_platform::Profile;
use slot2_text::FontChain;

/// The Latin UI font, compiled in so a card with no `System/Fonts/` still has text.
pub const UI_FONT: &[u8] = include_bytes!("../../../assets/fonts/OpenSans-Regular.ttf");

/// The CJK font's file name. Looked for in each of `UiCtx::font_dirs`, loaded lazily.
pub const CJK_FONT: &str = "NotoSansKR-Regular.otf";

/// Text sizes, in panel pixels. Chosen for a 640x480 safe area viewed at arm's length.
pub const PX_WORDMARK: f32 = 64.0;
pub const PX_TITLE: f32 = 24.0;
pub const PX_BODY: f32 = 16.0;
pub const PX_HINT: f32 = 14.0;

/// Shared state every screen draws with.
pub struct UiCtx {
    pub fonts: FontChain,
    pub faces: FaceCache,
    pub i18n: I18n,
    pub profile: Profile,
    pub safe: SafeArea,
    /// Where `CJK_FONT` (and later, a language pack's `font=`) is searched, in order.
    pub font_dirs: Vec<PathBuf>,
}

impl UiCtx {
    /// Build the context for a panel: the UI font from `UI_FONT`, the CJK font registered
    /// lazily from the first `font_dirs` entry that contains it (or, if none does, from the
    /// first dir anyway so the miss is logged once at use), the language loaded with
    /// `card_lang_dir` as the override directory.
    pub fn new(
        profile: Profile,
        lang: &str,
        font_dirs: Vec<PathBuf>,
        card_lang_dir: Option<&Path>,
    ) -> Self {
        let mut fonts = FontChain::new();
        // The embedded font cannot fail to parse; if it ever did, text would draw as tofu,
        // which is still a running frontend.
        let _ = fonts.push_bytes("OpenSans-Regular", UI_FONT.to_vec());
        let cjk = font_dirs
            .iter()
            .map(|d| d.join(CJK_FONT))
            .find(|p| p.is_file())
            .or_else(|| font_dirs.first().map(|d| d.join(CJK_FONT)));
        if let Some(p) = cjk {
            fonts.push_lazy_file(&p);
        }
        let i18n = I18n::load(lang, card_lang_dir)
            .or_else(|_| I18n::embedded(slot2_i18n::FALLBACK))
            .expect("the embedded English pack always loads");
        UiCtx {
            fonts,
            faces: FaceCache::default(),
            i18n,
            profile,
            safe: SafeArea::for_geometry(profile.geometry),
            font_dirs,
        }
    }
}

/// Draw a message's spans on one baseline starting at `(x, y)` (top-left of the line box),
/// `px` tall, in `color`. Returns the total width drawn. Button spans become a cap: a
/// rounded-looking rect (plain rect is fine in M0) of `color.with_alpha(0.25)` behind the
/// label drawn in `color`, with `px * 0.4` horizontal padding and `px * 0.3` gaps either
/// side. Implemented in `face.rs` by task 04.
pub fn draw_spans(
    canvas: &mut dyn Canvas,
    ctx: &mut UiCtx,
    spans: &[Span],
    px: f32,
    x: f32,
    y: f32,
    color: Color,
) -> f32 {
    face::draw_spans(canvas, ctx, spans, px, x, y, color)
}
