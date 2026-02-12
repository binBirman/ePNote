use crate::repo::asset_repo::AssetRepo;
use crate::repo::context::RepoContext;
use crate::repo::meta_repo::MetaRepo;
// use crate::repo::review_repo::ReviewRepo;

#[derive(Debug, Clone)]
pub struct Repo {
    pub assets: AssetRepo,
    pub meta: MetaRepo,
    // pub reviews: ReviewRepo,
}

impl Repo {}
