use crate::db::connection;
use crate::db::error::DbError;
use crate::domain::{
    ids::{MetaId, QuestionId},
    meta::Meta,
};
pub use rusqlite::Connection;

/// DAO for `Meta` using the lightweight `db` schema functions and repo converters.
pub struct MetaDao<'a> {
    conn: &'a Connection,
}

impl<'a> MetaDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 根据领域层 `MetaId` 查询元信息，找不到返回 `Ok(None)`。
    pub fn get_meta_by_id(&self, id: MetaId) -> Result<Option<Meta>, DbError> {
        let id_i64: i64 = i64::from(id);
        if let Some(row) = crate::db::select_meta_by_id(self.conn, id_i64)? {
            let m = crate::repo::meta_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            Ok(Some(m))
        } else {
            Ok(None)
        }
    }

    /// 插入元信息记录，返回新记录的自增 ID。
    pub fn meta_insert(&self, m: &Meta) -> Result<i64, DbError> {
        let row = crate::repo::meta_domain_to_row(m)
            .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;

        let id = crate::db::insert_meta(self.conn, row.question_id, &row.key, &row.value)?;
        Ok(id)
    }

    /// 按 ID 删除元信息（物理删除）。
    pub fn meta_delete(&self, id: MetaId) -> Result<(), DbError> {
        let id_i64: i64 = i64::from(id);
        crate::db::delete_meta(self.conn, id_i64)?;
        Ok(())
    }

    /// 列出某题目的所有元信息。
    pub fn list_meta_by_question(&self, question_id: QuestionId) -> Result<Vec<Meta>, DbError> {
        let qid_i64: i64 = i64::from(question_id);
        let rows = crate::db::select_meta_by_question(self.conn, qid_i64)?;
        let mut metas = Vec::new();
        for row in rows {
            let m = crate::repo::meta_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            metas.push(m);
        }
        Ok(metas)
    }

    /// 查询某题目指定 key 的元信息，找不到返回 `Ok(None)`。
    pub fn get_meta_by_question_key(
        &self,
        question_id: QuestionId,
        key: &str,
    ) -> Result<Option<Meta>, DbError> {
        let qid_i64: i64 = i64::from(question_id);
        if let Some(row) = crate::db::select_meta_by_question_key(self.conn, qid_i64, key)? {
            let m = crate::repo::meta_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            Ok(Some(m))
        } else {
            Ok(None)
        }
    }
}
