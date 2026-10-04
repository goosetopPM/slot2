//! The built-in packs' own contract, read from the source the binary ships.
//!
//! `EMBEDDED` holds the `.ftl` text itself, so what is checked here is the file a translator
//! edits: every canonical key is in `ko` as well as `en`, with the same variables and the same
//! button placeholders in the same order, and every placeholder is one the runtime knows. A
//! card pack is deliberately out of scope — it may be partial, and its missing keys fall back
//! to English exactly as before.
//!
//! The scanner is small and explicit on purpose: no regex crate, no production helper. It
//! understands the syntax these two packs use — a message id and its indented continuations,
//! `$variables`, and calls — and it fails on a key rather than skipping a form it does not
//! understand, because silence is how a typo gets shipped.

use std::collections::{BTreeMap, BTreeSet};

use slot2_i18n::{Button, I18n, EMBEDDED, FALLBACK};

/// The canonical number of keys. Both packs carry exactly this many, counted by hand and by
/// the scanner below; a new message belongs in `en` and `ko` together, and this number is
/// part of the contract that says so.
const KEYS: usize = 125;

/// The particle pairs `josa()` implements. From `crates/slot2-i18n/src/josa.rs` (the generic
/// `<받침 있음>/<받침 없음>` rule, with `으로/로` called out) and the same list the `ko` pack
/// documents for its translators.
const JOSA_PAIRS: &[&str] = &[
    "을/를",
    "이/가",
    "은/는",
    "과/와",
    "으로/로",
    "아/야",
    "이여/여",
];

/// Functions a built-in pack may call: the project's two, and the ones `fluent-bundle`
/// provides itself. Anything else is a typo or a missing function, and is reported as such.
const FUNCTIONS: &[&str] = &["BTN", "JOSA", "NUMBER", "DATETIME"];

/// One message: its id and the whole of its value, continuations included.
struct Message {
    id: String,
    body: String,
}

fn source(code: &str) -> &'static str {
    EMBEDDED
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, text)| *text)
        .unwrap_or_else(|| panic!("{code} is not an embedded pack"))
}

/// Every top-level message of a pack, in file order.
///
/// A message starts at a line beginning with `id =` and owns the indented lines that follow it
/// (the select variants of `states-count`, for one). Comments and blank lines are not values.
fn messages(code: &str) -> Vec<Message> {
    let mut out: Vec<Message> = Vec::new();
    let mut open: Option<Message> = None;
    for (number, line) in source(code).lines().enumerate() {
        let number = number + 1;
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indented = line.starts_with(' ') || line.starts_with('\t');
        if !indented && line.starts_with('}') {
            // The closing brace of a multi-line value belongs to the message that opened it.
            let Some(message) = open.as_mut() else {
                panic!("{code}:{number}: a `}}` with no message open");
            };
            message.body.push('\n');
            message.body.push_str(line.trim());
            continue;
        }
        if !indented {
            if let Some(message) = open.take() {
                out.push(message);
            }
            let Some((id, body)) = line.split_once('=') else {
                panic!("{code}:{number}: a line that is neither a message nor a continuation");
            };
            let id = id.trim();
            if !valid_id(id) {
                panic!("{code}:{number}: {id:?} is not a message id");
            }
            if out.iter().any(|m| m.id == id) {
                panic!("{code}:{number}: {id} is defined twice");
            }
            open = Some(Message {
                id: id.to_string(),
                body: body.trim().to_string(),
            });
        } else if let Some(message) = open.as_mut() {
            message.body.push('\n');
            message.body.push_str(trimmed);
        }
        // An unindented line that is not a message start closes the one before it: the `}` that
        // ends a select's variants is the only such line in these packs.
    }
    if let Some(message) = open.take() {
        out.push(message);
    }
    out
}

/// The project's id rule: a lower-case ASCII letter, then lower-case letters, digits and `-`.
fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// The `$variables` a value names, as a set. Order and repetition belong to the language.
fn variables(body: &str, key: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '$' {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < chars.len() && is_ident(chars[end]) {
            end += 1;
        }
        if end == start {
            panic!("{key}: a `$` that names nothing: {body:?}");
        }
        out.insert(chars[start..end].iter().collect());
        i = end;
    }
    out
}

