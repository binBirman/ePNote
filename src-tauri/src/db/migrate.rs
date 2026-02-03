use rusqlite::{Connection, Result};

//use crate::db::connection;

// 单条迁移描述
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

// 所有迁移，按 version 升序排列
const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "init_schema",
    sql: r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );

        INSERT INTO schema_version (version)
        SELECT 0
        WHERE NOT EXISTS (SELECT 1 FROM schema_version);

        CREATE TABLE question (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            state TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            deleted_at INTEGER
        );

        CREATE TABLE review (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            reviewed_at INTEGER NOT NULL,
            result TEXT NOT NULL,
            FOREIGN KEY(question_id) REFERENCES question(id)
        );

        CREATE TABLE asset (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            deleted_at INTEGER,
            FOREIGN KEY(question_id) REFERENCES question(id)
        );

        CREATE TABLE meta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            FOREIGN KEY(question_id) REFERENCES question(id)
        );
        "#,
}];

/*
    启动时调用，检查当前 schema 版本，依次应用未应用的迁移
    输入：
    - conn: 可变数据库连接引用
*/
pub fn migrate(conn: &mut Connection) -> Result<()> {
    ensure_schema_version_table(conn)?;

    let current_version = get_current_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version > current_version {
            apply_migration(conn, migration)?;
        }
    }

    Ok(())
}

// 工具函数，确保 schema_version 表存在
fn ensure_schema_version_table(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );
        "#,
        [],
    )?;
    Ok(())
}

// 工具函数，读取当前 schema version
fn get_current_version(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT version FROM schema_version LIMIT 1")?;

    let version = stmt.query_row([], |row| row.get(0)).unwrap_or(0);

    Ok(version)
}

// 工具函数，执行单条迁移
fn apply_migration(conn: &mut Connection, migration: &Migration) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute_batch(migration.sql)?;

    tx.execute(
        "UPDATE schema_version SET version = ?1",
        [migration.version],
    )?;

    tx.commit()?;

    Ok(())
}
