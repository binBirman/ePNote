//! Asset 存储管理
//!
//! 核心结构：AssetStore
//! 职责：
//!     - 保存资源
//!     - 移动到回收区
//!     - 生成资源路径

use std::path::PathBuf;
use std::time::SystemTime;

use uuid::Uuid;

use crate::asset::error::StorageError;
use crate::asset::fs::move_file;
use crate::asset::path::{PathBuilder, StorageLayout};

/// 资源元数据（纯文件系统信息，不包含业务引用）
#[derive(Debug, Clone)]
pub struct AssetMeta {
    pub id: Uuid,
    /// 存储中的相对路径（相对于 `AssetStore.root`）
    pub relative_path: PathBuf,
    pub size: u64,
    pub ext: String,
    pub created_at: SystemTime,
}

// 回收区条目不再使用独立结构，接口改为使用元组 `(id, ext, recycle_relative)`。

/// Asset 存储
pub struct AssetStore {
    root: PathBuf,
    builder: PathBuilder,
}

impl AssetStore {
    /// 使用存储根目录创建 `AssetStore`。
    pub fn new(root: PathBuf) -> Self {
        let layout = StorageLayout::new(root.clone());
        let builder = PathBuilder::new(layout);

        Self { root, builder }
    }

    /// 返回存储根目录（绝对路径）
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// 保存多个文件到存储中。行为：为每个源文件生成文件名，并将文件移动到存储下。
    /// 返回对应的 `AssetMeta` 列表。
    ///
    /// 注意：方法会移动源文件（如需保留请在调用前复制）。
    pub fn save_many(&self, src_paths: &[PathBuf]) -> Result<Vec<AssetMeta>, StorageError> {
        let mut metas = Vec::with_capacity(src_paths.len());

        for src in src_paths {
            // 1. 生成 id 和扩展名
            let id = Uuid::new_v4();
            let ext = src
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("bin")
                .to_string();

            // 2. 生成目标相对路径（相对于 root），使用 PathBuilder 来生成 timestamped 文件名
            let dst_rel = self
                .builder
                .build_asset_path(&id, &ext)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))
                .map_err(StorageError::Io)?;

            // 将相对路径转换为绝对路径（基于 store root）并移动文件到目标（会创建父目录）
            let dst = self.root.join(&dst_rel);
            move_file(src.as_path(), &dst)?;

            // 4. 获取元信息
            let md = std::fs::metadata(&dst)?;
            let size = md.len();
            let created_at = md.modified().unwrap_or(SystemTime::now());
            
            // 5. 记录相对路径（相对于 root）
            let relative = dst_rel;

            metas.push(AssetMeta {
                id,
                relative_path: relative,
                size,
                ext,
                created_at,
            });
        }

        Ok(metas)
    }

    /// 将给定的资源移动到回收区，返回生成的 `RecycleEntry` 列表。
    pub fn move_to_recycle(
        &self,
        assets: &[AssetMeta],
    ) -> Result<Vec<(Uuid, String, PathBuf)>, StorageError> {
        let mut res = Vec::with_capacity(assets.len());

        for a in assets {
            let src = self.root.join(&a.relative_path);

            // 回收目录: {root}/garbages/{timestamp}/{filename}
            let ts = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
            let recycle_dir = self.root.join("garbages").join(ts);
            let filename = src
                .file_name()
                .map(|s| s.to_os_string())
                .unwrap_or_else(|| a.id.to_string().into());
            let dst = recycle_dir.join(filename);

            move_file(&src, &dst)?;

            let recycle_relative = dst
                .strip_prefix(&self.root)
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|_| dst.clone());

            res.push((a.id, a.ext.clone(), recycle_relative));
        }

        Ok(res)
    }

    /// 从回收区恢复文件到其原始相对路径（由 `move_to_recycle` 保存的 `original_relative` 字段）。
    /// 返回恢复后的 `AssetMeta` 列表。
    pub fn restore(
        &self,
        entries: &[(Uuid, String, PathBuf)],
    ) -> Result<Vec<AssetMeta>, StorageError> {
        let mut res = Vec::with_capacity(entries.len());

        for (id, ext, recycle_relative) in entries {
            let src = self.root.join(recycle_relative);
            // 重新计算目标相对路径，使用 PathBuilder 生成 timestamped 文件名
            let dst_rel = self
                .builder
                .build_asset_path(id, ext)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))
                .map_err(StorageError::Io)?;

            // 目标绝对路径
            let dst = self.root.join(&dst_rel);
            move_file(&src, &dst)?;

            let md = std::fs::metadata(&dst)?;
            let size = md.len();
            let created_at = md.modified().unwrap_or(SystemTime::now());

            let relative = dst_rel;

            res.push(AssetMeta {
                id: *id,
                relative_path: relative,
                size,
                ext: ext.clone(),
                created_at,
            });
        }

        Ok(res)
    }

    /// 物理删除回收区中的指定条目（从存储根目录中删除回收路径文件）。
    pub fn delete_physical(&self, entries: &[(Uuid, String, PathBuf)]) -> Result<(), StorageError> {
        for (_id, _ext, recycle_relative) in entries {
            let p = self.root.join(recycle_relative);
            if p.exists() {
                std::fs::remove_file(&p)?;
            }
        }
        Ok(())
    }
}