/// A call in a value: its name and its arguments, as written.
struct Call {
    name: String,
    args: Vec<String>,
}

/// Every call a value makes, in order.
fn calls(body: &str, key: &str) -> Vec<Call> {
    let chars: Vec<char> = body.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        if !(chars[i].is_ascii_alphabetic() || chars[i] == '_') {
            i += 1;
            continue;
        }
        if start > 0 && is_ident(chars[start - 1]) {
            // The tail of an identifier already walked over.
            i += 1;
            continue;
        }
        let mut name_end = i;
        while name_end < chars.len() && is_ident(chars[name_end]) {
            name_end += 1;
        }
        let name: String = chars[start..name_end].iter().collect();
        let mut open = name_end;
        while open < chars.len() && chars[open].is_whitespace() {
            open += 1;
        }
        i = name_end;
        if open >= chars.len() || chars[open] != '(' {
            continue;
        }
        let (args, end) = arguments(&chars, open, key, &name);
        out.push(Call { name, args });
        i = end;
    }
    out
}

/// The argument list of a call starting at `open` (its `(`), split on top-level commas.
fn arguments(chars: &[char], open: usize, key: &str, name: &str) -> (Vec<String>, usize) {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut i = open;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '"' => {
                quoted = !quoted;
                if depth > 0 {
                    current.push(c);
                }
            }
            '(' if !quoted => {
                depth += 1;
                if depth > 1 {
                    current.push(c);
                }
            }
            ')' if !quoted => {
                depth -= 1;
                if depth == 0 {
                    if !current.trim().is_empty() {
                        args.push(current.trim().to_string());
                    }
                    return (args, i + 1);
                }
                current.push(c);
            }
            ',' if !quoted && depth == 1 => {
                args.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                if depth > 0 {
                    current.push(c);
                }
            }
        }
        i += 1;
    }
    panic!("{key}: {name}( is not closed");
}

/// The `index`th argument of a call, which must be a quoted string.
fn quoted_arg(call: &Call, key: &str, index: usize) -> String {
    let arg = call.args.get(index).unwrap_or_else(|| {
        panic!(
            "{key}: {} has {} argument(s), wanted #{}: {:?}",
            call.name,
            call.args.len(),
            index,
            call.args
        )
    });
    arg.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| panic!("{key}: {} wants a quoted argument, got {arg:?}", call.name))
        .to_string()
}

/// A call's arguments, which must be exactly `want`.
fn exactly(call: &Call, key: &str, want: usize) {
    assert_eq!(
        call.args.len(),
        want,
        "{key}: {} takes {want} argument(s), got {:?}",
        call.name,
        call.args
    );
}

fn pack(code: &str) -> BTreeMap<String, Message> {
    messages(code)
        .into_iter()
        .map(|message| (message.id.clone(), message))
        .collect()
}

fn canonical() -> BTreeMap<String, Message> {
    pack(FALLBACK)
}

// ---------------------------------------------------------------- keys

#[test]
fn both_packs_carry_the_same_keys_with_no_duplicates() {
    // `messages` rejects a duplicate id and an id outside the naming rule, so reaching the
    // counts below means both packs are well formed.
    let en = messages(FALLBACK);
    let ko = messages("ko");
    assert_eq!(en.len(), KEYS, "en does not carry every canonical key");
    assert_eq!(ko.len(), KEYS, "ko does not carry every canonical key");

    let en_keys: BTreeSet<&String> = en.iter().map(|m| &m.id).collect();
    let ko_keys: BTreeSet<&String> = ko.iter().map(|m| &m.id).collect();
    let missing: Vec<&String> = en_keys.difference(&ko_keys).copied().collect();
    let extra: Vec<&String> = ko_keys.difference(&en_keys).copied().collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "ko is missing {missing:?} and invented {extra:?}"
    );
}

#[test]
fn only_the_english_font_line_may_be_empty() {
    for code in [FALLBACK, "ko"] {
        for message in messages(code) {
            let body = message.body.trim();
            assert!(
                !body.is_empty(),
                "{code}: {} has no value at all",
                message.id
            );
            let empty = body == "\"\"" || body == "{ \"\" }";
            if empty {
                assert_eq!(
                    (code, message.id.as_str()),
                    (FALLBACK, "lang-font"),
                    "only the fallback pack's `lang-font` may be empty"
                );
            }
        }
    }
}

