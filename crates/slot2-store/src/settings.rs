//! What the player has changed, kept on the card beside the games.
//!
//! Two files, the two outer ends of the settings hierarchy (D-23): the frontend's own at
//! `System/slot2.ini`, which holds the volume level and the display offset of D-25, and each
//! game's at `System/games/<PLAT>/<stem>.ini`, holding only what was actually overridden. A
//! game nobody has touched has no file at all, and every unset key falls back to the
//! platform's default — so a card's settings folder reads as a list of decisions somebody
//! made, not a dump of everything that could be set.
//!
//! Both files are plain `key=value`, editable on a PC with a text editor, which is the only
//! way to reach them until the shelf menus exist. Unknown keys are left alone when a file is
//! rewritten: a newer SLOT2 writing a key this one has never heard of must not lose it on the
//! next save.

use crate::card::{Card, Cart};
use crate::ini::Ini;
use crate::Error;

/// How the game's picture is laid on the panel. Mirrors `slot2_gfx::ScalePolicy`, which
/// this crate does not depend on: the card owns the *file format*, and a spelling on a card
/// has to keep working even if the renderer's type is renamed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleMode {
    Integer,
    AspectFit,
    Fill,
}

impl ScaleMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            ScaleMode::Integer => "integer",
            ScaleMode::AspectFit => "aspect",
            ScaleMode::Fill => "fill",
        }
    }

    /// Case-insensitive, and `None` for anything unrecognised — a typo in a hand-edited
    /// file falls back to the default rather than refusing to start the game.
    pub fn parse(s: &str) -> Option<ScaleMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "integer" | "int" => Some(ScaleMode::Integer),
            "aspect" | "aspectfit" | "aspect_fit" => Some(ScaleMode::AspectFit),
            "fill" | "stretch" => Some(ScaleMode::Fill),
            _ => None,
        }
    }
}

/// Which built-in shader a game's picture runs through (D-10).
///
/// This store-side type owns the stable card format without depending on the renderer. The
/// later renderer implementation must map these choices at one boundary so a spelling on a
/// card keeps working if its internal types change. The platform's own default is not here —
/// this type only says what a person chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderPreset {
    /// The player turned shaders off for this game. Different from having said nothing: the
    /// platform's default preset, when there is one, must not come back and override this.
    Off,
    SharpBilinear,
    Lcd3x,
    ZfastCrt,
    Scanline,
}

impl ShaderPreset {
    /// The one spelling a save writes. Hand-edited files may use the spellings [`parse`]
    /// accepts, but a rewrite always comes back canonical.
    ///
    /// [`parse`]: ShaderPreset::parse
    pub const fn as_str(self) -> &'static str {
        match self {
            ShaderPreset::Off => "none",
            ShaderPreset::SharpBilinear => "sharp-bilinear",
            ShaderPreset::Lcd3x => "lcd3x",
            ShaderPreset::ZfastCrt => "zfast-crt",
            ShaderPreset::Scanline => "scanline",
        }
    }

    /// Case-insensitive, and `None` for anything unrecognised — a typo in a hand-edited file
    /// falls back to the platform's default rather than refusing to start the game. Falling
    /// back to [`ShaderPreset::Off`] instead would be worse than either: it would silently
    /// turn shaders off for a game whose only mistake was a misspelling.
    pub fn parse(s: &str) -> Option<ShaderPreset> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" | "off" => Some(ShaderPreset::Off),
            "sharp-bilinear" | "sharp_bilinear" | "sharpbilinear" => {
                Some(ShaderPreset::SharpBilinear)
            }
            "lcd3x" | "lcd-3x" => Some(ShaderPreset::Lcd3x),
            "zfast-crt" | "zfast_crt" | "zfastcrt" => Some(ShaderPreset::ZfastCrt),
            "scanline" | "scanlines" => Some(ShaderPreset::Scanline),
            _ => None,
        }
    }
}

