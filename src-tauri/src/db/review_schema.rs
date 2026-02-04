use rusqlite::Result;

use crate::db::error::DbError;
use crate::db::Connection;

#[derive(Debug, Clone)]
pub struct ReviewRow {
    pub id: i64,
    pub question_id: i64,
    pub content: String,
    pub created_at: i64,
}

/* 增加一条记录 */
pub fn insert_review(
    conn: &Connection,
    question_id: i64,
    content: &str,
    created_at: i64,
) -> Result<i64, DbError> {
    conn.execute(
        r#"
        INSERT INTO review (question_id, content, created_at)
        VALUES (?1, ?2, ?3)
        "#,
        (question_id, content, created_at),
    )?;

    Ok(conn.last_insert_rowid())
}

/* 用ID查找复习记录 */
pub fn select_review_by_id(conn: &Connection, id: i64) -> Result<Option<ReviewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, content, created_at
        FROM review
        WHERE id = ?1
        "#,
    )?;

    let review_iter = stmt.query_map((id,), |row| {
        Ok(ReviewRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    for review in review_iter {
        return Ok(Some(review?));
    }

    Ok(None)
}

/* 查找某题目的所有复习记录 */
pub fn select_reviews_by_question_id(
    conn: &Connection,
    question_id: i64,
) -> Result<Vec<ReviewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, content, created_at
        FROM review
        WHERE question_id = ?1
        "#,
    )?;

    let review_iter = stmt.query_map((question_id,), |row| {
        Ok(ReviewRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    review_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/* 查找某时间戳范围内的所有复习记录 */
pub fn select_reviews_by_time_range(
    conn: &Connection,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<ReviewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, content, created_at
        FROM review
        WHERE created_at BETWEEN ?1 AND ?2
        "#,
    )?;

    let review_iter = stmt.query_map((start_ts, end_ts), |row| {
        Ok(ReviewRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    review_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}
