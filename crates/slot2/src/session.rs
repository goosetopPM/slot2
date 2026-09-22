//! One cart, running. Owns the core, the audio chain and the per-frame bookkeeping; the
//! screen state machine in `app.rs` drives it and the UI draws around it.
//!
//! Implementation notes for task 09 (see tasks/09-play.md):
//!
//! ## Starting
//! `Session::start(card, cart, core_dir, sink_rate)`:
//! 1. `registry::def(platform)` for the core and its option presets.
//! 2. Find the core file: `core_dir.join(CoreId::file_name())`; missing → `Error::NoCore`.
//! 3. `Env { system_dir: card.bios_dir(), save_dir: card.root().join("Saves"), options, language: 0 }`.
//! 4. `Core::load(dylib, &cart.rom, env)`.
//! 5. Restore save RAM: `card.read_save(cart)` → `core.write_memory(Memory::SaveRam, …)`,
//!    ignoring a size mismatch (a save from another core layout is not fatal; log it).
//! 6. `Resampler::new(core.av_info().sample_rate as u32, sink_rate)`, a `Ring` of
//!    `sink_rate / 4` frames (250 ms), split; the `Producer` stays here and the caller takes
//!    the `Consumer` for its sink (`start` returns `(Session, Consumer)`).
//!
//! ## Per frame (`run_frame(held, volume)`)
//! `core.set_input(0, registry::mask_for(platform, held))` → `core.run()` →
//! `core.take_audio()` → resample into a scratch `Vec<i16>` → `volume.apply` →
//! `producer.write` (a full ring drops, never blocks) → if the core produced a new frame,
//! convert it with `Frame::to_rgba8` into a reusable buffer and mark `video_dirty`.
//! Track `frames_run` and, every `SAVE_EVERY_FRAMES` (600, ten seconds), flush save RAM:
//! `core.memory(Memory::SaveRam)` → `card.write_save` (which skips identical bytes).
//!
//! ## Drawing
//! `upload_video(canvas)` uploads or replaces the game texture when `video_dirty`, then
//! `draw(canvas, ctx)` draws it centred on the panel at the largest integer multiple of the
//! core's `base_width/base_height` that fits (`scale = min(panel_w / w, panel_h / h)`,
//! at least 1), with `Color::BLACK` behind. Geometry changes mid-game (`av_info()` changing)
//! must be picked up: read it each frame, and re-upload when the size changed.
//!
//! ## Stopping
//! `stop(self)`: flush save RAM one last time, write a resume state
//! (`core.serialize()` → `card.write_state(cart, StateKind::Resume, …, thumb)`) where the
//! thumbnail is the last frame scaled to fit inside 160x160 by nearest-neighbour
//! (`thumbnail()`), then drop the core. Errors are logged, not returned: losing a resume
//! state must not stop the player getting back to the shelf.
//! `save_state(kind)` / `load_state(kind)` do the same for the numbered slots.

use std::path::{Path, PathBuf};

use slot2_audio::{Consumer, Producer, Resampler, Volume};
use slot2_gfx::{Canvas, TexId};
use slot2_retro::{Core, LogicalButton};
use slot2_store::{Card, Cart, StateKind};
use slot2_ui::UiCtx;

/// Save RAM is flushed this often while playing.
pub const SAVE_EVERY_FRAMES: u64 = 600;

/// The resume thumbnail fits inside this box.
pub const THUMB_MAX: u32 = 160;

