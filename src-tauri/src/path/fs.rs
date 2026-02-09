use std::fs;
use std::path::Path;

use crate::path::error::StorageError;

pub fn ensure_parent(path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn move_file(src: &Path, dst: &Path) -> Result<(), StorageError> {
    ensure_parent(dst)?;
    fs::rename(src, dst)?;
    Ok(())
}
