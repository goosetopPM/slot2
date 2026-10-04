//! Overlay pictures: which PNG belongs to a platform and a panel, and the texture it becomes.
//!
//! D-11's overlay is one PNG per platform per geometry, drawn over the game and under the UI.
//! This module owns exactly two things: resolving a `(platform, geometry)` pair against the
//! card and this build's own pictures, and turning the winner into one texture that is
//! re-uploaded only when the sources change.
//!
//! What it deliberately does not own: whether an overlay should be drawn at all. A game's
//! `overlay` setting is read by the App, and a source set is handed here; this layer draws the
//! sources it is given and nothing else. It also does not touch the picture: no shader effect,
//! no crop, no scale policy, no dim and no clear — it lays its texture over the whole panel and
//! stops, so the caller decides the order and the screen keeps its own frame underneath.
//!
//! Built-in pictures are compiled in (`include_bytes!`), never read from `assets/` at run time:
//! a device image carries the binary, not the repository's asset tree. The table below holds the
//! sample D-11 named, and a table with one entry and a table with none behave identically for
//! every pair that is not in it — an overlay exists only where one is registered.

use std::path::{Path, PathBuf};

use slot2_gfx::{Canvas, Color, TexId};
use slot2_platform::Geometry;
use slot2_store::{Card, Platform};

/// The folder under `System/` a card's own overlays live in.
pub const CARD_OVERLAY_FOLDER: &str = "Overlays";

/// Whether a game's stored `overlay` setting means "draw the overlay for this game".
///
/// The one place the card's three states become a runtime answer. `None` is the absence of the
/// key, which inherits the platform's default — and D-11's default is no overlay anywhere, so
/// today it is the same answer as an explicit off. They are still read separately: a platform
/// default added later must reach the games that never said anything, and not the ones that
/// said no, and this match is the whole of that boundary.
///
/// Written out rather than `unwrap_or(false)`, which would hide the inheritance behind the
/// answer and make the two states indistinguishable at the one place that has to tell them
/// apart.
pub fn overlay_enabled(setting: Option<bool>) -> bool {
    match setting {
        None => false,
        Some(true) => true,
        Some(false) => false,
    }
}

/// The registry geometry a panel size is, or `None` when this build knows no overlay for it.
///
/// The three supported panels, matched as numbers and never parsed from a name: an unknown size
/// is a panel this build has no picture for, which is a reason to draw no overlay and not a
/// reason to refuse to launch a game.
pub fn geometry_for_panel(panel: (u32, u32)) -> Option<Geometry> {
    match panel {
        (640, 480) => Some(Geometry::W640H480),
        (720, 480) => Some(Geometry::W720H480),
        (720, 720) => Some(Geometry::W720H720),
        _ => None,
    }
}

/// One picture this build ships, for exactly one platform and one panel.
///
/// `id` is a stable name for logs and tests — a picture that is wrong in a log line is worth
/// naming. It is not part of the card format: D-11 has one PNG per pair, so there is nothing
/// for a file to choose between.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltInOverlay {
    pub platform: Platform,
    pub geometry: Geometry,
    pub id: &'static str,
    /// The PNG itself. A static slice, because a device image has no `assets/` to read.
    pub png: &'static [u8],
}

/// Every overlay this build ships.
///
/// One entry, for the pair D-11 named: the Game Boy on the RG CubeXX's square panel. Its
/// aperture is the Game Boy's own 160×144 frame at the default `ScalePolicy::Integer` on
/// 720×720 — 640×576 at (40, 72) — and `build/generate-overlays.py` writes it.
///
/// The other platforms and panels have no picture, and GBC does not share GB's: an overlay is
/// registered for one pair, and everything else resolves to nothing rather than to a stretch.
/// Registering the next pair is one more entry here and nothing else.
pub const BUILT_IN_OVERLAYS: &[BuiltInOverlay] = &[BuiltInOverlay {
    platform: Platform::Gb,
    geometry: Geometry::W720H720,
    id: "gb-cubexx-frame-v1",
    png: include_bytes!("../../../assets/overlays/GB/720x720.png"),
}];

