use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SanitizeError {
    #[error("path traversal detected")]
    PathTraversal,

    #[error("separator not allowed")]
    SeparatorNotAllowed,

    #[error("empty input")]
    Empty,
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("sanitize error: {0}")]
    Sanitize(#[from] SanitizeError),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
