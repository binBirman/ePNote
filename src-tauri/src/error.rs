#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] crate::db::error::DbError),

    #[error(transparent)]
    Path(#[from] crate::asset::error::PathError),

    // #[error(transparent)]
    // Repo(#[from] crate::repo::error::RepoError),
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal error")]
    Internal,
}