/// `System/Overlays/<PLAT>/<geometry>.png`, the one place a card's overlay can be.
///
/// The platform and geometry spellings are the registry's own (`GB`, `720x480`), so a picture
/// is named after the shelf and the panel it belongs to and after nothing else.
pub fn card_overlay_path(card: &Card, platform: Platform, geometry: Geometry) -> PathBuf {
    card.system_dir()
        .join(CARD_OVERLAY_FOLDER)
        .join(platform.folder())
        .join(format!("{geometry}.png"))
}

/// The pictures that could be drawn for one platform and panel: the card's own file first, the
/// built-in second.
///
/// Both may be present, and both may be absent: the card's picture is preferred but is not the
/// only chance, and a platform with no built-in is the ordinary case today.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlaySources {
    pub platform: Platform,
    pub geometry: Geometry,
    /// The card's own file, when there is a regular file at the path.
    pub card: Option<PathBuf>,
    /// The picture this build ships for the pair, if there is one.
    pub built_in: Option<BuiltInOverlay>,
}

impl OverlaySources {
    /// Whether there is anything to try at all.
    pub fn is_empty(&self) -> bool {
        self.card.is_none() && self.built_in.is_none()
    }
}

/// What one platform and panel should be drawn from, on this card and in this build.
///
/// Resolution is filesystem metadata and table lookup only: nothing here opens a file or
/// decodes anything, so a caller may ask this per launch (or per eject) without paying for the
/// picture. A directory where the picture belongs is not a picture — the card is somebody's
/// filesystem, and a folder named `720x480.png` is not a reason to fail a launch.
pub fn resolve(card: &Card, platform: Platform, geometry: Geometry) -> OverlaySources {
    resolve_with(BUILT_IN_OVERLAYS, card, platform, geometry)
}

/// [`resolve`] against an explicit table.
///
/// The production path passes [`BUILT_IN_OVERLAYS`]; this is the same function with the table
/// in hand, so the tests can register pictures of their own without depending on which ones the
/// build ships. A pair registered twice resolves to the **first** entry: a table is a list, and
/// a list with a duplicate is a mistake somebody can see in the order they wrote.
pub fn resolve_with(
    built_ins: &[BuiltInOverlay],
    card: &Card,
    platform: Platform,
    geometry: Geometry,
) -> OverlaySources {
    let path = card_overlay_path(card, platform, geometry);
    OverlaySources {
        platform,
        geometry,
        card: path.is_file().then_some(path),
        built_in: built_ins
            .iter()
            .find(|b| b.platform == platform && b.geometry == geometry)
            .copied(),
    }
}

/// A decoded overlay: straight (non-premultiplied) RGBA8, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayImage {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// Decode a PNG held in memory, which must be exactly `size` big.
pub fn decode_png_bytes(bytes: &[u8], size: (u32, u32)) -> Result<OverlayImage, String> {
    decode(std::io::Cursor::new(bytes), size)
}

/// Decode the PNG at `path`, which must be exactly `size` big.
///
/// The path is not part of any error message: the caller knows which file it asked for, and a
/// message that repeats it is one more place for a card's layout to leak into a log.
pub fn decode_png_file(path: &Path, size: (u32, u32)) -> Result<OverlayImage, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("the picture could not be opened ({e})"))?;
    decode(file, size)
}

