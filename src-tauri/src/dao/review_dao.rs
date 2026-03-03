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
}
