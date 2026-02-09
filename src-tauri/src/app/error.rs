#[derive(thiserror::Error, Debug)]
pub enum InitError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid DataRoot structure")]
    InvalidStructure,

    #[error("Instance error")]
    InstanceError,
}
