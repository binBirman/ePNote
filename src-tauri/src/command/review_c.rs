use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app::{AppInner, AppState};
use crate::domain::ids::QuestionId;
use crate::domain::question::Question;
use crate::server::ReviewManager;

/// 推荐结果数据
#[derive(Serialize, Deserialize)]
pub struct RecommendResultData {
    pub questions: Vec<QuestionData>,
    pub reasons: Vec<String>,
    pub subject: Option<String>,
}

/// 题目数据（简化版，用于推荐列表）
#[derive(Serialize, Deserialize)]
pub struct QuestionData {
    pub id: i64,
    pub name: Option<String>,
    pub state: String,
    pub created_at: String,
    pub last_review_at: Option<String>,
    pub correct_streak: i64,
    pub wrong_count: i64,
    pub due_at: Option<String>,
}

/// 处理复习结果
#[tauri::command]
pub fn process_review_comm(
    state: tauri::State<AppState>,
    question_id: i64,
    result: String, // "CORRECT", "WRONG", "FUZZY"
) -> Result<QuestionData, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let manager = ReviewManager::new(conn);
    let qid = QuestionId::from(question_id);

    // 转换 result 字符串到枚举
    let review_result = match result.as_str() {
        "CORRECT" => crate::domain::enums::ReviewResult::CORRECT,
        "WRONG" => crate::domain::enums::ReviewResult::WRONG,
        "FUZZY" => crate::domain::enums::ReviewResult::FUZZY,
        _ => return Err("Invalid review result".to_string()),
    };

    let question = manager.process_review(qid, review_result, crate::util::time::now_ts())?;
    Ok(question_to_data(question))
}

/// 获取推荐的复习题目
#[tauri::command]
pub fn recommend_questions_comm(
    state: tauri::State<AppState>,
    limit: Option<usize>,
    subject: Option<String>,
) -> Result<RecommendResultData, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let manager = ReviewManager::new(conn);
    let result = manager.recommend(limit, subject.as_deref())?;

    let questions: Vec<QuestionData> = result.questions.into_iter().map(question_to_data).collect();
    let reasons: Vec<String> = result.reasons.into_iter().map(|r| format!("{:?}", r)).collect();

    Ok(RecommendResultData {
        questions,
        reasons,
        subject: result.subject,
    })
}

/// 暂停题目
#[tauri::command]
pub fn suspend_question_comm(
    state: tauri::State<AppState>,
    question_id: i64,
) -> Result<QuestionData, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let manager = ReviewManager::new(conn);
    let qid = QuestionId::from(question_id);
    let question = manager.suspend(qid)?;
    Ok(question_to_data(question))
}

/// 恢复题目
#[tauri::command]
pub fn recover_question_comm(
    state: tauri::State<AppState>,
    question_id: i64,
) -> Result<QuestionData, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let manager = ReviewManager::new(conn);
    let qid = QuestionId::from(question_id);
    let question = manager.recover(qid)?;
    Ok(question_to_data(question))
}

/// 获取所有科目列表
#[tauri::command]
pub fn list_subjects_comm(
    state: tauri::State<AppState>,
) -> Result<Vec<String>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let manager = ReviewManager::new(conn);
    manager.list_subjects()
}

/// 辅助函数：将 Question 转换为 QuestionData
fn question_to_data(question: Question) -> QuestionData {
    QuestionData {
        id: i64::from(question.id),
        name: question.name,
        state: question.state.as_str().to_string(),
        created_at: question.created_at.as_i64().to_string(),
        last_review_at: question.last_review_at.map(|t| t.as_i64().to_string()),
        correct_streak: question.correct_streak,
        wrong_count: question.wrong_count,
        due_at: question.due_at.map(|t| t.as_i64().to_string()),
    }
}

/// 统计结果数据
#[derive(Serialize, Deserialize)]
pub struct StatsData {
    pub total_questions: i64,
    pub today_reviewed: i64,
    pub total_reviews: i64,
    pub correct_count: i64,
    pub wrong_count: i64,
    pub fuzzy_count: i64,
    pub state_counts: StateCountsData,
    pub today_pending: i64,
    /// 平均准确率
    pub average_accuracy: f64,
}

/// 状态统计
#[derive(Serialize, Deserialize)]
pub struct StateCountsData {
    pub new_count: i64,
    pub learning_count: i64,
    pub stable_count: i64,
    pub due_count: i64,
    pub suspended_count: i64,
}

/// 获取统计信息
#[tauri::command]
pub fn get_stats_comm(
    state: tauri::State<AppState>,
) -> Result<StatsData, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let manager = ReviewManager::new(conn);
    let stats = manager.get_stats()?;

    // 计算平均准确率（正确次数 / 总复习次数）
    let total = stats.correct_count + stats.wrong_count + stats.fuzzy_count;
    let average_accuracy = if total > 0 {
        (stats.correct_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(StatsData {
        total_questions: stats.total_questions,
        today_reviewed: stats.today_reviewed,
        total_reviews: stats.total_reviews,
        correct_count: stats.correct_count,
        wrong_count: stats.wrong_count,
        fuzzy_count: stats.fuzzy_count,
        state_counts: StateCountsData {
            new_count: stats.state_counts.new_count,
            learning_count: stats.state_counts.learning_count,
            stable_count: stats.state_counts.stable_count,
            due_count: stats.state_counts.due_count,
            suspended_count: stats.state_counts.suspended_count,
        },
        today_pending: stats.today_pending,
        average_accuracy,
    })
}
