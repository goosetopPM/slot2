//! The overlay asset layer: which PNG a platform and a panel resolve to, what a strict decode
//! makes of it, and the one texture the layer keeps.
//!
//! Every fixture here is generated in the test — small PNGs of each colour kind, the panel-sized
//! pictures the strict path demands, and a PNG whose header lies about its size. Nothing reads
//! the repository's assets, and nothing needs a GPU: the canvas below is the recording one with
//! the uploaded bytes kept, which is the only thing it does not already keep.

use std::fs;
use std::path::PathBuf;

use slot2_gfx::{Canvas, Color, Op, RecordingCanvas, TexId};
use slot2_platform::Geometry;
use slot2_store::{Card, Platform};

use slot2::overlay::{
    card_overlay_path, decode_png_bytes, decode_png_file, overlay_enabled, resolve, resolve_with,
    BuiltInOverlay, OverlayImage, OverlayLayer, OverlaySourceKind, OverlaySources,
    BUILT_IN_OVERLAYS,
};

/// The panel these tests draw on. 640x480, the safe area every other geometry contains.
const G: Geometry = Geometry::W640H480;

// ------------------------------------------------------------------ canvases

/// A `RecordingCanvas` that also keeps the bytes uploaded, so a decoded picture can be checked
/// pixel for pixel without inventing a texture format.
struct PixelCanvas {
    inner: RecordingCanvas,
    uploads: Vec<(TexId, u32, u32, Vec<u8>)>,
}

impl PixelCanvas {
    fn new(size: (u32, u32)) -> Self {
        PixelCanvas {
            inner: RecordingCanvas::new(size.0, size.1),
            uploads: Vec::new(),
        }
    }

    fn frees(&self, tex: TexId) -> usize {
        self.inner
            .ops
            .iter()
            .filter(|o| matches!(o, Op::Free(t) if *t == tex))
            .count()
    }

    fn images(&self) -> Vec<&Op> {
        self.inner
            .ops
            .iter()
            .filter(|o| matches!(o, Op::Image { .. }))
            .collect()
    }
}

impl Canvas for PixelCanvas {
    fn size(&self) -> (u32, u32) {
        self.inner.size()
    }

    fn clear(&mut self, color: Color) {
        self.inner.clear(color);
    }

    fn upload_rgba8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        let id = self.inner.upload_rgba8(w, h, data);
        self.uploads.push((id, w, h, data.to_vec()));
        id
    }

    fn upload_alpha8(&mut self, w: u32, h: u32, data: &[u8]) -> TexId {
        self.inner.upload_alpha8(w, h, data)
    }

    fn free(&mut self, tex: TexId) {
        self.inner.free(tex);
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.inner.rect(x, y, w, h, color);
    }

    fn image(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        self.inner.image(tex, x, y, w, h, tint);
    }

    fn image_uv(&mut self, tex: TexId, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], tint: Color) {
        self.inner.image_uv(tex, x, y, w, h, uv, tint);
    }
}

// ------------------------------------------------------------------ fixtures

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-overlay-{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

/// A card with its folders made, and its root.
fn card(tag: &str) -> (Card, PathBuf) {
    let root = scratch(tag);
    let card = Card::new(&root);
    card.ensure_layout();
    (card, root)
}

/// A PNG of `w`x`h` with `data` as its only image data, at the given colour kind and depth.
fn png_with(w: u32, h: u32, color: png::ColorType, depth: png::BitDepth, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w, h);
        encoder.set_color(color);
        encoder.set_depth(depth);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(data).unwrap();
    }
    out
}

/// The same, for a palette image with a transparency table.
fn png_indexed(w: u32, h: u32, palette: Vec<u8>, trns: Vec<u8>, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w, h);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_palette(palette);
        encoder.set_trns(trns);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(data).unwrap();
    }
    out
}

/// A panel-sized RGBA picture whose every pixel is black except two: one marker that carries
/// `seed`, so two fixtures never share bytes, and one fully transparent pixel, whose alpha a
/// straight-alpha decode has to leave alone.
fn panel_pixels(w: u32, h: u32, seed: u8) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    let marker = ((20 * w + 10) * 4) as usize;
    out[marker] = seed;
    out[marker + 1] = 0x11;
    out[marker + 2] = 0x22;
    out[marker + 3] = 0xFF;
    let clear = ((30 * w + 40) * 4) as usize;
    out[clear] = 0x30;
    out[clear + 1] = 0x40;
    out[clear + 2] = 0x50;
    out[clear + 3] = 0x00;
    out
}

/// A panel-sized RGBA PNG of [`panel_pixels`].
fn panel_png(geometry: Geometry, seed: u8) -> Vec<u8> {
    let (w, h) = geometry.size();
    png_with(
        w,
        h,
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &panel_pixels(w, h, seed),
    )
}

fn built_in(
    platform: Platform,
    geometry: Geometry,
    id: &'static str,
    png: Vec<u8>,
) -> BuiltInOverlay {
    BuiltInOverlay {
        platform,
        geometry,
        id,
        png: Box::leak(png.into_boxed_slice()),
    }
}

/// Put a picture on the card where its platform and geometry say it belongs.
fn put_on_card(card: &Card, platform: Platform, geometry: Geometry, bytes: &[u8]) -> PathBuf {
    let path = card_overlay_path(card, platform, geometry);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}

/// An empty set for a pair: resolved, and with nothing behind it.
fn empty_sources(platform: Platform, geometry: Geometry) -> OverlaySources {
    OverlaySources {
        platform,
        geometry,
        card: None,
        built_in: None,
    }
}

fn layer_for(sources: OverlaySources) -> OverlayLayer {
    let mut layer = OverlayLayer::default();
    layer.set_sources(sources);
    layer
}

fn draw(layer: &mut OverlayLayer, size: (u32, u32)) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(size);
    layer.draw(&mut canvas);
    canvas
}

// ------------------------------------------------------------------ resolution