// ---------------------------------------------------------------- variables and buttons

#[test]
fn variables_match_between_the_packs() {
    let en = canonical();
    let ko = pack("ko");
    for (id, message) in &en {
        let other = &ko[id];
        let want = variables(&message.body, id);
        let got = variables(&other.body, id);
        assert_eq!(
            want, got,
            "{id}: en names {want:?} and ko names {got:?} — a translation must keep every \
             variable and invent none"
        );
    }
}

#[test]
fn button_placeholders_match_and_are_real_buttons() {
    let en = canonical();
    let ko = pack("ko");
    for (id, message) in &en {
        let want: Vec<String> = calls(&message.body, id)
            .iter()
            .filter(|c| c.name == "BTN")
            .map(|c| {
                exactly(c, id, 1);
                quoted_arg(c, id, 0)
            })
            .collect();
        let got: Vec<String> = calls(&ko[id].body, id)
            .iter()
            .filter(|c| c.name == "BTN")
            .map(|c| {
                exactly(c, id, 1);
                quoted_arg(c, id, 0)
            })
            .collect();
        assert_eq!(
            want, got,
            "{id}: the button placeholders differ between the packs"
        );
        for button in &want {
            assert!(
                Button::from_id(button).is_some(),
                "{id}: BTN(\"{button}\") is not a button the frontend knows"
            );
        }
    }
}

#[test]
fn only_korean_uses_the_runtime_particles() {
    let en = canonical();
    let ko = pack("ko");
    for (id, message) in &en {
        assert!(
            calls(&message.body, id).iter().all(|c| c.name != "JOSA"),
            "{id}: English cannot use JOSA — the particle is the Korean pack's own"
        );
    }
    for (id, message) in &ko {
        for call in calls(&message.body, id).iter().filter(|c| c.name == "JOSA") {
            exactly(call, id, 2);
            let word = &call.args[0];
            let name = word
                .strip_prefix('$')
                .unwrap_or_else(|| panic!("{id}: JOSA wants a variable, got {word:?}"));
            assert!(
                variables(&message.body, id).contains(name),
                "{id}: JOSA(\"{word}\") names a variable the message does not use"
            );
            let pair = quoted_arg(call, id, 1);
            assert!(
                JOSA_PAIRS.contains(&pair.as_str()),
                "{id}: JOSA pair {pair:?} is not one the runtime implements ({JOSA_PAIRS:?})"
            );
        }
    }
}

#[test]
fn no_pack_carries_a_literal_button_cap() {
    for code in [FALLBACK, "ko"] {
        for message in messages(code) {
            for button in Button::ALL {
                let literal = format!("[{}]", button.label());
                assert!(
                    !message.body.contains(&literal),
                    "{code}: {} spells the cap {literal} out instead of using BTN(\"{}\")",
                    message.id,
                    button.id()
                );
            }
        }
    }
}

#[test]
fn every_call_is_one_the_runtime_has() {
    for code in [FALLBACK, "ko"] {
        for message in messages(code) {
            for call in calls(&message.body, &message.id) {
                assert!(
                    FUNCTIONS.contains(&call.name.as_str()),
                    "{code}: {} calls {}(), which the runtime does not provide",
                    message.id,
                    call.name
                );
            }
        }
    }
}

// ---------------------------------------------------------------- the packs load

#[test]
fn both_packs_load_and_know_every_key() {
    for code in [FALLBACK, "ko"] {
        let i18n = I18n::embedded(code).unwrap_or_else(|e| panic!("{code} does not load: {e}"));
        assert_eq!(i18n.code(), code);
        // The marker a missing key renders as, so the check below means what it says.
        let marker = i18n.t("no-such-key");
        assert_eq!(marker, "[no-such-key]");
        for id in canonical().keys() {
            assert_ne!(
                i18n.t(id),
                format!("[{id}]"),
                "{code}: {id} is not in this pack"
            );
        }
    }
}
