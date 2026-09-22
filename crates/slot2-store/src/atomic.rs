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
    if let Ok(existing) = std::fs::read(path) {
        if existing == bytes {
            return Ok(false);
        }
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Io(parent.to_path_buf(), e))?;
        }
    }

    let mut tmp_os = path.as_os_str().to_os_string();
    tmp_os.push(".tmp");
    let tmp_path = std::path::PathBuf::from(tmp_os);

    let res = (|| {
        {
            let mut f = std::fs::File::create(&tmp_path)?;
            use std::io::Write;
            f.write_all(bytes)?;
            f.sync_all()?;
        }

        #[cfg(windows)]
        {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }

        std::fs::rename(&tmp_path, path)?;
        Ok(())
    })();

    match res {
        Ok(_) => Ok(true),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp_path);
            Err(Error::Io(path.to_path_buf(), e))
        }
    }
}
