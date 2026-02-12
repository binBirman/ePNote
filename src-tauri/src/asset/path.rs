//! Asset 路径管理
//!
//! 负责生成真实文件路径、组织目录结构
//!
//! Asset 文件存储结构：
//! - 正常文件: assets/ab/cd/{uuid}.{ext}
//! - 回收文件: garbages/{logical_day}/{uuid}.{ext}
//!
//! logical_day 格式: YYYYMMDD

use std::path::{Path, PathBuf};

use crate::domain::ids::AssetId;
use crate::path::PhysicalPath;
use crate::util::time::logical_day;

/// Asset 存储路径管理器
#[derive(Debug, Clone)]
pub struct AssetPath {
    root: PathBuf,
}

impl AssetPath {
    /// 创建新的路径管理器
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// 获取根目录
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 获取 assets 目录
    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }

    /// 获取 garbages 目录
    pub fn garbages_dir(&self) -> PathBuf {
        self.root.join("garbages")
    }

    /// 根据 AssetId 生成相对路径（不包含文件名）
    ///
    /// 使用 UUID 前缀分层存储，避免单层目录文件过多
    fn asset_subdir(&self, id: AssetId) -> PathBuf {
        let s = id.0.simple().to_string();

        let p1 = &s[0..2];
        let p2 = &s[2..4];

        PathBuf::from(p1).join(p2)
    }

    /// 生成 Asset 存储路径（不含扩展名）
    pub fn asset_storage_path(&self, id: AssetId) -> PathBuf {
        self.assets_dir().join(self.asset_subdir(id))
    }

    /// 生成完整的 Asset 文件路径
    pub fn asset_file_path(&self, id: AssetId, ext: &str) -> PathBuf {
        let filename = format!("{}.{}", id.0.simple(), ext);
        self.asset_storage_path(id).join(filename)
    }

    /// 生成回收区子目录路径
    ///
    /// 使用 logical_day 作为回收日期标识，便于过期清理
    pub fn garbage_subdir(&self, logical_day: logical_day::LogicalDay) -> PathBuf {
        let day_str = format!("{}", logical_day.0);
        self.garbages_dir().join(&day_str)
    }

    /// 生成回收区文件路径
    ///
    /// 保持原扩展名，文件名不变
    pub fn garbage_file_path(
        &self,
        id: AssetId,
        ext: &str,
        logical_day: logical_day::LogicalDay,
    ) -> PathBuf {
        let filename = format!("{}.{}", id.0.simple(), ext);
        self.garbage_subdir(logical_day).join(filename)
    }

    /// 将 PathBuf 转换为 PhysicalPath
    pub fn to_physical(&self, path: PathBuf) -> PhysicalPath {
        PhysicalPath::new(path)
    }

    /// 生成临时路径（用于文件上传等场景）
    pub fn temp_path(&self, temp_id: &str) -> PathBuf {
        self.root.join("temp").join(temp_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::AssetId;
    use uuid::Uuid;

    #[test]
    fn test_asset_subdir() {
        let asset_path = AssetPath::new(PathBuf::from("/data"));

        // UUID: 12345678-1234-5678-1234-567812345678
        let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());

        let subdir = asset_path.asset_subdir(id);
        assert_eq!(subdir, PathBuf::from("12").join("34"));
    }

    #[test]
    fn test_asset_storage_path() {
        let asset_path = AssetPath::new(PathBuf::from("/data"));

        let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());

        let path = asset_path.asset_storage_path(id);
        assert_eq!(
            path,
            PathBuf::from("/data").join("assets").join("12").join("34")
        );
    }

    #[test]
    fn test_asset_file_path() {
        let asset_path = AssetPath::new(PathBuf::from("/data"));

        let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());

        let path = asset_path.asset_file_path(id, "jpg");
        let expected = PathBuf::from("/data")
            .join("assets")
            .join("12")
            .join("34")
            .join("12345678123456781234567812345678.jpg");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_garbage_subdir() {
        let asset_path = AssetPath::new(PathBuf::from("/data"));

        // LogicalDay(0) corresponds to 0000-01-01 (before shifting)
        let day = logical_day::LogicalDay(738156); // 2024-01-01

        let subdir = asset_path.garbage_subdir(day);
        assert_eq!(
            subdir,
            PathBuf::from("/data").join("garbages").join("738156")
        );
    }

    #[test]
    fn test_garbage_file_path() {
        let asset_path = AssetPath::new(PathBuf::from("/data"));

        let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());
        let day = logical_day::LogicalDay(738156);

        let path = asset_path.garbage_file_path(id, "jpg", day);
        let expected = PathBuf::from("/data")
            .join("garbages")
            .join("738156")
            .join("12345678123456781234567812345678.jpg");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_temp_path() {
        let asset_path = AssetPath::new(PathBuf::from("/data"));

        let path = asset_path.temp_path("upload_123");
        assert_eq!(path, PathBuf::from("/data/temp/upload_123"));
    }
}
