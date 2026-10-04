//! Per-core cheat validation: what a core's own parser will actually take.
//!
//! `retro_cheat_set` returns `void`, so a core that refuses a code says nothing at all: the
//! cheat is silently absent from the menu and the player has no way to tell a bad code from a
//! game that does not honour it. D-21 puts the pre-check here, per core, before the call.
//!
//! Two things make that worth doing rather than letting the core sort it out:
//!
//! - The pinned adapters copy a code into a fixed buffer or reconstruct it at fixed offsets,
//!   so a code that is the wrong shape is not "wrong" so much as a buffer overrun or a
//!   silently mangled unit waiting for a card that holds one. That is a boundary this layer
//!   has to know, because the core will not report it.
//! - A code the core cannot parse is a code the player thinks they turned on. Saying "this
//!   one is not in a form this core reads" before the call is the only chance to say it.
//!
//! What is checked is *syntax*, never whether a code means anything: whether `7E0000:01`
//! affects the game it was written for is the game's business.
//!
//! The answer is per (core, platform) pair, because the same core reads a different grammar
//! for a different console: mGBA's GBA units are not its Game Boy units. A pair the core does
//! not run is refused outright, and a core whose parser has not been read answers
//! [`CheatValidation::Unchecked`] — "nobody checked this" — rather than either verdict.

use crate::{CoreId, Platform};

/// How a core wants its cheats delivered.
///
/// The libretro entry point takes `(index, enabled, code)` and a core is free to use the flag
/// — but two of the pinned adapters here do not. mGBA's `retro_cheat_set` and FCEUmm's both
/// ignore the index and the flag and add every code they are handed to one enabled set, so for
/// those two the only way to leave an entry off is not to hand it over at all. gpSP, SNES9x
/// and Genesis Plus GX read the flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheatDelivery {
    /// Every entry of the file, in order, each with its own `enabled` flag.
    PassAllEntries,
    /// Only the entries that are on. The index handed over is still the file's own, so the
    /// core's idea of which entry is which does not shift when something is left out.
    EnabledEntriesOnly,
}

/// How `core` wants its cheats delivered.
///
/// Independent of [`validate_cheat`]: that one asks whether a code is in a form the core
/// parses, and this asks which entries to send it. A core can be unchecked and still know
/// what to do with an `enabled` flag, and it can validate every code and still ignore it.
pub const fn cheat_delivery(core: CoreId) -> CheatDelivery {
    match core {
        // Both pinned adapters' `retro_cheat_set` ignore `index` and `enabled` before adding
        // the code to an enabled set, so an entry sent as disabled is on.
        CoreId::Mgba | CoreId::Fceumm => CheatDelivery::EnabledEntriesOnly,
        // Gambatte's adapter keeps the code and the enabled flag per index and re-applies only
        // the enabled ones, so the flag it is handed is the flag that decides.
        CoreId::Gambatte | CoreId::Gpsp | CoreId::Snes9x | CoreId::GenesisPlusGx => {
            CheatDelivery::PassAllEntries
        }
    }
}

/// What this crate can say about a cheat code before a core is asked to take it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheatValidation {
    /// The core's own parser was read, and this code is in a form it takes.
    Validated,
    /// Nothing is known here about this core's syntax, so nothing is claimed. The code may
    /// still work perfectly: the frontend simply has no basis to say so.
    Unchecked,
}

/// Why a code was refused.
///
/// The core, the platform and a reason are kept because "which parser, for which console" is
/// the first question anyone asks, and the reason is written for a person reading a log line.
/// The code itself is deliberately not carried: a reason is a sentence about a code, not a
/// copy of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheatValidationError {
    core: CoreId,
    platform: Platform,
    reason: String,
}

impl CheatValidationError {
    /// The core whose syntax refused the code.
    pub fn core(&self) -> CoreId {
        self.core
    }

