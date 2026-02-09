use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

pub struct DataRootContext {
    pub root: PathBuf,
    pub assets_dir: PathBuf,
    pub trash_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub db_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct InstanceFile {
    pub instance_version: u32,
    pub schema_version: u32,
    pub asset_layout_version: u32,
    pub created_at: i64,
}

impl Default for InstanceFile {
    fn default() -> Self {
        Self {
            instance_version: 1,
            schema_version: 1,
            asset_layout_version: 1,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}
