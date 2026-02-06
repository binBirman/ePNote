use rusqlite::Result;

use crate::db::error::DbError;
use crate::db::Connection;

#[derive(Debug, Clone)]
pub struct AssetRow {
    pub id: i64,
    pub question_id: i64,
    pub type_: String,
    pub path: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}
/* 增加一条记录 */
pub fn insert_asset(
    conn: &Connection,
    question_id: i64,
    type_: &str,
    path: &str,
    created_at: i64,
) -> Result<i64, DbError> {
    conn.execute(
        r#"
        INSERT INTO asset (question_id, type, path, created_at)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        (question_id, type_, path, created_at),
    )?;

    Ok(conn.last_insert_rowid())
}
/* 删除一条记录 */
pub fn delete_asset(conn: &Connection, asset_id: i64, deleted_at: i64) -> Result<(), DbError> {
    conn.execute(
        r#"
        UPDATE asset
        SET deleted_at = ?1
        WHERE id = ?2
        "#,
        (deleted_at, asset_id),
    )?;

    Ok(())
}

/* 用ID查找资源 */
pub fn select_asset_by_id(conn: &Connection, id: i64) -> Result<Option<AssetRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, type, path, created_at, deleted_at
        FROM asset
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
    )?;

    let asset_iter = stmt.query_map((id,), |row| {
        Ok(AssetRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            type_: row.get(2)?,
            path: row.get(3)?,
            created_at: row.get(4)?,
            deleted_at: row.get(5)?,
        })
    })?;

    for asset in asset_iter {
        return Ok(Some(asset?));
    }

    Ok(None)
}

/* 查找某题目的所有资源 */
pub fn select_asset_by_question(
    conn: &Connection,
    question_id: i64,
) -> Result<Vec<AssetRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, type, path, created_at, deleted_at
        FROM asset
        WHERE question_id = ?1 AND deleted_at IS NULL
        "#,
    )?;

    let asset_iter = stmt.query_map((question_id,), |row| {
        Ok(AssetRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            type_: row.get(2)?,
            path: row.get(3)?,
            created_at: row.get(4)?,
            deleted_at: row.get(5)?,
        })
    })?;

    asset_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}