#[test]
fn the_card_path_of_every_platform_and_geometry_is_spelled_exactly() {
    let (card, root) = card("paths");
    let platforms = [
        (Platform::Gb, "GB"),
        (Platform::Gbc, "GBC"),
        (Platform::Gba, "GBA"),
        (Platform::Nes, "NES"),
        (Platform::Snes, "SNES"),
        (Platform::Md, "MD"),
        (Platform::Sms, "SMS"),
    ];
    let geometries = [
        (Geometry::W640H480, "640x480"),
        (Geometry::W720H480, "720x480"),
        (Geometry::W720H720, "720x720"),
    ];
    assert_eq!(
        platforms.len(),
        Platform::ALL.len(),
        "a platform is missing"
    );

    for (platform, folder) in platforms {
        for (geometry, name) in geometries {
            assert_eq!(
                card_overlay_path(&card, platform, geometry),
                root.join("System")
                    .join("Overlays")
                    .join(folder)
                    .join(format!("{name}.png")),
                "{platform:?}/{geometry:?}"
            );
        }
    }
}

#[test]
fn only_a_regular_file_is_a_card_override() {
    let (card, _root) = card("override");
    let png = panel_png(G, 0xC1);
    let path = card_overlay_path(&card, Platform::Gba, G);
    let table = [built_in(Platform::Gba, G, "sample", png.clone())];

    // Nothing on the card: the built-in is what is left, and the card source is absent.
    let missing = resolve_with(&table, &card, Platform::Gba, G);
    assert_eq!(missing.card, None);
    assert_eq!(missing.built_in.map(|b| b.id), Some("sample"));
    assert!(!missing.is_empty());

    // A directory where the picture belongs is not a picture: a card is somebody's filesystem.
    fs::create_dir_all(&path).unwrap();
    let directory = resolve_with(&table, &card, Platform::Gba, G);
    assert_eq!(directory.card, None, "a directory was taken for a picture");
    fs::remove_dir(&path).unwrap();

    // A regular file is the card's own source, and the built-in stays as the fallback.
    fs::write(&path, &png).unwrap();
    let file = resolve_with(&table, &card, Platform::Gba, G);
    assert_eq!(file.card.as_deref(), Some(path.as_path()));
    assert_eq!(file.built_in.map(|b| b.id), Some("sample"));

    // Resolving never opens anything: a file that is there but not a PNG still resolves, and
    // it is the decode that will refuse it.
    fs::write(&path, b"not a png").unwrap();
    assert!(resolve_with(&table, &card, Platform::Gba, G).card.is_some());
}

#[test]
fn a_built_in_is_chosen_by_platform_and_geometry_and_the_first_registration_wins() {
    let (card, _root) = card("builtins");
    let first = built_in(Platform::Gba, Geometry::W720H480, "first", vec![1]);
    let second = built_in(Platform::Gba, Geometry::W720H480, "second", vec![2]);
    let other_geometry = built_in(Platform::Gba, Geometry::W640H480, "other-geometry", vec![3]);
    let other_platform = built_in(Platform::Nes, Geometry::W720H480, "other-platform", vec![4]);
    let table = [first, other_geometry, second, other_platform];

    let got = |platform, geometry| {
        resolve_with(&table, &card, platform, geometry)
            .built_in
            .map(|b| b.id)
    };
    assert_eq!(got(Platform::Gba, Geometry::W720H480), Some("first"));
    assert_eq!(
        got(Platform::Gba, Geometry::W640H480),
        Some("other-geometry")
    );
    assert_eq!(
        got(Platform::Nes, Geometry::W720H480),
        Some("other-platform")
    );
    // Neither half of the pair selects on its own.
    assert_eq!(got(Platform::Nes, Geometry::W640H480), None);
    assert_eq!(got(Platform::Gba, Geometry::W720H720), None);
}

#[test]
fn a_pair_with_no_registration_and_nothing_on_the_card_resolves_to_no_sources() {
    // 640x480 is not a panel any built-in is registered for, and this card has no picture:
    // the two halves of the pair have to agree before a picture exists at all.
    let (card, _root) = card("empty");
    assert_eq!(G, Geometry::W640H480);

    let nothing = resolve(&card, Platform::Gba, G);
    assert!(nothing.is_empty());
    assert_eq!(nothing.card, None);
    assert_eq!(nothing.built_in, None);

    // And a layer handed that set draws nothing and reports nothing wrong: a game with no
    // overlay is not a game with a broken one.
    let mut layer = layer_for(nothing);
    let canvas = draw(&mut layer, G.size());
    assert!(canvas.uploads.is_empty());
    assert!(canvas.inner.ops.is_empty());
    assert_eq!(layer.used(), None);
    assert_eq!(layer.error(), None);
    assert_eq!(layer.texture(), None);
}

// ------------------------------------------------------------------ decoding

