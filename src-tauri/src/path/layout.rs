use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StorageLayout {
    root: PathBuf,
}

impl StorageLayout {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn asset_dir(&self, id: &Uuid) -> PathBuf {
        let s = id.simple().to_string();

        let p1 = &s[0..2];
        let p2 = &s[2..4];

        self.root.join("assets").join(p1).join(p2)
    }

    pub fn asset_file(&self, id: &Uuid, ext: &str) -> PathBuf {
        let filename = format!("{}.{}", id.simple(), ext);
        self.asset_dir(id).join(filename)
    }
}
