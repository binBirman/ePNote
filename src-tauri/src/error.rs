#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] crate::db::error::DbError),

    #[error(transparent)]
    Path(#[from] crate::asset::error::PathError),

    #[error(transparent)]
    Repo(#[from] crate::repo::error::RepoError),

    #[error(transparent)]
    Convert(#[from] crate::repo::error::ConvertError),

    #[error(transparent)]
    InitError(#[from] crate::app::error::InitError),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error")]
    Internal,
}

impl From<crate::asset::error::SanitizeError> for AppError {
    fn from(e: crate::asset::error::SanitizeError) -> Self {
        AppError::Path(e.into())
    }
}

impl From<crate::asset::error::StorageError> for AppError {
    fn from(e: crate::asset::error::StorageError) -> Self {
        AppError::Path(e.into())
    }
}

impl From<crate::asset::error::LogicalPathError> for AppError {
    fn from(e: crate::asset::error::LogicalPathError) -> Self {
        AppError::Path(e.into())
    }
}