#[test]
fn every_colour_kind_decodes_to_straight_rgba8() {
    // 4x2, so the fixtures are small: the strict size check is told what to expect.
    let size = (4u32, 2u32);
    let (w, h) = size;

    // RGBA, with one transparent pixel and one half-transparent: alpha stays as written.
    let rgba = [
        0x10, 0x20, 0x30, 0xFF, 0x40, 0x50, 0x60, 0x00, 0x70, 0x80, 0x90, 0x80, 0xAA, 0xBB, 0xCC,
        0xFF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x10,
    ];
    let got = decode_png_bytes(
        &png_with(w, h, png::ColorType::Rgba, png::BitDepth::Eight, &rgba),
        size,
    )
    .unwrap();
    assert_eq!((got.w, got.h), size);
    assert_eq!(got.rgba, rgba, "RGBA8 was not passed through untouched");
    assert_eq!(got.rgba[7], 0x00, "a transparent pixel lost its alpha");

    // RGB: opaque, channel for channel.
    let rgb = [
        1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    ];
    let got = decode_png_bytes(
        &png_with(w, h, png::ColorType::Rgb, png::BitDepth::Eight, &rgb),
        size,
    )
    .unwrap();
    assert_eq!(
        got.rgba,
        [
            1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255, 13, 14, 15, 255, 16, 17, 18,
            255, 19, 20, 21, 255, 22, 23, 24, 255
        ]
    );

    // Grayscale: one channel, three ways, opaque.
    let gray = [0u8, 1, 2, 3, 0x80, 0xFE, 0xFF, 0x40];
    let got = decode_png_bytes(
        &png_with(w, h, png::ColorType::Grayscale, png::BitDepth::Eight, &gray),
        size,
    )
    .unwrap();
    assert_eq!(
        got.rgba,
        [
            0, 0, 0, 255, 1, 1, 1, 255, 2, 2, 2, 255, 3, 3, 3, 255, 0x80, 0x80, 0x80, 255, 0xFE,
            0xFE, 0xFE, 255, 0xFF, 0xFF, 0xFF, 255, 0x40, 0x40, 0x40, 255
        ]
    );

    // Grayscale + alpha: the alpha is the pixel's own, including zero.
    let ga = [
        0x11u8, 0xFF, 0x22, 0x00, 0x33, 0x80, 0x44, 0x10, 0x55, 0x01, 0x66, 0xFE, 0x77, 0x7F, 0x88,
        0x40,
    ];
    let got = decode_png_bytes(
        &png_with(
            w,
            h,
            png::ColorType::GrayscaleAlpha,
            png::BitDepth::Eight,
            &ga,
        ),
        size,
    )
    .unwrap();
    assert_eq!(
        got.rgba,
        [
            0x11, 0x11, 0x11, 0xFF, 0x22, 0x22, 0x22, 0x00, 0x33, 0x33, 0x33, 0x80, 0x44, 0x44,
            0x44, 0x10, 0x55, 0x55, 0x55, 0x01, 0x66, 0x66, 0x66, 0xFE, 0x77, 0x77, 0x77, 0x7F,
            0x88, 0x88, 0x88, 0x40
        ]
    );

    // Palette with a transparency table: index 1 is fully transparent and stays so.
    let indexed = [0u8, 1, 0, 1, 1, 0, 1, 0];
    let got = decode_png_bytes(
        &png_indexed(w, h, vec![255, 0, 0, 0, 0, 255], vec![255, 0], &indexed),
        size,
    )
    .unwrap();
    let red = [255, 0, 0, 255];
    let blue_clear = [0, 0, 255, 0];
    let mut want = Vec::new();
    for index in indexed {
        want.extend_from_slice(if index == 0 { &red } else { &blue_clear });
    }
    assert_eq!(got.rgba, want, "the palette and its tRNS were not read");

    // Palette without a transparency table: every index is opaque.
    let got = decode_png_bytes(
        &png_indexed(
            w,
            h,
            vec![0, 255, 0, 255, 255, 0],
            vec![],
            &[0u8, 1, 0, 1, 0, 1, 0, 1],
        ),
        size,
    )
    .unwrap();
    assert_eq!(&got.rgba[..4], [0, 255, 0, 255]);
    assert_eq!(&got.rgba[4..8], [255, 255, 0, 255]);
    assert!(got.rgba.as_chunks::<4>().0.iter().all(|px| px[3] == 255));

    // 16-bit RGB: each sample is stripped to its high byte, which is what the png crate's own
    // `STRIP_16` does, and never a truncated channel order.
    let rgb16 = [
        0x11u8, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44, 0x55, 0x55, 0x66, 0x66, 0x77, 0x77, 0x88,
        0x88, 0x99, 0x99, 0xAA, 0xAA, 0xBB, 0xBB, 0xCC, 0xCC, 0xDD, 0xDD, 0xEE, 0xEE, 0xFF, 0xFF,
        0x00, 0x00, 0x01, 0x01, 0x02, 0x02, 0x03, 0x03, 0x04, 0x04, 0x05, 0x05, 0x06, 0x06, 0x07,
        0x07, 0x08, 0x08,
    ];
    let got = decode_png_bytes(
        &png_with(w, h, png::ColorType::Rgb, png::BitDepth::Sixteen, &rgb16),
        size,
    )
    .unwrap();
    assert_eq!(got.rgba.len(), (w * h * 4) as usize);
    assert_eq!(
        &got.rgba[..8],
        [0x11, 0x22, 0x33, 255, 0x44, 0x55, 0x66, 255],
        "a 16-bit picture was not stripped to 8-bit samples"
    );
    assert_eq!(
        &got.rgba[20..24],
        [0x00, 0x01, 0x02, 255],
        "a 16-bit sample pair was not read as its high byte"
    );
    assert_eq!(
        &got.rgba[28..32],
        [0x06, 0x07, 0x08, 255],
        "the last pixel's samples are not its high bytes"
    );
}

#[test]
fn a_header_that_claims_an_impossible_size_is_refused_before_any_buffer() {
    // A real 1x1 PNG whose IHDR has been rewritten to claim 40000x40000: trusting the header
    // first would ask for 6.4 GB here. The refusal is the size check, which runs before any
    // output buffer exists, and the message says what the picture claimed.
    let lying = header_that_lies(
        40_000,
        40_000,
        &png_with(
            1,
            1,
            png::ColorType::Rgba,
            png::BitDepth::Eight,
            &[0, 0, 0, 0xFF],
        ),
    );
    let err = decode_png_bytes(&lying, G.size()).unwrap_err();
    assert!(err.contains("40000x40000"), "{err}");
    assert!(err.contains("640x480"), "{err}");

    // A picture that is simply another panel's is refused the same way rather than stretched.
    let wrong = panel_png(Geometry::W720H480, 1);
    let err = decode_png_bytes(&wrong, G.size()).unwrap_err();
    assert!(err.contains("720x480"), "{err}");
    assert!(err.contains("640x480"), "{err}");
    assert!(
        decode_png_bytes(&wrong, Geometry::W720H480.size()).is_ok(),
        "the same picture is fine on its own panel"
    );

    // And where the geometry is the picture's, a header that lies the other way — claiming the
    // panel but holding one pixel — is a decode failure rather than a stretch.
    let thin = header_that_lies(
        G.size().0,
        G.size().1,
        &png_with(
            1,
            1,
            png::ColorType::Rgba,
            png::BitDepth::Eight,
            &[0, 0, 0, 0xFF],
        ),
    );
    assert!(decode_png_bytes(&thin, G.size()).is_err());
}