/// Every field is optional, and `None` means "whatever the platform says".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameSettings {
    /// A core's base name, as in `mgba_libretro`. The frontend falls back to the platform's
    /// default when the named core is not on the card — a setting must not make a game
    /// unlaunchable.
    pub core: Option<String>,
    pub scale: Option<ScaleMode>,
    /// Whether to crop the platform's overscan. Only the NES has any to crop.
    pub overscan: Option<bool>,
    /// Whether to keep a rewind ring. It costs a save state every tenth of a second and up
    /// to the platform's byte budget, which a slow core or a tight game may not want.
    pub rewind: Option<bool>,
    /// The shader this game wants. `None` is the absence of the key, which *inherits* the
    /// platform's default preset; [`ShaderPreset::Off`] is a decision the player made, and
    /// the two are not the same thing.
    pub shader: Option<ShaderPreset>,
    /// Whether this game wants the overlay picture for its own geometry (D-11).
    ///
    /// `None` is the absence of the key, which inherits the platform's default; `Some(true)`
    /// asks for the overlay on this game and `Some(false)` turns it off. The three are kept
    /// apart even while the default is "no overlay anywhere", because a future default must
    /// reach the games that never said anything and not the ones that said no.
    ///
    /// Nothing here looks at whether a picture exists. Overlays are one PNG per platform and
    /// geometry, and a game asking for one that is not on the card is the loader's silence to
    /// keep — erasing the decision here would turn a missing file into a forgotten choice.
    pub overlay: Option<bool>,
}

impl GameSettings {
    pub const KEY_CORE: &'static str = "core";
    pub const KEY_SCALE: &'static str = "scale";
    pub const KEY_OVERSCAN: &'static str = "overscan";
    pub const KEY_REWIND: &'static str = "rewind";
    pub const KEY_SHADER: &'static str = "shader";
    pub const KEY_OVERLAY: &'static str = "overlay";

    /// True when nothing is set, which is when the file should not exist.
    pub fn is_default(&self) -> bool {
        *self == GameSettings::default()
    }

    pub fn from_ini(ini: &Ini) -> GameSettings {
        GameSettings {
            core: ini
                .get(Self::KEY_CORE)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            scale: ini.get(Self::KEY_SCALE).and_then(ScaleMode::parse),
            overscan: ini.get(Self::KEY_OVERSCAN).and_then(parse_bool),
            rewind: ini.get(Self::KEY_REWIND).and_then(parse_bool),
            shader: ini.get(Self::KEY_SHADER).and_then(ShaderPreset::parse),
            overlay: ini.get(Self::KEY_OVERLAY).and_then(parse_bool),
        }
    }

    /// Write this onto `ini`, leaving every key it does not own untouched and removing the
    /// ones that are no longer set.
    pub fn apply_to(&self, ini: &mut Ini) {
        match &self.core {
            Some(c) => ini.set(Self::KEY_CORE, c),
            None => {
                ini.remove(Self::KEY_CORE);
            }
        }
        match self.scale {
            Some(s) => ini.set(Self::KEY_SCALE, s.as_str()),
            None => {
                ini.remove(Self::KEY_SCALE);
            }
        }
        match self.overscan {
            Some(b) => ini.set(Self::KEY_OVERSCAN, if b { "on" } else { "off" }),
            None => {
                ini.remove(Self::KEY_OVERSCAN);
            }
        }
        match self.rewind {
            Some(b) => ini.set(Self::KEY_REWIND, if b { "on" } else { "off" }),
            None => {
                ini.remove(Self::KEY_REWIND);
            }
        }
        match self.shader {
            Some(p) => ini.set(Self::KEY_SHADER, p.as_str()),
            None => {
                ini.remove(Self::KEY_SHADER);
            }
        }
        match self.overlay {
            Some(b) => ini.set(Self::KEY_OVERLAY, if b { "on" } else { "off" }),
            None => {
                ini.remove(Self::KEY_OVERLAY);
            }
        }
    }
}

