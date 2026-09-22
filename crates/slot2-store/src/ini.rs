//! The settings file format: `key = value` lines, `#` comments, UTF-8. No sections. Task 07
//! implements the four methods.
//!
//! - `parse`: trim each line; skip empty and `#` lines; split at the first `=`; trim both
//!   sides. A line without `=` is ignored. Later keys override earlier ones. Keys are
//!   case-sensitive.
//! - `to_string`: `key = value\n` per entry, keys sorted, so a rewrite is stable and diffs
//!   are small. Comments are not preserved (settings are machine-written).
//! - `load(path)`: missing file → empty `Ini`, unreadable → `Error::Io`.
//! - `save(path)`: through `atomic_write`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ini {
    map: BTreeMap<String, String>,
}

impl Ini {
    pub fn parse(text: &str) -> Ini {
        let _ = text;
        todo!("task 07")
    }

    pub fn load(path: &Path) -> Result<Ini, Error> {
        let _ = path;
        todo!("task 07")
    }

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        let _ = path;
        todo!("task 07")
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_owned(), value.to_owned());
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.map.remove(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl std::fmt::Display for Ini {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in &self.map {
            writeln!(f, "{k} = {v}")?;
        }
        Ok(())
    }
}
