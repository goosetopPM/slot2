//! One cart, running. Owns the core, the audio chain and the per-frame bookkeeping; the
//! screen state machine in `app.rs` drives it and the UI draws around it.
//!
//! Implementation notes for task 09 (see tasks/09-play.md):
//!
//! ## Starting
//! `Session::start(card, cart, core_dir, sink_rate, tuning)`:
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
use std::time::Duration;

use slot2_audio::{Consumer, Producer, Resampler, Volume, CHANNELS};
use slot2_gfx::{Canvas, TexId};
use slot2_retro::{Core, LogicalButton};
use slot2_store::{Card, Cart, StateKind};
use slot2_ui::UiCtx;

/// Save RAM is flushed this often while playing.
pub const SAVE_EVERY_FRAMES: u64 = 600;

/// What to pace a frame at when a core reports nonsense for its fps.
const FALLBACK_FPS: f64 = 60.0;

/// A core's library file name from the base name a settings file gives, accepting either
/// spelling: `mgba` and `mgba_libretro` mean the same core to a person typing it.
fn core_file_name(base: &str) -> String {
    let base = base.trim().trim_end_matches(".so").trim_end_matches(".dll");
    let base = if base.ends_with("_libretro") {
        base.to_string()
    } else {
        format!("{base}_libretro")
    };
    let ext = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    format!("{base}.{ext}")
}

/// Turn a core's reported fps into a frame period, rejecting values no console has.
fn frame_time_for(fps: f64) -> Duration {
    let fps = if fps.is_finite() && (1.0..=1000.0).contains(&fps) {
        fps
    } else {
        FALLBACK_FPS
    };
    Duration::from_secs_f64(1.0 / fps)
}

/// The share of a frame rewind may spend taking states before it starts taking them less
/// often. Three percent is under the noise of everything else in a frame.
pub const REWIND_FRAME_BUDGET: f64 = 0.03;

/// However slow the machine, a capture at least this often — past a second of granularity
/// rewind stops being rewind.
pub const MAX_REWIND_INTERVAL: u32 = 60;

