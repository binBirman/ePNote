#[derive(Debug, Clone)]
pub struct RepoContext {
    pub db: sqlx::SqlitePool,
    pub dataroot: std::path::PathBuf,
}