/// The spellings a person might reasonably write for "yes".
fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "on" | "true" | "yes" | "1" => Some(true),
        "off" | "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

/// The level the frontend starts a machine at: the same 70 `slot2_audio::Volume::default()`
/// uses, written down here because the card owns the file and the default that goes with it,
/// and because this crate does not depend on the audio one.
///
/// The boot path reads this rather than repeating the number: a second copy is a second thing
/// to keep in step with the file format.
pub const DEFAULT_VOLUME_LEVEL: u8 = 70;

/// The loudest level there is, and the top of the range a card may store.
pub const MAX_VOLUME_LEVEL: u8 = 100;

/// The key the player's chosen language is stored under in `System/slot2.ini` (D-23).
pub const LANGUAGE_KEY: &str = "language";

/// What a card that has never been set says.
///
/// English is the pack every other language falls back to for the keys it lacks, and the one
/// compiled into every build, so a card that says nothing is a card that always works. The store
/// does not read that fallback from `slot2-i18n` — it does not depend on it — and the two names
/// being the same today is a coincidence the App boundary is where to check.
pub const DEFAULT_LANGUAGE: &str = "en";

/// The longest a language code may be, in bytes.
///
/// A code is a file stem under `System/Lang`, not a sentence: `zh-Hant`, `pt-BR` and `ja_custom`
/// are all far shorter than this, and anything longer is a caller who has confused the code with
/// something else.
const MAX_LANGUAGE_CODE_BYTES: usize = 64;

/// The key the clock's display offset is stored under in `System/slot2.ini` (D-25).
pub const UTC_OFFSET_MINUTES_KEY: &str = "utc_offset_minutes";

/// Minutes added to UTC before a time is shown. Zero is UTC, which is what a card that has
/// never been set says.
///
/// The system clock and every stored timestamp stay UTC: this is a display offset, and the
/// clock applies it one moment before it draws. Nothing writes a local time back into a file
/// name, an mtime or the OS clock.
pub const DEFAULT_UTC_OFFSET_MINUTES: i32 = 0;

/// The furthest any inhabited place is from UTC, in minutes: Baker Island at -12, Kiritimati
/// at +14.
///
/// Written down here rather than taken from `slot2_platform::clock`, because the card owns
/// the *file format* and this crate does not depend on the platform one. The two ranges have
/// to agree — a value a card accepts and the clock then refuses would be a setting nobody can
/// explain — and the task that wires the two together checks exactly that.
pub const UTC_OFFSET_MINUTES_MIN: i32 = -720;
pub const UTC_OFFSET_MINUTES_MAX: i32 = 840;

/// What the frontend itself remembers between runs: `System/slot2.ini`.
///
/// This type owns one key of the file, the volume level. Everything else in it belongs to
/// whoever wrote it — the timezone's display offset, which [`Card::write_utc_offset_minutes`]
/// owns, a future version, or a person with a text editor — and survives a save here
/// untouched, the same way a game's file keeps the keys this version does not know.
///
/// Mute is deliberately *not* stored. It is a runtime toggle, and a machine that came back up
/// silent with nothing on screen to explain why would be worse than one that came back at the
/// level the player chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalSettings {
    /// `0..=100`, as the player sees it. Anything else on the card — a misspelling, a negative
    /// number, a level past the top — reads back as [`DEFAULT_VOLUME_LEVEL`] rather than
    /// refusing to boot or silently becoming 0 or 100.
    pub volume: u8,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        GlobalSettings {
            volume: DEFAULT_VOLUME_LEVEL,
        }
    }
}

impl GlobalSettings {
    pub const KEY_VOLUME: &'static str = "volume";