#[test]
fn malformed_data_is_an_error_string_rather_than_a_panic() {
    let good = panel_png(G, 1);
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", Vec::new()),
        ("signature only", good[..8].to_vec()),
        ("truncated in the data", good[..good.len() / 2].to_vec()),
        ("not a png at all", b"this is not a picture".to_vec()),
        ("header cut short", good[..20].to_vec()),
    ];
    for (what, bytes) in cases {
        match decode_png_bytes(&bytes, G.size()) {
            Err(e) => assert!(!e.is_empty(), "{what}: an error with no words"),
            Ok(_) => panic!("{what}: decoded anyway"),
        }
    }
}

#[test]
fn a_file_and_a_memory_slice_go_through_the_same_decoder() {
    let root = scratch("decode-file");
    fs::create_dir_all(&root).unwrap();
    let bytes = panel_png(G, 0xC1);
    let path = root.join("picture.png");
    fs::write(&path, &bytes).unwrap();

    assert_eq!(
        decode_png_file(&path, G.size()).unwrap(),
        decode_png_bytes(&bytes, G.size()).unwrap(),
        "the same picture decoded two ways"
    );

    // A path that is not there fails like any other bad picture, and the message does not
    // repeat the card's layout back at whoever reads the log.
    let missing = root.join("Overlays").join("GBA").join("640x480.png");
    let err = decode_png_file(&missing, G.size()).unwrap_err();
    assert!(!err.contains("640x480.png"), "{err}");
    assert!(!err.contains(&root.to_string_lossy().to_string()), "{err}");
}

/// A valid 1x1 PNG rebuilt with an IHDR that claims `w`x`h`: the shape of a file that would
/// cost an impossible allocation if its header were believed before it was checked.
fn header_that_lies(w: u32, h: u32, small: &[u8]) -> Vec<u8> {
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA, no interlace

    let mut out = small[..8].to_vec(); // the signature
    out.extend_from_slice(&chunk(b"IHDR", &ihdr));
    out.extend_from_slice(&small[8 + 25..]); // the chunks after IHDR, unchanged
    out
}

fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_input = kind.to_vec();
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    out
}

/// The PNG CRC-32, written out because the png crate keeps its own to itself.
fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, entry) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *entry = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for b in bytes {
        crc = table[((crc ^ *b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

// ------------------------------------------------------------------ drawing

#[test]
fn the_cards_own_picture_beats_the_built_in_and_its_pixels_are_uploaded() {
    let (card, _root) = card("card-wins");
    let card_pixels = panel_pixels(G.size().0, G.size().1, 0xC1);
    put_on_card(&card, Platform::Gba, G, &panel_png(G, 0xC1));
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];

    let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));
    let canvas = draw(&mut layer, G.size());

    assert_eq!(canvas.uploads.len(), 1, "the picture was not uploaded once");
    let (_, w, h, bytes) = &canvas.uploads[0];
    assert_eq!((*w, *h), G.size());
    assert_eq!(bytes, &card_pixels, "the built-in's pixels were uploaded");
    assert_eq!(layer.used(), Some(OverlaySourceKind::Card));
    assert_eq!(layer.error(), None);
}

#[test]
fn with_no_card_picture_the_built_in_is_uploaded() {
    let (card, _root) = card("built-in");
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];
    let want = panel_pixels(G.size().0, G.size().1, 0xB1);

    let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));
    let canvas = draw(&mut layer, G.size());
    assert_eq!(canvas.uploads.len(), 1);
    assert_eq!(canvas.uploads[0].3, want);
    assert_eq!(layer.used(), Some(OverlaySourceKind::BuiltIn));
    assert_eq!(layer.error(), None);
}

#[test]
fn a_corrupt_or_wrong_size_card_picture_falls_back_to_the_built_in_and_says_so() {
    let (card, _root) = card("fallback");
    let want = panel_pixels(G.size().0, G.size().1, 0xB1);
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];
    let path = card_overlay_path(&card, Platform::Gba, G);

    // Two ways for a card picture to be there and useless: cut in half, and the wrong panel.
    let truncated = panel_png(G, 0xC1);
    let wrong_panel = panel_png(Geometry::W720H480, 0xC2);
    let cases: [(&str, Vec<u8>); 3] = [
        ("truncated", truncated[..truncated.len() / 2].to_vec()),
        ("wrong panel", wrong_panel),
        ("not a picture", b"overlay".to_vec()),
    ];

    for (what, bytes) in cases {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &bytes).unwrap();

        let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));
        let canvas = draw(&mut layer, G.size());
        assert_eq!(canvas.uploads.len(), 1, "{what}: the built-in was not used");
        assert_eq!(
            canvas.uploads[0].3, want,
            "{what}: another picture was used"
        );
        assert_eq!(
            layer.used(),
            Some(OverlaySourceKind::BuiltIn),
            "{what}: the wrong source was reported"
        );
        let trouble = layer
            .error()
            .unwrap_or_else(|| panic!("{what}: the card's failure was dropped"))
            .to_string();
        assert!(trouble.contains("card's own picture"), "{what}: {trouble}");
        // A log line is not a place for the card's layout or a copy of the file.
        assert!(
            !trouble.contains(&path.to_string_lossy().to_string()),
            "{trouble}"
        );
        assert!(!trouble.contains("640x480.png"), "{trouble}");
    }
}

