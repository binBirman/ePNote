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

#[derive(Debug, Clone)]
pub struct LogicalPath {
    pub namespace: String,
    pub key: String,
}
