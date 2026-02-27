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
}
