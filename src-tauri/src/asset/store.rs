//! Asset 存储管理
//!
//! 核心结构：AssetStore
//! 职责：
//!     - 保存资源
//!     - 移动到回收区
//!     - 生成资源路径

use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::asset::path::AssetPath;
use crate::domain::ids::AssetId;
use crate::path::{ensure_parent, StorageError};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

/// Asset 存储管理器
///
/// 负责文件的物理存储操作，不包含业务逻辑
#[derive(Debug, Clone)]
pub struct AssetStore {
    path_manager: AssetPath,
}

impl AssetStore {
    /// 创建新的存储管理器
    pub fn new(path_manager: AssetPath) -> Self {
        Self { path_manager }
    }

    /// 获取路径管理器
    pub fn path(&self) -> &AssetPath {
        &self.path_manager
    }

    /// 从路径提取扩展名
    ///
    /// 返回小写的扩展名（不含点号），如果没有扩展名则返回空字符串
    pub(crate) fn extract_extension(file_path: &Path) -> String {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default()
    }

    /// 保存多个文件到存储
    ///
    /// 参数：
    ///   - file_paths: 原始文件路径列表
    ///
    /// 返回：
    ///   - Vec<AssetId>: 生成的 AssetId 列表
    ///   - Vec<String>: 存储后的相对路径列表
    ///
    /// 过程：
    ///   1. 为每个文件生成 AssetId (UUID)
    ///   2. 计算存储路径
    ///   3. 确保目录存在
    ///   4. 复制文件到存储位置
    pub fn save_many(
        &self,
        file_paths: &[PathBuf],
    ) -> Result<(Vec<AssetId>, Vec<String>), StorageError> {
        let mut asset_ids = Vec::with_capacity(file_paths.len());
        let mut stored_paths = Vec::with_capacity(file_paths.len());

        for src_path in file_paths {
            let result = self.save_one(src_path)?;
            asset_ids.push(result.0);
            stored_paths.push(result.1);
        }

        Ok((asset_ids, stored_paths))
    }

    /// 保存单个文件到存储
    ///
    /// 返回：
    ///   - AssetId: 生成的 AssetId
    ///   - String: 存储后的相对路径（相对于 root）
    pub(crate) fn save_one(&self, src_path: &Path) -> Result<(AssetId, String), StorageError> {
        // 1. 生成 AssetId
        let asset_id = AssetId(Uuid::new_v4());

        // 2. 提取扩展名
        let ext = Self::extract_extension(src_path);

        // 3. 计算存储路径
        let dst_path = self
            .path_manager
            .asset_file_path(asset_id, &ext)
            .map_err(|e| StorageError::Io(IoError::new(IoErrorKind::Other, e.to_string())))?;

        // 4. 确保目录存在
        ensure_parent(dst_path.as_path())?;

        // 5. 复制文件
        fs::copy(src_path, dst_path.as_path())?;

        // 6. 生成相对路径
        let relative_path = dst_path
            .as_path()
            .strip_prefix(self.path_manager.root())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| dst_path.as_path().to_string_lossy().to_string());

        Ok((asset_id, relative_path))
    }

    /// 移动单个 Asset 到回收区
    ///
    /// 参数：
    ///   - asset_id: Asset ID
    ///   - ext: 文件扩展名
    ///   - logical_day: 回收逻辑日
    ///
    /// 返回：
    ///   - String: 回收后的相对路径
    pub fn move_to_recycle_one(
        &self,
        asset_id: AssetId,
        ext: &str,
        logical_day: crate::util::time::logical_day::LogicalDay,
    ) -> Result<String, StorageError> {
        // 1. 获取源路径
        let src_path = self
            .path_manager
            .asset_file_path(asset_id, ext)
            .map_err(|e| StorageError::Io(IoError::new(IoErrorKind::Other, e.to_string())))?;

        // 检查源文件是否存在
        if !src_path.as_path().exists() {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Asset file not found: {:?}", src_path.as_path()),
            )));
        }

        // 2. 计算目标路径
        let dst_path = self
            .path_manager
            .garbage_file_path(asset_id, ext, logical_day);

        // 3. 移动文件
        crate::path::move_file(src_path.as_path(), dst_path.as_path())?;

        // 4. 生成相对路径
        let relative_path = dst_path
            .as_path()
            .strip_prefix(self.path_manager.root())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| dst_path.as_path().to_string_lossy().to_string());

        Ok(relative_path)
    }

    /// 移动多个 Asset 到回收区
    ///
    /// 参数：
    ///   - assets: Asset 列表，包含 id 和原始路径
    ///   - logical_day: 回收逻辑日
    ///
    /// 返回：
    ///   - Vec<String>: 回收后的相对路径列表
    ///
    /// 注意：
    ///   - 从原始路径提取扩展名
    ///   - 如果某个文件移动失败，继续处理剩余文件
    ///   - 返回成功的路径和错误信息
    pub fn move_to_recycle(
        &self,
        assets: &[(AssetId, String)],
        logical_day: crate::util::time::logical_day::LogicalDay,
    ) -> Result<(Vec<String>, Vec<(AssetId, StorageError)>), StorageError> {
        let mut success_paths = Vec::new();
        let mut errors = Vec::new();

        for (asset_id, original_path) in assets {
            let ext = Self::extract_extension(Path::new(original_path));

            match self.move_to_recycle_one(*asset_id, &ext, logical_day) {
                Ok(path) => success_paths.push(path),
                Err(e) => errors.push((*asset_id, e)),
            }
        }

        Ok((success_paths, errors))
    }

    /// 读取 Asset 文件内容
    ///
    /// 参数：
    ///   - asset_id: Asset ID
    ///   - ext: 文件扩展名
    pub fn read_asset(&self, asset_id: AssetId, ext: &str) -> Result<Vec<u8>, StorageError> {
        let path = self
            .path_manager
            .asset_file_path(asset_id, ext)
            .map_err(|e| StorageError::Io(IoError::new(IoErrorKind::Other, e.to_string())))?;
        Ok(fs::read(path.as_path())?)
    }

    /// 删除 Asset 文件（用于临时文件等场景）
    ///
    /// 警告：
    ///   - 正常删除应使用 move_to_recycle
    ///   - 此方法直接删除文件，不进入回收区
    pub fn delete_file(&self, asset_id: AssetId, ext: &str) -> Result<(), StorageError> {
        let path = self
            .path_manager
            .asset_file_path(asset_id, ext)
            .map_err(|e| StorageError::Io(IoError::new(IoErrorKind::Other, e.to_string())))?;
        Ok(fs::remove_file(path.as_path())?)
    }

    /// 检查 Asset 文件是否存在
    pub fn asset_exists(&self, asset_id: AssetId, ext: &str) -> bool {
        match self.path_manager.asset_file_path(asset_id, ext) {
            Ok(p) => p.as_path().exists(),
            Err(_) => false,
        }
    }
}
