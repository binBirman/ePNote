use crate::db::connection::Connection;
use crate::db::error::DbError;
use crate::domain::{
    enums::ReviewResult,
    ids::{QuestionId, ReviewId},
    review::Review,
};
use crate::repo::primitive::*;
use crate::util::time::Timestamp;

/// DAO for `Review` using the lightweight `db` schema functions and repo converters.
pub struct ReviewDao<'a> {
    conn: &'a Connection,
}

impl<'a> ReviewDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 根据领域层 `ReviewId` 查询复习记录，找不到返回 `Ok(None)`。
    pub fn get_by_id(&self, id: ReviewId) -> Result<Option<Review>, DbError> {
        let id_i64: i64 = i64::from(id);
        if let Some(row) = crate::db::select_review_by_id(self.conn, id_i64)? {
            let r = crate::repo::review_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            Ok(Some(r))
        } else {
            Ok(None)
        }
    }

    /// 列出某题目的所有复习记录。
    pub fn list_by_question(&self, question_id: QuestionId) -> Result<Vec<Review>, DbError> {
        let qid_i64: i64 = i64::from(question_id);
        let rows = crate::db::select_reviews_by_question_id(self.conn, qid_i64)?;
        let mut reviews = Vec::new();
        for row in rows {
            let r = crate::repo::review_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            reviews.push(r);
        }
        Ok(reviews)
    }

    /// 插入复习记录，返回新记录的自增 ID。
    pub fn insert(
        &self,
        qid: QuestionId,
        result: ReviewResult,
        created_at: Timestamp,
    ) -> Result<ReviewId, DbError> {
        let timestamp_i64: i64 = created_at.into();
        let result_str = result.as_str();
        let id =
            crate::db::insert_review(self.conn, i64::from(qid), result_str, timestamp_i64)?;
        Ok(ReviewId::from(id))
    }

    /// 统计总复习次数
    pub fn count_all(&self) -> Result<i64, DbError> {
        crate::db::count_reviews(self.conn)
    }

    /// 统计指定结果的复习次数
    pub fn count_by_result(&self, result: &str) -> Result<i64, DbError> {
        crate::db::count_reviews_by_result(self.conn, result)
    }

    /// 统计今日复习次数
    pub fn count_today(&self, today_start: i64) -> Result<i64, DbError> {
        crate::db::count_reviews_since(self.conn, today_start)
    }

    /// 统计今日指定结果的复习次数
    pub fn count_today_by_result(&self, today_start: i64, result: &str) -> Result<i64, DbError> {
        crate::db::count_reviews_since_by_result(self.conn, today_start, result)
    }

    /// 插入复习记录（使用字符串结果），返回新记录的自增 ID。
    pub fn insert_str(
        &self,
        qid: QuestionId,
        result: &str,
        created_at: Timestamp,
    ) -> Result<ReviewId, DbError> {
        let timestamp_i64: i64 = created_at.into();
        let id = crate::db::insert_review(self.conn, i64::from(qid), result, timestamp_i64)?;
        Ok(ReviewId::from(id))
    }

    /// 获取所有题目的错误率和复习次数（从 review_summary 视图）
    /// 返回 HashMap<question_id, (error_rate, review_count)>
    pub fn get_all_error_rates(&self) -> Result<std::collections::HashMap<i64, (f64, i64)>, DbError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT question_id, error_rate, review_count
            FROM review_summary
            "#
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, i64>(2)?))
        })?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (qid, rate, count) = row?;
            map.insert(qid, (rate, count));
        }

        Ok(map)
    }

    /// 每个科目的错误率统计（不分时间）。`subject_filter = None` 即全部。
    pub fn subject_error_stats(
        &self,
        subject_filter: Option<&str>,
    ) -> Result<Vec<crate::db::SubjectStatRow>, DbError> {
        crate::db::select_subject_error_stats(self.conn, subject_filter)
    }

    /// 每日 × 科目 复习行为时间序列。
    ///
    /// `subject_filter = None` 即"全部科目"；`start_day_bucket` / `end_day_bucket`
    /// 为 `None` 即不限时间范围（拉全部历史）。具体数值由前端按日历月算并
    /// 把 unix sec 减 10800（即本地月初 - 3h）转 day_bucket。
    pub fn review_daily_series(
        &self,
        subject_filter: Option<&str>,
        start_day_bucket: Option<i64>,
        end_day_bucket: Option<i64>,
    ) -> Result<Vec<crate::db::DailySeriesRow>, DbError> {
        crate::db::select_review_daily_series(
            self.conn,
            subject_filter,
            start_day_bucket,
            end_day_bucket,
        )
    }
}
