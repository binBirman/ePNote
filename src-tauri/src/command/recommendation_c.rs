//! 推荐系统命令层

use crate::app::appstate::AppState;
use crate::app::config;
use crate::dao::meta_dao::MetaDao;
use crate::dao::question_dao::QuestionDao;
use crate::dao::recommendation_dao::{DailyReviewStatus, RecommendationDao, ReviewRecord};
use crate::dao::review_dao::ReviewDao;
use crate::server::recommendation::{
    DailyRecommendation, PreviewRecommendationItem, RecommendationStats, RecommendationSystem,
    RecommendedQuestion,
};

/// 获取每日推荐
#[tauri::command]
pub fn get_daily_recommendation_comm(
    state: tauri::State<AppState>,
) -> Result<DailyRecommendation, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let rs = RecommendationSystem::new(conn);
    let settings = config::load_settings();
    rs.get_daily_recommendation(&settings.subjects, settings.per_subject_daily_limit)
        .map_err(|e| e.to_string())
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
    let settings = config::load_settings();
    let default_limit = settings.default_review_limit as i64;
    dao.get_pending_recommendations(limit.unwrap_or(default_limit)).map_err(|e| e.to_string())
}

/// 预览推荐（对所有题目评分，标记入选/落选，不写库）
#[tauri::command]
pub fn preview_recommendation_comm(
    state: tauri::State<AppState>,
    show_score_detail: bool,
    show_exclusion_reason: bool,
) -> Result<Vec<PreviewRecommendationItem>, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let rs = RecommendationSystem::new(conn);
    let settings = config::load_settings();
    rs.preview_recommendation(
        show_score_detail,
        show_exclusion_reason,
        &settings.subjects,
        settings.per_subject_daily_limit,
    )
    .map_err(|e| e.to_string())
}

/// 重新生成今日推荐（删除缓存后重新生成，记录操作日志）
#[tauri::command]
pub fn regenerate_daily_recommendation_comm(
    state: tauri::State<AppState>,
) -> Result<DailyRecommendation, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };

    let now = crate::util::time::now_ts();
    let day = crate::util::time::LogicalDay::from(now).0 as i64;

    let recommendation_dao = RecommendationDao::new(conn);

    // 获取旧推荐数量（用于日志）
    let old_count = recommendation_dao
        .get_by_day(day)
        .map_err(|e| e.to_string())?
        .map(|qs| qs.len())
        .unwrap_or(0);

    // 删除今日缓存
    recommendation_dao
        .delete_by_day(day)
        .map_err(|e| e.to_string())?;

    // 重新生成
    let rs = RecommendationSystem::new(conn);
    let settings = config::load_settings();
    let daily = rs
        .get_daily_recommendation(&settings.subjects, settings.per_subject_daily_limit)
        .map_err(|e| e.to_string())?;

    let new_count = daily.questions.len();

    // 记录操作日志
    write_regen_log(day, old_count, new_count);

    Ok(daily)
}

/// 获取推荐统计概览
#[tauri::command]
pub fn get_recommendation_stats_comm(
    state: tauri::State<AppState>,
) -> Result<RecommendationStats, String> {
    let guard = state.inner.lock().unwrap();
    let conn = match &*guard {
        Some(inner) => &inner.db,
        None => return Err("App not initialized".to_string()),
    };
    let rs = RecommendationSystem::new(conn);
    let settings = config::load_settings();
    rs.get_recommendation_stats(&settings.subjects)
        .map_err(|e| e.to_string())
}

/// 将重新生成操作写入日志文件
fn write_regen_log(day: i64, old_count: usize, new_count: usize) {
    let root = match config::load_root() {
        Ok(Some(r)) => r,
        _ => return,
    };

    let log_path = root.join("recommendation_operations.log");
    let timestamp = crate::util::time::now_ts().as_i64();
    let entry = format!(
        "[timestamp={}] REGENERATE: day={}, old_count={}, new_count={}\n",
        timestamp, day, old_count, new_count
    );

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", entry);
    }
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
            reason: None,
            score_detail: None,
        });
    }

    // 填充错误率
    let error_rates = review_dao.get_all_error_rates().map_err(|e| e.to_string())?;
    for q in result.iter_mut() {
        if let Some(&(error_rate, _)) = error_rates.get(&q.question_id) {
            q.error_rate = Some(error_rate);
        }
    }

    Ok(result)
}