#[test]
fn nothing_valid_anywhere_draws_nothing_and_keeps_both_failures() {
    let (card, _root) = card("all-broken");
    put_on_card(&card, Platform::Gba, G, b"not a png");
    let table = [built_in(
        Platform::Gba,
        G,
        "sample",
        b"not a png either".to_vec(),
    )];

    let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));
    let canvas = draw(&mut layer, G.size());
    assert!(canvas.uploads.is_empty(), "something was uploaded");
    assert!(canvas.images().is_empty(), "something was drawn");
    assert_eq!(layer.used(), None);
    assert_eq!(layer.texture(), None);
    let trouble = layer
        .error()
        .expect("both failures were dropped")
        .to_string();
    assert!(trouble.contains("card's own picture"), "{trouble}");
    assert!(trouble.contains("built-in picture sample"), "{trouble}");

    // And the second frame does not try again, or the card would be read sixty times a second.
    let again = draw(&mut layer, G.size());
    assert!(again.uploads.is_empty());
    assert!(again.images().is_empty());
    assert_eq!(layer.error().map(str::to_string), Some(trouble));
}

#[test]
fn a_failed_source_is_not_retried_until_the_sources_change() {
    let (card, _root) = card("no-retry");
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];
    let path = put_on_card(&card, Platform::Gba, G, b"not a png");
    let good = panel_pixels(G.size().0, G.size().1, 0xC1);

    let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));
    let mut canvas = PixelCanvas::new(G.size());
    layer.draw(&mut canvas);
    assert_eq!(canvas.uploads.len(), 1, "the built-in was not used");
    let trouble = layer
        .error()
        .expect("the card failure was dropped")
        .to_string();

    // The card's picture is fixed, but the layer was not asked to look again.
    fs::write(&path, panel_png(G, 0xC1)).unwrap();
    layer.draw(&mut canvas);
    layer.draw(&mut canvas);
    assert_eq!(
        canvas.uploads.len(),
        1,
        "a failed source was retried per frame"
    );
    assert_eq!(layer.used(), Some(OverlaySourceKind::BuiltIn));
    assert_eq!(layer.error().map(str::to_string), Some(trouble));

    // Setting the same value again is not a change either.
    layer.set_sources(resolve_with(&table, &card, Platform::Gba, G));
    layer.draw(&mut canvas);
    assert_eq!(canvas.uploads.len(), 1);

    // An empty set and back is a new attempt, and now the card's picture wins.
    layer.set_sources(empty_sources(Platform::Gba, G));
    layer.draw(&mut canvas);
    assert_eq!(canvas.uploads.len(), 1);
    assert_eq!(layer.used(), None);
    assert_eq!(layer.error(), None);

    layer.set_sources(resolve_with(&table, &card, Platform::Gba, G));
    layer.draw(&mut canvas);
    assert_eq!(canvas.uploads.len(), 2, "the new attempt did not happen");
    assert_eq!(canvas.uploads[1].3, good);
    assert_eq!(layer.used(), Some(OverlaySourceKind::Card));
    assert_eq!(layer.error(), None);
}

#[test]
fn a_successful_draw_is_one_plain_image_over_the_whole_panel() {
    let (card, _root) = card("draw");
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];
    let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));
    let canvas = draw(&mut layer, G.size());

    let (w, h) = G.size();
    assert_eq!(canvas.uploads.len(), 1);
    assert_eq!(canvas.images().len(), 1, "{:?}", canvas.inner.ops);
    match canvas.images()[0] {
        Op::Image {
            tex,
            x,
            y,
            w: iw,
            h: ih,
            uv,
            tint,
        } => {
            assert_eq!(*tex, canvas.uploads[0].0);
            assert_eq!((*x, *y, *iw, *ih), (0.0, 0.0, w as f32, h as f32));
            assert_eq!(*uv, [0.0, 0.0, 1.0, 1.0]);
            assert_eq!(*tint, Color::WHITE);
        }
        other => panic!("the overlay was not a plain image: {other:?}"),
    }
    // Nothing else: no clear (the game's frame is underneath), no rect, no effect, no update.
    assert!(
        !canvas.inner.ops.iter().any(|o| matches!(
            o,
            Op::Clear(_) | Op::Rect { .. } | Op::ImageEffect { .. } | Op::UpdateRgba8 { .. }
        )),
        "{:?}",
        canvas.inner.ops
    );
    assert_eq!(layer.texture(), Some(canvas.uploads[0].0));
}

#[test]
fn the_same_sources_draw_one_image_per_frame_and_upload_once() {
    let (card, _root) = card("cache");
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];
    let sources = resolve_with(&table, &card, Platform::Gba, G);
    let mut layer = layer_for(sources.clone());
    let mut canvas = PixelCanvas::new(G.size());

    for _ in 0..3 {
        layer.draw(&mut canvas);
    }
    assert_eq!(canvas.uploads.len(), 1, "the picture was uploaded again");
    assert_eq!(canvas.images().len(), 3, "a frame drew no overlay");
    assert!(canvas.inner.ops.iter().all(|o| !matches!(o, Op::Free(_))));

    // The same value set again is not a change: no free, no decode, no upload.
    layer.set_sources(sources);
    layer.draw(&mut canvas);
    assert_eq!(canvas.uploads.len(), 1);
    assert_eq!(canvas.images().len(), 4);
    assert!(canvas.inner.ops.iter().all(|o| !matches!(o, Op::Free(_))));
}

