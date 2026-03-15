//! 推荐系统命令层

use crate::app::appstate::AppState;
use crate::dao::meta_dao::MetaDao;
use crate::dao::question_dao::QuestionDao;
use crate::dao::recommendation_dao::{DailyReviewStatus, RecommendationDao, ReviewRecord};
use crate::dao::review_dao::ReviewDao;
use crate::server::recommendation::{DailyRecommendation, RecommendationSystem, RecommendedQuestion};

/// 获取每日推荐
#[tauri::command]
pub fn get_daily_recommendation_comm(
    state: tauri::State<AppState>,
    target_count: Option<i64>,
) -> Result<DailyRecommendation, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let rs = RecommendationSystem::new(conn);
    rs.get_daily_recommendation(target_count.unwrap_or(10)).map_err(|e| e.to_string())
}

/// 获取推荐题目列表（用于复习会话）- 只返回未复习的题目
#[tauri::command]
pub fn get_recommendation_list_comm(
    state: tauri::State<AppState>,
    limit: Option<i64>,
) -> Result<Vec<RecommendedQuestion>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let dao = RecommendationDao::new(conn);
    dao.get_pending_recommendations(limit.unwrap_or(10)).map_err(|e| e.to_string())
}

/// 获取今日复习状态
#[tauri::command]
pub fn get_daily_review_status_comm(
    state: tauri::State<AppState>,
) -> Result<DailyReviewStatus, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let dao = RecommendationDao::new(conn);
    dao.get_daily_review_status().map_err(|e| e.to_string())
}

/// 获取今日复习记录（用于总结页面）
#[tauri::command]
pub fn get_today_review_records_comm(
    state: tauri::State<AppState>,
) -> Result<Vec<ReviewRecord>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let dao = RecommendationDao::new(conn);
    dao.get_today_review_records().map_err(|e| e.to_string())
}

/// 根据题目ID列表获取题目（用于练习模式）
#[tauri::command]
pub fn get_questions_by_ids_comm(
    state: tauri::State<AppState>,
    question_ids: Vec<i64>,
) -> Result<Vec<RecommendedQuestion>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    if question_ids.is_empty() {
        return Ok(vec![]);
    }

    let question_dao = QuestionDao::new(conn);
    let meta_dao = MetaDao::new(conn);
    let review_dao = ReviewDao::new(conn);
    let subject_key = "system.Subject";

    let questions = question_dao.get_by_ids(&question_ids).map_err(|e| e.to_string())?;

    let mut result: Vec<RecommendedQuestion> = Vec::new();

    for q in questions {
        let subject = meta_dao
            .get_by_question_key(q.id.clone(), subject_key)
            .ok()
            .flatten()
            .map(|m| m.value);

        result.push(RecommendedQuestion {
            question_id: i64::from(q.id),
            name: q.name,
            score: 0.0, // 不需要评分
            state: q.state.as_str().to_string(),
            due_at: q.due_at.map(|t| t.as_i64()),
            correct_streak: q.correct_streak,
            wrong_count: q.wrong_count,
            last_result: q.last_result.map(|r| r.as_str().to_string()),
            error_rate: None,
            subject,
        });
    }

    // 填充错误率
    let error_rates = review_dao.get_all_error_rates().map_err(|e| e.to_string())?;
    for q in result.iter_mut() {
        if let Some(rate) = error_rates.get(&q.question_id) {
            q.error_rate = Some(*rate);
        }
    }

    Ok(result)
}
