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
}

/* 输出所有科目 */
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

/*
    用ID查找题目视图
    输入：
        id: 题目ID，必填
    输出：
        若找到，返回Some(ViewRow)
*/
pub fn select_view_by_id(conn: &Connection, id: i64) -> Result<ViewRow, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at
        FROM show_view
        WHERE id = ?1
        "#,
    )?;

    let question_iter = stmt.query_map((id,), |row| {
        Ok(ViewRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            subject: row.get(5)?,
            last_reviewed_at: row.get(6)?,
        })
    })?;

    for question in question_iter {
        return Ok(question?);
    }

    Err(DbError::NotFound)
}

/*
    用名称查找题目
    输入：
        name: 题目名称，必填
    输出：
        若找到，返回Some(ViewRow)
*/
pub fn select_views_by_name(conn: &Connection, name: &str) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at
        FROM show_view
        WHERE name = ?1
        "#,
    )?;

    let question_iter = stmt.query_map((name,), |row| {
        Ok(ViewRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            subject: row.get(5)?,
            last_reviewed_at: row.get(6)?,
        })
    })?;

    let mut results = Vec::new();
    for question in question_iter {
        results.push(question?);
    }

    if results.is_empty() {
        Err(DbError::NotFound)
    } else {
        Ok(results)
    }
}

/*
    分页列出未删除题目视图
    输入：
        limit: 每页记录数
        offset: 偏移量
    输出：
        返回未删除题目视图列表
*/
/// 分页列出未删除题目视图
/// 参数顺序为 `(offset, limit)` 以配合上层 DAO 的调用习惯；内部按 SQL 需要传入 `(limit, offset)`。
pub fn select_views_page(
    conn: &Connection,
    offset: i64,
    limit: i64,
) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at
        FROM show_view
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map((limit, offset), |row| {
        Ok(ViewRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            subject: row.get(5)?,
            last_reviewed_at: row.get(6)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    分页列出特定状态未删除题目视图
    输入：
        offset: 偏移量
        limit: 每页记录数
        state: 题目状态
    输出：
        返回未删除题目视图列表
*/
pub fn select_views_page_by_state(
    conn: &Connection,
    offset: i64,
    limit: i64,
    state: &str,
) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at
        FROM show_view
        WHERE deleted_at IS NULL AND state = ?3
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map((limit, offset, state), |row| {
        Ok(ViewRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            subject: row.get(5)?,
            last_reviewed_at: row.get(6)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    分页列出特定科目未删除题目视图
    输入：
        offset: 偏移量
        limit: 每页记录数
        subject：题目科目
    输出：
        返回未删除题目视图列表
*/
pub fn select_views_page_by_subject(
    conn: &Connection,
    offset: i64,
    limit: i64,
    subject: &str,
) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at
        FROM show_view
        WHERE deleted_at IS NULL AND subject = ?3
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map((limit, offset, subject), |row| {
        Ok(ViewRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            subject: row.get(5)?,
            last_reviewed_at: row.get(6)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    分页列出已删除题目视图
    输入：
        offset: 偏移量
        limit: 每页记录数
    输出：
        返回已删除题目视图列表
*/
pub fn select_deleted_views_page(
    conn: &Connection,
    offset: i64,
    limit: i64,
) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at
        FROM show_view
        WHERE deleted_at IS NOT NULL
        ORDER BY deleted_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map((limit, offset), |row| {
        Ok(ViewRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            subject: row.get(5)?,
            last_reviewed_at: row.get(6)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn select_views_page_by_subject_and_state(
    conn: &Connection,
    offset: i64,
    limit: i64,
    subject: &str,
    state: &str,
) -> Result<Vec<ViewRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at, subject, last_reviewed_at
        FROM show_view
        WHERE deleted_at IS NULL AND subject = ?3 AND state = ?4
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map((limit, offset, subject, state), |row| {
        Ok(ViewRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            subject: row.get(5)?,
            last_reviewed_at: row.get(6)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
