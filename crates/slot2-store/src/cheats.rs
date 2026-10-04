//! Per-game RetroArch cheat files: `System/cheats/<PLAT>/<stem>.cht` (D-21).
//!
//! One file per cart, in the shape libretro-database ships and RetroArch reads:
//!
//! ```text
//! cheats = 2
//!
//! cheat0_desc = "Infinite lives"
//! cheat0_code = "7E007C9A"
//! cheat0_enable = false
//!
//! cheat1_desc = "Max hearts"
//! cheat1_code = "7E13F2FF"
//! cheat1_enable = true
//! ```
//!
//! This module only turns that text into ordered records. Handing a code to the core is
//! `retro_cheat_set`, and whether a code means anything on a platform is the core's business:
//! nothing here is validated, converted, uppercased or re-spelled, and a multi-part code is
//! left as the one string the file wrote.
//!
//! The rules, all of them deliberate:
//!
//! - The declared `cheats = N` decides how many records there are, and they come back in
//!   index order `0..N` whatever order the lines were written in. `N = 0` is the empty file,
//!   and a file with no count at all is not a cheat file.
//! - The count is what is trusted, so every declared index has to be complete: a missing,
//!   out-of-range or duplicated field fails the whole load rather than returning a list the
//!   file does not actually describe. It is checked against the entries the file actually
//!   wrote, and never used as the size of anything before it has been.
//! - Values are double-quoted (what the official database writes) or bare. Inside quotes, `\"`
//!   and `\\` are the only escapes; everything else — `#`, `=`, `+`, spaces, punctuation — is
//!   content, kept exactly as written.
//! - Keys this version does not own are ignored, so a file carrying RetroArch's other fields
//!   (handler, search, rumble…) still loads.
//!
//! Reading is the whole of it: nothing here writes a `.cht`, toggles an entry, or looks for
//! cheats the card does not name.

use std::collections::BTreeMap;
use std::path::Path;

use crate::Error;

/// One cheat, as the card describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cheat {
    /// What the player reads in the menu, exactly as the file spelled it.
    pub description: String,
    /// The code as written: possibly several parts joined by `+`, in the core's own spelling.
    pub code: String,
    /// Whether the file says this cheat is on. A missing `cheatN_enable` is off.
    pub enabled: bool,
}

/// Read one `.cht`. A file that is not there is no cheats, not an error.
pub fn read(path: &Path) -> Result<Vec<Cheat>, Error> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    // Invalid UTF-8 arrives here as an I/O error, which is what it is: bytes this layer cannot
    // decode, from a path the caller has to be told about.
    let text = std::fs::read_to_string(path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
    parse(&text)
}

/// The fields one index has offered so far. `None` is "the file has not written this yet",
/// which is also how a duplicate is noticed.
#[derive(Default)]
struct Entry {
    desc: Option<String>,
    code: Option<String>,
    enable: Option<bool>,
}

