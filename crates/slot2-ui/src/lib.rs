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

pub mod about_sticker;
pub mod art;
pub mod cheat_menu;
pub mod core_picker;
pub mod device_menu;
pub mod display_menu;
pub mod face;
pub mod hud;
pub mod image;
pub mod in_game_menu;
pub mod insert;
pub mod label;
pub mod language_picker;
pub mod layout;
pub mod overlay_menu;
pub mod overscan_menu;
pub mod power_menu;
pub mod refusal;
pub mod shader_menu;
pub mod shelf;
pub mod shelf_menu;
pub mod shelf_view;
pub mod skin;
pub mod splash;
pub mod state_switcher;
pub mod svg;
pub mod timezone_menu;
pub mod toast;
pub mod wallpaper;

pub use about_sticker::{AboutInfo, AboutSticker};
pub use art::ArtCache;
pub use cheat_menu::CheatMenu;
pub use core_picker::CorePicker;
pub use device_menu::{DeviceMenu, DeviceSetting};
pub use display_menu::{DisplayChoice, DisplayMenu};
pub use face::FaceCache;
pub use hud::Hud;
pub use in_game_menu::{InGameChoice, InGameMenu};
pub use language_picker::{LanguageOption, LanguagePicker};
pub use layout::{SafeArea, SAFE_H, SAFE_W};
pub use overlay_menu::OverlayMenu;
pub use overscan_menu::OverscanMenu;
pub use power_menu::{PowerChoice, PowerMenu};
pub use shader_menu::ShaderMenu;
pub use shelf::{Placement, Shelf, SIDE_ALPHA, SIDE_SCALE};
pub use shelf_menu::{ShelfAvailability, ShelfChoice, ShelfMenu, SHELF_CHOICES};
pub use shelf_view::ShelfView;
pub use splash::Splash;
pub use state_switcher::StateSwitcher;
pub use timezone_menu::TimezoneMenu;

use std::path::{Component, Path, PathBuf};

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
    /// Where a language pack's preferred font (its `lang-font`) and `CJK_FONT` are searched,
    /// in order: the card's own `System/Fonts` first, the build's assets after it.
    pub font_dirs: Vec<PathBuf>,
}

impl UiCtx {
    /// Build the context for a panel: the language loaded with `card_lang_dir` as the override
    /// directory, and the font chain that language asks for — its own preferred font first,
    /// then the UI font from `UI_FONT`, then the CJK font registered lazily from the first
    /// `font_dirs` entry that contains it (or, if none does, from the first dir anyway so the
    /// miss is logged once at use).
    ///
    /// The language comes first because it decides the chain: a pack that names a font is a
    /// request, and a request that did not load is not a request — a fallback to English uses
    /// English's own (empty) preference rather than the font of a pack nobody is reading.
    /// Nothing here reads a font file: the preferred slot and the CJK slot are registered by
    /// path, and the first glyph that needs them is what parses them.
    pub fn new(
        profile: Profile,
        lang: &str,
        font_dirs: Vec<PathBuf>,
        card_lang_dir: Option<&Path>,
    ) -> Self {
        let i18n = I18n::load(lang, card_lang_dir)
            .or_else(|_| I18n::embedded(slot2_i18n::FALLBACK))
            .expect("the embedded English pack always loads");

        let mut fonts = FontChain::new();
        let wanted = i18n.font();
        let preferred = preferred_font_path(&font_dirs, wanted.clone());
        if let Some(p) = preferred.as_ref() {
            fonts.push_lazy_file(p);
        } else if let Some(name) = wanted.as_deref() {
            // A pack asking for a font this machine does not have is worth one line: the UI
            // still runs, in the default fonts, and that is exactly what is hard to notice.
            eprintln!(
                "slot2: language {:?} prefers the font {name:?}, which is not a usable file here; \
                 keeping the default fonts",
                i18n.code()
            );
        }
        // The embedded font cannot fail to parse; if it ever did, text would draw as tofu,
        // which is still a running frontend.
        let _ = fonts.push_bytes("OpenSans-Regular", UI_FONT.to_vec());
        let cjk = font_dirs
            .iter()
            .map(|d| d.join(CJK_FONT))
            .find(|p| p.is_file())
            .or_else(|| font_dirs.first().map(|d| d.join(CJK_FONT)));
        if let Some(p) = cjk {
            // A language whose preferred font is the CJK font itself — Korean's is — gets it
            // registered once, in front: the slot that draws Hangul is the one the pack asked
            // for, and one file is not parsed twice.
            if preferred.as_ref() != Some(&p) {
                fonts.push_lazy_file(&p);
            }
        }
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

/// The file a pack's `lang-font` names, if that name is a plain file name one of `font_dirs`
/// really holds, in the caller's own order.
///
/// Only a bare file name is allowed. A pack is card data, and a value carrying a separator, a
/// root or a `.`/`..` component would let a card point the frontend at a file outside the
/// directories it was given. The name is used exactly as the pack spells it — no extension is
/// tried on its behalf and no case is folded — and the first directory that has it wins.
fn preferred_font_path(font_dirs: &[PathBuf], name: Option<String>) -> Option<PathBuf> {
    let name = name?;
    // Windows and Linux disagree about the separator, so both are refused on both.
    if name.contains('/') || name.contains('\\') {
        return None;
    }
    let mut parts = Path::new(&name).components();
    match (parts.next(), parts.next()) {
        (Some(Component::Normal(_)), None) => {}
        _ => return None,
    }
    font_dirs
        .iter()
        .map(|d| d.join(&name))
        .find(|p| p.is_file())
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
