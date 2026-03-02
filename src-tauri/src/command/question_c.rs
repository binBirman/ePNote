use serde::Deserialize;
use tauri::State;

use crate::app::{AppInner, AppState};
use crate::domain::QuestionId;
use crate::server::question_manager::*;

#[tauri::command]
pub fn create_question_comm(
    state: tauri::State<AppState>,
    name: String,
    question_image_paths: Vec<String>,
    answer_image_paths: Vec<String>,
    subject: Option<String>,
    knowledge_points: Vec<String>,
) -> Result<String, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let store = match &*guard {
        Some(inner) => &inner.asset_store,
        None => return Err("App not initialized".to_string()),
    };
    match create_question(
        &conn,
        &store,
        name,
        question_image_paths,
        answer_image_paths,
        subject,
        knowledge_points,
    ) {
        Ok(qid) => {
            println!("Question created with ID: {}", qid.0);
            Ok(qid.0.to_string())
        }
        Err(e) => {
            eprintln!("Failed to create question: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn delete_question_comm(state: tauri::State<AppState>, id: i64) -> Result<String, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let qid = QuestionId::from(id);
    match delete_question(&conn, qid) {
        Ok(_) => {
            println!("Question deleted with ID: {}", id);
            Ok(id.to_string())
        }
        Err(e) => {
            eprintln!("Failed to delete question: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn restore_question_comm(state: tauri::State<AppState>, id: i64) -> Result<String, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let qid = QuestionId::from(id);
    match restore_question(&conn, qid) {
        Ok(_) => {
            println!("Question restored with ID: {}", id);
            Ok(id.to_string())
        }
        Err(e) => {
            eprintln!("Failed to restore question: {:?}", e);
            Err(e.to_string())
        }
    }
}