    /// The platform the code was checked for.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// What was wrong with it, without the code itself.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    fn new(core: CoreId, platform: Platform, reason: impl Into<String>) -> Self {
        CheatValidationError {
            core,
            platform,
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for CheatValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} on {:?}: {}",
            self.core.base_name(),
            self.platform,
            self.reason
        )
    }
}

impl std::error::Error for CheatValidationError {}

/// Check one cheat code against the core that would be handed it, for one console.
///
/// Pure: nothing here reads a file, a core or any state, so the same call always answers the
/// same way and a card full of codes can be checked before a core is loaded.
///
/// The order is deliberate. A core that does not run the console is a caller mistake, and the
/// syntax of a code for it is not a question worth asking — that is answered first. Then the
/// two rules that hold for every core: an empty code is nothing to apply, and a code with a
/// NUL inside it cannot be a C string at all (`Core::set_cheat` refuses it one layer down as
/// well). Only then does the core's own grammar decide, and a core whose grammar this crate
/// has not read answers `Unchecked` instead of guessing.
pub fn validate_cheat(
    core: CoreId,
    platform: Platform,
    code: &str,
) -> Result<CheatValidation, CheatValidationError> {
    if !core.supports_platform(platform) {
        return Err(CheatValidationError::new(
            core,
            platform,
            "this core does not run that console",
        ));
    }

    if code.is_empty() {
        return Err(CheatValidationError::new(
            core,
            platform,
            "the code is empty",
        ));
    }
    if code.as_bytes().contains(&0) {
        return Err(CheatValidationError::new(
            core,
            platform,
            "the code has a NUL inside it",
        ));
    }

    match (core, platform) {
        (CoreId::Snes9x, _) => validate_snes9x(code),
        // The GBA adapter reconstructs each unit at fixed offsets, so the units and the bytes
        // between them are both part of the grammar.
        (CoreId::Mgba, Platform::Gba) => validate_mgba_gba(code),
        // One Game Boy branch, one grammar, for both of the Game Boy's colour depths.
        (CoreId::Mgba, Platform::Gb | Platform::Gbc) => validate_mgba_gb(platform, code),
        (CoreId::Fceumm, Platform::Nes) => validate_fceumm(code),
        // No syntax contract for these yet, and no guesses: the caller is told the code is
        // unchecked, which is the honest answer for a parser this crate has not read.
        _ => Ok(CheatValidation::Unchecked),
    }
}

// ---------------------------------------------------------------- SNES9x

/// The pinned SNES9x adapter's code buffer, minus the byte its terminator needs.
const SNES9X_MAX_BYTES: usize = 255;

/// The separators the adapter splits a code on. Nothing else separates tokens: a tab or a
/// newline ends up *inside* a token and makes it unparseable, which is how those fail here too.
const SNES9X_SEPARATORS: [u8; 5] = *b"+,.; ";

/// SNES Game Genie codes are eight characters from this alphabet, in either case.
const GAME_GENIE_ALPHABET: &[u8] = b"DF4709156BC8A23E";

/// A code the pinned SNES9x adapter takes: one or more tokens it can parse, within its buffer.
fn validate_snes9x(code: &str) -> Result<CheatValidation, CheatValidationError> {
    // Bytes, not characters: the buffer the core copies into is measured in bytes, and a code
    // of eighty Hangul syllables is five times the length it looks.
    if code.len() > SNES9X_MAX_BYTES {
        return Err(CheatValidationError::new(
            CoreId::Snes9x,
            Platform::Snes,
            format!(
                "the code is {} bytes, and the core's buffer takes {SNES9X_MAX_BYTES}",
                code.len()
            ),
        ));
    }

    let mut tokens = 0usize;
    for token in code.split(|c: char| c.is_ascii() && SNES9X_SEPARATORS.contains(&(c as u8))) {
        // A separator run — leading, trailing or several in a row — leaves the core with
        // nothing to parse, and is skipped the same way here.
        if token.is_empty() {
            continue;
        }
        tokens += 1;
        if !snes9x_token_ok(token) {
            return Err(CheatValidationError::new(
                CoreId::Snes9x,
                Platform::Snes,
                format!("token {tokens} is not a code form this core parses"),
            ));
        }
    }

    if tokens == 0 {
        return Err(CheatValidationError::new(
            CoreId::Snes9x,
            Platform::Snes,
            "the code is nothing but separators",
        ));
    }
    Ok(CheatValidation::Validated)
}

