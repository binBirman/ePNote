use std::path::PathBuf;
use uuid::Uuid;

/// 存储布局，用于将 UUID 分桶到稳定的目录结构下。
///
/// 例如：`/root/assets/ab/cd/ab...` 这种方式可以避免单目录过多文件。
#[derive(Debug, Clone)]
pub struct StorageLayout {
    root: PathBuf,
}

impl StorageLayout {
    /// 创建一个新的 `StorageLayout`，`root` 是存储根目录。
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// 返回配置的根目录引用。
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// 根据 `id` 生成分桶目录（不包含文件名），例如 `/root/assets/ab/cd`。
    pub fn asset_dir(&self, id: &Uuid) -> PathBuf {
        let s = id.simple().to_string();

        let p1 = &s[0..2];
        let p2 = &s[2..4];

        self.root.join("assets").join(p1).join(p2)
    }

    /// 根据 `id` 和扩展名 `ext` 返回完整的文件路径（含文件名）。
    pub fn asset_file(&self, id: &Uuid, ext: &str) -> PathBuf {
        let filename = format!("{}.{}", id.simple(), ext);
        self.asset_dir(id).join(filename)
    }
}