#[test]
fn changing_the_sources_frees_the_old_texture_exactly_once() {
    let (card, _root) = card("free");
    let table = [
        built_in(Platform::Gba, G, "first", panel_png(G, 0xB1)),
        built_in(Platform::Gba, G, "second", panel_png(G, 0xB2)),
    ];
    let mut layer = layer_for(resolve_with(&table[..1], &card, Platform::Gba, G));
    let mut canvas = PixelCanvas::new(G.size());
    layer.draw(&mut canvas);
    let first = layer.texture().expect("a texture");
    assert_eq!(canvas.uploads.len(), 1);

    // A different set: the old texture goes — but only when there is a canvas to free it on.
    layer.set_sources(resolve_with(&table[1..], &card, Platform::Gba, G));
    assert!(canvas.inner.ops.iter().all(|o| !matches!(o, Op::Free(_))));
    layer.draw(&mut canvas);
    assert_eq!(canvas.frees(first), 1, "the old texture was not freed once");
    assert_eq!(canvas.uploads.len(), 2);
    let second = layer.texture().expect("a texture");
    assert_ne!(second, first, "the same texture served a new picture");

    // And to nothing at all: the texture goes with it, and no new work happens.
    layer.set_sources(empty_sources(Platform::Gba, G));
    layer.draw(&mut canvas);
    assert_eq!(canvas.frees(second), 1, "the texture did not go");
    assert_eq!(canvas.uploads.len(), 2);
    assert_eq!(layer.texture(), None);
    assert_eq!(layer.used(), None);
    assert_eq!(layer.error(), None);

    // One more frame: nothing left to free, nothing drawn.
    let before = canvas.inner.ops.len();
    layer.draw(&mut canvas);
    assert_eq!(canvas.inner.ops.len(), before);

    // `clear_sources` is the same thing as an empty set, and a texture outlives neither.
    layer.set_sources(resolve_with(&table[..1], &card, Platform::Gba, G));
    layer.draw(&mut canvas);
    let third = layer.texture().expect("a texture");
    layer.clear_sources();
    layer.draw(&mut canvas);
    assert_eq!(canvas.frees(third), 1);
    assert_eq!(layer.texture(), None);
}

#[test]
fn release_frees_once_and_leaves_nothing_behind() {
    let (card, _root) = card("release");
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];
    let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));
    let mut canvas = PixelCanvas::new(G.size());
    layer.draw(&mut canvas);
    let tex = layer.texture().expect("a texture");

    layer.release(&mut canvas);
    assert_eq!(canvas.frees(tex), 1);
    assert_eq!(layer.texture(), None);
    assert_eq!(layer.used(), None);
    assert_eq!(layer.error(), None);

    // Releasing again is not a second free, and afterwards there is nothing to draw.
    layer.release(&mut canvas);
    assert_eq!(canvas.frees(tex), 1);
    let before = canvas.inner.ops.len();
    layer.draw(&mut canvas);
    assert_eq!(canvas.inner.ops.len(), before);
    assert_eq!(layer.texture(), None);

    // A source set after a release is a new picture, not a remembered one.
    layer.set_sources(resolve_with(&table, &card, Platform::Gba, G));
    layer.draw(&mut canvas);
    assert_eq!(canvas.uploads.len(), 2);
    assert!(layer.texture().is_some());
}

#[test]
fn a_canvas_of_the_wrong_size_uploads_and_draws_nothing_but_says_so() {
    let (card, _root) = card("wrong-canvas");
    let table = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];
    let mut layer = layer_for(resolve_with(&table, &card, Platform::Gba, G));

    let mut wrong = PixelCanvas::new((320, 240));
    layer.draw(&mut wrong);
    assert!(
        wrong.uploads.is_empty(),
        "a wrong-sized canvas got a texture"
    );
    assert!(
        wrong.inner.ops.is_empty(),
        "a wrong-sized canvas got a draw"
    );
    assert_eq!(layer.texture(), None);
    let trouble = layer
        .error()
        .expect("the mismatch said nothing")
        .to_string();
    assert!(
        trouble.contains("640x480") && trouble.contains("320x240"),
        "{trouble}"
    );

    // The panel's own canvas still works: the attempt was not spent on the wrong one, and an
    // error belongs to the moment that made it.
    let canvas = draw(&mut layer, G.size());
    assert_eq!(canvas.uploads.len(), 1);
    assert_eq!(canvas.images().len(), 1);
    assert_eq!(layer.error(), None);
    assert_eq!(layer.used(), Some(OverlaySourceKind::BuiltIn));

    // And the mismatch again, with a texture already on screen: the error comes back, and the
    // texture is left where it is rather than drawn on a canvas that is not its panel.
    let mut wrong = PixelCanvas::new((720, 720));
    layer.draw(&mut wrong);
    assert!(wrong.inner.ops.is_empty());
    assert!(layer.error().is_some());
}

#[test]
fn a_picture_that_is_not_a_file_is_never_opened() {
    // The resolver asks the filesystem whether the path is a regular file and nothing more:
    // a directory and a missing path are the same "no card source", so neither can make the
    // layer open or decode anything.
    let (card, _root) = card("no-open");
    let path = card_overlay_path(&card, Platform::Gba, G);
    fs::create_dir_all(&path).unwrap();
    let built = [built_in(Platform::Gba, G, "sample", panel_png(G, 0xB1))];

    let sources = resolve_with(&built, &card, Platform::Gba, G);
    assert_eq!(sources.card, None);
    assert!(!sources.is_empty());

    let mut layer = layer_for(sources);
    let canvas = draw(&mut layer, G.size());
    assert_eq!(canvas.uploads.len(), 1);
    assert_eq!(layer.used(), Some(OverlaySourceKind::BuiltIn));
    assert_eq!(layer.error(), None);

    // The directory is still a directory: nothing here removed or replaced it.
    assert!(path.is_dir());
}

// ------------------------------------------------------------------ the built-in picture

/// The Game Boy's own frame. The aperture the picture leaves is the integer placement of it.
const GB_NATIVE: (u32, u32) = (160, 144);

/// The pair and the id the production table is expected to register.
const GB_CUBEXX_ID: &str = "gb-cubexx-frame-v1";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One pixel of a decoded picture, as straight RGBA8.
fn pixel(image: &OverlayImage, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * image.w + x) * 4) as usize;
    [
        image.rgba[i],
        image.rgba[i + 1],
        image.rgba[i + 2],
        image.rgba[i + 3],
    ]
}

/// The built-in picture, decoded the way the layer decodes it.
fn built_in_image() -> OverlayImage {
    let entry = BUILT_IN_OVERLAYS
        .first()
        .expect("the build ships no overlay at all");
    let geometry = Geometry::W720H720;
    decode_png_bytes(entry.png, geometry.size()).expect("the built-in picture did not decode")
}