/// One token in any of the adapter's three forms, and nothing else.
fn snes9x_token_ok(token: &str) -> bool {
    let b = token.as_bytes();
    match b.len() {
        // Pro Action Replay: eight hex digits.
        8 => b.iter().all(u8::is_ascii_hexdigit),
        // Two forms are nine bytes long, and which one it is is decided by the character in
        // the middle: `AAAAAA:VV` and `XXXX-XXXX`.
        9 if b[6] == b':' => {
            b[..6].iter().all(u8::is_ascii_hexdigit) && b[7..].iter().all(u8::is_ascii_hexdigit)
        }
        9 if b[4] == b'-' => {
            b[..4].iter().all(|c| game_genie_char(*c)) && b[5..].iter().all(|c| game_genie_char(*c))
        }
        _ => false,
    }
}

fn game_genie_char(c: u8) -> bool {
    GAME_GENIE_ALPHABET.contains(&c.to_ascii_uppercase())
}

// ---------------------------------------------------------------- mGBA

/// One byte that separates mGBA's code units: `+`, or one of the six bytes C `isspace` takes
/// in the C locale — space, tab, line feed, vertical tab, form feed and carriage return.
fn mgba_separator(b: u8) -> bool {
    b == b'+' || matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// A code the GBA adapter can reconstruct.
///
/// Its units include the separator that belongs to them — `AAAAAAAA+VVVV` is one CodeBreaker
/// unit, not two tokens — and the adapter rebuilds them at fixed offsets. So the grammar is a
/// sequence of units with exactly one separator byte between them:
///
/// ```text
/// AAAAAAAA+VVVV        CodeBreaker    8 hex, a separator, 4 hex
/// AAAAAAAA+VVVVVVVV    GameShark/PAR  8 hex, a separator, 8 hex
/// AAAAAAAA:VVVVVVVV    VBA 32-bit     8 hex, a colon, 8 hex
/// ```
///
/// Anything that does not consume the whole string is refused: a leading, trailing or doubled
/// separator shifts every later unit, a `+`-joined pair that is not one of those three shapes
/// is a unit the adapter will not find, and the shorter VBA forms are real to the parser but
/// never reach it through this adapter, so they are not forms this frontend hands over.
fn validate_mgba_gba(code: &str) -> Result<CheatValidation, CheatValidationError> {
    if !code.is_ascii() {
        return Err(mgba_non_ascii(Platform::Gba));
    }

    let b = code.as_bytes();
    // `ok[i]` is "the bytes from `i` are a non-empty sequence of units, with one separator
    // between each". Filling it from the end makes the check linear in the code's length,
    // which matters because a card's file is not the only thing that can write one.
    let mut ok = vec![false; b.len() + 1];
    for i in (0..b.len()).rev() {
        for len in [13usize, 17] {
            let Some(unit) = b.get(i..i + len) else {
                continue;
            };
            if !gba_unit_ok(unit) {
                continue;
            }
            let after = i + len;
            // End of the string, or one separator and another unit: not "zero separators", so
            // a doubled or trailing separator never becomes a valid end.
            if after == b.len() || (mgba_separator(b[after]) && ok[after + 1]) {
                ok[i] = true;
            }
        }
    }

    if ok[0] {
        Ok(CheatValidation::Validated)
    } else {
        Err(CheatValidationError::new(
            CoreId::Mgba,
            Platform::Gba,
            "the code is not a sequence of units this core's adapter rebuilds",
        ))
    }
}

/// One 13-byte or 17-byte GBA unit, whatever the byte in position 8 is.
fn gba_unit_ok(unit: &[u8]) -> bool {
    let hex = |s: &[u8]| s.iter().all(u8::is_ascii_hexdigit);
    match unit.len() {
        // CodeBreaker: eight hex, one separator, four hex.
        13 => mgba_separator(unit[8]) && hex(&unit[..8]) && hex(&unit[9..]),
        // GameShark/PAR: eight hex, one separator, eight hex. VBA 32-bit: eight hex, a colon,
        // eight hex.
        17 => hex(&unit[..8]) && (unit[8] == b':' || mgba_separator(unit[8])) && hex(&unit[9..]),
        _ => false,
    }
}

/// A code the Game Boy adapter takes: units of one of five shapes, separated by one byte.
///
/// Unlike the GBA's, a Game Boy unit is one token — nothing inside it is a separator — so the
/// string splits cleanly, and an empty piece is a separator with no unit beside it:
///
/// ```text
/// XXXXXXXX      GameShark
/// XXX-XXX       Game Genie
/// XXX-XXX-XXX   Game Genie with compare
/// XXXXXX-XX     CodeBreaker
/// XXXX:XX       VBA
/// ```
fn validate_mgba_gb(
    platform: Platform,
    code: &str,
) -> Result<CheatValidation, CheatValidationError> {
    if !code.is_ascii() {
        return Err(mgba_non_ascii(platform));
    }

    let mut units = 0usize;
    for unit in code.split(|c: char| c.is_ascii() && mgba_separator(c as u8)) {
        if unit.is_empty() {
            return Err(CheatValidationError::new(
                CoreId::Mgba,
                platform,
                "a separator with no unit on one of its sides",
            ));
        }
        units += 1;
        if !gb_unit_ok(unit.as_bytes()) {
            return Err(CheatValidationError::new(
                CoreId::Mgba,
                platform,
                format!("unit {units} is not a code form this core parses"),
            ));
        }
    }

    Ok(CheatValidation::Validated)
}

/// One Game Boy unit, in any of the adapter's five forms and nothing else.
fn gb_unit_ok(b: &[u8]) -> bool {
    let hex = |s: &[u8]| s.iter().all(u8::is_ascii_hexdigit);
    match b.len() {
        // Game Genie `XXX-XXX` or VBA `XXXX:XX`.
        7 => {
            (b[3] == b'-' && hex(&b[..3]) && hex(&b[4..]))
                || (b[4] == b':' && hex(&b[..4]) && hex(&b[5..]))
        }
        // GameShark `XXXXXXXX`.
        8 => hex(b),
        // CodeBreaker `XXXXXX-XX`.
        9 => b[6] == b'-' && hex(&b[..6]) && hex(&b[7..]),
        // Game Genie with compare `XXX-XXX-XXX`.
        11 => b[3] == b'-' && b[7] == b'-' && hex(&b[..3]) && hex(&b[4..7]) && hex(&b[8..]),
        _ => false,
    }
}

fn mgba_non_ascii(platform: Platform) -> CheatValidationError {
    CheatValidationError::new(
        CoreId::Mgba,
        platform,
        "the core tests every byte for whitespace, so the code has to be ASCII",
    )
}

// ---------------------------------------------------------------- FCEUmm

/// The pinned FCEUmm adapter's code buffer, minus the byte its terminator needs: it copies the
/// input with `strlcpy` into 1024 bytes, so anything longer is silently cut short.
const FCEUMM_MAX_BYTES: usize = 1023;

/// The six bytes the adapter splits a code on. Nothing else separates tokens: a tab or a
/// newline ends up *inside* a token and makes it unparseable, which is how those fail here too.
const FCEUMM_SEPARATORS: [u8; 6] = *b"+,;._ ";

/// NES Game Genie codes are six or eight characters from this alphabet, in either case.
const FCEUMM_GAME_GENIE_ALPHABET: &[u8] = b"APZLGITYEOXUKSVN";

/// A code the pinned FCEUmm adapter takes: one or more tokens it can parse, within its buffer.
///
/// The adapter drops a token it cannot parse and carries on with the rest, which is exactly
/// the quiet half-applied set this check exists to prevent — so one bad token fails the whole
/// code here, while the core would have applied the others.
fn validate_fceumm(code: &str) -> Result<CheatValidation, CheatValidationError> {
    if !code.is_ascii() {
        return Err(CheatValidationError::new(
            CoreId::Fceumm,
            Platform::Nes,
            "the core upper-cases every byte itself, so the code has to be ASCII",
        ));
    }
    if code.len() > FCEUMM_MAX_BYTES {
        return Err(CheatValidationError::new(
            CoreId::Fceumm,
            Platform::Nes,
            format!(
                "the code is {} bytes, and the core's buffer takes {FCEUMM_MAX_BYTES}",
                code.len()
            ),
        ));
    }

    let mut tokens = 0usize;
    for token in code.split(|c: char| c.is_ascii() && FCEUMM_SEPARATORS.contains(&(c as u8))) {
        // A separator run — leading, trailing or several in a row — leaves the core with
        // nothing to parse, and is skipped the same way here.
        if token.is_empty() {
            continue;
        }
        tokens += 1;
        if !fceumm_token_ok(token) {
            return Err(CheatValidationError::new(
                CoreId::Fceumm,
                Platform::Nes,
                format!("token {tokens} is not a code form this core parses"),
            ));
        }
    }

    if tokens == 0 {
        return Err(CheatValidationError::new(
            CoreId::Fceumm,
            Platform::Nes,
            "the code is nothing but separators",
        ));
    }
    Ok(CheatValidation::Validated)
}

/// One token in any of the adapter's four forms, and nothing else.
///
/// ```text
/// AAAA:VV        raw              4 hex, a colon, 2 hex
/// AAAA?CC:VV     raw with compare 4 hex, a question mark, 2 hex, a colon, 2 hex
/// XXXXXX         NES Game Genie   6 or 8 characters of its own alphabet
/// XXXXXXXX       Pro Action Replay 8 hex
/// ```
///
/// The lengths decide which form is even possible, and the alphabets decide between the two
/// that share eight characters. The adapter's own `strtoul` and `sscanf` do not check that
/// their conversion succeeded, and its Pro Action Replay helper answers success on length
/// alone — none of that is repeated here: a digit that is not ASCII hex is refused.
fn fceumm_token_ok(token: &str) -> bool {
    let b = token.as_bytes();
    let hex = |s: &[u8]| s.iter().all(u8::is_ascii_hexdigit);

    match b.len() {
        // Game Genie, six characters: hex has no such form, so the alphabet is the question.
        6 => b.iter().all(|c| fceumm_game_genie_char(*c)),
        // `AAAA:VV`.
        7 => b[4] == b':' && hex(&b[..4]) && hex(&b[5..]),
        // Game Genie, eight characters, or Pro Action Replay: eight hex. The two alphabets
        // overlap — a token of nothing but `A` and `E` is both — and the adapter asks Game
        // Genie first, so which parser a core uses is decided by that overlap. It does not
        // decide acceptance: either reading of such a token is a form this core takes, and
        // the same byte string reaches the same game either way.
        8 => hex(b) || b.iter().all(|c| fceumm_game_genie_char(*c)),
        // `AAAA?CC:VV`.
        10 => b[4] == b'?' && b[7] == b':' && hex(&b[..4]) && hex(&b[5..7]) && hex(&b[8..]),
        _ => false,
    }
}

fn fceumm_game_genie_char(c: u8) -> bool {
    FCEUMM_GAME_GENIE_ALPHABET.contains(&c.to_ascii_uppercase())
}
