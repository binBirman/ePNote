use serde::Deserialize;
use tauri::State;

use crate::app::AppState;
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
    let conn = state.db.lock().unwrap();
    match create_question(
        &conn,
        &state.asset,
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
    let conn = state.db.lock().unwrap();
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
    let conn = state.db.lock().unwrap();
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
