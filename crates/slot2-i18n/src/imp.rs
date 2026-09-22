//! The Fluent side: bundles, custom functions, formatting, span splitting.
//!
//! Implementation notes for task 01 (see tasks/01-i18n.md):
//! - `Inner` holds the chosen language's `FluentBundle<FluentResource>` and, when the code
//!   is not `en`, a second bundle for `en` as fallback. Lookup tries the language first.
//! - Register two functions on every bundle:
//!   - `JOSA(word, pair)` → `crate::josa::attach(word, pair)`. `word` may arrive as a
//!     string or a number (numbers are formatted plainly, no grouping).
//!   - `BTN(id)` → a marker the span splitter can find later: `"\u{E000}<id>\u{E001}"`.
//!     Private-use code points, so they never collide with real text.
//! - Unicode isolation marks: create bundles with `FluentBundle::new_concurrent` or set
//!   `set_use_isolating(false)` so no U+2068/U+2069 appear in output.
//! - `format` renders and then replaces each marker with `[LABEL]` via `Button::label`;
//!   an unknown id renders as `[?id]`. Missing key (in both bundles) → `[key]`.
//! - `spans` renders the same way but splits at the markers into `Span::Btn` / `Span::Text`,
//!   merging adjacent text and dropping empty text. Unknown ids become text `[?id]`.
//! - Parse errors: `FluentResource::try_new` returns `(resource, errors)`; treat errors as
//!   fatal for that file and report the first one in `Error::Parse` with the file name.

use std::path::Path;

use crate::{Arg, Error, Span};
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

type Bundle = FluentBundle<FluentResource>;

pub struct Inner {
    code: String,
    bundle: Bundle,
    fallback: Option<Bundle>,
}

impl Inner {
    pub fn code(&self) -> &str {
        &self.code
    }

    fn raw_format(&self, key: &str, args: &[(&str, Arg)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            let val = match v {
                Arg::Str(s) => FluentValue::from(s.to_string()),
                Arg::Num(n) => {
                    // Default formatting already prints `2.0` as `2`; forcing zero fraction
                    // digits through the options leaves a trailing `.` in this version.
                    let opts = fluent_bundle::types::FluentNumberOptions {
                        use_grouping: false,
                        ..Default::default()
                    };
                    FluentValue::Number(fluent_bundle::types::FluentNumber::new(*n, opts))
                }
            };
            fluent_args.set(*k, val);
        }

        let mut res = None;
        if let Some(msg) = self.bundle.get_message(key) {
            if let Some(pattern) = msg.value() {
                let mut errors = vec![];
                let s = self
                    .bundle
                    .format_pattern(pattern, Some(&fluent_args), &mut errors);
                res = Some(s.into_owned());
            }
        }

        if res.is_none() {
            if let Some(fallback) = &self.fallback {
                if let Some(msg) = fallback.get_message(key) {
                    if let Some(pattern) = msg.value() {
                        let mut errors = vec![];
                        let s = fallback.format_pattern(pattern, Some(&fluent_args), &mut errors);
                        res = Some(s.into_owned());
                    }
                }
            }
        }

        res.unwrap_or_else(|| format!("[{}]", key))
    }

    pub fn format(&self, key: &str, args: &[(&str, Arg)]) -> String {
        let s = self.raw_format(key, args);
        let mut out = String::new();
        let mut current = s.as_str();
        while let Some(start) = current.find('\u{E000}') {
            out.push_str(&current[..start]);
            current = &current[start + 3..];
            if let Some(end) = current.find('\u{E001}') {
                let id = &current[..end];
                if let Some(btn) = crate::button::Button::from_id(id) {
                    out.push_str(&format!("[{}]", btn.label()));
                } else {
                    out.push_str(&format!("[?{}]", id));
                }
                current = &current[end + 3..];
            } else {
                break;
            }
        }
        out.push_str(current);
        out
    }