#[test]
fn the_production_table_registers_one_pair_under_one_id() {
    assert_eq!(BUILT_IN_OVERLAYS.len(), 1, "another picture was registered");
    let entry = &BUILT_IN_OVERLAYS[0];
    assert_eq!(entry.platform, Platform::Gb);
    assert_eq!(entry.geometry, Geometry::W720H720);
    assert_eq!(entry.id, GB_CUBEXX_ID);
    assert!(!entry.png.is_empty(), "the picture compiled in is empty");

    // One pair, one id: a repeat would be a picture nobody can reach, or two names for one.
    for (i, a) in BUILT_IN_OVERLAYS.iter().enumerate() {
        for b in &BUILT_IN_OVERLAYS[i + 1..] {
            assert!(
                a.platform != b.platform || a.geometry != b.geometry,
                "{:?}/{:?} is registered twice",
                a.platform,
                a.geometry
            );
            assert_ne!(a.id, b.id, "two pictures share the id {}", a.id);
        }
    }
}

#[test]
fn the_transparent_rectangle_is_exactly_where_integer_scaling_puts_a_game_boy_frame() {
    // The one placement the picture is cut for: the default `ScalePolicy::Integer` on 720x720,
    // asked of the same function the session draws the game through rather than worked out
    // again here. GB is square-pixel, so the aspect argument is (1, 1) and Integer ignores it.
    let want = slot2_gfx::place(
        slot2_gfx::ScalePolicy::Integer,
        GB_NATIVE,
        (1, 1),
        Geometry::W720H720.size(),
    );
    assert_eq!((want.x, want.y, want.w, want.h), (40, 72, 640, 576));

    let image = built_in_image();
    assert_eq!((image.w, image.h), (720, 720));
    assert_eq!(image.rgba.len(), 720 * 720 * 4, "not a whole RGBA8 panel");

    // Every fully transparent pixel, and where they reach: the game's rectangle and nothing
    // else in the picture may be see-through.
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    let mut opaque_outside = 0usize;
    for y in 0..image.h {
        for x in 0..image.w {
            let px = pixel(&image, x, y);
            if px == [0, 0, 0, 0] {
                bounds = Some(match bounds {
                    None => (x, y, x, y),
                    Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                });
            } else if px[3] != 255 {
                opaque_outside += 1;
            }
        }
    }
    let (x0, y0, x1, y1) = bounds.expect("nothing in the picture is transparent");
    assert_eq!(
        (x0, y0, x1 + 1 - x0, y1 + 1 - y0),
        (40, 72, 640, 576),
        "the see-through rectangle is not the game's"
    );
    assert_eq!(
        opaque_outside, 0,
        "a pixel that is neither the game nor an opaque case"
    );

    // And the game's rectangle itself carries no colour at all: an opaque or tinted one would
    // sit on top of the game it is supposed to frame.
    for y in want.y as u32..(want.y + want.h) as u32 {
        for x in want.x as u32..(want.x + want.w) as u32 {
            assert_eq!(pixel(&image, x, y), [0, 0, 0, 0], "{x},{y} is not clear");
        }
    }
}

#[test]
fn the_case_around_the_aperture_is_opaque_and_uses_more_than_one_colour() {
    let image = built_in_image();
    let want = slot2_gfx::place(
        slot2_gfx::ScalePolicy::Integer,
        GB_NATIVE,
        (1, 1),
        Geometry::W720H720.size(),
    );
    let inside = |x: u32, y: u32| {
        x >= want.x as u32
            && x < (want.x + want.w) as u32
            && y >= want.y as u32
            && y < (want.y + want.h) as u32
    };

    let mut colours: Vec<[u8; 4]> = Vec::new();
    for y in 0..image.h {
        for x in 0..image.w {
            if inside(x, y) {
                continue;
            }
            let px = pixel(&image, x, y);
            assert_eq!(px[3], 255, "{x},{y} in the case is see-through");
            if !colours.contains(&px) {
                colours.push(px);
            }
        }
    }
    colours.sort_unstable();
    assert!(
        colours.len() >= 3,
        "the case is one flat colour: {colours:?}"
    );

    // The rim is a colour of its own: the first pixel on each side of the aperture is not the
    // base the far corner is made of.
    let corner = pixel(&image, 0, 0);
    for (x, y) in [
        (40 - 1, 72 + 100),
        (680, 72 + 100),
        (40 + 100, 72 - 1),
        (40 + 100, 648),
    ] {
        let rim = pixel(&image, x, y);
        assert_ne!(rim, corner, "{x},{y}: the rim is the base colour");
        assert_eq!(rim[3], 255);
    }
}

#[test]
fn only_the_registered_pair_resolves_to_a_built_in() {
    let (card, _root) = card("production-pairs");
    let geometries = [Geometry::W640H480, Geometry::W720H480, Geometry::W720H720];
    let mut registered = 0;
    let mut absent = 0;

    for platform in Platform::ALL {
        for geometry in geometries {
            let sources = resolve(&card, platform, geometry);
            assert_eq!(sources.card, None, "{platform:?}/{geometry:?}");
            if platform == Platform::Gb && geometry == Geometry::W720H720 {
                assert_eq!(sources.built_in.map(|b| b.id), Some(GB_CUBEXX_ID));
                assert!(!sources.is_empty());
                registered += 1;
            } else {
                assert_eq!(
                    sources.built_in, None,
                    "{platform:?}/{geometry:?} has a picture"
                );
                assert!(sources.is_empty(), "{platform:?}/{geometry:?}");
                absent += 1;
            }
        }
    }
    assert_eq!(registered, 1);
    assert_eq!(absent, 20, "the other pairs are not all empty");
    assert_eq!(Platform::ALL.len() * 3, registered + absent);
}

