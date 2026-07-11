//! 推荐数据访问层

use crate::db::error::DbError;
use crate::server::recommendation::RecommendedQuestion;
use crate::util::time::{ClockConfig, LogicalDay};
use rusqlite::Connection;

/// 每日复习状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyReviewStatus {
    pub recommended_count: i64,    // 今日推荐题目数
    pub reviewed_count: i64,       // 今日已复习题目数
    pub is_completed: bool,        // 是否完成全部推荐
}

/// DAO for recommendation table and review_summary view
pub struct RecommendationDao<'a> {
    conn: &'a Connection,
}

impl<'a> RecommendationDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 获取指定日期的推荐列表
    pub fn get_by_day(&self, day: i64) -> Result<Option<Vec<RecommendedQuestion>>, DbError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                r.question_id,
                q.name,
                r.score,
                q.state,
                q.due_at,
                q.correct_streak,
                q.wrong_count,
                q.last_result,
                rs.error_rate,
                r.subject,
                r.reason
            FROM recommendation r
            JOIN question q ON r.question_id = q.id
            LEFT JOIN review_summary rs ON r.question_id = rs.question_id
            WHERE r.day = ?1
              AND q.deleted_at IS NULL
            ORDER BY r.score DESC
            "#
        )?;

        let rows = stmt.query_map([day], |row| {
            let reason_text: Option<String> = row.get(10)?;
            let reason = reason_text
                .filter(|t| t != "null")
                .and_then(|t| serde_json::from_str(&t).ok());
            Ok(RecommendedQuestion {
                question_id: row.get(0)?,
                name: row.get(1)?,
                score: row.get(2)?,
                state: row.get(3)?,
                due_at: row.get(4)?,
                correct_streak: row.get(5)?,
                wrong_count: row.get(6)?,
                last_result: row.get(7)?,
                error_rate: row.get(8)?,
                subject: row.get(9)?,
                reason,
                score_detail: None,
                review_count: 0,
                created_at: 0,
            })
        })?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row?);
        }

        if questions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(questions))
        }
    }

    /// 批量插入推荐记录
    pub fn insert_batch(
        &self,
        day: i64,
        questions: &[RecommendedQuestion],
    ) -> Result<(), DbError> {
        // 先删除当天的旧推荐
        self.conn.execute("DELETE FROM recommendation WHERE day = ?1", [day])?;

        // 插入新推荐
        let mut stmt = self.conn.prepare(
            "INSERT INTO recommendation (day, question_id, score, subject, reason) VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;

        for q in questions {
            let reason_json = serde_json::to_string(&q.reason).unwrap_or_else(|_| "null".to_string());
            stmt.execute(rusqlite::params![day, q.question_id, q.score, q.subject, reason_json])?;
        }

        Ok(())
    }

    /// 删除指定日期的推荐
    pub fn delete_by_day(&self, day: i64) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM recommendation WHERE day = ?1", [day])?;
        Ok(())
    }

    /// 清理旧推荐记录，只保留当前逻辑天的记录
    /// 每次生成新推荐时调用，确保推荐表不会无限增长
    pub fn cleanup_old_recommendations(&self, current_day: i64) -> Result<usize, DbError> {
        let deleted = self.conn.execute(
            "DELETE FROM recommendation WHERE day < ?1",
            [current_day],
        )?;
        Ok(deleted)
    }

    /// 获取今日复习状态
    pub fn get_daily_review_status(&self) -> Result<DailyReviewStatus, DbError> {
        let now = crate::util::time::now_ts();
        let cfg = ClockConfig::default();
        let day = LogicalDay::from_timestamp(now, &cfg).0;

        // 获取今天推荐的题目数量
        let recommended_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM recommendation WHERE day = ?1",
            [day],
            |row| row.get(0)
        )?;

        // 如果没有推荐，返回未完成状态
        if recommended_count == 0 {
            return Ok(DailyReviewStatus {
                recommended_count: 0,
                reviewed_count: 0,
                is_completed: false,
            });
        }

        // 获取今天的时间范围
        let (day_start, day_end) = crate::util::time::range_of_day(LogicalDay(day), &cfg);

        // 获取今天复习过的题目数量（在推荐范围内的）
        let reviewed_count: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(DISTINCT r.question_id)
            FROM review r
            INNER JOIN recommendation rec ON r.question_id = rec.question_id
            WHERE rec.day = ?1
              AND r.reviewed_at >= ?2
              AND r.reviewed_at < ?3
            "#,
            rusqlite::params![day, day_start.0, day_end.0],
            |row| row.get(0)
        )?;

        Ok(DailyReviewStatus {
            recommended_count,
            reviewed_count,
            is_completed: reviewed_count >= recommended_count,
        })
    }

    /// 获取今日复习记录详情
    pub fn get_today_review_records(&self) -> Result<Vec<ReviewRecord>, DbError> {
        let now = crate::util::time::now_ts();
        let cfg = ClockConfig::default();
        let day = LogicalDay::from_timestamp(now, &cfg).0;
        let (day_start, day_end) = crate::util::time::range_of_day(LogicalDay(day), &cfg);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                r.question_id,
                q.name,
                r.result,
                r.reviewed_at,
                rec.subject
            FROM review r
            INNER JOIN recommendation rec ON r.question_id = rec.question_id
            LEFT JOIN question q ON r.question_id = q.id
            WHERE rec.day = ?1
              AND r.reviewed_at >= ?2
              AND r.reviewed_at < ?3
            ORDER BY r.reviewed_at DESC
            "#
        )?;

        let rows = stmt.query_map(rusqlite::params![day, day_start.0, day_end.0], |row| {
            Ok(ReviewRecord {
                question_id: row.get(0)?,
                question_name: row.get(1)?,
                result: row.get(2)?,
                reviewed_at: row.get(3)?,
                subject: row.get(4)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    /// 获取今日未复习的推荐题目
    ///
    /// `subject`：可选的科目筛选。
    /// - `None` 不过滤，返回所有科目。
    /// - `Some("未分类")` 只返回未标注科目的题目。
    /// - `Some("其他")` 只返回该科目。
    pub fn get_pending_recommendations(
        &self,
        limit: i64,
        subject: Option<&str>,
    ) -> Result<Vec<RecommendedQuestion>, DbError> {
        let now = crate::util::time::now_ts();
        let cfg = ClockConfig::default();
        let day = LogicalDay::from_timestamp(now, &cfg).0;
        let (day_start, day_end) = crate::util::time::range_of_day(LogicalDay(day), &cfg);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                r.question_id,
                q.name,
                r.score,
                q.state,
                q.due_at,
                q.correct_streak,
                q.wrong_count,
                q.last_result,
                rs.error_rate,
                r.subject,
                r.reason
            FROM recommendation r
            JOIN question q ON r.question_id = q.id
            LEFT JOIN review_summary rs ON r.question_id = rs.question_id
            WHERE r.day = ?1
              AND q.deleted_at IS NULL
              AND (?5 IS NULL OR r.subject = ?5)
              AND NOT EXISTS (
                  SELECT 1 FROM review rev
                  WHERE rev.question_id = r.question_id
                    AND rev.reviewed_at >= ?2
                    AND rev.reviewed_at < ?3
              )
            ORDER BY r.score DESC
            LIMIT ?4
            "#
        )?;

        let rows = stmt.query_map(
            rusqlite::params![day, day_start.0, day_end.0, limit, subject],
            |row| {
            let reason_text: Option<String> = row.get(10)?;
            let reason = reason_text
                .filter(|t| t != "null")
                .and_then(|t| serde_json::from_str(&t).ok());
            Ok(RecommendedQuestion {
                question_id: row.get(0)?,
                name: row.get(1)?,
                score: row.get(2)?,
                state: row.get(3)?,
                due_at: row.get(4)?,
                correct_streak: row.get(5)?,
                wrong_count: row.get(6)?,
                last_result: row.get(7)?,
                error_rate: row.get(8)?,
                subject: row.get(9)?,
                reason,
                score_detail: None,
                review_count: 0,
                created_at: 0,
            })
        })?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row?);
        }

        Ok(questions)
    }
}

/// 复习记录
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReviewRecord {
    pub question_id: i64,
    pub question_name: Option<String>,
    pub result: String,
    pub reviewed_at: i64,
    pub subject: Option<String>,
}
