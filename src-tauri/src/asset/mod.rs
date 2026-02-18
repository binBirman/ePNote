//! Asset 模块
//!
//! 文件系统层，负责资源文件的存储和管理
//!
//! 模块结构：
//! - path: 路径管理，生成真实文件路径、组织目录结构
//! - store: 存储管理，保存资源、移动到回收区
//! - garbage: 回收区管理，扫描回收区、清理过期文件
//!
//! 关键规则：
//! - Asset 是文件系统层，不是业务层
//! - 资源从属于 Question，不得设计独立 Asset 业务系统
//! - 所有时间使用 LogicalDay
//! - Asset 删除进入回收区，不允许自动删除

pub mod asset_path;
pub mod garbage;
pub mod store;

mod tests {
    mod asset_path_test;
    mod garbage_test;
    mod store_test;
}

pub use asset_path::AssetPath;
pub use garbage::{GarbageEntry, GarbageManager, GarbageStats};
pub use store::AssetStore;
