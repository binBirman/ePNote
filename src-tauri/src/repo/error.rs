#[derive(Debug)]
pub enum RepoError {
    Db(sqlx::Error),
    Io(std::io::Error),
    NotFound,
    InvalidData,
}

// ------------------------------------------------------------
// Convert Error
// ------------------------------------------------------------

#[derive(Debug)]
pub enum ConvertError {
    InvalidQuestionState(String),
    InvalidReviewResult(String),
    InvalidAssetType(String),
    InvalidMetaKey(String),
    InvalidLogicalPath(String),
    InvalidLogicalDay(i64),
}

pub type ConvertResult<T> = Result<T, ConvertError>;