    pub fn spans(&self, key: &str, args: &[(&str, Arg)]) -> Vec<Span> {
        let s = self.raw_format(key, args);
        let mut out = Vec::new();
        let mut current = s.as_str();

        let push_text = |t: &str, list: &mut Vec<Span>| {
            if !t.is_empty() {
                if let Some(Span::Text(last)) = list.last_mut() {
                    last.push_str(t);
                } else {
                    list.push(Span::Text(t.to_string()));
                }
            }
        };

        while let Some(start) = current.find('\u{E000}') {
            push_text(&current[..start], &mut out);
            current = &current[start + 3..];
            if let Some(end) = current.find('\u{E001}') {
                let id = &current[..end];
                if let Some(btn) = crate::button::Button::from_id(id) {
                    out.push(Span::Btn(btn));
                } else {
                    push_text(&format!("[?{}]", id), &mut out);
                }
                current = &current[end + 3..];
            } else {
                break;
            }
        }
        push_text(current, &mut out);
        out
    }
}

fn load_bundle(code: &str, card_lang_dir: Option<&Path>) -> Result<Option<Bundle>, Error> {
    let mut builtin_src = None;
    for &(emb_code, emb_src) in crate::EMBEDDED {
        if emb_code == code {
            builtin_src = Some(emb_src);
            break;
        }
    }

    let mut card_src = None;
    if let Some(dir) = card_lang_dir {
        let path = dir.join(format!("{}.ftl", code));
        if path.is_file() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                card_src = Some((path, s));
            }
        }
    }

    if builtin_src.is_none() && card_src.is_none() {
        return Ok(None);
    }

    let langid: LanguageIdentifier = code.parse().unwrap_or_else(|_| "en-US".parse().unwrap());
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.set_use_isolating(false);

    bundle
        .add_function("JOSA", |positional, _named| {
            let word_str = match positional.first() {
                Some(FluentValue::String(w)) => w.to_string(),
                Some(FluentValue::Number(n)) => n.as_string().to_string(),
                _ => String::new(),
            };
            let pair = match positional.get(1) {
                Some(FluentValue::String(p)) => p.to_string(),
                _ => String::new(),
            };
            FluentValue::from(crate::josa::attach(&word_str, &pair))
        })
        .unwrap();

    bundle
        .add_function("BTN", |positional, _named| {
            let id = match positional.first() {
                Some(FluentValue::String(s)) => s.to_string(),
                _ => return FluentValue::Error,
            };
            FluentValue::from(format!("\u{E000}{id}\u{E001}"))
        })
        .unwrap();

    if let Some(src) = builtin_src {
        let res = match FluentResource::try_new(src.to_string()) {
            Ok(r) => r,
            Err((_, mut errs)) => {
                return Err(Error::Parse(format!("{}.ftl: {}", code, errs.remove(0))));
            }
        };
        bundle.add_resource(res).unwrap();
    }

    if let Some((path, src)) = card_src {
        let res = match FluentResource::try_new(src) {
            Ok(r) => r,
            Err((_, mut errs)) => {
                let filename = path.file_name().unwrap().to_string_lossy();
                return Err(Error::Parse(format!("{}: {}", filename, errs.remove(0))));
            }
        };
        bundle.add_resource_overriding(res);
    }

    Ok(Some(bundle))
}

pub fn load(code: &str, card_lang_dir: Option<&Path>) -> Result<Inner, Error> {
    let main_bundle = match load_bundle(code, card_lang_dir)? {
        Some(b) => b,
        None => return Err(Error::UnknownLanguage(code.to_string())),
    };

    let fallback = if code != "en" {
        load_bundle("en", card_lang_dir)?
    } else {
        None
    };

    Ok(Inner {
        code: code.to_string(),
        bundle: main_bundle,
        fallback,
    })
}

pub fn available(card_lang_dir: Option<&Path>) -> Vec<String> {
    let mut langs = Vec::new();
    for (code, _) in crate::EMBEDDED {
        langs.push(code.to_string());
    }
    if let Some(dir) = card_lang_dir {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("ftl") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                langs.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    langs.sort();
    langs.dedup();
    langs
}
