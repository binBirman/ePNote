#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] crate::db::error::DbError),

    // #[error(transparent)]
    // Repo(#[from] RepoError),
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal error")]
    Internal,
}
