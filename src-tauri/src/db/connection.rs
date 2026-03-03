pub use rusqlite::Connection;

use rusqlite::Result;
use std::path::Path;

pub fn open_db(data_root: &Path) -> Result<Connection> {
    let db_path = data_root.join("db.sqlite");
    let conn = Connection::open(db_path)?;
    // 启用外键约束
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}
