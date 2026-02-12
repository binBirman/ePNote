use crate::domain::asset::Asset;
use crate::repo::context::RepoContext;
use crate::repo::error::RepoError;

#[derive(Debug, Clone)]
pub struct AssetRepo {
    ctx: RepoContext,
}

impl AssetRepo {
    pub fn new(ctx: RepoContext) -> Self {
        Self { ctx }
    }

    pub async fn insert(&self, asset: Asset, file: &[u8]) -> Result<(), RepoError> {
        // 1 写文件
        // 2 写数据库
        Ok(())
    }
}
