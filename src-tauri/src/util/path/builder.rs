use std::path::PathBuf;
use uuid::Uuid;

use crate::util::path::{error::PathError, sanitize::sanitize_filename, StorageLayout};

/// 路径构建器：负责在给定 `StorageLayout` 下安全地构建文件路径。
#[derive(Debug, Clone)]
pub struct PathBuilder {
    layout: StorageLayout,
}

impl PathBuilder {
    /// 使用给定的 `StorageLayout` 创建 `PathBuilder`。
    pub fn new(layout: StorageLayout) -> Self {
        Self { layout }
    }

    /// 返回引用到内部 `StorageLayout`。
    pub fn layout(&self) -> &StorageLayout {
        &self.layout
    }

    /// 根据 `id` 与扩展名 `ext` 构建资产文件的完整 `PathBuf`。
    ///
    /// 在构建前会对扩展名做 `sanitize` 校验，避免包含路径分隔符或非法字符。
    pub fn build_asset_path(&self, id: &Uuid, ext: &str) -> Result<PathBuf, PathError> {
        let safe_ext = sanitize_filename(ext)?;

        let path: PathBuf = self.layout.asset_file(id, &safe_ext);

        Ok(path)
    }
}
