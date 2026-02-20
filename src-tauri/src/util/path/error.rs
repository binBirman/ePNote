use std::io;
use thiserror::Error;

/// path 模块的 sanitize（清理）相关错误类型
///
/// - 用于表达对文件名/路径的业务约束（空、隐藏文件等）
/// - 用于描述检测到的安全问题（路径穿越、非法字符等）
#[derive(Debug, Error)]
pub enum SanitizeError {
    /// 输入为空
    #[error("empty input")]
    Empty,

    /// 不允许以 `.` 开头的隐藏文件名
    #[error("hidden files not allowed")]
    HiddenFile,

    /// 检测到路径穿越（path traversal），应记录安全日志
    #[error("path traversal detected")]
    PathTraversal,

    /// 检测到非法字符
    #[error("illegal character detected")]
    IllegalChar,

    /// 不允许以点结尾（Windows 特殊行为）
    #[error("trailing dot not allowed")]
    TrailingDot,

    /// Windows 保留名（例如 CON、NUL）不允许
    #[error("reserved name not allowed")]
    ReservedName,
}

/// 解析逻辑路径（LogicalPath）时可能的错误
#[derive(Debug, Error)]
pub enum LogicalPathError {
    /// 空路径
    #[error("empty path")]
    Empty,

    /// 命名空间为空（占位，保留）
    #[error("empty namespace")]
    EmptyNamespace,
}

/// 通用的路径相关错误封装，用于上层统一处理
#[derive(Debug, Error)]
pub enum PathError {
    /// 来自 sanitize 的错误
    #[error("sanitize error: {0}")]
    Sanitize(#[from] SanitizeError),

    /// 逻辑路径解析错误
    #[error("logical path parse error: {0}")]
    ParseLogical(#[from] LogicalPathError),
}

/// 与文件系统操作相关的错误（包装 `std::io::Error`）
#[derive(Debug, Error)]
pub enum StorageError {
    /// IO 错误
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
