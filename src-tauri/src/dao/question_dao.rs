use core::time;

use crate::db::connection;
use crate::db::error::DbError;
use crate::domain::{enums::QuestionState, ids::QuestionId, question::Question};
use crate::util::time::Timestamp;
pub use rusqlite::Connection;

/// DAO for `Question` using the lightweight `db` schema functions and repo converters.
pub struct QuestionDao<'a> {
    conn: &'a Connection,
}

impl<'a> QuestionDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 根据领域层 `QuestionId` 查询题目，找不到返回 `Ok(None)`。
    pub fn get_by_id(&self, id: QuestionId) -> Result<Option<Question>, DbError> {
        let id_i64: i64 = i64::from(id);
        if let Some(row) = crate::db::select_question_by_id(self.conn, id_i64)? {
            let q = crate::repo::question_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            Ok(Some(q))
        } else {
            Ok(None)
        }
    }

    /// 插入题目，返回新记录的自增 ID。
    pub fn insert(
        &self,
        name: Option<&str>,
        state: QuestionState,
        created_at: Timestamp,
    ) -> Result<QuestionId, DbError> {
        let timestamp_i64: i64 = created_at.into();
        let state_str = state.as_str();
        let id = crate::db::insert_question(self.conn, name, &state_str, timestamp_i64)?;
        Ok(QuestionId::from(id))
    }

    /// 修改题目名称（只传入要改的信息）。
    pub fn update_name(&self, id: QuestionId, name: Option<&str>) -> Result<(), DbError> {
        let id_i64: i64 = i64::from(id);
        crate::db::update_question_name(self.conn, id_i64, name)?;
        Ok(())
    }

    /// 修改题目删除时间（只传入要改的信息）。
    pub fn update_deleted_at(
        &self,
        id: QuestionId,
        deleted_at: Option<Timestamp>,
    ) -> Result<(), DbError> {
        let id_i64: i64 = i64::from(id);
        let del_opt: Option<i64> = deleted_at.map(|t| i64::from(t));
        crate::db::update_question_deleted_at(self.conn, id_i64, del_opt)?;
        Ok(())
    }

    /// 修改题目状态。
    pub fn update_state(&self, id: QuestionId, state: QuestionState) -> Result<(), DbError> {
        let id_i64: i64 = i64::from(id);
        crate::db::update_question_state(self.conn, id_i64, state.as_str())?;
        Ok(())
    }

    /// 更新题目复习相关字段（状态转移后调用）。
    pub fn update_review_fields(
        &self,
        id: QuestionId,
        last_review_at: Option<Timestamp>,
        last_result: Option<&str>,
        correct_streak: i64,
        wrong_count: i64,
        due_at: Option<Timestamp>,
    ) -> Result<(), DbError> {
        let id_i64: i64 = i64::from(id);
        let last_review_i64: Option<i64> = last_review_at.map(|t| t.as_i64());
        let due_at_i64: Option<i64> = due_at.map(|t| t.as_i64());
        crate::db::update_question_review_fields(
            self.conn,
            id_i64,
            last_review_i64,
            last_result,
            correct_streak,
            wrong_count,
            due_at_i64,
        )?;
        Ok(())
    }

    /// 查询指定状态的题目列表。
    pub fn list_by_state(&self, state: QuestionState) -> Result<Vec<Question>, DbError> {
        let rows = crate::db::select_questions_by_state(self.conn, state.as_str())?;
        let mut questions = Vec::new();
        for row in rows {
            let q = crate::repo::question_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            questions.push(q);
        }
        Ok(questions)
    }

    /// 查询指定复习结果的题目列表。
    pub fn list_by_last_result(&self, result: &str) -> Result<Vec<Question>, DbError> {
        let rows = crate::db::select_questions_by_last_result(self.conn, result)?;
        let mut questions = Vec::new();
        for row in rows {
            let q = crate::repo::question_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            questions.push(q);
        }
        Ok(questions)
    }

    /// 查询已到期的题目列表（due_at <= now）。
    pub fn list_due_questions(&self, now: Timestamp) -> Result<Vec<Question>, DbError> {
        let rows = crate::db::select_due_questions(self.conn, now.as_i64())?;
        let mut questions = Vec::new();
        for row in rows {
            let q = crate::repo::question_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            questions.push(q);
        }
        Ok(questions)
    }

    /// 查询从未复习过的题目（NEW 状态）。
    pub fn list_new_questions(&self) -> Result<Vec<Question>, DbError> {
        self.list_by_state(QuestionState::NEW)
    }

    /// 查询长期未复习的题目（last_review_at 较早，且不是 NEW 状态）。
    pub fn list_stale_questions(&self, days_threshold: i64) -> Result<Vec<Question>, DbError> {
        let rows = crate::db::select_stale_questions(self.conn, days_threshold)?;
        let mut questions = Vec::new();
        for row in rows {
            let q = crate::repo::question_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            questions.push(q);
        }
        Ok(questions)
    }
}
