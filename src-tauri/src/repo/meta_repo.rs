use crate::repo::context::RepoContext;

#[derive(Debug, Clone)]
pub struct MetaRepo {
    ctx: RepoContext,
}

impl MetaRepo {
    pub fn new(ctx: RepoContext) -> Self {
        Self { ctx }
    }
}
