use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SanitizeError {

    // 下面结构错误类用于进行业务约束

    #[error("empty input")]
    Empty,

    #[error("hidden files not allowed")]
    HiddenFile,

    // 下面几个错误类用于防御攻击

    #[error("path traversal detected")] //防止目录逃逸攻击，需被安全日志记录
    PathTraversal,

    #[error("illegal character detected")] //防止非法字符攻击
    IllegalChar,

    // 下面几个错误类用于防止不可预测的系统行为

    #[error("trailing dot not allowed")]
    TrailingDot, // 处理Windows路径末尾点导致的异常

    #[error("reserved name not allowed")]
    ReservedName, // 处理Windows保留名导致的异常
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("sanitize error: {0}")]
    Sanitize(#[from] SanitizeError),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
