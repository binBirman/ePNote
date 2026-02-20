//! 路径工具箱模块，包含路径清理、逻辑路径、物理路径、存储布局和相关工具函数。
//!
//! 目标：对文件名与路径进行安全校验、在存储布局下生成稳定的存储路径、
//! 并提供对物理文件系统的常用操作（创建父目录、移动文件等）。

mod builder;
mod error;
mod fs;
mod layout;
mod logical_path;
mod physical_path;
mod sanitize;

mod tests {
    mod security_test;
    mod test;
}

pub use builder::PathBuilder;
pub use error::{PathError, SanitizeError, StorageError};
pub use fs::{ensure_parent, move_file};
pub use layout::StorageLayout;
pub use logical_path::LogicalPath;
pub use physical_path::PhysicalPath;