    /// The settings a written file describes. Only the key this type owns is looked at.
    pub fn from_ini(ini: &Ini) -> GlobalSettings {
        GlobalSettings {
            volume: ini
                .get(Self::KEY_VOLUME)
                .and_then(parse_level)
                .unwrap_or(DEFAULT_VOLUME_LEVEL),
        }
    }

    /// Write this onto `ini`, leaving every key it does not own untouched. The default removes
    /// the key rather than storing it: an untouched machine and a machine set back to 70 are
    /// the same state, and the file should say so by not being there.
    ///
    /// The range is the caller's to have checked — [`Card::write_global_settings`] is the path
    /// that does, and refuses an out-of-range level before any file is touched.
    pub fn apply_to(&self, ini: &mut Ini) {
        if self.volume == DEFAULT_VOLUME_LEVEL {
            ini.remove(Self::KEY_VOLUME);
        } else {
            ini.set(Self::KEY_VOLUME, &self.volume.to_string());
        }
    }
}

/// A volume level as it may be written on a card: whole decimal digits only.
///
/// A leading `+`, a fractional part, a stray space or a negative sign is not a level, and
/// neither is anything past [`MAX_VOLUME_LEVEL`]; all of them mean "no setting here", which
/// reads back as the default.
fn parse_level(s: &str) -> Option<u8> {
    let s = s.trim();
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<u8>().ok().filter(|v| *v <= MAX_VOLUME_LEVEL)
}

/// An offset as it may be written on a card: an optional sign and whole decimal digits.
///
/// Anything else means "no setting here" rather than a corrected one. A fraction of a minute,
/// an exponent, an interior space, a sentence after the number and a value past the range all
/// read back as UTC: clamping would store a decision nobody made, and wrapping would turn a
/// typo into a plausible-looking clock.
fn parse_utc_offset_minutes(s: &str) -> Option<i32> {
    let mins: i32 = s.trim().parse().ok()?;
    (UTC_OFFSET_MINUTES_MIN..=UTC_OFFSET_MINUTES_MAX)
        .contains(&mins)
        .then_some(mins)
}

/// A language code as it may be written on a card, which is to say as it may be spelled as the
/// stem of a file under `System/Lang`.
///
/// This is a *file name* rule and not a language tag rule: the store does not implement BCP-47,
/// does not know which languages exist, and does not care whether the pack is on the card. What
/// it refuses is what could not be a single file in that folder — an empty code, one past the
/// length bound, one with a path separator or a control character in it, `.` or `..`, and any
/// code with whitespace in it, which includes the spaces a caller might wrap one in by mistake.
/// The INI grammar's own padding around a value is the parser's business and is gone before this
/// is asked.
///
/// Everything else is kept exactly as it was spelled: `pt-BR` does not become `pt-br`, and an
/// installed-looking code is not the store's call.
fn parse_language_code(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_LANGUAGE_CODE_BYTES {
        return None;
    }
    if s == "." || s == ".." {
        return None;
    }
    let usable = !s
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '/' || c == '\\');
    usable.then_some(s)
}
impl Card {
    /// What has been set for this game. A missing or unreadable file is no settings, not an
    /// error: a card that cannot be read still has to show a shelf.
    pub fn read_settings(&self, cart: &Cart) -> GameSettings {
        match Ini::load(&self.game_settings_path(cart)) {
            Ok(ini) => GameSettings::from_ini(&ini),
            Err(_) => GameSettings::default(),
        }
    }

