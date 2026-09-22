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
    _todo: (),
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").finish_non_exhaustive()
    }
}

impl Session {
    pub fn start(
        card: &Card,
        cart: &Cart,
        core_dir: &Path,
        sink_rate: u32,
    ) -> Result<(Session, Consumer), Error> {
        let _ = (card, cart, core_dir, sink_rate);
        todo!("task 09")
    }

    pub fn cart(&self) -> &Cart {
        todo!("task 09")
    }

    /// Advance one frame with the buttons currently held.
    pub fn run_frame(&mut self, held: &[LogicalButton], volume: &Volume) {
        let _ = (held, volume);
        todo!("task 09")
    }

    /// Upload the newest picture if there is one. Call before `draw`.
    pub fn upload_video(&mut self, canvas: &mut dyn Canvas) {
        let _ = canvas;
        todo!("task 09")
    }

    /// Draw the game, integer-scaled and centred on the panel.
    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &UiCtx) {
        let _ = (canvas, ctx);
        todo!("task 09")
    }

    /// The last frame as RGBA8 with its size, for thumbnails and tests.
    pub fn last_frame(&self) -> Option<(u32, u32, &[u8])> {
        todo!("task 09")
    }

    /// Nearest-neighbour downscale of the last frame to fit `THUMB_MAX`, as
    /// `(w, h, rgba)`. `None` before the first frame.
    pub fn thumbnail(&self) -> Option<(u32, u32, Vec<u8>)> {
        todo!("task 09")
    }

    pub fn save_state(&mut self, card: &Card, kind: StateKind) -> Result<(), Error> {
        let _ = (card, kind);
        todo!("task 09")
    }

    pub fn load_state(&mut self, card: &Card, kind: StateKind) -> Result<(), Error> {
        let _ = (card, kind);
        todo!("task 09")
    }

    /// Flush save RAM now. Returns whether anything was written.
    pub fn flush_save(&mut self, card: &Card) -> Result<bool, Error> {
        let _ = card;
        todo!("task 09")
    }

    /// Flush the save, write the resume state, and let the core go. Logs its own errors.
    pub fn stop(self, card: &Card) {
        let _ = card;
        todo!("task 09")
    }

    /// How many frames have run since `start`.
    pub fn frames_run(&self) -> u64 {
        todo!("task 09")
    }
}

/// Unused imports guard: these types are part of the contract above.
#[allow(dead_code)]
fn _types(_: Option<TexId>, _: Option<Producer>, _: Option<Resampler>, _: Option<Core>) {}