/// The one decoder both sources go through, so a card picture and a built-in one are held to
/// the same rules.
///
/// The header is read first and its size must be the panel's exactly. Nothing is allocated
/// until then, so a PNG claiming 40000x40000 costs one refused header rather than 6.4 GB — and
/// a picture that is not the panel's size is a broken overlay rather than something to stretch.
fn decode<R: std::io::Read>(source: R, size: (u32, u32)) -> Result<OverlayImage, String> {
    let mut decoder = png::Decoder::new(source);
    // Palettes and tRNS become RGB/RGBA, sub-byte grayscale becomes 8-bit and 16-bit samples
    // become 8-bit, all by the png crate's own transformations rather than by hand. Straight
    // alpha stays straight: nothing here premultiplies it or drops it.
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("the PNG header was refused ({e})"))?;

    let (w, h) = reader.info().size();
    if (w, h) != size {
        return Err(format!(
            "the picture is {w}x{h} and this panel draws {}x{}",
            size.0, size.1
        ));
    }
    let pixels = (w as usize)
        .checked_mul(h as usize)
        .ok_or_else(|| format!("a {w}x{h} picture is too large to hold"))?;
    let expected = pixels
        .checked_mul(4)
        .ok_or_else(|| format!("a {w}x{h} RGBA8 picture is too large to hold"))?;

    // The size is known to be the panel's, so this buffer is bounded by the panel: the header
    // is what decides, and it was checked above.
    let raw = reader.output_buffer_size();
    let channels = match reader.output_color_type().0 {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        other => return Err(format!("a {other:?} picture is not an overlay")),
    };
    if raw != pixels * channels {
        return Err(format!(
            "a {w}x{h} picture does not hold the pixels its header describes"
        ));
    }

    let mut buf = vec![0u8; raw];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("the picture did not decode ({e})"))?;
    if (info.width, info.height) != size || info.buffer_size() != raw {
        return Err(format!(
            "the picture decoded to {}x{} rather than {w}x{h}",
            info.width, info.height
        ));
    }
    if info.bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "the picture is not 8 bits per sample ({:?})",
            info.bit_depth
        ));
    }

    let rgba = to_rgba8(&buf, pixels, info.color_type)?;
    if rgba.len() != expected {
        return Err(format!(
            "a {w}x{h} picture is not {expected} bytes of RGBA8"
        ));
    }
    Ok(OverlayImage { w, h, rgba })
}

/// Straight RGBA8 from whatever the png crate handed back, for the four 8-bit colour types it
/// can produce after the transformations above. Written as a match with no catch-all
/// conversion: a type nobody expected is an error, not a silent guess about channel order.
fn to_rgba8(data: &[u8], pixels: usize, color_type: png::ColorType) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(pixels * 4);
    match color_type {
        png::ColorType::Rgba => out.extend_from_slice(&data[..pixels * 4]),
        png::ColorType::Rgb => {
            for px in data[..pixels * 3].as_chunks::<3>().0 {
                out.extend_from_slice(&[px[0], px[1], px[2], 0xFF]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for px in data[..pixels * 2].as_chunks::<2>().0 {
                out.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
            }
        }
        png::ColorType::Grayscale => {
            for g in &data[..pixels] {
                out.extend_from_slice(&[*g, *g, *g, 0xFF]);
            }
        }
        other => return Err(format!("a {other:?} picture is not an overlay")),
    }
    Ok(out)
}

/// Which source a drawn texture came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlaySourceKind {
    Card,
    BuiltIn,
}

/// The overlay texture for one screen: resolved sources, at most one upload, and what happened.
///
/// One attempt per set of sources. The first canvas that sees a source set decodes it, and
/// every later frame draws the texture it made; a set that failed is not retried frame after
/// frame — a card whose picture is broken would otherwise be read and refused sixty times a
/// second. Handing the layer a different set (or an empty one) is what asks for a new attempt.
#[derive(Debug, Default)]
pub struct OverlayLayer {
    sources: Option<OverlaySources>,
    /// The sources changed since the last canvas saw them: the old texture is freed, and a new
    /// attempt is made, on the next canvas access.
    pending: bool,
    /// Whether the current sources have already had their one attempt.
    attempted: bool,
    tex: Option<TexId>,
    used: Option<OverlaySourceKind>,
    error: Option<String>,
}

impl OverlayLayer {
    /// Point the layer at a resolved set. Setting the same value again is not a change: the
    /// texture stays and nothing is decoded.
    pub fn set_sources(&mut self, sources: OverlaySources) {
        self.retarget(Some(sources));
    }

    /// Leave the layer with nothing to draw. The texture, if there is one, goes at the next
    /// canvas access like any other source change.
    pub fn clear_sources(&mut self) {
        self.retarget(None);
    }

