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

/*
    删除指定题目的所有复习记录
    输入：
        question_id: 题目ID
    输出：
        若删除成功，返回空值
*/
pub fn delete_reviews_by_question(conn: &Connection, question_id: i64) -> Result<(), DbError> {
    conn.execute(
        r#"
        DELETE FROM review WHERE question_id = ?1
        "#,
        (question_id,),
    )?;
    Ok(())
}

/// 每科目错误率统计行（不分时间）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubjectStatRow {
    pub subject: String,
    pub review_count: i64,
    pub correct_count: i64,
    pub wrong_count: i64,
    pub fuzzy_count: i64,
}

/// 每日 × 科目 复习行为时间序列。
/// `day_bucket` 是 UTC 秒数 / 86400 后向下取整；前端按需格式化为日期字符串。
#[derive(Debug, Clone, serde::Serialize)]
pub struct DailySeriesRow {
    pub day_bucket: i64,
    pub subject: String,
    pub review_count: i64,
    pub correct_count: i64,
    pub wrong_count: i64,
}

/// 每个科目的错误率统计。
/// `subject_filter = None` 即"全部科目"；`Some(name)` 即只取给定科目。
/// 没有 subject meta 的题记为 `"__未分类__"`（前端用 settings 隐藏）。
pub fn select_subject_error_stats(
    conn: &Connection,
    subject_filter: Option<&str>,
) -> Result<Vec<SubjectStatRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            COALESCE(m.value, '__未分类__') AS subject,
            COUNT(*) AS review_count,
            SUM(CASE WHEN LOWER(r.result) = 'correct' THEN 1 ELSE 0 END) AS correct_count,
            SUM(CASE WHEN LOWER(r.result) = 'wrong'   THEN 1 ELSE 0 END) AS wrong_count,
            SUM(CASE WHEN LOWER(r.result) = 'fuzzy'   THEN 1 ELSE 0 END) AS fuzzy_count
        FROM review r
        LEFT JOIN meta m
            ON m.question_id = r.question_id
            AND m.key = 'system.Subject'
        WHERE (?1 IS NULL OR m.value = ?1)
        GROUP BY COALESCE(m.value, '__未分类__')
        ORDER BY review_count DESC, subject ASC
        "#,
    )?;

    let iter = stmt.query_map((subject_filter,), |row| {
        Ok(SubjectStatRow {
            subject: row.get(0)?,
            review_count: row.get(1)?,
            correct_count: row.get(2)?,
            wrong_count: row.get(3)?,
            fuzzy_count: row.get(4)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// 每日 × 科目 复习行为时间序列（按"逻辑日"聚合）。
///
/// `day_bucket` 是 `(r.reviewed_at + offset_sec - cutoff_sec) / 86400`，由调用方提供。
/// 这与 `LogicalDay` 抽象保持一致；offset_sec = 时区偏移秒，cutoff_sec = 切日秒。
/// 前端按日历月过滤时也按同一公式算 start_day_bucket / end_day_bucket，
/// 后端只要 `WHERE day_bucket BETWEEN ? AND ?` 即可。
pub fn select_review_daily_series(
    conn: &Connection,
    subject_filter: Option<&str>,
    start_day_bucket: Option<i64>,
    end_day_bucket: Option<i64>,
    offset_seconds: i64,
    cutoff_seconds: i64,
) -> Result<Vec<DailySeriesRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            ((r.reviewed_at + ?4 - ?5) / 86400) AS day_bucket,
            COALESCE(m.value, '__未分类__') AS subject,
            COUNT(*) AS review_count,
            SUM(CASE WHEN LOWER(r.result) = 'correct' THEN 1 ELSE 0 END) AS correct_count,
            SUM(CASE WHEN LOWER(r.result) = 'wrong'   THEN 1 ELSE 0 END) AS wrong_count
        FROM review r
        LEFT JOIN meta m
            ON m.question_id = r.question_id
            AND m.key = 'system.Subject'
        WHERE (?1 IS NULL OR m.value = ?1)
          AND (?2 IS NULL OR ((r.reviewed_at + ?4 - ?5) / 86400) >= ?2)
          AND (?3 IS NULL OR ((r.reviewed_at + ?4 - ?5) / 86400) <= ?3)
        GROUP BY day_bucket, COALESCE(m.value, '__未分类__')
        ORDER BY day_bucket ASC, COALESCE(m.value, '__未分类__') ASC
        "#,
    )?;

    let iter = stmt.query_map(
        (subject_filter, start_day_bucket, end_day_bucket, offset_seconds, cutoff_seconds),
        |row| {
            Ok(DailySeriesRow {
                day_bucket: row.get(0)?,
                subject: row.get(1)?,
                review_count: row.get(2)?,
                correct_count: row.get(3)?,
                wrong_count: row.get(4)?,
            })
        },
    )?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
