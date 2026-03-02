use rusqlite::Connection;
use serde::Deserialize;
use serde::Serialize;
use tauri::State;

use crate::app::appstate::{AppInner, AppState};
use crate::dao::ViewDao;
use crate::domain::{QuestionState, View};
use crate::server::show_question_view::*;
// use crate::server::show_question_view::{
//     list_available_questions_page, list_deleted_questions_page, list_questions_by_state_page,
//     list_questions_by_subject_page,
// };
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
                last_review: LogicalDay::from(v.last_reviewed_at).to_string(),
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

#[tauri::command]
pub fn show_subjects(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let subjects = crate::db::select_all_subjects(conn).unwrap_or_default();
    print!("查询到的科目数量 = {}\n", subjects.len());
    Ok(subjects)
}

#[tauri::command]
pub fn show_states(_state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    // 返回所有 QuestionState 的字符串表示
    let states = vec![
        crate::domain::QuestionState::NEW.as_str().to_string(),
        crate::domain::QuestionState::LEARNING.as_str().to_string(),
        crate::domain::QuestionState::STABLE.as_str().to_string(),
        crate::domain::QuestionState::DUE.as_str().to_string(),
        crate::domain::QuestionState::SUSPENDED.as_str().to_string(),
    ];
    Ok(states)
}

#[tauri::command]
pub fn show_list_available_questions_page(
    state: tauri::State<AppState>,
    page: usize,
    page_size: usize,
) -> Result<Vec<ActiveQuestion>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized\n".to_string()),
    };
    let views = list_available_questions_page(&conn, page, page_size).unwrap_or_default();
    let result = ActiveQuestion::new(views);
    print!("查询到的可用题目数量 = {}\n", result.len());
    Ok(result)
}

#[tauri::command]
pub fn show_list_deleted_questions_page(
    state: tauri::State<AppState>,
    page: usize,
    page_size: usize,
) -> Result<Vec<DeleteQuestion>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized\n".to_string()),
    };
    let views = list_deleted_questions_page(&conn, page, page_size).unwrap_or_default();
    let result = DeleteQuestion::new(views);
    print!("查询到的已删除题目数量 = {}\n", result.len());
    Ok(result)
}

#[tauri::command]
pub fn show_list_available_questions_by_state_page(
    state: tauri::State<AppState>,
    question_state: String,
    page: usize,
    page_size: usize,
) -> Result<Vec<ActiveQuestion>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized\n".to_string()),
    };
    let state = QuestionState::try_from(question_state.clone())
        .map_err(|e| format!("invalid question state: {:?}", e))?;
    let views = list_questions_by_state_page(&conn, state, page, page_size).unwrap_or_default();
    let result = ActiveQuestion::new(views);
    print!(
        "查询到的状态为 {:?} 的题目数量 = {}\n",
        question_state,
        result.len()
    );
    Ok(result)
}

#[tauri::command]
pub fn show_list_available_questions_by_subject_page(
    state: tauri::State<AppState>,
    subject: String,
    page: usize,
    page_size: usize,
) -> Result<Vec<ActiveQuestion>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized\n".to_string()),
    };
    let views =
        list_questions_by_subject_page(&conn, subject.clone(), page, page_size).unwrap_or_default();
    let result = ActiveQuestion::new(views);
    print!(
        "查询到的科目为 {:?} 的题目数量 = {}\n",
        subject.clone(),
        result.len()
    );
    Ok(result)
}
