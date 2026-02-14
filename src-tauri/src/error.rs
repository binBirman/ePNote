#[derive(Debug, thiserror::Error)]
pub enum PathErrorEnum {
    #[error(transparent)]
    PathError(#[from] crate::util::path::PathError),
    #[error(transparent)]
    SanitizeError(#[from] crate::util::path::SanitizeError),
    #[error(transparent)]
    StorageError(#[from] crate::util::path::StorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Path(#[from] PathErrorEnum),

    #[error(transparent)]
    Db(#[from] crate::db::error::DbError),

    // #[error(transparent)]
    // Repo(#[from] RepoError),
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal error")]
    Internal,
}
