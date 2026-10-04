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
//! `start_named(card, cart, core_dir, core, sink_rate, tuning)` is the same list with the core
//! named instead of resolved: the Core row's choice, which must be one of this platform's own
//! candidates and whose library is built from the core directory. There is no fallback there —
//! a core that cannot run the console, or whose library is missing, is an error.
//!
//! ## Cheats
//! `start` reads the cart's `.cht` once (`Card::read_cheats`) and, once the core has the game,
//! puts the whole file on it before the first frame: one `reset_cheats`, then every entry in
//! file order with the file's index as the core's index, disabled entries included. A file
//! that cannot be read or applied fails the launch — half a set is worse than none. Toggling
//! an entry re-applies the whole set after a reset, so what the core holds is always what the
//! session says; nothing is written back to the card, and a new session starts from the file's
//! own `enable` values again (D-21: writing cheat files is the desktop's job).
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
//! ## States
//! Each session's states live in the namespace of the core it opened:
//! `States/<PLAT>/<stem>/<core-base-name>/{resume,<n>}.state`. A libretro state is a core's own
//! serialization, so two cores for one game keep two sets and neither can read the other's; a
//! library this frontend does not ship gets a deterministic namespace of its own. `resolve_core`
//! below is the one place that decides both which library opens and which namespace that is —
//! the shelf's Resume scan calls the same function, so the hint and the launch cannot disagree.
//!
//! ## Stopping
//! `stop(self)`: flush save RAM one last time, write a resume state
//! (`core.serialize()` → `card.scoped_write_state(cart, namespace, StateKind::Resume, …, thumb)`)
//! where the
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

