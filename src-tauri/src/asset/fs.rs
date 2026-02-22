use std::fs;
use std::io;
use std::path::Path;

use crate::asset::error::StorageError;
use uuid::Uuid;

/// 确保目标路径的父目录存在，如果不存在则递归创建。
pub fn ensure_parent(path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// 原子拷贝：先拷贝到临时文件再重命名到目标路径。
fn copy_atomic(src: &Path, dst: &Path) -> Result<(), io::Error> {
    let parent = dst
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "destination has no parent"))?;
    fs::create_dir_all(parent)?;
    let tmp = dst.with_extension(format!("tmp-{}", Uuid::new_v4().simple()));
    fs::copy(src, &tmp)?;
    fs::rename(&tmp, dst)?;
    Ok(())
}

/// 在文件系统中移动文件，优先使用 `rename`，在跨设备等场景失败时退回到 copy + remove。
pub fn move_file(src: &Path, dst: &Path) -> Result<(), StorageError> {
    ensure_parent(dst)?;
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => {
            // fallback to copy + remove
            copy_atomic(src, dst).map_err(StorageError::Io)?;
            fs::remove_file(src).map_err(StorageError::Io)?;
            Ok(())
        }
    }
}

/// 将字节写入目标文件，使用临时文件+重命名保证原子性。
pub fn write_bytes_atomic(dst: &Path, data: &[u8]) -> Result<(), StorageError> {
    ensure_parent(dst)?;
    let tmp = dst.with_extension(format!("tmp-{}", Uuid::new_v4().simple()));
    fs::write(&tmp, data).map_err(StorageError::Io)?;
    fs::rename(&tmp, dst).map_err(StorageError::Io)?;
    Ok(())
}

/// 如果存在则删除文件，否则返回 Ok(())。
pub fn remove_file(path: &Path) -> Result<(), StorageError> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
