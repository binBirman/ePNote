//！ 物理地址，专用于存储文件在文件系统中的实际路径，提供一些路径相关的操作和验证功能。

use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PhysicalPath {
    pub(crate) inner: PathBuf,
}

impl PhysicalPath {
    pub fn new(inner: PathBuf) -> Self {
        Self { inner }
    }

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