    fn retarget(&mut self, sources: Option<OverlaySources>) {
        if self.sources == sources {
            return;
        }
        self.sources = sources;
        self.used = None;
        self.error = None;
        self.attempted = false;
        self.pending = true;
    }

    /// Which source the texture on screen came from. `None` before the first successful draw,
    /// and after a set where every source failed.
    pub fn used(&self) -> Option<OverlaySourceKind> {
        self.used
    }

    /// The last thing that went wrong, card fallback included: a built-in that covered a
    /// broken card picture still leaves the card's failure here to be read. `None` when the
    /// sources were drawn, or when there was nothing to try.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// The texture on screen, if there is one. For callers that need the id rather than the
    /// picture — tests, and any later code that wants to free it itself.
    pub fn texture(&self) -> Option<TexId> {
        self.tex
    }

    /// Free the texture now and forget the sources, leaving the layer as it started.
    ///
    /// Called with a canvas because a texture belongs to the canvas that made it, and dropping
    /// a layer is not a reason to reach for GL.
    pub fn release(&mut self, canvas: &mut dyn Canvas) {
        if let Some(tex) = self.tex.take() {
            canvas.free(tex);
        }
        self.sources = None;
        self.pending = false;
        self.attempted = false;
        self.used = None;
        self.error = None;
    }

    /// Draw the overlay over the whole panel, or nothing at all.
    ///
    /// One attempt per source set: the picture is opened, decoded and uploaded the first time
    /// the sources are seen, and only drawn afterwards. An empty set, or a set where both
    /// sources failed, draws nothing — the game's own frame is the whole screen then.
    pub fn draw(&mut self, canvas: &mut dyn Canvas) {
        if self.pending {
            // A texture belongs to the canvas that made it, so the free waits for one. This is
            // also the only place a source change costs anything.
            if let Some(tex) = self.tex.take() {
                canvas.free(tex);
            }
            self.pending = false;
            self.attempted = false;
            self.used = None;
            self.error = None;
        }

        let Some(sources) = self.sources.clone() else {
            return;
        };
        let (w, h) = sources.geometry.size();
        if canvas.size() != (w, h) {
            // An overlay is a panel-sized picture: another geometry's file must not be
            // stretched over this panel, and a canvas that is not the panel cannot show it.
            self.error = Some(format!(
                "an overlay for {w}x{h} cannot be drawn on a {}x{} canvas",
                canvas.size().0,
                canvas.size().1
            ));
            return;
        }

        if !self.attempted {
            self.attempted = true;
            self.attempt(canvas, &sources);
        }
        if let Some(tex) = self.tex {
            canvas.image(tex, 0.0, 0.0, w as f32, h as f32, Color::WHITE);
        }
    }

    /// The one attempt for the current sources: the card's own picture first, the built-in
    /// second, and both failures kept.
    fn attempt(&mut self, canvas: &mut dyn Canvas, sources: &OverlaySources) {
        let size = sources.geometry.size();
        let mut trouble: Vec<String> = Vec::new();

        if let Some(path) = sources.card.clone() {
            match decode_png_file(&path, size) {
                Ok(image) => {
                    self.tex = Some(canvas.upload_rgba8(image.w, image.h, &image.rgba));
                    self.used = Some(OverlaySourceKind::Card);
                    self.error = None;
                    return;
                }
                // The card's picture is the card's business: a broken one falls back rather
                // than taking a working built-in down with it.
                Err(e) => trouble.push(format!("the card's own picture was not used ({e})")),
            }
        }

        if let Some(built_in) = sources.built_in {
            match decode_png_bytes(built_in.png, size) {
                Ok(image) => {
                    self.tex = Some(canvas.upload_rgba8(image.w, image.h, &image.rgba));
                    self.used = Some(OverlaySourceKind::BuiltIn);
                    self.error = (!trouble.is_empty()).then(|| trouble.join("; "));
                    return;
                }
                Err(e) => trouble.push(format!(
                    "the built-in picture {} was not used ({e})",
                    built_in.id
                )),
            }
        }

        self.used = None;
        self.error = (!trouble.is_empty()).then(|| trouble.join("; "));
    }
}