#[derive(Debug)]
pub enum Error {
    /// No core file for this platform in the core directory.
    NoCore(PathBuf),
    Retro(slot2_retro::Error),
    Store(slot2_store::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoCore(p) => write!(f, "no core at {}", p.display()),
            Error::Retro(e) => write!(f, "{e}"),
            Error::Store(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<slot2_retro::Error> for Error {
    fn from(e: slot2_retro::Error) -> Self {
        Error::Retro(e)
    }
}
impl From<slot2_store::Error> for Error {
    fn from(e: slot2_store::Error) -> Self {
        Error::Store(e)
    }
}

pub struct Session {
    cart: Cart,
    core: Core,
    producer: Producer,
    resampler: Resampler,
    frames_run: u64,
    video_dirty: bool,
    video_buffer: Vec<u8>,
    tex_id: Option<TexId>,
    last_width: u32,
    last_height: u32,
    scratch_audio: Vec<i16>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("cart", &self.cart)
            .field("frames_run", &self.frames_run)
            .finish_non_exhaustive()
    }
}

impl Session {
    pub fn start(
        card: &Card,
        cart: &Cart,
        core_dir: &Path,
        sink_rate: u32,
    ) -> Result<(Session, Consumer), Error> {
        let retro_platform = match cart.platform {
            slot2_store::Platform::Gb => slot2_retro::Platform::Gb,
            slot2_store::Platform::Gbc => slot2_retro::Platform::Gbc,
            slot2_store::Platform::Gba => slot2_retro::Platform::Gba,
            slot2_store::Platform::Nes => slot2_retro::Platform::Nes,
            slot2_store::Platform::Snes => slot2_retro::Platform::Snes,
            slot2_store::Platform::Md => slot2_retro::Platform::Md,
            slot2_store::Platform::Sms => slot2_retro::Platform::Sms,
        };

        let def = slot2_retro::def(retro_platform);
        let dylib = core_dir.join(def.default_core.file_name());
        if !dylib.exists() {
            return Err(Error::NoCore(dylib));
        }

        let mut options: Vec<(String, String)> = def
            .options
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        // Skip BIOS if it doesn't exist
        if def.platform == slot2_retro::Platform::Gba
            && !card.bios_dir().join("gba_bios.bin").exists()
        {
            options.push(("mgba_skip_bios".to_string(), "ON".to_string()));
        }

        let env = slot2_retro::Env {
            system_dir: card.bios_dir(),
            save_dir: card.root().join("Saves").join(cart.platform.folder()),
            options,
            language: 0,
        };

        let mut core = Core::load(&dylib, &cart.rom, env)?;

        if let Some(save) = card.read_save(cart) {
            if let Err(e) = core.write_memory(slot2_retro::Memory::SaveRam, &save) {
                eprintln!("slot2: save RAM size mismatch: {e}");
            }
        }

        let av = core.av_info();
        let resampler = Resampler::new(av.sample_rate as u32, sink_rate);
        let (producer, consumer) = slot2_audio::Ring::new((sink_rate / 4) as usize).split();

        let session = Session {
            cart: cart.clone(),
            core,
            producer,
            resampler,
            frames_run: 0,
            video_dirty: false,
            video_buffer: Vec::new(),
            tex_id: None,
            last_width: 0,
            last_height: 0,
            scratch_audio: Vec::new(),
        };

        Ok((session, consumer))
    }

    pub fn cart(&self) -> &Cart {
        &self.cart
    }

    /// Advance one frame with the buttons currently held.
    pub fn run_frame(&mut self, held: &[LogicalButton], volume: &Volume) {
        let retro_platform = match self.cart.platform {
            slot2_store::Platform::Gb => slot2_retro::Platform::Gb,
            slot2_store::Platform::Gbc => slot2_retro::Platform::Gbc,
            slot2_store::Platform::Gba => slot2_retro::Platform::Gba,
            slot2_store::Platform::Nes => slot2_retro::Platform::Nes,
            slot2_store::Platform::Snes => slot2_retro::Platform::Snes,
            slot2_store::Platform::Md => slot2_retro::Platform::Md,
            slot2_store::Platform::Sms => slot2_retro::Platform::Sms,
        };

        self.core.set_input(
            0,
            slot2_retro::mask_for(retro_platform, held.iter().copied()),
        );
        self.core.run();

        let samples = self.core.take_audio();
        self.scratch_audio.clear();
        self.resampler.process(&samples, &mut self.scratch_audio);
        volume.apply(&mut self.scratch_audio);
        self.producer.write(&self.scratch_audio);

        if let Some(frame) = self.core.frame() {
            if frame.width != self.last_width || frame.height != self.last_height {
                self.last_width = frame.width;
                self.last_height = frame.height;
                self.video_dirty = true;
            }
            self.video_buffer = frame.to_rgba8();
            self.video_dirty = true;
        }

        self.frames_run += 1;
    }

    /// Upload the newest picture if there is one. Call before `draw`.
    pub fn upload_video(&mut self, canvas: &mut dyn Canvas) {
        if self.video_dirty {
            if let Some(tex) = self.tex_id {
                canvas.free(tex);
            }
            self.tex_id =
                Some(canvas.upload_rgba8(self.last_width, self.last_height, &self.video_buffer));
            self.video_dirty = false;
        }
    }

    /// Draw the game, integer-scaled and centred on the panel.
    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &UiCtx) {
        let _ = ctx;
        if let Some(tex) = self.tex_id {
            let (pw, ph) = canvas.size();
            let sw = self.last_width as f32;
            let sh = self.last_height as f32;

            let scale = (pw as f32 / sw).min(ph as f32 / sh).floor().max(1.0);
            let dw = sw * scale;
            let dh = sh * scale;
            let dx = (pw as f32 - dw) / 2.0;
            let dy = (ph as f32 - dh) / 2.0;

            canvas.clear(slot2_gfx::Color::BLACK);
            canvas.image(tex, dx, dy, dw, dh, slot2_gfx::Color::WHITE);
        }
    }

    /// The last frame as RGBA8 with its size, for thumbnails and tests.
    pub fn last_frame(&self) -> Option<(u32, u32, &[u8])> {
        if self.video_buffer.is_empty() {
            None
        } else {
            Some((self.last_width, self.last_height, &self.video_buffer))
        }
    }

    /// Nearest-neighbour downscale of the last frame to fit `THUMB_MAX`, as
    /// `(w, h, rgba)`. `None` before the first frame.
    pub fn thumbnail(&self) -> Option<(u32, u32, Vec<u8>)> {
        let (w, h, data) = self.last_frame()?;
        let scale = if w > h {
            THUMB_MAX as f32 / w as f32
        } else {
            THUMB_MAX as f32 / h as f32
        };
        let tw = (w as f32 * scale).floor() as u32;
        let th = (h as f32 * scale).floor() as u32;
        let mut tdata = Vec::with_capacity((tw * th * 4) as usize);
        for y in 0..th {
            for x in 0..tw {
                let sx = (x as f32 / scale).floor() as u32;
                let sy = (y as f32 / scale).floor() as u32;
                let sx = sx.min(w - 1);
                let sy = sy.min(h - 1);
                let i = ((sy * w + sx) * 4) as usize;
                tdata.extend_from_slice(&data[i..i + 4]);
            }
        }
        Some((tw, th, tdata))
    }

    pub fn save_state(&mut self, card: &Card, kind: StateKind) -> Result<(), Error> {
        let data = self.core.serialize()?;
        let thumb = self.thumbnail();
        let t = thumb.as_ref().map(|(w, h, rgba)| slot2_store::Thumb {
            width: *w,
            height: *h,
            rgba,
        });
        card.write_state(&self.cart, kind, &data, t)?;
        Ok(())
    }

    pub fn load_state(&mut self, card: &Card, kind: StateKind) -> Result<(), Error> {
        let data = card
            .read_state(&self.cart, kind)
            .ok_or_else(|| Error::Retro(slot2_retro::Error::State("no such state".into())))?;
        self.core.unserialize(&data)?;
        self.resampler.reset();
        Ok(())
    }

    /// Flush save RAM now. Returns whether anything was written.
    pub fn flush_save(&mut self, card: &Card) -> Result<bool, Error> {
        if let Some(save) = self.core.memory(slot2_retro::Memory::SaveRam) {
            Ok(card.write_save(&self.cart, &save)?)
        } else {
            Ok(false)
        }
    }

    /// Flush the save, write the resume state, and let the core go. Logs its own errors.
    pub fn stop(mut self, card: &Card) {
        if let Err(e) = self.flush_save(card) {
            eprintln!("slot2: error flushing save: {e}");
        }
        if let Ok(state) = self.core.serialize() {
            let thumb = self.thumbnail();
            let t = thumb.as_ref().map(|(w, h, rgba)| slot2_store::Thumb {
                width: *w,
                height: *h,
                rgba,
            });
            if let Err(e) = card.write_state(&self.cart, StateKind::Resume, &state, t) {
                eprintln!("slot2: error writing resume state: {e}");
            }
        }
    }

    /// How many frames have run since `start`.
    pub fn frames_run(&self) -> u64 {
        self.frames_run
    }
}

/// Unused imports guard: these types are part of the contract above.
#[allow(dead_code)]
fn _types(_: Option<TexId>, _: Option<Producer>, _: Option<Resampler>, _: Option<Core>) {}
