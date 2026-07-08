use crate::dao::meta_dao::MetaDao;
use crate::db::error::DbError;
use crate::db::schema::view_schema::{
    select_deleted_views_page, select_view_active_by_id, select_view_by_id, select_views_by_name,
    select_views_classified, select_views_search_fuzzy,
};
use crate::domain::enums::QuestionState;
use crate::domain::ids::QuestionId;
use crate::domain::View;
use crate::util::time::Timestamp;
use rusqlite::Connection;
use std::convert::TryFrom;

/// DAO for `Question` using the lightweight `db` schema functions and repo converters.
pub struct ViewDao<'a> {
    conn: &'a Connection,
}

const KP_META_KEY: &str = "system.KnowledgePoint";

impl<'a> ViewDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 根据领域层 `QuestionId` 查询题目，找不到返回 `Ok(None)`。
    pub fn get_by_id(&self, id: QuestionId) -> Result<View, DbError> {
        let id_i64: i64 = i64::from(id);
        let row = select_view_by_id(self.conn, id_i64)?;
        let md = MetaDao::new(self.conn);
        let knowledge_points = md.get_values_by_question_key(id, KP_META_KEY)?;
        view_from_row(row, &knowledge_points)
    }

    /// 根据名称精确匹配题目。
    pub fn get_by_name(&self, name: &str) -> Result<Vec<View>, DbError> {
        let rows = select_views_by_name(self.conn, name)?;
        let md = MetaDao::new(self.conn);
        let mut views = Vec::new();
        for row in rows {
            let knowledge_points =
                md.get_values_by_question_key(QuestionId::from(row.id), KP_META_KEY)?;
            views.push(view_from_row(row, &knowledge_points)?);
        }
        Ok(views)
    }

    /// 分页输出已删除题目。
    pub fn list_deleted(&self, offset: i64, limit: i64) -> Result<Vec<View>, DbError> {
        let rows = select_deleted_views_page(self.conn, offset, limit)?;
        self.attach_kps(rows)
    }

    /// 分类查询：`subject` 和 `state` 都可选；同时为 `None` 时返回全部分页结果。
    pub fn list_classified(
        &self,
        subject: Option<&str>,
        state: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<View>, DbError> {
        let rows = select_views_classified(self.conn, subject, state, limit, offset)?;
        self.attach_kps(rows)
    }

    /// 按 ID 精确搜索未删除题目，返回 0 或 1 条。
    pub fn search_by_id(&self, id: i64) -> Result<Option<View>, DbError> {
        let Some(row) = select_view_active_by_id(self.conn, id)? else {
            return Ok(None);
        };
        let md = MetaDao::new(self.conn);
        let kps = md.get_values_by_question_key(QuestionId::from(row.id), KP_META_KEY)?;
        Ok(Some(view_from_row(row, &kps)?))
    }

    /// 模糊搜索：`query` 任意字符串，按 `%query%` 匹配 `name` 或知识点；
    /// 可叠加 `subject` / `state` 过滤（Mode B）。
    pub fn search_fuzzy(
        &self,
        query: &str,
        subject: Option<&str>,
        state: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<View>, DbError> {
        let pattern = format!("%{}%", query);
        let rows = select_views_search_fuzzy(self.conn, &pattern, subject, state, limit, offset)?;
        self.attach_kps(rows)
    }

    /// 批量为 `ViewRow` 列表补充 `knowledge_points`，避免 N+1。
    fn attach_kps(
        &self,
        rows: Vec<crate::db::schema::view_schema::ViewRow>,
    ) -> Result<Vec<View>, DbError> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let qids: Vec<i64> = rows.iter().map(|r| r.id).collect();
        let md = MetaDao::new(self.conn);
        let mut kp_map = md.list_values_by_question_ids(&qids, KP_META_KEY)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let kps = kp_map.remove(&row.id).unwrap_or_default();
            out.push(view_from_row(row, &kps)?);
        }
        Ok(out)
    }
}

fn view_from_row(
    row: crate::db::schema::view_schema::ViewRow,
    knowledge_points: &[String],
) -> Result<View, DbError> {
    let state = QuestionState::try_from(row.state.clone())
        .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
    Ok(View {
        id: QuestionId::from(row.id),
        name: row.name.clone(),
        state,
        created_at: Timestamp::from(row.created_at),
        deleted_at: row.deleted_at.map(Timestamp::from),
        subject: row.subject.clone().unwrap_or_default(),
        knowledge_points: knowledge_points.to_vec(),
        last_reviewed_at: Timestamp::from(row.last_reviewed_at.unwrap_or(0)),
    })
}