fn parse(text: &str) -> Result<Vec<Cheat>, Error> {
    let mut declared: Option<u32> = None;
    let mut entries: BTreeMap<u32, Entry> = BTreeMap::new();

    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // The first `=` is the syntax one; a `=` inside a quoted value comes later and stays
        // content. A line with no `=` at all carries no key, and the settings ini reads the
        // same way.
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        if key == "cheats" {
            let n = parse_count(value, line_no)?;
            if declared.replace(n).is_some() {
                return Err(invalid(line_no, "cheats is declared twice"));
            }
            continue;
        }

        let Some((index, field)) = split_key(key) else {
            continue;
        };
        // Only the three owned fields touch the map. An entry created by anything else would
        // be an index the file never claimed, which is exactly what the gap and range checks
        // below are looking for.
        match field {
            "desc" => {
                let entry = entries.entry(index).or_default();
                if entry.desc.is_some() {
                    return Err(invalid(
                        line_no,
                        &format!("cheat{index}_desc is written twice"),
                    ));
                }
                entry.desc = Some(decode(value, key, line_no)?);
            }
            "code" => {
                let entry = entries.entry(index).or_default();
                if entry.code.is_some() {
                    return Err(invalid(
                        line_no,
                        &format!("cheat{index}_code is written twice"),
                    ));
                }
                entry.code = Some(decode(value, key, line_no)?);
            }
            "enable" => {
                let entry = entries.entry(index).or_default();
                if entry.enable.is_some() {
                    return Err(invalid(
                        line_no,
                        &format!("cheat{index}_enable is written twice"),
                    ));
                }
                entry.enable = Some(parse_enable(&decode(value, key, line_no)?, key, line_no)?);
            }
            // RetroArch's other per-cheat fields: handler, search, rumble and so on. They are
            // not this version's to apply, and they are not an index of a cheat either.
            _ => {}
        }
    }

    let Some(count) = declared else {
        return Err(Error::Invalid(
            "the cheat file does not declare cheats = N".into(),
        ));
    };

    // Validation walks the entries the file actually wrote, in index order. Nothing here is
    // sized or bounded by the declared count: a damaged file claiming four billion cheats and
    // holding none is refused by the first check it cannot pass, not by making room for a
    // list that was never in the file.
    let mut cheats = Vec::with_capacity(entries.len());
    for (index, entry) in entries {
        if index >= count {
            return Err(Error::Invalid(format!(
                "cheat{index} is written but cheats = {count}"
            )));
        }
        if index as usize != cheats.len() {
            // The file skipped an index. Everything below this one is complete, so the first
            // index it never wrote is the one the list has reached.
            let missing = cheats.len();
            return Err(Error::Invalid(format!(
                "cheat{missing}_desc and cheat{missing}_code are missing (cheats = {count})"
            )));
        }
        let Some(description) = entry.desc else {
            return Err(Error::Invalid(format!(
                "cheat{index}_desc is missing (cheats = {count})"
            )));
        };
        let Some(code) = entry.code else {
            return Err(Error::Invalid(format!(
                "cheat{index}_code is missing (cheats = {count})"
            )));
        };
        if description.trim().is_empty() {
            return Err(Error::Invalid(format!("cheat{index}_desc is empty")));
        }
        if code.trim().is_empty() {
            return Err(Error::Invalid(format!("cheat{index}_code is empty")));
        }
        cheats.push(Cheat {
            description,
            code,
            enabled: entry.enable.unwrap_or(false),
        });
    }
    if cheats.len() != count as usize {
        // The declared count is a claim about the file, and this is where it turns out to be
        // wrong: the highest index present is `cheats.len() - 1`, so this one was not written.
        let missing = cheats.len();
        return Err(Error::Invalid(format!(
            "cheat{missing}_desc and cheat{missing}_code are missing (cheats = {count})"
        )));
    }
    Ok(cheats)
}

/// `cheat12_desc` → `(12, "desc")`. Anything else — including RetroArch keys with no index,
/// like a bare `cheat` — is not this version's to read.
fn split_key(key: &str) -> Option<(u32, &str)> {
    let rest = key.strip_prefix("cheat")?;
    let (digits, field) = rest.split_once('_')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((digits.parse::<u32>().ok()?, field))
}

fn parse_count(value: &str, line_no: usize) -> Result<u32, Error> {
    let n = decode(value, "cheats", line_no)?;
    n.trim()
        .parse::<u32>()
        .map_err(|_| invalid(line_no, "cheats is not a whole number"))
}

fn parse_enable(value: &str, key: &str, line_no: usize) -> Result<bool, Error> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(invalid(line_no, &format!("{key} is not true or false"))),
    }
}

/// One value: a double-quoted string with `\"` and `\\` escapes, or bare text taken as it is.
///
/// A value that opens a quote has to close it, and nothing but whitespace may follow: a file
/// that says one thing and then another is not read as the first thing.
fn decode(value: &str, key: &str, line_no: usize) -> Result<String, Error> {
    let Some(inner) = value.strip_prefix('"') else {
        return Ok(value.to_owned());
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if chars.as_str().trim().is_empty() {
                    return Ok(out);
                }
                return Err(invalid(
                    line_no,
                    &format!("{key} has text after its closing quote"),
                ));
            }
            '\\' => match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                _ => return Err(invalid(line_no, &format!("{key} has an unknown escape"))),
            },
            c => out.push(c),
        }
    }
    Err(invalid(
        line_no,
        &format!("{key} opens a quote it never closes"),
    ))
}

fn invalid(line_no: usize, what: &str) -> Error {
    Error::Invalid(format!("line {line_no}: {what}"))
}