#[test]
fn the_cards_own_picture_still_wins_and_a_broken_one_falls_back_to_the_built_in() {
    let geometry = Geometry::W720H720;
    let (w, h) = geometry.size();
    let want = built_in_image().rgba;
    let (card, _root) = card("production-precedence");

    // A valid picture for the pair beats the built-in, whether or not one is registered.
    let card_pixels = panel_pixels(w, h, 0xC1);
    put_on_card(&card, Platform::Gb, geometry, &panel_png(geometry, 0xC1));
    let mut layer = layer_for(resolve(&card, Platform::Gb, geometry));
    let canvas = draw(&mut layer, geometry.size());
    assert_eq!(canvas.uploads.len(), 1);
    assert_eq!(canvas.uploads[0].3, card_pixels, "the built-in was used");
    assert_eq!(layer.used(), Some(OverlaySourceKind::Card));
    assert_eq!(layer.error(), None);

    // Broken or the wrong panel: the production picture is what the layer falls back to, and
    // the card's failure is kept beside it.
    let path = card_overlay_path(&card, Platform::Gb, geometry);
    let truncated = panel_png(geometry, 0xC1);
    let cases: [(&str, Vec<u8>); 3] = [
        ("truncated", truncated[..truncated.len() / 2].to_vec()),
        ("wrong panel", panel_png(Geometry::W720H480, 0xC2)),
        ("not a picture", b"overlay".to_vec()),
    ];
    for (what, bytes) in cases {
        fs::write(&path, &bytes).unwrap();
        let mut layer = layer_for(resolve(&card, Platform::Gb, geometry));
        let canvas = draw(&mut layer, geometry.size());
        assert_eq!(canvas.uploads.len(), 1, "{what}: nothing was drawn");
        assert_eq!(
            canvas.uploads[0].3, want,
            "{what}: another picture was used"
        );
        assert_eq!(layer.used(), Some(OverlaySourceKind::BuiltIn), "{what}");
        let trouble = layer
            .error()
            .unwrap_or_else(|| panic!("{what}: the card's failure was dropped"))
            .to_string();
        assert!(trouble.contains("card's own picture"), "{what}: {trouble}");
    }
}

#[test]
fn a_layer_with_only_the_built_in_uploads_once_and_draws_every_frame() {
    let geometry = Geometry::W720H720;
    let (w, h) = geometry.size();
    let (card, _root) = card("production-layer");
    let sources = resolve(&card, Platform::Gb, geometry);
    assert_eq!(sources.card, None);
    assert_eq!(sources.built_in.map(|b| b.id), Some(GB_CUBEXX_ID));

    let mut layer = layer_for(sources);
    let mut canvas = PixelCanvas::new((w, h));
    for _ in 0..3 {
        layer.draw(&mut canvas);
    }
    assert_eq!(canvas.uploads.len(), 1, "the picture was uploaded again");
    assert_eq!(canvas.images().len(), 3, "a frame drew no overlay");
    assert!(canvas.inner.ops.iter().all(|o| !matches!(o, Op::Free(_))));

    let (tex, uw, uh, bytes) = &canvas.uploads[0];
    assert_eq!((*uw, *uh), (w, h), "not a panel-sized texture");
    assert_eq!(
        bytes,
        &built_in_image().rgba,
        "the bytes are not the picture"
    );
    match canvas.images()[0] {
        Op::Image {
            tex: itex,
            x,
            y,
            w: iw,
            h: ih,
            uv,
            tint,
        } => {
            assert_eq!(*itex, *tex);
            assert_eq!((*x, *y, *iw, *ih), (0.0, 0.0, w as f32, h as f32));
            assert_eq!(*uv, [0.0, 0.0, 1.0, 1.0]);
            assert_eq!(*tint, Color::WHITE);
        }
        other => panic!("the overlay was not a plain image: {other:?}"),
    }
}

#[test]
fn the_sample_does_not_turn_the_platform_default_on() {
    // A registered picture is not a default: a game that never asked for one still gets none.
    assert!(!overlay_enabled(None), "the sample turned the default on");
    assert!(!overlay_enabled(Some(false)));
    assert!(overlay_enabled(Some(true)));
}

/// The interpreter to run the generator with, if either usual name is on PATH.
fn python() -> Option<&'static str> {
    for name in ["python", "python3"] {
        let ok = std::process::Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success());
        if ok {
            return Some(name);
        }
    }
    None
}

#[test]
fn the_generator_writes_the_same_bytes_every_time_and_the_picture_does_not_move() {
    // The picture is a generated file, so the check that it is still the generator's is the
    // generator: run it twice, and neither run may change a byte of what is committed.
    let Some(python) = python() else {
        eprintln!("no python on PATH — skipping the generator determinism check");
        return;
    };
    let root = repo();
    let script = root.join("build").join("generate-overlays.py");
    let picture = root
        .join("assets")
        .join("overlays")
        .join("GB")
        .join("720x720.png");
    assert!(script.is_file(), "the generator is missing");
    assert!(picture.is_file(), "the picture is missing");

    let before = fs::read(&picture).unwrap();
    let digest = sha256(&before);
    assert_eq!(
        digest, "f370d0270ad2cc1c503d12abeef5bd67ee0711b68864985468b502dafed0dfe2",
        "the committed picture is not the one this source generates"
    );

    for run in 1..=2 {
        let out = std::process::Command::new(python)
            .arg(&script)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|e| panic!("run {run}: the generator did not start: {e}"));
        assert!(
            out.status.success(),
            "run {run}: the generator failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let after = fs::read(&picture).unwrap();
        assert_eq!(sha256(&after), digest, "run {run} wrote another picture");
        assert_eq!(after, before, "run {run} moved a byte of the picture");
    }
}

/// The SHA-256 of a byte slice, as lowercase hex.
///
/// Written out rather than pulled in: nothing in this tree already has one, and the check is
/// about a digest a person can compare with `Get-FileHash` rather than about two vectors of
/// two megabytes compared once.
fn sha256(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut message = bytes.to_vec();
    let bits = (bytes.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bits.to_be_bytes());

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    for block in message.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}
