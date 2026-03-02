use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app::{AppInner, AppState};
use crate::domain::{enums::AssetType, enums::MetaKey, enums::SystemMetaKey, ids::QuestionId};
use crate::server::question_manager::{
    add_question_images, create_question, delete_question, delete_question_image,
    get_question_detail, rename_question, restore_question,
};
use crate::util::time::LogicalDay;

#[derive(Serialize, Deserialize)]
pub struct QuestionImageData {
    pub path: String,
    pub asset_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct QuestionInfoData {
    pub id: i64,
    pub name: Option<String>,
    pub state: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
    pub subject: Option<String>,
    pub knowledge_points: Vec<String>,
    pub question_images: Vec<QuestionImageData>,
    pub answer_images: Vec<QuestionImageData>,
    pub last_reviewed_at: Option<String>,
}

#[tauri::command]
pub fn create_question_comm(
    state: tauri::State<AppState>,
    name: String,
    question_image_paths: Vec<String>,
    answer_image_paths: Vec<String>,
    subject: Option<String>,
    knowledge_points: Vec<String>,
) -> Result<String, String> {
    // Debug: Print received image paths
    println!("[DEBUG] create_question_comm received:");
    println!("  name: {}", name);
    println!("  question_image_paths count: {}", question_image_paths.len());
    for (i, p) in question_image_paths.iter().enumerate() {
        println!("    [{}]: {}", i, p);
    }
    println!("  answer_image_paths count: {}", answer_image_paths.len());
    for (i, p) in answer_image_paths.iter().enumerate() {
        println!("    [{}]: {}", i, p);
    }

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

#[tauri::command]
pub fn get_question_detail_comm(
    state: tauri::State<AppState>,
    id: i64,
) -> Result<QuestionInfoData, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let store = match &*guard {
        Some(inner) => &inner.asset_store,
        None => return Err("App not initialized".to_string()),
    };
    let store_root = store.root().clone();
    let qid = QuestionId::from(id);

    match get_question_detail(&conn, qid) {
        Ok(q_info) => {
            // Extract question info
            let question = q_info.question;

            // Extract subject from metas
            let subject = q_info
                .metas
                .iter()
                .find(|m| matches!(m.key, MetaKey::System(SystemMetaKey::Subject)))
                .map(|m| m.value.clone());

            // Extract knowledge points from metas
            let knowledge_points: Vec<String> = q_info
                .metas
                .iter()
                .filter(|m| matches!(m.key, MetaKey::System(SystemMetaKey::KnowledgePoint)))
                .map(|m| m.value.clone())
                .collect();

            // Extract question images from assets (convert to full path)
            // 使用正斜杠以兼容 asset 协议
            let question_images: Vec<QuestionImageData> = q_info
                .assets
                .iter()
                .filter(|a| a.asset_type == AssetType::QUESTION && a.deleted_at.is_none())
                .map(|a| {
                    let full_path = store_root.join(a.path.as_str());
                    // 转换为正斜杠格式，兼容 asset 协议
                    QuestionImageData {
                        path: full_path.to_string_lossy().replace('\\', "/"),
                        asset_id: Some(a.id.0.to_string()),
                    }
                })
                .collect();

            // Extract answer images from assets (convert to full path)
            let answer_images: Vec<QuestionImageData> = q_info
                .assets
                .iter()
                .filter(|a| a.asset_type == AssetType::ANSWER && a.deleted_at.is_none())
                .map(|a| {
                    let full_path = store_root.join(a.path.as_str());
                    // 转换为正斜杠格式，兼容 asset 协议
                    QuestionImageData {
                        path: full_path.to_string_lossy().replace('\\', "/"),
                        asset_id: Some(a.id.0.to_string()),
                    }
                })
                .collect();

            // Get last reviewed date from reviews
            let last_reviewed_at = q_info.reviews.first().map(|r| {
                LogicalDay::from(r.reviewed_at).to_string()
            });

            Ok(QuestionInfoData {
                id: question.id.into(),
                name: question.name,
                state: question.state.as_str().to_string(),
                created_at: LogicalDay::from(question.created_at).to_string(),
                deleted_at: question.deleted_at.map(|ts| LogicalDay::from(ts).to_string()),
                subject,
                knowledge_points,
                question_images,
                answer_images,
                last_reviewed_at,
            })
        }
        Err(e) => {
            eprintln!("Failed to get question detail: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateQuestionInput {
    pub name: Option<String>,
    pub subject: Option<String>,
    pub knowledge_points: Option<Vec<String>>,
}

#[tauri::command]
pub fn update_question_comm(
    state: tauri::State<AppState>,
    id: i64,
    data: UpdateQuestionInput,
) -> Result<String, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    // 更新题目名称
    if let Some(name) = &data.name {
        if let Err(e) = rename_question(conn, QuestionId::from(id), name.clone()) {
            eprintln!("Failed to update question name: {:?}", e);
            return Err(e.to_string());
        }
    }

    // TODO: 后续可以添加更新科目和知识点的功能
    // 目前后端只支持更新名称

    println!("Question updated with ID: {}", id);
    Ok(id.to_string())
}

/// 添加题目的图片
#[tauri::command]
pub fn add_question_images_comm(
    state: tauri::State<AppState>,
    id: i64,
    image_paths: Vec<String>,
    image_type: String,
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

    let asset_type = match image_type.as_str() {
        "question" => AssetType::QUESTION,
        "answer" => AssetType::ANSWER,
        _ => return Err("Invalid image type".to_string()),
    };

    match add_question_images(
        &conn,
        &store,
        QuestionId::from(id),
        image_paths,
        asset_type,
    ) {
        Ok(_) => {
            println!("Images added to question {}", id);
            Ok(id.to_string())
        }
        Err(e) => {
            eprintln!("Failed to add images: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 删除题目的图片
#[tauri::command]
pub fn delete_question_image_comm(
    state: tauri::State<AppState>,
    asset_id: String,
) -> Result<String, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let asset_id_clone = asset_id.clone();
    match delete_question_image(&conn, asset_id) {
        Ok(_) => {
            println!("Image deleted: {}", asset_id_clone);
            Ok(asset_id_clone)
        }
        Err(e) => {
            eprintln!("Failed to delete image: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 读取图片文件并返回 base64 编码数据
#[tauri::command]
pub fn get_image_base64(path: String) -> Result<String, String> {
    use std::fs;

    // 读取文件
    let data = fs::read(&path).map_err(|e| format!("Failed to open file: {}", e))?;

    // 转换为 base64
    let base64_str = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);

    // 根据文件扩展名确定 MIME 类型
    let mime_type = if path.to_lowercase().ends_with(".png") {
        "image/png"
    } else if path.to_lowercase().ends_with(".jpg") || path.to_lowercase().ends_with(".jpeg") {
        "image/jpeg"
    } else if path.to_lowercase().ends_with(".gif") {
        "image/gif"
    } else if path.to_lowercase().ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    };

    Ok(format!("data:{};base64,{}", mime_type, base64_str))
}
