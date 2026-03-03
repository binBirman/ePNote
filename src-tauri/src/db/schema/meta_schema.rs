use rusqlite::Result;

use crate::db::error::DbError;
use crate::db::Connection;

#[derive(Debug, Clone)]
pub struct MetaRow {
    pub id: i64,
    pub question_id: i64,
    pub key: String,
    pub value: String,
}
/* 增加一条记录 */
pub fn insert_meta(
    conn: &Connection,
    question_id: i64,
    key: &str,
    value: &str,
) -> Result<i64, DbError> {
    conn.execute(
        r#"
        INSERT INTO meta (question_id, key, value)
        VALUES (?1, ?2, ?3)
        "#,
        (question_id, key, value),
    )?;

    Ok(conn.last_insert_rowid())
}
/* 删除一条记录 */
pub fn delete_meta(conn: &Connection, meta_id: i64) -> Result<(), DbError> {
    conn.execute(
        r#"
        DELETE FROM meta
        WHERE id = ?1
        "#,
        (meta_id,),
    )?;

    Ok(())
}

/*
    用ID查找元信息
    输入：
        id: 元信息ID，必填
    输出：
        若找到，返回Some(MetaRow)
*/
pub fn select_meta_by_id(conn: &Connection, id: i64) -> Result<Option<MetaRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, key, value
        FROM meta
        WHERE id = ?1
        "#,
    )?;

    let meta_iter = stmt.query_map((id,), |row| {
        Ok(MetaRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            key: row.get(2)?,
            value: row.get(3)?,
        })
    })?;

    for meta in meta_iter {
        return Ok(Some(meta?));
    }

    Ok(None)
}

/* 查找某题目的所有元信息 */
pub fn select_meta_by_question(
    conn: &Connection,
    question_id: i64,
) -> Result<Vec<MetaRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, key, value
        FROM meta
        WHERE question_id = ?1
        "#,
    )?;

    let meta_iter = stmt.query_map((question_id,), |row| {
        Ok(MetaRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            key: row.get(2)?,
            value: row.get(3)?,
        })
    })?;

    meta_iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/* 查找某题目某key的元信息 */
pub fn select_meta_by_question_key(
    conn: &Connection,
    question_id: i64,
    key: &str,
) -> Result<Option<MetaRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, key, value
        FROM meta
        WHERE question_id = ?1 AND key = ?2
        "#,
    )?;

    let meta_iter = stmt.query_map((question_id, key), |row| {
        Ok(MetaRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            key: row.get(2)?,
            value: row.get(3)?,
        })
    })?;

    for meta in meta_iter {
        return Ok(Some(meta?));
    }

    Ok(None)
}

pub fn select_meta_values_by_question_key(
    conn: &Connection,
    question_id: i64,
    key: &str,
) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT value
        FROM meta
        WHERE question_id = ?1 AND key = ?2
        "#,
    )?;

    let rows = stmt.query_map((question_id, key), |row| {
        let v: String = row.get(0)?;
        Ok(v)
    })?;

    let mut values = Vec::new();
    for v in rows {
        values.push(v?);
    }

    Ok(values)
}

/*
    查询所有不重复的科目值
    输入：
        key: 元信息 key，如 "system.Subject"
    输出：
        返回所有不重复的值列表
*/
pub fn select_distinct_values_by_key(
    conn: &Connection,
    key: &str,
) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT DISTINCT value
        FROM meta
        WHERE key = ?1
        ORDER BY value
        "#,
    )?;

    let rows = stmt.query_map((key,), |row| {
        let v: String = row.get(0)?;
        Ok(v)
    })?;

    let mut values = Vec::new();
    for v in rows {
        values.push(v?);
    }

    Ok(values)
}
