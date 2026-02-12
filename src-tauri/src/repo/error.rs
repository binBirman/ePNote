#[derive(Debug)]
pub enum RepoError {
    Db(sqlx::Error),
    Io(std::io::Error),
    NotFound,
    InvalidData,
}
