use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error(transparent)]
    Db(sqlx::Error),

    #[error(transparent)]
    Io(std::io::Error),

    #[error("not found")]
    NotFound,

    #[error("invalid data")]
    InvalidData,
}

// ------------------------------------------------------------
// Convert Error
// ------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("invalid question state: {0}")]
    InvalidQuestionState(String),

    #[error("invalid review result: {0}")]
    InvalidReviewResult(String),

    #[error("invalid asset type: {0}")]
    InvalidAssetType(String),

    #[error("invalid meta key: {0}")]
    InvalidMetaKey(String),

    #[error("invalid logical path: {0}")]
    InvalidLogicalPath(String),

    #[error("invalid logical day: {0}")]
    InvalidLogicalDay(i64),
}

pub type ConvertResult<T> = Result<T, ConvertError>;
