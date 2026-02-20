//! 物理路径类型，用于表示文件系统中的真实路径。
//!
//! `PhysicalPath` 包装了 `PathBuf`，提供安全的访问方法并实现 `Deref`/`AsRef`。

use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PhysicalPath {
    pub(crate) inner: PathBuf,
}

impl PhysicalPath {
    /// 构造一个新的 `PhysicalPath`。
    pub fn new(inner: PathBuf) -> Self {
        Self { inner }
    }

    /// 返回内部 `Path` 的引用。
    pub fn as_path(&self) -> &Path {
        &self.inner
    }
}

impl Deref for PhysicalPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl AsRef<Path> for PhysicalPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}