/// The fastest fast forward on offer. Past this the core is the bottleneck anyway and the
/// picture is unreadable.
pub const MAX_SPEED: u32 = 8;

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
    retro_platform: slot2_retro::Platform,
    rewind: slot2_retro::Rewind,
    rewind_on: bool,
    /// The worst capture seen so far, which is what the interval is budgeted against.
    rewind_cost: std::time::Duration,
    speed: u32,
    aspect: slot2_retro::Aspect,
    overscan: slot2_retro::Overscan,
    scale: slot2_gfx::ScalePolicy,
    producer: Producer,
    resampler: Resampler,
    frame_time: Duration,
    audio_frames: u64,
    audio_dropped: u64,
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
        tuning: slot2_retro::Tuning,
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
        let settings = card.read_settings(cart);

        // A named core that is not on the card falls back to the platform's own, with a
        // line in the log. A setting is allowed to be wrong; it is not allowed to make a
        // game unlaunchable, and the player who typed it will not see a panic.
        let dylib = match settings.core.as_deref() {
            Some(name) => {
                let named = core_dir.join(core_file_name(name));
                if named.exists() {
                    named
                } else {
                    eprintln!(
                        "slot2: {} asks for core {name}, which is not on the card; using {}",
                        cart.stem,
                        def.default_core.base_name()
                    );
                    core_dir.join(def.default_core.file_name())
                }
            }
            None => core_dir.join(def.default_core.file_name()),
        };
        if !dylib.exists() {
            return Err(Error::NoCore(dylib));
        }

        // A BIOS on the card is the player asking for the real boot sequence; none means
        // the core's own. Nothing else about the launch depends on it.
        let bios_present = def
            .bios
            .iter()
            .any(|name| card.bios_dir().join(name).exists());
        let options = slot2_retro::options_for(retro_platform, bios_present, tuning);

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
            retro_platform,
            rewind: slot2_retro::Rewind::new(def.rewind),
            rewind_on: settings.rewind.unwrap_or(true),
            rewind_cost: std::time::Duration::ZERO,
            speed: 1,
            aspect: def.aspect,
            overscan: if settings.overscan.unwrap_or(true) {
                def.overscan
            } else {
                slot2_retro::Overscan::NONE
            },
            scale: match settings.scale {
                Some(slot2_store::ScaleMode::Integer) => slot2_gfx::ScalePolicy::Integer,
                Some(slot2_store::ScaleMode::AspectFit) => slot2_gfx::ScalePolicy::AspectFit,
                Some(slot2_store::ScaleMode::Fill) => slot2_gfx::ScalePolicy::Fill,
                None => slot2_gfx::ScalePolicy::default(),
            },
            producer,
            resampler,
            frame_time: frame_time_for(av.fps),
            audio_frames: 0,
            audio_dropped: 0,
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

    /// How long one core frame should take, from the core's own fps.
    ///
    /// The loop must pace on this and not on a flat 1/60: a GBA runs at 59.7275 fps, so
    /// calling `run_frame` sixty times a second makes the core produce 0.46 % more audio
    /// than the codec can play. The ring fills within a second and every frame's tail is
    /// then thrown away — sixty splices a second, which is heard as a grainy buzz under
    /// the music rather than as distinct clicks.
    pub fn frame_time(&self) -> Duration {
        self.frame_time
    }

    /// `(frames produced, frames the ring had no room for, queued now, ring capacity)`.
    /// A healthy game drops nothing and sits near half.
    pub fn audio_health(&self) -> (u64, u64, usize, usize) {
        (
            self.audio_frames,
            self.audio_dropped,
            self.producer.queued_frames(),
            self.producer.capacity_frames(),
        )
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
        // Fast forward is extra core frames inside one displayed frame. Only the last one's
        // picture is shown and none of their audio is kept: a stream played at three times
        // the rate is three times the pitch, and the ring would overflow producing it.
        for _ in 1..self.speed.max(1) {
            self.core.run();
            let _ = self.core.take_audio();
            self.frames_run += 1;
        }
        self.core.run();

        // A core may retime itself mid-game (`SET_SYSTEM_AV_INFO`); reading this back is a
        // struct copy, so do it every frame rather than trusting what `start` saw.
        let av = self.core.av_info();
        self.frame_time = frame_time_for(av.fps);
        if av.sample_rate as u32 != self.resampler.rates().0 {
            self.resampler
                .set_rates(av.sample_rate as u32, self.resampler.rates().1);
        }

        // Steer the output rate on how full the ring is, before converting this frame.
        self.resampler.set_output_trim(slot2_audio::drc_trim(
            self.producer.queued_frames(),
            self.producer.capacity_frames(),
        ));

        let samples = self.core.take_audio();
        self.scratch_audio.clear();
        if self.speed <= 1 {
            self.resampler.process(&samples, &mut self.scratch_audio);
            volume.apply(&mut self.scratch_audio);
            let took = self.producer.write(&self.scratch_audio);
            self.audio_frames += (self.scratch_audio.len() / CHANNELS) as u64;
            self.audio_dropped += ((self.scratch_audio.len() - took) / CHANNELS) as u64;
        }

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

        if self.rewind_on && self.rewind.should_capture(self.frames_run) {
            let began = std::time::Instant::now();
            match self.core.serialize() {
                Ok(state) => self.rewind.push(state),
                Err(e) => {
                    eprintln!("slot2: rewind off, this core cannot serialize: {e}");
                    self.rewind_on = false;
                    self.rewind.clear();
                }
            }
            self.budget_rewind(began.elapsed());
        }
    }

    /// Keep the cost of rewinding under [`REWIND_FRAME_BUDGET`] by taking states less often.
    ///
    /// A capture is a `retro_serialize` and a delta against the last one, and both are
    /// mostly memory traffic — so what it costs depends on the machine and the console, not
    /// on anything that can be written down here. Measured on a Cortex-A53, the same core
    /// as the device: a NES state costs half a percent of a frame, a Mega Drive state
    /// twenty-two percent. Rather than pick a number per platform from one machine's
    /// timings, this measures what it actually costs where it actually runs and doubles the
    /// interval until it fits.
    ///
    /// The worst that can happen is coarser rewind on a slow box, which is what anyone
    /// would choose over a stutter every tenth of a second.
    fn budget_rewind(&mut self, took: std::time::Duration) {
        self.rewind_cost = self.rewind_cost.max(took);
        let interval = self.rewind.interval();
        if interval == 0 || interval >= MAX_REWIND_INTERVAL {
            return;
        }
        let per_frame = self.rewind_cost.div_f64(interval as f64);
        if per_frame <= self.frame_time.mul_f64(REWIND_FRAME_BUDGET) {
            return;
        }
        let wider = (interval * 2).min(MAX_REWIND_INTERVAL);
        self.rewind.set_interval(wider);
        eprintln!(
            "slot2: rewind capture takes {:.1} ms; taking one every {wider} frames instead of {interval}",
            self.rewind_cost.as_secs_f64() * 1000.0
        );
    }

    /// Step the game back one rewind capture. False when there is nothing behind it.
    ///
    /// The ring is spent as it is walked, so holding rewind runs out rather than looping,
    /// and playing forward from wherever it stopped fills it again.
    pub fn rewind_step(&mut self) -> bool {
        let Some(state) = self.rewind.pop() else {
            return false;
        };
        if let Err(e) = self.core.unserialize(state) {
            eprintln!("slot2: rewind failed: {e}");
            self.rewind.clear();
            return false;
        }
        // The picture has to follow the state back, or the screen keeps showing where the
        // player was until the next frame runs.
        if let Some(frame) = self.core.frame() {
            self.last_width = frame.width;
            self.last_height = frame.height;
            self.video_buffer = frame.to_rgba8();
            self.video_dirty = true;
        }
        true
    }

    /// How many captures are still behind the current moment, and what they cost.
    pub fn rewind_state(&self) -> (usize, usize) {
        (self.rewind.depth(), self.rewind.bytes())
    }

    /// `(frames between captures, the worst capture seen)`. The interval widens itself on a
    /// machine where captures are expensive; this is how the log says so.
    pub fn rewind_pace(&self) -> (u32, std::time::Duration) {
        (self.rewind.interval(), self.rewind_cost)
    }

    pub fn rewind_enabled(&self) -> bool {
        self.rewind_on
    }

    /// Turning rewind off frees the ring; turning it on starts a new one from here.
    pub fn set_rewind(&mut self, on: bool) {
        if self.rewind_on != on {
            self.rewind.clear();
        }
        self.rewind_on = on;
    }

    /// Core frames per displayed frame. 1 is normal speed; anything more is silent.
    pub fn speed(&self) -> u32 {
        self.speed
    }

    pub fn set_speed(&mut self, times: u32) {
        self.speed = times.clamp(1, MAX_SPEED);
    }

    /// Upload the newest picture if there is one. Call before `draw`.
    pub fn upload_video(&mut self, canvas: &mut dyn Canvas) {
        if !self.video_dirty {
            return;
        }
        let (w, h) = (self.last_width, self.last_height);
        // The usual case is the same frame size as last time, which is a rewrite of the
        // pixels rather than a new texture. Only a geometry change — a Mega Drive going
        // from 256 to 320 across, a SNES screen turning hi-res — reallocates.
        let rewritten = self
            .tex_id
            .is_some_and(|tex| canvas.update_rgba8(tex, w, h, &self.video_buffer));
        if !rewritten {
            if let Some(tex) = self.tex_id {
                canvas.free(tex);
            }
            self.tex_id = Some(canvas.upload_rgba8(w, h, &self.video_buffer));
        }
        self.video_dirty = false;
    }

    /// Draw the game, integer-scaled and centred on the panel.
    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &UiCtx) {
        let _ = ctx;
        let Some(tex) = self.tex_id else { return };

        let src = (self.last_width, self.last_height);
        let o = self.overscan;
        // Everything downstream works on the visible picture, not the buffer: a cropped NES
        // frame is 256×224, and both the placement and its aspect have to agree about that.
        let shown = slot2_gfx::cropped_size(src, o.left, o.top, o.right, o.bottom);
        let aspect = self.aspect.display(shown);
        let r = slot2_gfx::place(self.scale, shown, aspect, canvas.size());
        let uv = slot2_gfx::sub_uv(src, o.left, o.top, o.right, o.bottom);

        canvas.clear(slot2_gfx::Color::BLACK);
        canvas.image_uv(
            tex,
            r.x as f32,
            r.y as f32,
            r.w as f32,
            r.h as f32,
            uv,
            slot2_gfx::Color::WHITE,
        );
    }

    /// How the picture is laid on the panel, and what is cropped off it first.
    pub fn scale(&self) -> slot2_gfx::ScalePolicy {
        self.scale
    }

    pub fn set_scale(&mut self, policy: slot2_gfx::ScalePolicy) {
        self.scale = policy;
    }

    /// Turn the platform's default overscan crop on or off. Off is the safe answer for a
    /// game that draws to the edge; on hides the rubbish a television never showed.
    pub fn set_overscan(&mut self, crop: bool) {
        self.overscan = if crop {
            slot2_retro::def(self.retro_platform).overscan
        } else {
            slot2_retro::Overscan::NONE
        };
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
    /// The core's state right now, for tests and for anything that wants to compare two
    /// moments. `None` when the core refuses to serialize.
    pub fn core_state(&mut self) -> Option<Vec<u8>> {
        self.core.serialize().ok()
    }

    pub fn frames_run(&self) -> u64 {
        self.frames_run
    }
}

/// Unused imports guard: these types are part of the contract above.
#[allow(dead_code)]
fn _types(_: Option<TexId>, _: Option<Producer>, _: Option<Resampler>, _: Option<Core>) {}
