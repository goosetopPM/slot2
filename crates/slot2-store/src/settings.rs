//! What a player has changed about one game, kept beside the card's other state.
//!
//! One file per cart at `System/games/<PLAT>/<stem>.ini`, holding only what was actually
//! overridden. A game nobody has touched has no file at all, and every unset key falls back
//! to the platform's default — so a card's settings folder reads as a list of decisions
//! somebody made, not a dump of everything that could be set.
//!
//! The file is plain `key=value`, editable on a PC with a text editor, which is the only
//! way to reach these until the in-game menu exists. Unknown keys are left alone when the
//! file is rewritten: a newer SLOT2 writing a key this one has never heard of must not lose
//! it on the next save.

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
}

impl GameSettings {
    pub const KEY_CORE: &'static str = "core";
    pub const KEY_SCALE: &'static str = "scale";
    pub const KEY_OVERSCAN: &'static str = "overscan";

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
    /// defaults remove the file rather than leaving an empty one behind.
    pub fn write_settings(&self, cart: &Cart, s: &GameSettings) -> Result<(), Error> {
        let path = self.game_settings_path(cart);
        let mut ini = Ini::load(&path).unwrap_or_else(|_| Ini::parse(""));
        s.apply_to(&mut ini);
        if ini.is_empty() {
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        ini.save(&path)
    }
}
