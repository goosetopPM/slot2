//! Write a file so that it is either the old content or the new, never half of the new.
//! Task 07 implements `atomic_write`:
//!
//! - Create the parent directory if missing.
//! - Write `bytes` to `<path>.tmp` (same directory, so the rename is one filesystem),
//!   `sync_all` it, then `rename` over `path`. On Windows `rename` fails if the target
//!   exists, so remove the target first there (`#[cfg(windows)]`); on unix a rename
//!   replaces atomically.
//! - On any error, remove the `.tmp` if it exists and return `Error::Io(path, e)`.
//! - If `path` already holds exactly `bytes`, do nothing and return `Ok(false)`; otherwise
//!   write and return `Ok(true)`. (A frontend flushes save RAM every few seconds; a card
//!   should not wear for identical bytes.)

use std::path::Path;

use crate::Error;

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<bool, Error> {
    let _ = (path, bytes);
    todo!("task 07")
}
