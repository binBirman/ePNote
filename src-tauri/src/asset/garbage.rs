//! 回收区管理
//!
//! 职责：
//!   - 管理回收区
//!   - 支持移动到回收区（由 AssetStore 提供）
//!   - 扫描回收区
//!   - 清理过期文件
//!
//! 注意：
//!   - 不允许自动删除
//!   - 清理需要显式调用

use crate::asset::error::StorageError;
use crate::asset::fs::{ensure_parent, move_file, remove_file};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::Uuid;

/// 回收区条目
#[derive(Debug, Clone)]
pub struct GarbageEntry {
    /// 如果文件名可解析出 uuid，则填入，否则为 None
    pub id: Option<Uuid>,
    /// 回收区下的相对路径（相对于存储根目录）
    pub recycle_relative: PathBuf,
    /// 原始相对路径（如果可得），目前可能为 None
    pub original_relative: Option<PathBuf>,
    pub size: u64,
    pub moved_at: SystemTime,
}

/// 扫描回收区，返回所有在 `{root}/garbages` 下的文件条目。
pub fn scan_recycle(root: &Path) -> Result<Vec<GarbageEntry>, StorageError> {
    let recycle_root = root.join("garbages");
    if !recycle_root.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();

    fn visit(dir: &Path, root: &Path, out: &mut Vec<GarbageEntry>) -> Result<(), StorageError> {
        for entry in fs::read_dir(dir)? {
            let e = entry?;
            let p = e.path();
            if p.is_dir() {
                visit(&p, root, out)?;
            } else if p.is_file() {
                let md = fs::metadata(&p)?;
                let size = md.len();
                let moved_at = md.modified().unwrap_or(SystemTime::now());

                // 尝试从文件名解析 UUID
                let id = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| Uuid::parse_str(s).ok());

                let recycle_relative = p
                    .strip_prefix(root)
                    .map(|x| x.to_path_buf())
                    .unwrap_or_else(|_| p.clone());

                out.push(GarbageEntry {
                    id,
                    recycle_relative,
                    original_relative: None,
                    size,
                    moved_at,
                });
            }
        }
        Ok(())
    }

    visit(&recycle_root, root, &mut out)?;
    Ok(out)
}

/// 列出符合 `cutoff`（早于该时间点）的可清理回收条目。
pub fn list_deletable(root: &Path, cutoff: SystemTime) -> Result<Vec<GarbageEntry>, StorageError> {
    let all = scan_recycle(root)?;
    Ok(all.into_iter().filter(|g| g.moved_at < cutoff).collect())
}

/// 根据回收条目显式删除文件（会删除文件本身，但不会递归删除目录，若需可扩展）。
///
/// 注意：此函数为显式删除接口，不会进行时间过滤；调用方负责权限与确认。
pub fn delete_entries(root: &Path, entries: &[GarbageEntry]) -> Result<(), StorageError> {
    for e in entries {
        let p = root.join(&e.recycle_relative);
        // 仅删除文件
        if p.exists() {
            remove_file(&p)?;
        }
    }
    Ok(())
}
