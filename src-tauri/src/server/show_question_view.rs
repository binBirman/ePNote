//! 该模块负责管理题目视图（`show_view`），提供查询接口。
//! 题目视图是一个数据库视图，包含题目的基本信息和一些常用的元信息（如学科）。
//! 视图的定义在 `src-tauri/src/db/migrate.rs` 中的迁移脚本里，查询接口在 `ViewDao` 中实现。
//! 题目管理器（`question_manager.rs`）会调用 `ViewDao` 来获取题目视图数据，并将其转换为领域模型 `Question`，供上层调用。
//! 主要功能为向UI层提供题目基本信息，但不包含图片等具体信息。
//! 只在题目管理与查询界面使用。
use crate::dao::meta_dao::MetaDao;
use crate::dao::ViewDao;
use crate::domain::ids::QuestionId;
use crate::domain::view::View;
use crate::error::AppError;
use rusqlite::Connection;

/// 列出所有科目（去重）。
pub fn list_subjects(conn: &Connection) -> Result<Vec<String>, AppError> {
    let md = MetaDao::new(conn);
    Ok(md.list_all_subjects()?)
}

/// 按页显示已删除题目。
pub fn list_deleted_questions_page(
    conn: &Connection,
    page: usize,
    page_size: usize,
) -> Result<Vec<View>, AppError> {
    let vd = ViewDao::new(conn);
    let offset = page * page_size;
    Ok(vd.list_deleted(offset as i64, page_size as i64)?)
}

/// 显示按 id 查找题目的结果（任意状态，含已删除）。
pub fn find_question_by_id(conn: &Connection, id: QuestionId) -> Result<View, AppError> {
    let vd = ViewDao::new(conn);
    Ok(vd.get_by_id(id)?)
}

/// 显示按 name 查找题目的结果（精确匹配，可能多条）。
pub fn find_question_by_name(conn: &Connection, name: String) -> Result<Vec<View>, AppError> {
    let vd = ViewDao::new(conn);
    Ok(vd.get_by_name(&name)?)
}

/// 分类查询：按 `subject` 与 `state` 过滤，`page` 为 0-indexed。
pub fn classify_questions(
    conn: &Connection,
    subject: Option<String>,
    question_state: Option<String>,
    page: usize,
    page_size: usize,
) -> Result<Vec<View>, AppError> {
    let vd = ViewDao::new(conn);
    let offset = (page * page_size) as i64;
    let limit = page_size as i64;
    Ok(vd.list_classified(
        subject.as_deref().filter(|s| !s.is_empty()),
        question_state.as_deref().filter(|s| !s.is_empty()),
        offset,
        limit,
    )?)
}

/// 搜索查询：`query` 是用户输入的字符串。
/// - 若能解析为 `i64`，则走 ID 精确模式（0 或 1 行，`page` 必须为 0）。
/// - 否则走模糊模式：`name` 或知识点（`meta.key='system.KnowledgePoint'`）LIKE `%query%`。
/// - 可叠加 `subject` / `state`（Mode B）。
/// - `query` 去除首尾空格后若为空，直接返回空列表（不查库）。
pub fn search_questions(
    conn: &Connection,
    query: String,
    subject: Option<String>,
    question_state: Option<String>,
    page: usize,
    page_size: usize,
) -> Result<Vec<View>, AppError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let vd = ViewDao::new(conn);
    let subject_filter = subject.as_deref().filter(|s| !s.is_empty());
    let state_filter = question_state.as_deref().filter(|s| !s.is_empty());

    if let Ok(id) = trimmed.parse::<i64>() {
        // ID 精确模式：忽略分页（page>0 一律返回空），只取唯一行
        if page != 0 {
            return Ok(Vec::new());
        }
        return Ok(vd.search_by_id(id)?.into_iter().collect());
    }

    // 模糊模式
    let offset = (page * page_size) as i64;
    let limit = page_size as i64;
    Ok(vd.search_fuzzy(trimmed, subject_filter, state_filter, offset, limit)?)
}