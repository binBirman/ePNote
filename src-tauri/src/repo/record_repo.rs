use crate::repo::context::RepoContext;

#[derive(Debug, Clone)]
pub struct RecordRepo {
    ctx: RepoContext,
}

impl RecordRepo {
    pub fn new(ctx: RepoContext) -> Self {
        Self { ctx }
    }
}