    /// Save, keeping any keys a future version wrote. Settings that are back to their
    /// defaults remove the file rather than leaving an empty one behind; a file that is there
    /// and cannot be removed is an error, not a save that happened.
    ///
    /// The file's current contents are read fallibly: a settings file that cannot be read is
    /// not an empty one. Writing would otherwise replace keys this version never saw with
    /// the defaults of the keys it knows, which is somebody's settings thrown away.
    pub fn write_settings(&self, cart: &Cart, s: &GameSettings) -> Result<(), Error> {
        let path = self.game_settings_path(cart);
        let mut ini = Ini::load(&path)?;
        s.apply_to(&mut ini);
        if ini.is_empty() {
            // Nothing to keep: the player is back to the platform's defaults. A missing file
            // is already that state, so it is not a failure — deleting it is the only part
            // that can go wrong, and the caller has to hear about that.
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    return Err(Error::Io(path, e));
                }
            }
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        ini.save(&path)
    }

    /// What the frontend itself remembers. A missing or unreadable file is the defaults, not
    /// an error: a card that cannot be read still has to boot a frontend, and a level is not
    /// worth refusing to start over.
    ///
    /// Reading never creates, rewrites or removes the file.
    pub fn read_global_settings(&self) -> GlobalSettings {
        match Ini::load(&self.settings_path()) {
            Ok(ini) => GlobalSettings::from_ini(&ini),
            Err(_) => GlobalSettings::default(),
        }
    }

    /// Save, keeping every key this version does not own.
    ///
    /// A level outside `0..=100` is the caller's mistake and is refused before the file is
    /// read, let alone written: clamping it would store a decision nobody made. Settings that
    /// are back to their defaults remove the key, and a file left with nothing in it goes
    /// rather than sitting there empty; a file that is there and cannot be removed is an
    /// error, not a save that happened.
    ///
    /// The file's current contents are read fallibly, for the same reason the game's file is:
    /// a settings file that cannot be read is not an empty one, and writing over it would
    /// replace a future version's keys with this one's silence.
    pub fn write_global_settings(&self, s: &GlobalSettings) -> Result<(), Error> {
        if s.volume > MAX_VOLUME_LEVEL {
            return Err(Error::Invalid(format!(
                "volume {} is outside 0..={MAX_VOLUME_LEVEL}",
                s.volume
            )));
        }
        let path = self.settings_path();
        let mut ini = Ini::load(&path)?;
        s.apply_to(&mut ini);
        if ini.is_empty() {
            // Nothing to keep: the frontend is at its defaults, and a missing file is already
            // that state. A file that is not there was the whole of the wanted result, so the
            // only part that can go wrong is a delete, and the caller has to hear about that.
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    return Err(Error::Io(path, e));
                }
            }
            return Ok(());
        }
        ini.save(&path)
    }

    /// The offset the clock adds to UTC before showing a time, in minutes (D-25).
    ///
    /// Its own API rather than a field of [`GlobalSettings`], because the two settings are
    /// saved at different moments: a volume flush that did not know about a timezone would
    /// read the default where the card holds a real offset and write that back. Reading this
    /// is the same shape as reading the volume — a missing file, a missing key, a file that
    /// cannot be read, bytes that are not UTF-8 and a directory where the file belongs all
    /// mean UTC rather than an error — and it never creates, rewrites or removes anything.
    pub fn read_utc_offset_minutes(&self) -> i32 {
        match Ini::load(&self.settings_path()) {
            Ok(ini) => ini
                .get(UTC_OFFSET_MINUTES_KEY)
                .and_then(parse_utc_offset_minutes)
                .unwrap_or(DEFAULT_UTC_OFFSET_MINUTES),
            Err(_) => DEFAULT_UTC_OFFSET_MINUTES,
        }
    }

    /// Save the clock's display offset, keeping every key this one does not own.
    ///
    /// An offset outside the range is the caller's mistake and is refused before the file is
    /// read, let alone written: a card has no business holding minutes the clock would then
    /// refuse. Nothing here touches the system clock, an mtime or the environment variable a
    /// host developer starts the frontend with — the value is a number on a card and stays
    /// one.
    ///
    /// UTC is the absence of the key, so going back to it removes the key and then a file left
    /// with nothing in it. The file's other keys are read fallibly and written back as they
    /// were: a settings file that cannot be read is not an empty one, and a volume this version
    /// cannot parse is the player's volume rather than this write's business.
    pub fn write_utc_offset_minutes(&self, minutes: i32) -> Result<(), Error> {
        if !(UTC_OFFSET_MINUTES_MIN..=UTC_OFFSET_MINUTES_MAX).contains(&minutes) {
            return Err(Error::Invalid(format!(
                "utc offset {minutes} is outside {UTC_OFFSET_MINUTES_MIN}..={UTC_OFFSET_MINUTES_MAX}"
            )));
        }
        let path = self.settings_path();
        let mut ini = Ini::load(&path)?;
        if minutes == DEFAULT_UTC_OFFSET_MINUTES {
            ini.remove(UTC_OFFSET_MINUTES_KEY);
        } else {
            // A plain signed decimal, never a `+`: one spelling per value, so a file that has
            // been rewritten twice is the same file.
            ini.set(UTC_OFFSET_MINUTES_KEY, &minutes.to_string());
        }
        if ini.is_empty() {
            // Nothing to keep: the clock is on UTC, and a missing file is already that state.
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    return Err(Error::Io(path, e));
                }
            }
            return Ok(());
        }
        ini.save(&path)
    }

    /// The language the player chose, as the stem of a pack under `System/Lang`.
    ///
    /// Its own API rather than a field of [`GlobalSettings`], for the same reason the time zone
    /// has one: the volume is written on a delay of its own, and a flush that read the language
    /// as a field would write the default back over a card that holds a real code. A missing file,
    /// a missing key, a file that cannot be read, bytes that are not UTF-8 and a code that could
    /// not be a file name all mean [`DEFAULT_LANGUAGE`] rather than an error — a card that cannot
    /// be read still has to boot, and English is always there — and nothing here creates,
    /// rewrites or removes anything.
    ///
    /// Which languages exist, and whether the pack for this code parses, is not this crate's
    /// question: a card-only pack is placed under `System/Lang` by hand, and the frontend is what
    /// asks `slot2-i18n` whether it can load. Removing a code this build cannot speak would throw
    /// away a choice a later build could honour.
    pub fn read_language(&self) -> String {
        match Ini::load(&self.settings_path()) {
            Ok(ini) => ini
                .get(LANGUAGE_KEY)
                .and_then(parse_language_code)
                .map(str::to_owned)
                .unwrap_or_else(|| DEFAULT_LANGUAGE.to_owned()),
            Err(_) => DEFAULT_LANGUAGE.to_owned(),
        }
    }

    /// Save the player's language choice, keeping every key this one does not own.
    ///
    /// A code that could not be a single file under `System/Lang` is the caller's mistake and is
    /// refused before the file is read, let alone written: a card has no business holding a
    /// spelling the frontend would then have to refuse. The code is written exactly as it was
    /// given — no lowercasing, no tag rewriting — so a file that has been rewritten twice is the
    /// same file.
    ///
    /// English is the absence of the key, because a card that says nothing is already English:
    /// going back to it removes the key and then a file left with nothing in it. The file's other
    /// keys are read fallibly and written back as they were: a settings file that cannot be read
    /// is not an empty one, and a volume this version cannot parse is the player's volume rather
    /// than this write's business.
    pub fn write_language(&self, code: &str) -> Result<(), Error> {
        let code = parse_language_code(code).ok_or_else(|| {
            Error::Invalid(format!(
                "language {code:?} is not a usable pack file stem under System/Lang"
            ))
        })?;
        let path = self.settings_path();
        let mut ini = Ini::load(&path)?;
        if code == DEFAULT_LANGUAGE {
            ini.remove(LANGUAGE_KEY);
        } else {
            ini.set(LANGUAGE_KEY, code);
        }
        if ini.is_empty() {
            // Nothing to keep: the frontend is back to English, and a missing file is already
            // that state.
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    return Err(Error::Io(path, e));
                }
            }
            return Ok(());
        }
        ini.save(&path)
    }
}
