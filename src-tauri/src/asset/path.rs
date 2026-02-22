//! Asset 路径管理
//!
//! 负责生成真实文件路径、组织目录结构
//!
/// Asset 文件存储结构：
/// - 正常文件（按月份分桶）: assets/{YYYY}/{MM}/{timestamp}_{uuid}.{ext}
/// - 回收文件: garbages/{logical_day}/{uuid}.{ext}
use chrono::{TimeZone, Utc};
use std::path::PathBuf;
use uuid::Uuid;

use crate::asset::{error::*, sanitize::sanitize_filename};

/// 存储布局，用于将 UUID 分桶到稳定的目录结构下。
///
/// 例如：`/root/assets/ab/cd/ab...` 这种方式可以避免单目录过多文件。
#[derive(Debug, Clone)]
pub struct StorageLayout {
    root: PathBuf,
}

impl StorageLayout {
    /// 创建一个新的 `StorageLayout`，`root` 是存储根目录。
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// 返回配置的根目录引用。
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// 根据时间戳（毫秒）生成按月分桶相对目录（不包含文件名），例如 `assets/{type}/2026/02`。
    /// 新生成的路径均返回相对路径，调用方可通过 `StorageLayout::root()` 与相对路径拼接得到绝对路径。
    pub fn asset_dir_by_ts(&self, ts_millis: i64) -> PathBuf {
        let dt = Utc.timestamp_millis_opt(ts_millis).unwrap();
        let y = dt.format("%Y").to_string();
        let m = dt.format("%m").to_string();

        PathBuf::from("assets").join(y).join(m)
    }
}

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

    /// 构建资产文件的相对路径 `PathBuf`（相对于存储根 `root`）。
    /// 根据时间戳、id 与扩展名 ext 生成文件名，格式：{timestamp_ms}_{uuid}.{ext}
    /// 在构建前会对扩展名做 `sanitize` 校验，避免包含路径分隔符或非法字符。
    pub fn build_asset_path(&self, id: &Uuid, ext: &str) -> Result<PathBuf, PathError> {
        let safe_ext = sanitize_filename(ext)?;

        // 使用时间戳作为分桶依据（按月），并将同一时间戳用于文件名以保持一致性
        let ts = Utc::now().timestamp_millis();
        let filename = format!("{}_{}.{}", ts, id.simple(), safe_ext);

        // 返回相对路径：assets/{type}/{YYYY}/{MM}/{filename}
        Ok(self.layout.asset_dir_by_ts(ts).join(filename))
    }

    /// 将逻辑路径转换为物理路径，确保文件名安全。
    pub fn build_physical_path(&self, logical: &PathBuf) -> Result<PathBuf, PathError> {
        let file_name = logical
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| LogicalPathError::InvalidLogicalPath)?;

        let safe_file_name = sanitize_filename(file_name)?;

        Ok(self.layout.root().join(safe_file_name))
    }
}
