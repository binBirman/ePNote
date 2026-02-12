use crate::repo::asset_repo::AssetRepo;
use crate::repo::context::RepoContext;
use crate::repo::meta_repo::MetaRepo;
use crate::repo::record_repo::RecordRepo;

#[derive(Debug, Clone)]
pub struct Repo {
    pub assets: AssetRepo,
    pub meta: MetaRepo,
    pub records: RecordRepo,
}

impl Repo {
    pub fn new(ctx: RepoContext) -> Self {
        Self {
            assets: AssetRepo::new(ctx.clone()),
            meta: MetaRepo::new(ctx.clone()),
            records: RecordRepo::new(ctx),
        }
    }
}
