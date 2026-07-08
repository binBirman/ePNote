use serde::Serialize;
use tauri::State;

use crate::app::appstate::AppState;
use crate::domain::View;
use crate::server::show_question_view;
use crate::util::time::LogicalDay;

#[derive(Serialize)]
pub struct ActiveQuestion {
    pub id: i64,
    pub subject: String,
    pub title: String,
    pub status: String,
    pub knowledge_points: Vec<String>,
    pub created_at: String,
    pub last_review: String,
}
impl ActiveQuestion {
    pub fn new(views: Vec<View>) -> Vec<ActiveQuestion> {
        views
            .into_iter()
            .map(|v| Self {
                id: v.id.into(),
                subject: v.subject,
                title: v.name.unwrap_or_default(),
                status: v.state.as_str().to_string(),
                knowledge_points: v.knowledge_points,
                created_at: LogicalDay::from(v.created_at).to_string(),
                last_review: if v.last_reviewed_at.0 == 0 {
                    String::new()
                } else {
                    LogicalDay::from(v.last_reviewed_at).to_string()
                },
            })
            .collect()
    }
}

#[derive(Serialize)]
pub struct DeleteQuestion {
    pub id: i64,
    pub subject: String,
    pub title: String,
    pub status: String,
    pub knowledge_points: Vec<String>,
    pub deleted_at: String,
}
impl DeleteQuestion {
    pub fn new(views: Vec<View>) -> Vec<DeleteQuestion> {
        views
            .into_iter()
            .map(|v| Self {
                id: v.id.into(),
                subject: v.subject,
                title: v.name.unwrap_or_default(),
                status: v.state.as_str().to_string(),
                knowledge_points: v.knowledge_points,
                deleted_at: v
                    .deleted_at
                    .map(|ts| LogicalDay::from(ts).to_string())
                    .unwrap_or_default(),
            })
            .collect()
    }
}

/// 取得应用数据库连接的辅助函数；未初始化返回错误。
fn conn_or_err<'a>(
    state: &'a State<'_, AppState>,
) -> Result<std::sync::MutexGuard<'a, Option<crate::app::appstate::AppInner>>, String> {
    let guard = state.inner.lock().unwrap();
    if guard.is_none() {
        return Err("App not initialized".to_string());
    }
    Ok(guard)
}

/// 列出所有科目。
#[tauri::command]
pub fn show_subjects(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let guard = conn_or_err(&state)?;
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    show_question_view::list_subjects(conn).map_err(|e| e.to_string())
}

/// 分类查询：按科目 / 状态分页过滤；`page` 0-indexed。
#[tauri::command]
pub fn classify_questions(
    state: tauri::State<AppState>,
    subject: Option<String>,
    question_state: Option<String>,
    page: usize,
    page_size: usize,
) -> Result<Vec<ActiveQuestion>, String> {
    let guard = conn_or_err(&state)?;
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let views = show_question_view::classify_questions(
        conn,
        subject,
        question_state,
        page,
        page_size,
    )
    .map_err(|e| e.to_string())?;
    Ok(ActiveQuestion::new(views))
}

/// 搜索查询：单字符串输入，后端按"纯数字 → ID 精确"或"模糊"分发；可叠加 subject/state。
#[tauri::command]
pub fn search_questions(
    state: tauri::State<AppState>,
    query: String,
    subject: Option<String>,
    question_state: Option<String>,
    page: usize,
    page_size: usize,
) -> Result<Vec<ActiveQuestion>, String> {
    let guard = conn_or_err(&state)?;
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let views = show_question_view::search_questions(
        conn,
        query,
        subject,
        question_state,
        page,
        page_size,
    )
    .map_err(|e| e.to_string())?;
    Ok(ActiveQuestion::new(views))
}

/// 分页列出已删除题目。
#[tauri::command]
pub fn show_list_deleted_questions_page(
    state: tauri::State<AppState>,
    page: usize,
    page_size: usize,
) -> Result<Vec<DeleteQuestion>, String> {
    let guard = conn_or_err(&state)?;
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let views =
        show_question_view::list_deleted_questions_page(conn, page, page_size)
            .map_err(|e| e.to_string())?;
    Ok(DeleteQuestion::new(views))
}