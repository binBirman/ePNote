//! 推荐系统命令层

use crate::app::appstate::AppState;
use crate::dao::recommendation_dao::{DailyReviewStatus, RecommendationDao, ReviewRecord};
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
