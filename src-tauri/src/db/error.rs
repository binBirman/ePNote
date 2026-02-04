use rusqlite::Error as RusqliteError;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    //Sqlite封装了rusqlite（SQLite 数据库库）产生的所有数据库相关错误，只要你的数据库操作出错，都会自动转换为这个错误类型。
    #[error("sqlite error: {0}")]
    Sqlite(#[from] RusqliteError),

    //迁移错误
    #[error("migration failed: {0}")]
    Migration(String),

    //记录未找到
    #[error("record not found")]
    NotFound,
}
