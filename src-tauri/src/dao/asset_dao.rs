use crate::db::connection;
use crate::db::error::DbError;
use crate::domain::{
    asset::Asset,
    ids::{AssetId, QuestionId},
};
use crate::util::time::Timestamp;
pub use rusqlite::Connection;

/// DAO for `Asset` using the lightweight `db` schema functions and repo converters.
pub struct AssetDao<'a> {
    conn: &'a Connection,
}

impl<'a> AssetDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 根据领域层 `AssetId` 查询资源，找不到返回 `Ok(None)`。
    pub fn get_asset_by_id(&self, id: AssetId) -> Result<Option<Asset>, DbError> {
        let id_i64: i64 = i64::from(id);
        if let Some(row) = crate::db::select_asset_by_id(self.conn, id_i64)? {
            let a = crate::repo::asset_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            Ok(Some(a))
        } else {
            Ok(None)
        }
    }

    /// 插入资源记录，返回新记录的自增 ID。
    pub fn asset_insert(&self, a: &Asset) -> Result<i64, DbError> {
        let row = crate::repo::asset_domain_to_row(a)
            .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;

        let id = crate::db::insert_asset(
            self.conn,
            row.question_id,
            &row.type_,
            &row.path,
            row.created_at,
        )?;
        Ok(id)
    }

    /// 逻辑删除资源：
    /// 1. 调用 `now_ts()` 生成删除时间戳
    /// 2. 将删除时间写入数据库（`deleted_at` 字段）并返回该时间戳
    pub fn asset_delete(&self, id: AssetId) -> Result<Timestamp, DbError> {
        let id_i64: i64 = i64::from(id);
        let ts = crate::util::time::now_ts();
        crate::db::delete_asset(self.conn, id_i64, ts.as_i64())?;
        Ok(ts)
    }

    /// 逻辑恢复资源
    pub fn asset_restore(&self, id: AssetId) -> Result<(), DbError> {
        let id_i64: i64 = i64::from(id);
        crate::db::restore_asset(self.conn, id_i64)?;
        Ok(())
    }

    /// 列出某题目的所有资源（不包含已删除的）。
    pub fn list_asset_by_question(&self, question_id: QuestionId) -> Result<Vec<Asset>, DbError> {
        let qid_i64: i64 = i64::from(question_id);
        let rows = crate::db::select_asset_by_question(self.conn, qid_i64)?;
        let mut assets = Vec::new();
        for row in rows {
            let a = crate::repo::asset_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            assets.push(a);
        }
        Ok(assets)
    }
}
