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

#[derive(Debug, Clone)]
pub struct LogicalPath {
    pub namespace: String, // 顶层挂载点
    pub key: String,       // 该挂载点下的相对路径
}
