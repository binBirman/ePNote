//! 逻辑地址，专用于存储文件相对于根目录的路径，提供一些路径相关的操作和验证功能。

use crate::util::path::error::SanitizeError;
use crate::util::path::PathError;
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LogicalPath {
    pub(crate) inner: PathBuf,
}

impl LogicalPath {
    /// 创建一个新的 `LogicalPath`，不做额外校验。
    pub fn new(inner: PathBuf) -> Self {
        Self { inner }
    }

    /// 返回内部 `Path` 引用，用于文件系统相关操作。
    pub fn as_path(&self) -> &Path {
        &self.inner
    }

    /// 将逻辑路径以字符串形式返回（UTF-8 宽松转换）。
    pub fn as_str(&self) -> String {
        self.inner.to_string_lossy().into_owned()
    }
}

impl TryFrom<&str> for LogicalPath {
    type Error = PathError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // 执行 sanitize/验证/规范化
        let p = PathBuf::from(s);
        if p.components().any(|c| c == std::path::Component::ParentDir) {
            // 若包含上级目录组件，则视为路径穿越攻击
            return Err(SanitizeError::PathTraversal.into());
        }
        Ok(LogicalPath::new(p))
    }
}

impl Deref for LogicalPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl AsRef<Path> for LogicalPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}
