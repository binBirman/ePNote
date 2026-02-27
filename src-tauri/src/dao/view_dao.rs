use crate::db::connection;
use crate::db::error::DbError;
use crate::db::schema::view_schema::{
    select_deleted_views_page, select_view_by_id, select_views_by_name, select_views_page,
    select_views_page_by_state, select_views_page_by_subject,
};
use crate::domain::enums::QuestionState;
use crate::domain::ids::QuestionId;
use crate::domain::View;
use crate::util::time::Timestamp;
pub use rusqlite::Connection;
use std::convert::TryFrom;

/// DAO for `Question` using the lightweight `db` schema functions and repo converters.

pub struct ViewDao<'a> {
    conn: &'a Connection,
}

impl<'a> ViewDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 根据领域层 `QuestionId` 查询题目，找不到返回 `Ok(None)`。
    pub fn get_by_id(&self, id: QuestionId) -> Result<View, DbError> {
        let id_i64: i64 = i64::from(id);
        let row = select_view_by_id(self.conn, id_i64)?;

        let state = QuestionState::try_from(row.state.clone())
            .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
        let view = View {
            id: QuestionId::from(row.id),
            name: row.name.clone(),
            state,
            created_at: Timestamp::from(row.created_at),
            deleted_at: row.deleted_at.map(Timestamp::from),
            subject: if row.subject.is_empty() {
                None
            } else {
                Some(row.subject.clone())
            },
            last_reviewed_at: Timestamp::from(row.last_reviewed_at),
        };

        Ok(view)
    }

    /// 根据领域层 `QuestionName` 查询题目，找不到返回 `Ok(None)`。
    pub fn get_by_name(&self, name: &str) -> Result<Vec<View>, DbError> {
        // `select_views_by_name` now returns a vector of rows (or Err if not found).
        let rows = select_views_by_name(self.conn, name)?;
        let mut views = Vec::new();
        for row in rows {
            let state = QuestionState::try_from(row.state.clone())
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            views.push(View {
                id: QuestionId::from(row.id),
                name: row.name.clone(),
                state,
                created_at: Timestamp::from(row.created_at),
                deleted_at: row.deleted_at.map(Timestamp::from),
                subject: if row.subject.is_empty() {
                    None
                } else {
                    Some(row.subject.clone())
                },
                last_reviewed_at: Timestamp::from(row.last_reviewed_at),
            });
        }

        Ok(views)
    }

    /// 分页输出题目列表（不包含已删除题目）。
    pub fn list(&self, offset: i64, limit: i64) -> Result<Vec<View>, DbError> {
        let rows = select_views_page(self.conn, offset, limit)?;
        let mut views = Vec::new();
        for row in rows {
            let state = QuestionState::try_from(row.state.clone())
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            views.push(View {
                id: QuestionId::from(row.id),
                name: row.name.clone(),
                state,
                created_at: Timestamp::from(row.created_at),
                deleted_at: row.deleted_at.map(Timestamp::from),
                subject: if row.subject.is_empty() {
                    None
                } else {
                    Some(row.subject.clone())
                },
                last_reviewed_at: Timestamp::from(row.last_reviewed_at),
            });
        }
        if views.is_empty() {
            Err(DbError::NotFound)
        } else {
            Ok(views)
        }
    }

    /// 分页输已删除出题目列表。
    pub fn list_deleted(&self, offset: i64, limit: i64) -> Result<Vec<View>, DbError> {
        let rows = select_deleted_views_page(self.conn, offset, limit)?;
        let mut views = Vec::new();
        for row in rows {
            let state = QuestionState::try_from(row.state.clone())
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            views.push(View {
                id: QuestionId::from(row.id),
                name: row.name.clone(),
                state,
                created_at: Timestamp::from(row.created_at),
                deleted_at: row.deleted_at.map(Timestamp::from),
                subject: if row.subject.is_empty() {
                    None
                } else {
                    Some(row.subject.clone())
                },
                last_reviewed_at: Timestamp::from(row.last_reviewed_at),
            });
        }
        if views.is_empty() {
            Err(DbError::NotFound)
        } else {
            Ok(views)
        }
    }

    /// 分页提取某状态的题目列表（不包含已删除题目）。
    pub fn list_by_state(
        &self,
        state: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<View>, DbError> {
        let rows = select_views_page_by_state(self.conn, offset, limit, state)?;
        let mut views = Vec::new();
        for row in rows {
            let st = QuestionState::try_from(row.state.clone())
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            views.push(View {
                id: QuestionId::from(row.id),
                name: row.name.clone(),
                state: st,
                created_at: Timestamp::from(row.created_at),
                deleted_at: row.deleted_at.map(Timestamp::from),
                subject: if row.subject.is_empty() {
                    None
                } else {
                    Some(row.subject.clone())
                },
                last_reviewed_at: Timestamp::from(row.last_reviewed_at),
            });
        }
        if views.is_empty() {
            Err(DbError::NotFound)
        } else {
            Ok(views)
        }
    }

    /// 分页提取某科目的所有题目视图列表（不包含已删除题目）
    pub fn list_by_subject(
        &self,
        subject: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<View>, DbError> {
        // 返回全部匹配 subject 的视图（不分页）
        let rows = select_views_page_by_subject(self.conn, offset, limit, subject)?;
        let mut views = Vec::new();
        for row in rows {
            let st = QuestionState::try_from(row.state.clone())
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            views.push(View {
                id: QuestionId::from(row.id),
                name: row.name.clone(),
                state: st,
                created_at: Timestamp::from(row.created_at),
                deleted_at: row.deleted_at.map(Timestamp::from),
                subject: if row.subject.is_empty() {
                    None
                } else {
                    Some(row.subject.clone())
                },
                last_reviewed_at: Timestamp::from(row.last_reviewed_at),
            });
        }
        if views.is_empty() {
            Err(DbError::NotFound)
        } else {
            Ok(views)
        }
    }
}
