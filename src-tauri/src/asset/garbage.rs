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

use std::fs;
use std::time::SystemTime;

use crate::asset::asset_path::AssetPath;
use crate::util::path::StorageError;
use crate::util::time::logical_day;

/// 回收区扫描结果

/// 回收区扫描结果
#[derive(Debug, Clone)]
pub struct GarbageEntry {
    /// 文件路径（相对于 root）
    pub path: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 文件修改时间（系统时间）
    pub modified: SystemTime,
    /// 回收逻辑日
    pub logical_day: logical_day::LogicalDay,
}

/// 回收区统计信息
#[derive(Debug, Clone, Default)]
pub struct GarbageStats {
    /// 文件总数
    pub file_count: usize,
    /// 总大小（字节）
    pub total_size: u64,
    /// 按逻辑日分组的文件数量
    pub count_by_day: Vec<(logical_day::LogicalDay, usize)>,
}

/// 回收区管理器
#[derive(Debug, Clone)]
pub struct GarbageManager {
    path_manager: AssetPath,
}

impl GarbageManager {
    /// 创建新的回收区管理器
    pub fn new(path_manager: AssetPath) -> Self {
        Self { path_manager }
    }

    /// 从引用创建新的回收区管理器
    pub fn from_ref(path_manager: &AssetPath) -> Self {
        Self {
            path_manager: path_manager.clone(),
        }
    }

    /// 获取路径管理器
    pub fn path(&self) -> &AssetPath {
        &self.path_manager
    }

    /// 扫描回收区，获取所有文件信息
    ///
    /// 返回：
    ///   - Vec<GarbageEntry>: 回收区文件列表
    ///
    /// 过程：
    ///   1. 遍历 garbages 目录下的所有子目录
    ///   2. 从子目录名解析 logical_day
    ///   3. 收集每个文件的信息
    pub fn scan_garbage(&self) -> Result<Vec<GarbageEntry>, StorageError> {
        let garbage_dir = self.path_manager.garbages_dir();

        // 如果回收区不存在，返回空列表
        if !garbage_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();

        // 遍历所有日期子目录
        for entry in fs::read_dir(&garbage_dir)? {
            let entry = entry?;
            let path = entry.path();

            // 跳过非目录
            if !path.is_dir() {
                continue;
            }

            // 从目录名解析 logical_day
            let dir_name = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                StorageError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid directory name",
                ))
            })?;

            let logical_day: i32 = dir_name.parse().map_err(|_| {
                StorageError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid logical_day: {}", dir_name),
                ))
            })?;

            let logical_day = logical_day::LogicalDay(logical_day);

            // 遍历目录下的文件
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if !file_path.is_file() {
                    continue;
                }

                // 获取文件元数据
                let metadata = file_entry.metadata()?;
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

                // 生成相对路径
                let relative_path = file_path
                    .strip_prefix(self.path_manager.root())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

                entries.push(GarbageEntry {
                    path: relative_path,
                    size: metadata.len(),
                    modified,
                    logical_day,
                });
            }
        }

        Ok(entries)
    }

    /// 扫描回收区并生成统计信息
    pub fn get_stats(&self) -> Result<GarbageStats, StorageError> {
        let entries = self.scan_garbage()?;

        let mut stats = GarbageStats::default();
        stats.file_count = entries.len();
        stats.total_size = entries.iter().map(|e| e.size).sum();

        // 按逻辑日分组统计
        let mut day_counts = std::collections::HashMap::new();
        for entry in &entries {
            *day_counts.entry(entry.logical_day).or_insert(0) += 1;
        }

        stats.count_by_day = {
            let mut vec: Vec<_> = day_counts.into_iter().collect();
            vec.sort_by_key(|(day, _)| day.0);
            vec
        };

        Ok(stats)
    }

    /// 清理指定逻辑日及之前的所有文件
    ///
    /// 参数：
    ///   - before_day: 清除此逻辑日之前的文件
    ///
    /// 返回：
    ///   - Vec<String>: 已删除文件的相对路径列表
    ///
    /// 警告：
    ///   - 此操作不可逆
    ///   - 需要用户确认后调用
    pub fn cleanup_before(
        &self,
        before_day: logical_day::LogicalDay,
    ) -> Result<Vec<String>, StorageError> {
        let entries = self.scan_garbage()?;
        let mut deleted_paths = Vec::new();

        for entry in &entries {
            if entry.logical_day < before_day {
                let full_path = self.path_manager.root().join(&entry.path);
                fs::remove_file(&full_path)?;
                deleted_paths.push(entry.path.clone());
            }
        }

        // 清理空目录
        self.cleanup_empty_dirs()?;

        Ok(deleted_paths)
    }

    /// 清理指定逻辑日的所有文件
    ///
    /// 参数：
    ///   - target_day: 清除此逻辑日的所有文件
    ///
    /// 返回：
    ///   - Vec<String>: 已删除文件的相对路径列表
    ///
    /// 警告：
    ///   - 此操作不可逆
    ///   - 需要用户确认后调用
    pub fn cleanup_day(
        &self,
        target_day: logical_day::LogicalDay,
    ) -> Result<Vec<String>, StorageError> {
        let entries = self.scan_garbage()?;
        let mut deleted_paths = Vec::new();

        for entry in &entries {
            if entry.logical_day == target_day {
                let full_path = self.path_manager.root().join(&entry.path);
                fs::remove_file(&full_path)?;
                deleted_paths.push(entry.path.clone());
            }
        }

        // 清理该日的空目录
        let day_dir = self.path_manager.garbage_subdir(target_day);
        if day_dir.exists() && fs::read_dir(&day_dir)?.next().is_none() {
            fs::remove_dir(&day_dir)?;
        }

        Ok(deleted_paths)
    }

    /// 清理空目录
    ///
    /// 删除 garbages 下所有空的日期目录
    fn cleanup_empty_dirs(&self) -> Result<(), StorageError> {
        let garbage_dir = self.path_manager.garbages_dir();

        if !garbage_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&garbage_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // 检查是否为空目录
                if fs::read_dir(&path)?.next().is_none() {
                    fs::remove_dir(&path)?;
                }
            }
        }

        Ok(())
    }

    /// 检查是否需要清理（返回过期文件列表）
    ///
    /// 参数：
    ///   - keep_days: 保留天数
    ///   - current_day: 当前逻辑日
    ///
    /// 返回：
    ///   - Vec<GarbageEntry>: 过期文件列表
    ///
    /// 注意：
    ///   - 此方法只检查，不删除
    ///   - 需要用户确认后调用 cleanup_before
    pub fn check_expiration(
        &self,
        keep_days: i32,
        current_day: logical_day::LogicalDay,
    ) -> Result<Vec<GarbageEntry>, StorageError> {
        let entries = self.scan_garbage()?;
        let threshold_day = logical_day::LogicalDay(current_day.0 - keep_days);

        let expired: Vec<_> = entries
            .into_iter()
            .filter(|e| e.logical_day < threshold_day)
            .collect();

        Ok(expired)
    }

    /// 计算过期文件占用的空间
    ///
    /// 参数：
    ///   - keep_days: 保留天数
    ///   - current_day: 当前逻辑日
    ///
    /// 返回：
    ///   - u64: 过期文件总大小（字节）
    pub fn expired_size(
        &self,
        keep_days: i32,
        current_day: logical_day::LogicalDay,
    ) -> Result<u64, StorageError> {
        let expired = self.check_expiration(keep_days, current_day)?;
        let total_size: u64 = expired.iter().map(|e| e.size).sum();
        Ok(total_size)
    }
}
