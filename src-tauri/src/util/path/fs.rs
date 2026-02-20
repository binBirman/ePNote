use std::fs;
use std::path::Path;

use crate::util::path::error::StorageError;

/// 确保目标路径的父目录存在，如果不存在则递归创建。
pub fn ensure_parent(path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// 在文件系统中移动文件，并确保目标父目录存在。
///
/// 语义：如果目标父目录不存在会先创建，然后执行重命名（移动）。
pub fn move_file(src: &Path, dst: &Path) -> Result<(), StorageError> {
    ensure_parent(dst)?;
    fs::rename(src, dst)?;
    Ok(())
}