/// Put a cheat set on the core: one reset, then the entries this core is owed.
///
/// `identity` is the core the code is about to be handed to, read from the library that was
/// actually opened. The delivery policy lives in `slot2-retro::quirks` because it is a fact
/// about that core's own `retro_cheat_set`: mGBA's ignores the `enabled` flag, so leaving an
/// entry off is the only way to have it off, while every other core here reads the flag.
///
/// The index handed over is always the file's own index — never a position in the subset — so
/// a later change to one entry cannot shift what the core believes the others are. An unknown
/// core (`None`, an external library the card named) keeps the older behaviour of being sent
/// everything, flag and all, which is what it was launched with before this policy existed.
fn apply_cheats(
    core: &mut Core,
    identity: Option<slot2_retro::CoreId>,
    cheats: &[slot2_store::Cheat],
) -> Result<(), Error> {
    let delivery = match identity {
        Some(core_id) => slot2_retro::cheat_delivery(core_id),
        None => slot2_retro::CheatDelivery::PassAllEntries,
    };

    core.reset_cheats();
    for (index, cheat) in cheats.iter().enumerate() {
        if delivery == slot2_retro::CheatDelivery::EnabledEntriesOnly && !cheat.enabled {
            continue;
        }
        core.set_cheat(index as u32, cheat.enabled, &cheat.code)?;
    }
    Ok(())
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
    /// A cheat index this session has no entry for.
    NoCheat(usize),
    /// A core the caller named that cannot run this platform's games.
    ///
    /// An error rather than a fallback: a caller that named one core is asking for that core,
    /// and quietly opening another would leave what it asked for and what is running apart.
    UnsupportedCore(slot2_retro::CoreId),
    /// Applying cheats failed and putting the previous set back failed with it. Both
    /// messages are kept: either one alone would hide the other.
    Cheat(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoCore(p) => write!(f, "no core at {}", p.display()),
            Error::Retro(e) => write!(f, "{e}"),
            Error::Store(e) => write!(f, "{e}"),
            Error::NoCheat(index) => write!(f, "no cheat at index {index}"),
            Error::UnsupportedCore(core) => {
                write!(f, "{} cannot run this console", core.base_name())
            }
            Error::Cheat(message) => write!(f, "{message}"),
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

/// The registry's name for the shelf a cart came off.
///
/// In one place because three callers need it — a launch, a frame, a scan — and a fourth copy
/// is how a platform gets missed.
pub(crate) fn retro_platform(platform: slot2_store::Platform) -> slot2_retro::Platform {
    match platform {
        slot2_store::Platform::Gb => slot2_retro::Platform::Gb,
        slot2_store::Platform::Gbc => slot2_retro::Platform::Gbc,
        slot2_store::Platform::Gba => slot2_retro::Platform::Gba,
        slot2_store::Platform::Nes => slot2_retro::Platform::Nes,
        slot2_store::Platform::Snes => slot2_retro::Platform::Snes,
        slot2_store::Platform::Md => slot2_retro::Platform::Md,
        slot2_store::Platform::Sms => slot2_retro::Platform::Sms,
    }
}

/// Which core a launch will open, and the namespace its states live in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CoreChoice {
    /// The library that will be opened.
    pub(crate) dylib: PathBuf,
    /// What that library is, when it is one this frontend ships.
    pub(crate) core_id: Option<slot2_retro::CoreId>,
    /// Where its states live. Never empty, and never able to leave the card.
    pub(crate) namespace: slot2_store::StateNamespace,
}

/// Resolve the core a game's settings file asks for, without opening anything.
///
/// A launch and the shelf's scan of Resume states need the same answer, and a second copy of
/// this would be a second answer to "what does this settings file mean". The rules are the
/// ones the launch path always had: a name that is not on the card falls back to the platform's
/// own core, so does a core that cannot run this console, and a library this frontend does not
/// ship opens the way it always did.
///
/// `quiet` is for the scan, which runs for every cart on every rescan: a fallback is worth one
/// line at the moment a game starts, not one line per shelf visit.
pub(crate) fn resolve_core(card: &Card, cart: &Cart, core_dir: &Path, quiet: bool) -> CoreChoice {
    let platform = retro_platform(cart.platform);
    let def = slot2_retro::def(platform);
    let mut chosen = core_dir.join(def.default_core.file_name());
    let mut fell_back = None;

    match card.read_settings(cart).core.as_deref().map(str::trim) {
        // A core's name is a file name inside the core directory, never a path: a setting that
        // could name `..\..\anything` would be a way to open a library from anywhere on the
        // card, which is not what this setting is for.
        Some(name) if is_path_like(name) => {
            fell_back = Some(format!("names {name}, which is not a core file name"));
        }
        Some(name) => {
            let named = core_dir.join(core_file_name(name));
            if named.is_file() {
                match slot2_retro::CoreId::from_library_path(&named) {
                    Some(core) if !core.supports_platform(platform) => {
                        fell_back = Some(format!(
                            "asks for {}, which cannot run {platform:?} games",
                            core.base_name()
                        ));
                    }
                    _ => chosen = named,
                }
            } else {
                fell_back = Some(format!("asks for core {name}, which is not on the card"));
            }
        }
        None => {}
    }

    if let Some(why) = fell_back.filter(|_| !quiet) {
        eprintln!(
            "slot2: {} {why}; using {}",
            cart.stem,
            def.default_core.base_name()
        );
    }

    let core_id = slot2_retro::CoreId::from_library_path(&chosen);
    let namespace = namespace_for(&chosen, core_id);
    CoreChoice {
        dylib: chosen,
        core_id,
        namespace,
    }
}

/// A name that could walk out of the core directory.
fn is_path_like(name: &str) -> bool {
    name.is_empty()
        || name == "."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name == ".."
}

/// The namespace of a platform's own core: where a card's flat states belong, and where a
/// game starts when nothing else says otherwise.
pub(crate) fn default_namespace(platform: slot2_store::Platform) -> slot2_store::StateNamespace {
    namespace_of_core(slot2_retro::def(retro_platform(platform)).default_core)
}

/// The namespace a core this frontend ships keeps its states in.
fn namespace_of_core(core: slot2_retro::CoreId) -> slot2_store::StateNamespace {
    slot2_store::StateNamespace::new(core.base_name())
        .expect("a core's base name is a lowercase ASCII name")
}

/// The namespace a library's states live in.
///
/// A core this frontend ships is named by its canonical base name — `mgba_libretro`,
/// `gambatte_libretro` — so a card that has been through several versions still finds its
/// states where the core's own name says they are. An external library gets its stem when that
/// is a name this store takes, and otherwise a deterministic spelling of its bytes: the same
/// library lands in the same directory on every machine and every run, which a seeded hasher
/// could not promise.
fn namespace_for(path: &Path, core_id: Option<slot2_retro::CoreId>) -> slot2_store::StateNamespace {
    match core_id {
        Some(core) => namespace_of_core(core),
        None => external_namespace(path),
    }
}

/// The namespace for a library this frontend does not ship: `external_<hex>_<hex>`.
///
/// The first part is the first bytes of the stem spelled out, the second is FNV-1a over the
/// whole stem. No `DefaultHasher`, no seed: a directory name on a card has to mean the same
/// thing tomorrow and on the machine next door.
fn external_namespace(path: &Path) -> slot2_store::StateNamespace {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    // A stem that is already a plain name is its own namespace, which is what a core called
    // `mystery_libretro` should get.
    let canonical = stem.to_ascii_lowercase();
    if let Ok(namespace) = slot2_store::StateNamespace::new(&canonical) {
        return namespace;
    }

    // 9 + 19 * 2 + 1 + 16 = 64 bytes, the most a namespace may be.
    const PREFIX_BYTES: usize = 19;
    let mut name = String::from("external_");
    for byte in stem.as_bytes().iter().take(PREFIX_BYTES) {
        name.push_str(&format!("{byte:02x}"));
    }
    name.push('_');
    name.push_str(&format!("{:016x}", fnv1a64(stem.as_bytes())));
    slot2_store::StateNamespace::new(&name)
        .expect("the external spelling is 64 bytes of lowercase ASCII at most")
}

/// FNV-1a, 64 bit: a fixed arithmetic recipe with no seed and no table, so the same bytes give
/// the same number in every process — and this number ends up in a directory name on a card.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub struct Session {
    cart: Cart,
    core: Core,
    /// Which core the library that was opened is, when it is one this frontend ships. `None`
    /// for an external library the card named, which is a core this crate knows nothing about.
    core_id: Option<slot2_retro::CoreId>,
    /// Where this session's states live: one directory per core, under the game. A libretro
    /// state is a core's own serialization and no other core can read it, so the core that was
    /// opened is the only one whose states this session reads or writes.
    namespace: slot2_store::StateNamespace,
    retro_platform: slot2_retro::Platform,
    /// The cheats this session wants on, in file order, exactly as the card holds them.
    ///
    /// The index in this slice is the file's index, which is the index the core is given —
    /// but not every entry here is necessarily *on* the core: `CheatDelivery::EnabledEntriesOnly`
    /// cores (mGBA, FCEUmm) are handed only the enabled ones. This is the desired list, and
    /// which entries the core holds is the delivery policy's business.
    cheats: Vec<slot2_store::Cheat>,
    rewind: slot2_retro::Rewind,
    rewind_on: bool,
    /// The worst capture seen so far, which is what the interval is budgeted against.
    rewind_cost: std::time::Duration,
    speed: u32,
    aspect: slot2_retro::Aspect,
    overscan: slot2_retro::Overscan,
    scale: slot2_gfx::ScalePolicy,
    /// The effect the game picture is drawn through, or `None` for the ordinary image draw.
    ///
    /// Set once from the card at launch and changeable at run time. A missing key lands here
    /// as the platform's own default (which is what a later `set_shader_effect` moves), and an
    /// explicit `Off` lands here as `None`: the two are different things on the card and this
    /// field is the one place they stop being. Nothing about the running game depends on it:
    /// it is renderer state, like the scale policy.
    shader: Option<slot2_gfx::ShaderEffect>,
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
        // Which core this launch opens, and where its states live: one resolver, shared with
        // the shelf's scan, so the two cannot disagree about what a settings file means.
        let choice = resolve_core(card, cart, core_dir, false);
        Session::open(card, cart, &choice, sink_rate, tuning)
    }

    /// Open one named core for this cart, without asking the settings file.
    ///
    /// The Core row has already let the player choose, and the choice is about to be written
    /// to the card; coming back through `resolve_core` here would be a second answer to
    /// "which library", and a launch that quietly fell back to another core would leave the
    /// setting and the running game disagreeing about what is being played. So the core is
    /// named, its library path is built from the core directory rather than handed in, and a
    /// combination this platform does not run is an error rather than a fallback.
    pub fn start_named(
        card: &Card,
        cart: &Cart,
        core_dir: &Path,
        core: slot2_retro::CoreId,
        sink_rate: u32,
        tuning: slot2_retro::Tuning,
    ) -> Result<(Session, Consumer), Error> {
        if !slot2_retro::supported_cores(retro_platform(cart.platform)).contains(&core) {
            return Err(Error::UnsupportedCore(core));
        }
        let choice = CoreChoice {
            dylib: core_dir.join(core.file_name()),
            core_id: Some(core),
            namespace: namespace_of_core(core),
        };
        Session::open(card, cart, &choice, sink_rate, tuning)
    }

    /// Load a resolved core for a cart and start it.
    ///
    /// The one place a `Session` is built, so a core found through the settings and a core
    /// named by the player cannot drift apart in what the core is handed: the options, the
    /// cheat delivery policy and the state namespace all come from `choice`.
    fn open(
        card: &Card,
        cart: &Cart,
        choice: &CoreChoice,
        sink_rate: u32,
        tuning: slot2_retro::Tuning,
    ) -> Result<(Session, Consumer), Error> {
        let CoreChoice {
            dylib,
            core_id,
            namespace,
        } = choice;
        let retro_platform = retro_platform(cart.platform);

        let def = slot2_retro::def(retro_platform);
        let settings = card.read_settings(cart);
        // Read once, here: the file belongs to this launch, and one that cannot be read or
        // understood stops it rather than quietly starting a game with no cheats.
        let cheats = card.read_cheats(cart)?;

        if !dylib.exists() {
            return Err(Error::NoCore(dylib.clone()));
        }

        // A BIOS on the card is the player asking for the real boot sequence; none means
        // the core's own. Nothing else about the launch depends on it.
        let bios_present = def
            .bios
            .iter()
            .any(|name| card.bios_dir().join(name).exists());
        // The options belong to the core that will read them: an alternative core must not be
        // handed the platform default's `mgba_*` keys. A core this crate does not know keeps
        // the platform default's options, which is what it was launched with before cores were
        // identified at all.
        let options = match core_id {
            Some(core) => {
                slot2_retro::options_for_core(*core, retro_platform, bios_present, tuning)
                    .unwrap_or_else(|| {
                        slot2_retro::options_for(retro_platform, bios_present, tuning)
                    })
            }
            None => slot2_retro::options_for(retro_platform, bios_present, tuning),
        };

        let env = slot2_retro::Env {
            system_dir: card.bios_dir(),
            save_dir: card.root().join("Saves").join(cart.platform.folder()),
            options,
            language: 0,
        };

        let mut core = Core::load(dylib, &cart.rom, env)?;

        if let Some(save) = card.read_save(cart) {
            if let Err(e) = core.write_memory(slot2_retro::Memory::SaveRam, &save) {
                eprintln!("slot2: save RAM size mismatch: {e}");
            }
        }

        // The whole set goes on before the first frame, so the game never runs one without
        // the cheats it is supposed to have. A failure takes the attempt back out again: a
        // core holding half a set behaves like neither game.
        if let Err(e) = apply_cheats(&mut core, *core_id, &cheats) {
            core.reset_cheats();
            return Err(e);
        }

        let av = core.av_info();
        let resampler = Resampler::new(av.sample_rate as u32, sink_rate);
        let (producer, consumer) = slot2_audio::Ring::new((sink_rate / 4) as usize).split();

        let session = Session {
            cart: cart.clone(),
            core,
            core_id: *core_id,
            namespace: namespace.clone(),
            retro_platform,
            cheats,
            rewind: slot2_retro::Rewind::new(def.rewind),
            rewind_on: settings.rewind.unwrap_or(true),
            rewind_cost: std::time::Duration::ZERO,
            speed: 1,
            aspect: def.aspect,
            overscan: Session::overscan_for(retro_platform, settings.overscan),
            scale: Session::policy_for(settings.scale),
            shader: Session::shader_effect_for(retro_platform, settings.shader),
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

    /// Which of the cores this frontend ships is running, from the library that was opened.
    ///
    /// `None` when the card named a library this crate does not know — an external core still
    /// runs, it just has no per-core facts here — and `Some` for the core a fallback landed on
    /// when the named one was not on the card.
    pub fn core_id(&self) -> Option<slot2_retro::CoreId> {
        self.core_id
    }

    /// Where this session's states live: the namespace of the core that was opened, whether
    /// or not this frontend ships it.
    ///
    /// The App reads and writes states through this and nothing else, so a second core for the
    /// same game never sees — or overwrites — the first one's Resume or numbered states.
    pub fn state_namespace(&self) -> &slot2_store::StateNamespace {
        &self.namespace
    }

    /// The cheats this session wants on, in file order. The index in this slice is the index
    /// the core was given, which is the index `set_cheat_enabled` takes. An entry that is off
    /// is still in the list; whether the core was handed it depends on the core (see
    /// `apply_cheats`).
    ///
    /// Read only, and only about right now: the card's file stays the record of what the
    /// game starts with (D-21).
    pub fn cheats(&self) -> &[slot2_store::Cheat] {
        &self.cheats
    }

    /// Turn one entry on or off for this session, and put the whole set back on the core.
    ///
    /// A reset and a full ordered re-apply, not a second call for the one index: what the
    /// core holds has to end up as what this session says, and the entry that changed is not
    /// the only thing the core may be carrying from a moment ago.
    ///
    /// Nothing is written to the card — `cheatN_enable` in the file belongs to the desktop
    /// tool (D-21) — so starting the game again begins from the file. If the re-apply fails,
    /// the core goes back to the previous set and the session's own state stays with it; if
    /// even that fails, the error carries both messages rather than only the last one.
    pub fn set_cheat_enabled(&mut self, index: usize, enabled: bool) -> Result<(), Error> {
        let Some(cheat) = self.cheats.get(index) else {
            return Err(Error::NoCheat(index));
        };
        if cheat.enabled == enabled {
            // The core is already holding this set, so there is nothing to say to it.
            return Ok(());
        }

        let mut wanted = self.cheats.clone();
        wanted[index].enabled = enabled;

        if let Err(e) = apply_cheats(&mut self.core, self.core_id, &wanted) {
            self.core.reset_cheats();
            return match apply_cheats(&mut self.core, self.core_id, &self.cheats) {
                Ok(()) => Err(e),
                Err(back) => Err(Error::Cheat(format!(
                    "{e}; putting the previous cheats back also failed: {back}"
                ))),
            };
        }
        self.cheats = wanted;
        Ok(())
    }

    /// Advance one frame with the buttons currently held.
    pub fn run_frame(&mut self, held: &[LogicalButton], volume: &Volume) {
        let retro_platform = retro_platform(self.cart.platform);

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
        // The placement, the crop and the tint are worked out once and are the same whichever
        // way the picture goes out; the effect changes how the quad is sampled, not where it
        // is. A shader is never a reason to recompute the geometry or to fall back to the
        // whole texture.
        let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);

        canvas.clear(slot2_gfx::Color::BLACK);
        match self.shader {
            Some(effect) => {
                canvas.image_effect_uv(tex, x, y, w, h, uv, slot2_gfx::Color::WHITE, effect)
            }
            None => canvas.image_uv(tex, x, y, w, h, uv, slot2_gfx::Color::WHITE),
        }
    }

    /// The renderer policy a stored scale means, or `ScalePolicy::default()` when the game has
    /// no override of its own.
    ///
    /// Here rather than at one of its callers because there are two: the launch path reads a
    /// card, and the Display menu changes the setting mid-game. A second copy of this mapping is
    /// a second answer to "what does Fill mean".
    pub fn policy_for(scale: Option<slot2_store::ScaleMode>) -> slot2_gfx::ScalePolicy {
        match scale {
            Some(slot2_store::ScaleMode::Integer) => slot2_gfx::ScalePolicy::Integer,
            Some(slot2_store::ScaleMode::AspectFit) => slot2_gfx::ScalePolicy::AspectFit,
            Some(slot2_store::ScaleMode::Fill) => slot2_gfx::ScalePolicy::Fill,
            None => slot2_gfx::ScalePolicy::default(),
        }
    }

    /// The renderer effect a stored shader setting means on this platform, or `None` for the
    /// ordinary image draw.
    ///
    /// The one place a card's spelling becomes a renderer's type, because it is the only place
    /// that knows all three sides: `slot2-store` owns the file format, `slot2-retro` owns the
    /// platform's own default and `slot2-gfx` owns the programs, and none of them has to know
    /// the others. Written as an exhaustive match rather than through the enums' order, their
    /// spellings or the platform's name, so a new preset in the store or a new platform
    /// default stops compiling here and somebody decides what it draws.
    ///
    /// The three meanings, in the order the card's two states and its silence have to be read:
    /// an explicit `Off` is the plain draw on every platform; any other preset is what the
    /// game asked for, whatever the platform's own default is; and a missing key inherits
    /// `PlatformDef::shader_default`. The last is what the platform is here for, and it is a
    /// default rather than a rewrite — nothing on this path writes a computed value back to
    /// the card, so "shaders off" and "never said anything" stay different on disk.
    pub fn shader_effect_for(
        platform: slot2_retro::Platform,
        preset: Option<slot2_store::ShaderPreset>,
    ) -> Option<slot2_gfx::ShaderEffect> {
        match preset {
            Some(slot2_store::ShaderPreset::Off) => None,
            Some(slot2_store::ShaderPreset::SharpBilinear) => {
                Some(slot2_gfx::ShaderEffect::SharpBilinear)
            }
            Some(slot2_store::ShaderPreset::Lcd3x) => Some(slot2_gfx::ShaderEffect::Lcd3x),
            Some(slot2_store::ShaderPreset::ZfastCrt) => Some(slot2_gfx::ShaderEffect::ZfastCrt),
            Some(slot2_store::ShaderPreset::Scanline) => Some(slot2_gfx::ShaderEffect::Scanline),
            None => Some(match slot2_retro::def(platform).shader_default {
                slot2_retro::PlatformShader::SharpBilinear => {
                    slot2_gfx::ShaderEffect::SharpBilinear
                }
                slot2_retro::PlatformShader::Lcd3x => slot2_gfx::ShaderEffect::Lcd3x,
                slot2_retro::PlatformShader::ZfastCrt => slot2_gfx::ShaderEffect::ZfastCrt,
                slot2_retro::PlatformShader::Scanline => slot2_gfx::ShaderEffect::Scanline,
            }),
        }
    }

    /// The effect this session draws its game picture through, or `None` for the plain draw.
    pub fn shader_effect(&self) -> Option<slot2_gfx::ShaderEffect> {
        self.shader
    }

    /// Change the effect at run time, for the Display menu and the shelf.
    ///
    /// Renderer state only, exactly like `set_scale`: no card is read or written, the core is
    /// not restarted or reset, no frame is run, and the game texture is left where it is. The
    /// next draw uses the new effect and the one after that could use another.
    pub fn set_shader_effect(&mut self, effect: Option<slot2_gfx::ShaderEffect>) {
        self.shader = effect;
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
        self.set_overscan_setting(Some(crop));
    }

    /// The crop a stored overscan setting means on this platform.
    ///
    /// The one place the card's silence becomes a crop, because it is the only place that knows
    /// both sides: `slot2-store` owns the file's two states and `slot2-retro` owns the platform's
    /// own answer. Written as an exhaustive match rather than through `unwrap_or`, so the three
    /// meanings stay three: `None` is the absence of a setting and inherits
    /// `PlatformDef::overscan`, `Some(true)` asks for that same crop as this game's own decision,
    /// and `Some(false)` asks for the whole frame. A platform with nothing to crop answers
    /// `Overscan::NONE` for two of the three, which is why the App only offers the choice where
    /// there is a difference to make.
    ///
    /// Nothing on this path writes a computed value back to the card: "crop" and "never said
    /// anything" stay different on disk however alike they look on screen.
    pub fn overscan_for(
        platform: slot2_retro::Platform,
        setting: Option<bool>,
    ) -> slot2_retro::Overscan {
        match setting {
            None | Some(true) => slot2_retro::def(platform).overscan,
            Some(false) => slot2_retro::Overscan::NONE,
        }
    }

    /// The crop this session is drawing with.
    pub fn overscan(&self) -> slot2_retro::Overscan {
        self.overscan
    }

    /// Change the crop at run time, for the Display menu's overscan screen.
    ///
    /// Renderer state only, exactly like `set_scale` and `set_shader_effect`: no card is read or
    /// written, the core is not restarted or reset, no frame is run, and the game texture is left
    /// where it is. The next draw works out its placement, aspect and UV again from the new crop,
    /// over the same picture the core last produced.
    pub fn set_overscan_setting(&mut self, setting: Option<bool>) {
        self.overscan = Session::overscan_for(self.retro_platform, setting);
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
        card.scoped_write_state(&self.cart, &self.namespace, kind, &data, t)?;
        Ok(())
    }

    pub fn load_state(&mut self, card: &Card, kind: StateKind) -> Result<(), Error> {
        let data = card
            .scoped_read_state(&self.cart, &self.namespace, kind)
            .ok_or_else(|| Error::Retro(slot2_retro::Error::State("no such state".into())))?;
        self.core.unserialize(&data)?;
        self.resampler.reset();
        // The captures behind this moment belong to a future that no longer happened.
        // Stepping back into it would put the player in a game that was never played, which
        // is why a state jump is the end of the chain rather than a point in it.
        self.rewind.clear();
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
            if let Err(e) =
                card.scoped_write_state(&self.cart, &self.namespace, StateKind::Resume, &state, t)
            {
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

#[cfg(test)]
mod tests {
    //! The core resolver: which library a settings file means, and which directory that
    //! library's states go in. The launch and the shelf's Resume scan both ask this one
    //! function, so what is pinned here is the whole of what a settings file can mean.
    //!
    //! Nothing is opened: the questions are about a name and a path, so an empty file with
    //! the right name is the whole fixture.

    use super::*;
    use slot2_store::{GameSettings, Platform};

    /// A card with one GBA cart, its settings file, and a directory of stub libraries.
    fn fixture(tag: &str, core: Option<&str>) -> (Card, Cart, PathBuf) {
        let root = std::env::temp_dir().join(format!("slot2-resolve-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let card = Card::new(root.join("card"));
        card.ensure_layout();
        let dir = root.join("cores");
        std::fs::create_dir_all(&dir).unwrap();
        let cart = Cart {
            platform: Platform::Gba,
            stem: "arm".into(),
            title: "arm".into(),
            rom: card.games_dir(Platform::Gba).join("arm.gba"),
        };
        if let Some(name) = core {
            patch_core(&card, &cart, Some(name));
        }
        (card, cart, dir)
    }

    /// A file named like a library, with nothing in it. The resolver never opens one.
    fn stub(dir: &Path, name: &str) {
        std::fs::write(dir.join(name), b"not a library").unwrap();
    }

    fn patch_core(card: &Card, cart: &Cart, core: Option<&str>) {
        let settings = GameSettings {
            core: core.map(str::to_owned),
            ..Default::default()
        };
        card.write_settings(cart, &settings).unwrap();
    }

    fn ns(name: &str) -> slot2_store::StateNamespace {
        slot2_store::StateNamespace::new(name).unwrap()
    }

    #[test]
    fn a_game_with_no_setting_takes_the_platforms_own_core_and_its_namespace() {
        let (card, cart, dir) = fixture("plain", None);
        let choice = resolve_core(&card, &cart, &dir, true);
        assert_eq!(
            choice.dylib,
            dir.join(slot2_retro::CoreId::Mgba.file_name())
        );
        assert_eq!(choice.core_id, Some(slot2_retro::CoreId::Mgba));
        assert_eq!(choice.namespace, ns("mgba_libretro"));
        assert_eq!(
            choice.namespace,
            default_namespace(Platform::Gba),
            "the namespace a scan caches a resume against must be the one a launch uses"
        );
    }

    #[test]
    fn an_official_core_is_taken_only_when_it_is_on_the_card_and_can_run_the_console() {
        let (card, cart, dir) = fixture("gpsp", Some("gpsp"));

        // Not on the card: the platform's own core, which is what a card that names a core
        // it does not have has always got.
        let choice = resolve_core(&card, &cart, &dir, true);
        assert_eq!(choice.core_id, Some(slot2_retro::CoreId::Mgba));
        assert_eq!(choice.namespace, ns("mgba_libretro"));

        // On the card: what the setting asked for, spelled either way a person would.
        stub(&dir, &slot2_retro::CoreId::Gpsp.file_name());
        for name in ["gpsp", "gpsp_libretro"] {
            patch_core(&card, &cart, Some(name));
            let choice = resolve_core(&card, &cart, &dir, true);
            assert_eq!(choice.core_id, Some(slot2_retro::CoreId::Gpsp), "{name}");
            assert_eq!(choice.namespace, ns("gpsp_libretro"), "{name}");
        }

        // An official core that cannot run this console: the file is there, and opening it
        // would be a game that cannot start. The namespace follows the core that opens.
        stub(&dir, &slot2_retro::CoreId::Gambatte.file_name());
        patch_core(&card, &cart, Some("gambatte_libretro"));
        let choice = resolve_core(&card, &cart, &dir, true);
        assert_eq!(choice.core_id, Some(slot2_retro::CoreId::Mgba));
        assert_eq!(choice.namespace, ns("mgba_libretro"));

        // The same library on a console it does run: opened, with its own namespace.
        let gb = Cart {
            platform: Platform::Gb,
            ..cart.clone()
        };
        patch_core(&card, &gb, Some("gambatte_libretro"));
        let choice = resolve_core(&card, &gb, &dir, true);
        assert_eq!(choice.core_id, Some(slot2_retro::CoreId::Gambatte));
        assert_eq!(choice.namespace, ns("gambatte_libretro"));
    }

    #[test]
    fn a_setting_that_names_a_path_never_reaches_outside_the_core_directory() {
        let (card, cart, dir) = fixture("paths", None);
        // Every one of these is a path rather than a core's file name, including the empty
        // string and the trailing-extension spellings a person might type. None of them may
        // pick a library from anywhere on the card.
        let names = [
            "",
            ".",
            "..",
            "../mgba_libretro",
            "..\\..\\mgba_libretro",
            "/tmp/mgba_libretro",
            "C:\\cores\\mgba_libretro",
            "sub/dir/mgba_libretro",
        ];
        for name in names {
            patch_core(&card, &cart, Some(name));
            let choice = resolve_core(&card, &cart, &dir, true);
            assert_eq!(choice.core_id, Some(slot2_retro::CoreId::Mgba), "{name:?}");
            assert_eq!(
                choice.dylib,
                dir.join(slot2_retro::CoreId::Mgba.file_name()),
                "{name:?}"
            );
            assert_eq!(choice.dylib.parent(), Some(dir.as_path()), "{name:?}");
            assert_eq!(choice.namespace, ns("mgba_libretro"), "{name:?}");
        }
    }

    #[test]
    fn a_library_this_frontend_does_not_ship_gets_a_deterministic_namespace() {
        // A stem that is already a name this store takes is its own namespace, lowercased.
        assert_eq!(
            external_namespace(Path::new("mystery_libretro.dll")),
            ns("mystery_libretro")
        );
        assert_ne!(
            external_namespace(Path::new("mystery_libretro.dll")),
            external_namespace(Path::new("another_libretro.dll")),
            "two libraries must not share one set of states"
        );

        // A stem this store will not take gets a fixed spelling of its bytes. Pinned rather
        // than merely compared with itself: this string is a directory name on a card, and a
        // later version that changed the recipe would stop finding what it wrote before.
        for (stem, expected) in [
            (
                "core!_libretro",
                "external_636f7265215f6c6962726574726f_73fdf30215bf810f",
            ),
            (
                "My Core_libretro",
                "external_4d7920436f72655f6c6962726574726f_be538c6f12df829e",
            ),
        ] {
            let path = PathBuf::from(format!("{stem}.dll"));
            let got = external_namespace(&path);
            assert_eq!(got.as_str(), expected, "{stem}");
            assert!(
                got.as_str().len() <= 64,
                "a namespace is a directory name and has a ceiling: {}",
                got.as_str().len()
            );
            assert_eq!(
                external_namespace(&path),
                got,
                "the answer moved between runs"
            );
        }
    }

    #[test]
    fn an_unknown_library_on_the_card_opens_and_keeps_its_own_states() {
        let (card, cart, dir) = fixture("external", Some("mystery"));
        stub(&dir, &core_file_name("mystery"));

        let choice = resolve_core(&card, &cart, &dir, true);
        assert_eq!(
            choice.core_id, None,
            "an unknown library was given an identity"
        );
        assert_eq!(choice.dylib, dir.join(core_file_name("mystery")));
        assert_eq!(choice.namespace, ns("mystery_libretro"));
        assert_ne!(
            choice.namespace,
            default_namespace(Platform::Gba),
            "an external core must not land in the platform's own states"
        );
    }
}
