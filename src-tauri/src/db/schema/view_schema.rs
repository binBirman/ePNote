use rusqlite::Result;

use crate::db::error::DbError;
use crate::db::Connection;

#[derive(Debug, Clone)]
pub struct ViewRow {
    pub id: i64,
    pub name: Option<String>,
    pub state: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
    pub subject: Option<String>,
    pub last_reviewed_at: Option<i64>,
    pub wrong_count: i64,
    pub error_rate: Option<f64>,
}

const KP_META_KEY: &str = "system.KnowledgePoint";

/// 从 show_view 的 SELECT 行构建 ViewRow。
/// 调用方必须按以下顺序列出列：id, name, state, created_at, deleted_at,
/// subject, last_reviewed_at, wrong_count, error_rate。
fn row_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<ViewRow> {
    Ok(ViewRow {
        id: row.get(0)?,
        name: row.get(1)?,
        state: row.get(2)?,
        created_at: row.get(3)?,
        deleted_at: row.get(4)?,
        subject: row.get(5)?,
        last_reviewed_at: row.get(6)?,
        wrong_count: row.get(7)?,
        error_rate: row.get(8)?,
    })
}

/// 列出所有不重复的科目（来自 `show_view`，过滤空值）。
pub fn select_all_subjects(conn: &Connection) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT DISTINCT subject
        FROM show_view
        "#,
    )?;

    let subjects_iter = stmt.query_map((), |row| {
        let s: Option<String> = row.get(0)?;
        Ok(s)
    })?;

    let subjects = subjects_iter
        .filter_map(|r| match r {
            Ok(opt) => opt,
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    Ok(subjects)
}

/// 用 ID 查找题目视图（包含已删除题目）。
pub fn select_view_by_id(conn: &Connection, id: i64) -> Result<ViewRow, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at,
               wrong_count, error_rate
        FROM show_view
        WHERE id = ?1
        "#,
    )?;

    let mut rows = stmt.query((id,))?;
    if let Some(row) = rows.next()? {
        Ok(row_from(row)?)
    } else {
        Err(DbError::NotFound)
    }
}

/// 按名称精确查找题目视图（多个共享同一名称时返回多条）。
pub fn select_views_by_name(conn: &Connection, name: &str) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at,
               wrong_count, error_rate
        FROM show_view
        WHERE name = ?1
        "#,
    )?;

    let iter = stmt.query_map((name,), row_from)?;
    let rows: Vec<ViewRow> = iter.collect::<Result<_, _>>()?;

    if rows.is_empty() {
        Err(DbError::NotFound)
    } else {
        Ok(rows)
    }
}

/// 分页列出已删除题目视图。
pub fn select_deleted_views_page(
    conn: &Connection,
    offset: i64,
    limit: i64,
) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at,
               wrong_count, error_rate
        FROM show_view
        WHERE deleted_at IS NOT NULL
        ORDER BY deleted_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map((limit, offset), row_from)?;
    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// 分类查询：按 `subject` 与 `state` 过滤未删除题目，两者均可选。
///
/// 0-indexed 分页：`offset = page * page_size`。
pub fn select_views_classified(
    conn: &Connection,
    subject: Option<&str>,
    state: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ViewRow>, DbError> {
    let subject_present = subject.is_some();
    let state_present = state.is_some();

    let mut sql = String::from(
        "SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at, \
                wrong_count, error_rate \
         FROM show_view \
         WHERE deleted_at IS NULL",
    );
    if subject_present {
        sql.push_str(" AND subject = ?");
    }
    if state_present {
        sql.push_str(" AND state = ?");
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

    let mut stmt = conn.prepare(&sql)?;

    // 用具体类型让 rusqlite 能正确绑定 NULL
    let subject_param: Option<String> = subject.map(String::from);
    let state_param: Option<String> = state.map(String::from);

    // 占位符顺序：subject?, state?, limit, offset
    let iter = match (subject_present, state_present) {
        (true, true) => stmt.query_map(
            rusqlite::params![subject_param, state_param, limit, offset],
            row_from,
        )?,
        (true, false) => stmt.query_map(
            rusqlite::params![subject_param, limit, offset],
            row_from,
        )?,
        (false, true) => stmt.query_map(
            rusqlite::params![state_param, limit, offset],
            row_from,
        )?,
        (false, false) => stmt.query_map(rusqlite::params![limit, offset], row_from)?,
    };

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// 按 ID 精确查找未删除题目，返回 `Option`（找不到/已删除都给 None）。
pub fn select_view_active_by_id(
    conn: &Connection,
    id: i64,
) -> Result<Option<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at,
               wrong_count, error_rate
        FROM show_view
        WHERE deleted_at IS NULL AND id = ?1
        "#,
    )?;

    let mut rows = stmt.query((id,))?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_from(row)?))
    } else {
        Ok(None)
    }
}

/// 模糊搜索：`pattern` 必须已经包含 `%` 通配符。
/// 匹配 `name` 或知识点（`meta.key = 'system.KnowledgePoint'` 的 `value`），用 EXISTS 子查询。
/// 可叠加 `subject` / `state` 过滤（Mode B）。
pub fn select_views_search_fuzzy(
    conn: &Connection,
    pattern: &str,
    subject: Option<&str>,
    state: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ViewRow>, DbError> {
    let subject_present = subject.is_some();
    let state_present = state.is_some();

    let mut sql = String::from(
        "SELECT v.id, v.name, v.state, v.created_at, v.deleted_at, v.subject, v.last_reviewed_at, \
                v.wrong_count, v.error_rate \
         FROM show_view v \
         WHERE v.deleted_at IS NULL \
           AND (v.name LIKE ? \
             OR EXISTS (SELECT 1 FROM meta m \
                        WHERE m.question_id = v.id \
                          AND m.key = '",
    );
    sql.push_str(KP_META_KEY);
    sql.push_str("' AND m.value LIKE ?))");
    if subject_present {
        sql.push_str(" AND v.subject = ?");
    }
    if state_present {
        sql.push_str(" AND v.state = ?");
    }
    sql.push_str(" ORDER BY v.created_at DESC LIMIT ? OFFSET ?");

    let mut stmt = conn.prepare(&sql)?;

    let subject_param: Option<String> = subject.map(String::from);
    let state_param: Option<String> = state.map(String::from);

    // 占位符顺序：pattern(name), pattern(kp), subject?, state?, limit, offset
    let iter = match (subject_present, state_present) {
        (true, true) => stmt.query_map(
            rusqlite::params![pattern, pattern, subject_param, state_param, limit, offset],
            row_from,
        )?,
        (true, false) => stmt.query_map(
            rusqlite::params![pattern, pattern, subject_param, limit, offset],
            row_from,
        )?,
        (false, true) => stmt.query_map(
            rusqlite::params![pattern, pattern, state_param, limit, offset],
            row_from,
        )?,
        (false, false) => stmt.query_map(
            rusqlite::params![pattern, pattern, limit, offset],
            row_from,
        )?,
    };

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}