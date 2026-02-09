use std::path::PathBuf;
use uuid::Uuid;

use crate::path::{error::PathError, sanitize::sanitize_filename, PhysicalPath, StorageLayout};

#[derive(Clone)]
pub struct PathBuilder {
    layout: StorageLayout,
}

impl PathBuilder {
    pub fn new(layout: StorageLayout) -> Self {
        Self { layout }
    }

    pub fn build_asset_path(&self, id: &Uuid, ext: &str) -> Result<PhysicalPath, PathError> {
        let safe_ext = sanitize_filename(ext)?;

        let path: PathBuf = self.layout.asset_file(id, &safe_ext);

        Ok(PhysicalPath::new(path))
    }
}
