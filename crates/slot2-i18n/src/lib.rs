//! Fluent bundles, JOSA/BTN functions, language pack discovery.
//!
//! One `I18n` per running frontend, rebuilt when the language changes. Strings come out as
//! either plain `String` (`t`, `t_args`) or as a list of `Span`s (`spans`) when a message
//! contains `BTN(...)` placeholders the UI must draw as button caps in place.
//!
//! Language packs are Fluent `.ftl` files. `en` and `ko` are compiled in; a card may add or
//! override any language with `System/Lang/<code>.ftl`, which loads on top of the built-in
//! text for that code (card wins) with `en` underneath as the fallback for missing keys.
//!
//! Design: docs/DESIGN.md §8, decisions D-12, D-13.

pub mod button;
pub mod josa;

pub use button::Button;
pub use josa::josa;

use std::fmt;
use std::path::Path;

/// One piece of a rendered message. Text is drawn with the font chain; a button is drawn
/// as that button's cap glyph, at that position in the sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Span {
    Text(String),
    Btn(Button),
}

/// A value passed to a message. Mirrors the two Fluent value kinds the UI needs.
#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    Str(String),
    Num(f64),
}

impl From<&str> for Arg {
    fn from(s: &str) -> Self {
        Arg::Str(s.to_owned())
    }
}
impl From<String> for Arg {
    fn from(s: String) -> Self {
        Arg::Str(s)
    }
}
impl From<i64> for Arg {
    fn from(n: i64) -> Self {
        Arg::Num(n as f64)
    }
}
impl From<u32> for Arg {
    fn from(n: u32) -> Self {
        Arg::Num(n as f64)
    }
}
impl From<usize> for Arg {
    fn from(n: usize) -> Self {
        Arg::Num(n as f64)
    }
}

#[derive(Debug)]
pub enum Error {
    /// No built-in pack and no card pack for this code.
    UnknownLanguage(String),
    /// A `.ftl` did not parse. The message names the file and the first parse error.
    Parse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownLanguage(c) => write!(f, "unknown language {c}"),
            Error::Parse(m) => write!(f, "ftl parse error: {m}"),
        }
    }
}
impl std::error::Error for Error {}

/// Built-in packs, compiled in from `assets/lang/`. Order matters nowhere; lookup is by code.
pub const EMBEDDED: &[(&str, &str)] = &[
    ("en", include_str!("../../../assets/lang/en.ftl")),
    ("ko", include_str!("../../../assets/lang/ko.ftl")),
];

/// The code every other language falls back to for keys it lacks.
pub const FALLBACK: &str = "en";

/// A loaded language: its bundle plus the fallback bundle.
pub struct I18n {
    inner: imp::Inner,
}

impl fmt::Debug for I18n {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("I18n")
            .field("code", &self.inner.code())
            .finish_non_exhaustive()
    }
}

impl I18n {
    /// Load a built-in language by code (`en`, `ko`). `en` is also loaded as the fallback.
    pub fn embedded(code: &str) -> Result<Self, Error> {
        Self::load(code, None)
    }

    /// Load `code` from the built-ins and, if `card_lang_dir` holds `<code>.ftl`, layer that
    /// on top (card text wins for keys both define). A card-only language — a code with no
    /// built-in — is fine: it loads over `en`.
    pub fn load(code: &str, card_lang_dir: Option<&Path>) -> Result<Self, Error> {
        imp::load(code, card_lang_dir).map(|inner| I18n { inner })
    }

    /// Codes available: built-ins plus any `<code>.ftl` in `card_lang_dir`, deduplicated,
    /// sorted. What the language picker lists.
    pub fn available(card_lang_dir: Option<&Path>) -> Vec<String> {
        imp::available(card_lang_dir)
    }

    pub fn code(&self) -> &str {
        self.inner.code()
    }

    /// The pack's `lang-name`, for the picker ("한국어").
    pub fn name(&self) -> String {
        self.t("lang-name")
    }

    /// The pack's preferred UI font file name, if it names one (`lang-font`); `None` when
    /// the message is empty or missing.
    pub fn font(&self) -> Option<String> {
        let f = self.t("lang-font");
        let f = f.trim();
        if f.is_empty() || f.starts_with('[') {
            None
        } else {
            Some(f.to_owned())
        }
    }

    /// A message with no arguments. A missing key renders as `[key]` — visible, never a
    /// panic — after trying the fallback language.
    pub fn t(&self, key: &str) -> String {
        self.t_args(key, &[])
    }

    /// A message with arguments. Any `BTN(...)` in the result is rendered as the button's
    /// short label in square brackets, e.g. `[MENU]`, which is what a log line or a toast
    /// with no cap glyphs should show. The UI uses `spans` instead.
    pub fn t_args(&self, key: &str, args: &[(&str, Arg)]) -> String {
        self.inner.format(key, args)
    }

    /// The message split at its `BTN(...)` placeholders, in order. Adjacent text is merged
    /// into one `Span::Text`; there are never two `Text` spans in a row and never an empty
    /// one. A message with no buttons is a single `Text` span.
    pub fn spans(&self, key: &str, args: &[(&str, Arg)]) -> Vec<Span> {
        self.inner.spans(key, args)
    }
}

mod imp;
