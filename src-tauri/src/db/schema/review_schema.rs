use rusqlite::Result;

use crate::db::error::DbError;
use crate::db::Connection;

#[derive(Debug, Clone)]
pub struct ReviewRow {
    pub id: i64,
    pub question_id: i64,
    pub result: String,
    pub reviewed_at: i64,
}

/* 增加一条记录 */
pub fn insert_review(
    conn: &Connection,
    question_id: i64,
    result: &str,
    reviewed_at: i64,
) -> Result<i64, DbError> {
    conn.execute(
        r#"
        INSERT INTO review (question_id, result, reviewed_at)
        VALUES (?1, ?2, ?3)
        "#,
        (question_id, result, reviewed_at),
    )?;

    Ok(conn.last_insert_rowid())
}

/* 用ID查找复习记录 */
pub fn select_review_by_id(conn: &Connection, id: i64) -> Result<Option<ReviewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE id = ?1
        "#,
    )?;

    let review_iter = stmt.query_map((id,), |row| {
        Ok(ReviewRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            result: row.get(2)?,
            reviewed_at: row.get(3)?,
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
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE question_id = ?1
        "#,
    )?;

    let review_iter = stmt.query_map((question_id,), |row| {
        Ok(ReviewRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            result: row.get(2)?,
            reviewed_at: row.get(3)?,
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
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE reviewed_at BETWEEN ?1 AND ?2
        "#,
    )?;

    let review_iter = stmt.query_map((start_ts, end_ts), |row| {
        Ok(ReviewRow {
            id: row.get(0)?,
            question_id: row.get(1)?,
            result: row.get(2)?,
            reviewed_at: row.get(3)?,
        })
    })?;

    review_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/*
    统计总复习次数
    输出：
        返回总复习次数
*/
pub fn count_reviews(conn: &Connection) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM review
        "#,
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/*
    统计指定结果的复习次数
    输入：
        result: 复习结果
    输出：
        返回复习次数
*/
pub fn count_reviews_by_result(conn: &Connection, result: &str) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM review WHERE result = ?1
        "#,
        [result],
        |row| row.get(0),
    )?;
    Ok(count)
}

/*
    统计指定时间之后的复习次数
    输入：
        since: 时间戳
    输出：
        返回复习次数
*/
pub fn count_reviews_since(conn: &Connection, since: i64) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM review WHERE reviewed_at >= ?1
        "#,
        [since],
        |row| row.get(0),
    )?;
    Ok(count)
}

/*
    统计指定时间之后指定结果的复习次数
    输入：
        since: 时间戳
        result: 复习结果
    输出：
        返回复习次数
*/
pub fn count_reviews_since_by_result(conn: &Connection, since: i64, result: &str) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM review WHERE reviewed_at >= ?1 AND result = ?2
        "#,
        (since, result),
        |row| row.get(0),
    )?;
    Ok(count)
}
