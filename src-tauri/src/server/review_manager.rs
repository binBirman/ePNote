//! 复习管理器 - 处理复习相关业务逻辑
//!
//! 该模块负责：
//! - 处理复习结果并更新题目状态
//! - 检查题目是否到期需要复习
//! - 暂停/恢复题目的复习
//! - 根据推荐算法返回待复习题目列表

use crate::dao::meta_dao::MetaDao;
use crate::dao::question_dao::QuestionDao;
use crate::dao::review_dao::ReviewDao;
use crate::dao::Connection;
use crate::domain::enums::{MetaKey, QuestionState, ReviewResult, SystemMetaKey};
use crate::domain::ids::QuestionId;
use crate::domain::question::Question;
use crate::domain::state_machine::QuestionStateMachine;
use crate::util::time::{now_ts, range_of_day, LogicalDay, Timestamp};

/// 推荐的默认最大数量
const DEFAULT_RECOMMEND_LIMIT: usize = 10;

/// 长期未复习的天数阈值
const STALE_DAYS_THRESHOLD: i64 = 30;

/// 科目元信息的 key
const SUBJECT_META_KEY: &str = "system.Subject";

/// 复习管理器
pub struct ReviewManager<'a> {
    question_dao: QuestionDao<'a>,
    review_dao: ReviewDao<'a>,
    meta_dao: MetaDao<'a>,
}

/// 推荐结果
#[derive(Debug, Clone)]
pub struct RecommendResult {
    /// 推荐的题目列表
    pub questions: Vec<Question>,
    /// 每个题目对应的推荐原因
    pub reasons: Vec<RecommendReason>,
    /// 推荐的科目（如果有）
    pub subject: Option<String>,
}

/// 推荐原因
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendReason {
    /// 最近一次复习结果为"错误"
    WrongResult,
    /// 新录入但尚未复习
    NewQuestion,
    /// 最近复习结果为"模糊"
    FuzzyResult,
    /// 到达建议复习时间
    DueReview,
    /// 长期未复习
    StaleReview,
}

/// 统计结果
#[derive(Debug, Clone)]
pub struct StatsResult {
    /// 总题目数
    pub total_questions: i64,
    /// 今日已复习数
    pub today_reviewed: i64,
    /// 总复习次数
    pub total_reviews: i64,
    /// 正确次数
    pub correct_count: i64,
    /// 错误次数
    pub wrong_count: i64,
    /// 模糊次数
    pub fuzzy_count: i64,
    /// 各状态题目数
    pub state_counts: StateCounts,
    /// 今日待复习数
    pub today_pending: i64,
}

/// 状态统计
#[derive(Debug, Clone)]
pub struct StateCounts {
    pub new_count: i64,
    pub learning_count: i64,
    pub stable_count: i64,
    pub suspended_count: i64,
}

impl<'a> ReviewManager<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            question_dao: QuestionDao::new(conn),
            review_dao: ReviewDao::new(conn),
            meta_dao: MetaDao::new(conn),
        }
    }

    /// 处理复习结果
    ///
    /// # 参数
    /// - question_id: 题目ID
    /// - result: 复习结果
    /// - now: 当前时间
    ///
    /// # 返回
    /// 返回更新后的题目
    pub fn process_review(
        &self,
        question_id: QuestionId,
        result: ReviewResult,
        now: Timestamp,
    ) -> Result<Question, String> {
        // 获取题目
        let question = self
            .question_dao
            .get_by_id(question_id)
            .map_err(|e| format!("failed to get question: {}", e))?
            .ok_or("question not found")?;

        // 检查是否被暂停
        if question.state == QuestionState::SUSPENDED {
            return Err("cannot review a suspended question".to_string());
        }

        // 使用状态机处理复习结果
        let transition = QuestionStateMachine::process_review(&question, result.clone(), now);

        // 保存 result_str，因为在调用 insert 后无法再次使用 result
        let result_str = result.as_str();

        // 记录复习历史
        self.review_dao
            .insert(question_id, result, now)
            .map_err(|e| format!("failed to insert review: {}", e))?;

        // 更新题目状态
        self.question_dao
            .update_state(question_id, transition.new_state)
            .map_err(|e| format!("failed to update state: {}", e))?;

        // 更新复习字段
        self.question_dao
            .update_review_fields(
                question_id,
                Some(now),
                Some(result_str),
                transition.correct_streak,
                transition.wrong_count,
                transition.due_at,
            )
            .map_err(|e| format!("failed to update review fields: {}", e))?;

        // 返回更新后的题目
        self.question_dao
            .get_by_id(question_id)
            .map_err(|e| format!("failed to get updated question: {}", e))?
            .ok_or("question not found after update".to_string())
    }

    /// 暂停题目（任意非 SUSPENDED 状态 → SUSPENDED）。
    /// 把原状态写入 `system.PreSuspendState` meta，恢复时还原。
    pub fn suspend(&self, question_id: QuestionId) -> Result<Question, String> {
        // 获取题目
        let question = self
            .question_dao
            .get_by_id(question_id)
            .map_err(|e| format!("failed to get question: {}", e))?
            .ok_or("question not found")?;

        // 已经暂停的题目不能再次暂停
        if question.state == QuestionState::SUSPENDED {
            return Err("question already suspended".to_string());
        }

        // 1. 记下旧状态到 meta（先清掉可能残留的记录，避免叠加）
        let pre_suspend_key = MetaKey::System(SystemMetaKey::PreSuspendState);
        self.meta_dao
            .delete_by_question_and_key(question_id, pre_suspend_key.clone())
            .map_err(|e| format!("failed to clear pre-suspend meta: {}", e))?;
        self.meta_dao
            .insert(question_id, pre_suspend_key, question.state.as_str())
            .map_err(|e| format!("failed to save pre-suspend state: {}", e))?;

        // 2. 状态机暂停
        let transition = QuestionStateMachine::suspend(&question);

        // 3. 更新题目状态
        self.question_dao
            .update_state(question_id, transition.new_state)
            .map_err(|e| format!("failed to update state: {}", e))?;

        // 4. 更新 due_at = None（SUSPENDED 状态）
        self.question_dao
            .update_review_fields(
                question_id,
                question.last_review_at,
                question.last_result.as_ref().map(|r| r.as_str()),
                transition.correct_streak,
                transition.wrong_count,
                transition.due_at,
            )
            .map_err(|e| format!("failed to update review fields: {}", e))?;

        // 返回更新后的题目
        self.question_dao
            .get_by_id(question_id)
            .map_err(|e| format!("failed to get updated question: {}", e))?
            .ok_or("question not found after update".to_string())
    }

    /// 恢复题目（SUSPENDED → 暂停前的状态，缺省回退 LEARNING）。
    /// 恢复后清掉 `system.PreSuspendState` meta。
    pub fn recover(&self, question_id: QuestionId) -> Result<Question, String> {
        // 获取题目
        let question = self
            .question_dao
            .get_by_id(question_id)
            .map_err(|e| format!("failed to get question: {}", e))?
            .ok_or("question not found")?;

        // 只有暂停的题目才能恢复
        if question.state != QuestionState::SUSPENDED {
            return Err("only suspended questions can be recovered".to_string());
        }

        // 1. 读旧状态（缺省回退 LEARNING）
        let pre_suspend_key = MetaKey::System(SystemMetaKey::PreSuspendState);
        let key_str = pre_suspend_key.as_str();
        let target_state = self
            .meta_dao
            .get_values_by_question_key(question_id, &key_str)
            .map_err(|e| format!("failed to read pre-suspend state: {}", e))?
            .first()
            .and_then(|s| QuestionState::from_str(s))
            .unwrap_or(QuestionState::LEARNING);

        let now = now_ts();

        // 2. 状态机恢复
        let transition = QuestionStateMachine::recover(&question, target_state, now);

        // 3. 更新题目状态
        self.question_dao
            .update_state(question_id, transition.new_state)
            .map_err(|e| format!("failed to update state: {}", e))?;

        // 4. 更新 review 字段（due_at = now）
        self.question_dao
            .update_review_fields(
                question_id,
                question.last_review_at,
                question.last_result.as_ref().map(|r| r.as_str()),
                transition.correct_streak,
                transition.wrong_count,
                transition.due_at,
            )
            .map_err(|e| format!("failed to update review fields: {}", e))?;

        // 5. 清掉 pre-suspend meta
        self.meta_dao
            .delete_by_question_and_key(question_id, pre_suspend_key)
            .map_err(|e| format!("failed to clear pre-suspend meta: {}", e))?;

        // 返回更新后的题目
        self.question_dao
            .get_by_id(question_id)
            .map_err(|e| format!("failed to get updated question: {}", e))?
            .ok_or("question not found after update".to_string())
    }

    /// 获取推荐的复习题目
    ///
    /// # 参数
    /// - limit: 最大推荐数量，默认 10
    /// - subject: 可选的科目筛选
    ///
    /// # 返回
    /// 返回推荐结果
    pub fn recommend(&self, limit: Option<usize>, subject: Option<&str>) -> Result<RecommendResult, String> {
        let limit = limit.unwrap_or(DEFAULT_RECOMMEND_LIMIT);
        let now = now_ts();

        let mut questions = Vec::new();
        let mut reasons = Vec::new();
        let mut added_ids = std::collections::HashSet::new();

        // 辅助函数：检查题目是否进入推荐池
        // 1. SUSPENDED 状态：用户暂停复习，不参与推荐（无论怎么筛选都不选）
        // 2. 科目筛选：subject 为 None 或 "" 时全选；否则 meta 的科目严格匹配
        let subject_matches = |q: &Question, subject: Option<&str>| -> bool {
            if q.state == QuestionState::SUSPENDED {
                return false;
            }
            if subject.is_none() || subject == Some("") {
                return true;
            }
            let subject = subject.unwrap();
            // 查询题目的科目
            match self.meta_dao.get_by_question_key(q.id.clone(), SUBJECT_META_KEY) {
                Ok(Some(meta)) => meta.value == subject,
                Ok(None) => {
                    // 没有科目信息的题目，如果筛选全部则推荐，否则不推荐
                    false
                }
                Err(_) => false, // 查询失败时不推荐
            }
        };

        let add_question = |q: Question, reason: RecommendReason,
                             questions: &mut Vec<Question>,
                             reasons: &mut Vec<RecommendReason>,
                             added_ids: &mut std::collections::HashSet<i64>| {
            let id = i64::from(q.id);
            if !added_ids.contains(&id) {
                added_ids.insert(id);
                questions.push(q);
                reasons.push(reason);
            }
        };

        // 1. 最近一次复习结果为"错误"的题目（优先级最高）
        if questions.len() < limit {
            if let Ok(wrong_questions) = self.question_dao.list_by_last_result("WRONG") {
                for q in wrong_questions.into_iter().take(limit - questions.len()) {
                    if subject_matches(&q, subject) {
                        add_question(q, RecommendReason::WrongResult, &mut questions, &mut reasons, &mut added_ids);
                    }
                }
            }
        }

        // 2. 新录入但尚未复习的题目（NEW 状态）
        if questions.len() < limit {
            if let Ok(new_questions) = self.question_dao.list_new_questions() {
                for q in new_questions.into_iter().take(limit - questions.len()) {
                    if subject_matches(&q, subject) {
                        add_question(q, RecommendReason::NewQuestion, &mut questions, &mut reasons, &mut added_ids);
                    }
                }
            }
        }

        // 3. 最近复习结果为"模糊"的题目
        if questions.len() < limit {
            if let Ok(fuzzy_questions) = self.question_dao.list_by_last_result("FUZZY") {
                for q in fuzzy_questions.into_iter().take(limit - questions.len()) {
                    if subject_matches(&q, subject) {
                        add_question(q, RecommendReason::FuzzyResult, &mut questions, &mut reasons, &mut added_ids);
                    }
                }
            }
        }

        // 4. 到达建议复习时间的题目（按 due_at 时间筛选，与 state 无关）
        if questions.len() < limit {
            if let Ok(due_questions) = self.question_dao.list_due_questions(now) {
                for q in due_questions.into_iter().take(limit - questions.len()) {
                    if subject_matches(&q, subject) {
                        add_question(q, RecommendReason::DueReview, &mut questions, &mut reasons, &mut added_ids);
                    }
                }
            }
        }

        // 5. 长期未复习的题目（兜底）
        if questions.len() < limit {
            if let Ok(stale_questions) = self.question_dao.list_stale_questions(STALE_DAYS_THRESHOLD) {
                for q in stale_questions.into_iter().take(limit - questions.len()) {
                    if subject_matches(&q, subject) {
                        add_question(q, RecommendReason::StaleReview, &mut questions, &mut reasons, &mut added_ids);
                    }
                }
            }
        }

        Ok(RecommendResult {
            questions,
            reasons,
            subject: subject.map(|s| s.to_string()),
        })
    }

    /// 获取所有科目列表
    pub fn list_subjects(&self) -> Result<Vec<String>, String> {
        self.meta_dao
            .list_all_subjects()
            .map_err(|e| format!("failed to list subjects: {}", e))
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> Result<StatsResult, String> {
        // 计算今日开始时间（凌晨 00:00:00）
        let now = now_ts();
        let today_start = self.get_today_start_timestamp(now);

        // 总题目数
        let total_questions = self.question_dao
            .count_all()
            .map_err(|e| format!("failed to count questions: {}", e))?;

        // 今日已复习数
        let today_reviewed = self.review_dao
            .count_today(today_start)
            .map_err(|e| format!("failed to count today reviews: {}", e))?;

        // 总复习次数
        let total_reviews = self.review_dao
            .count_all()
            .map_err(|e| format!("failed to count all reviews: {}", e))?;

        // 正确次数
        let correct_count = self.review_dao
            .count_by_result("CORRECT")
            .map_err(|e| format!("failed to count correct: {}", e))?;

        // 错误次数
        let wrong_count = self.review_dao
            .count_by_result("WRONG")
            .map_err(|e| format!("failed to count wrong: {}", e))?;

        // 模糊次数
        let fuzzy_count = self.review_dao
            .count_by_result("FUZZY")
            .map_err(|e| format!("failed to count fuzzy: {}", e))?;

        // 各状态题目数
        let new_count = self.question_dao
            .count_by_state("NEW")
            .map_err(|e| format!("failed to count NEW: {}", e))?;
        let learning_count = self.question_dao
            .count_by_state("LEARNING")
            .map_err(|e| format!("failed to count LEARNING: {}", e))?;
        let stable_count = self.question_dao
            .count_by_state("STABLE")
            .map_err(|e| format!("failed to count STABLE: {}", e))?;
        let suspended_count = self.question_dao
            .count_by_state("SUSPENDED")
            .map_err(|e| format!("failed to count SUSPENDED: {}", e))?;

        // 今日待复习数 = NEW + LEARNING
        let today_pending = new_count + learning_count;

        Ok(StatsResult {
            total_questions,
            today_reviewed,
            total_reviews,
            correct_count,
            wrong_count,
            fuzzy_count,
            state_counts: StateCounts {
                new_count,
                learning_count,
                stable_count,
                suspended_count,
            },
            today_pending,
        })
    }

    /// 计算今日开始的时间戳（逻辑天，凌晨 03:00:00）
    fn get_today_start_timestamp(&self, now: Timestamp) -> i64 {
        let day = LogicalDay::from(now);
        let (day_start, _day_end) = range_of_day(day);
        day_start.as_i64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::Connection;
    use crate::db::migrate;
    use crate::domain::ids::QuestionId;
    use crate::util::time::now_ts;

    fn setup() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        conn
    }

    fn insert_question(
        conn: &Connection,
        name: &str,
        state: QuestionState,
    ) -> QuestionId {
        let now = now_ts();
        conn.execute(
            "INSERT INTO question (name, state, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![name, state.as_str(), now.as_i64()],
        )
        .unwrap();
        QuestionId::from(conn.last_insert_rowid())
    }

    #[test]
    fn test_recommend_reason_variants() {
        // 验证推荐原因可以创建
        let _ = RecommendReason::WrongResult;
        let _ = RecommendReason::NewQuestion;
        let _ = RecommendReason::FuzzyResult;
        let _ = RecommendReason::DueReview;
        let _ = RecommendReason::StaleReview;
    }

    #[test]
    fn test_suspend_writes_pre_suspend_meta() {
        let conn = setup();
        let qid = insert_question(&conn, "题", QuestionState::STABLE);

        let mgr = ReviewManager::new(&conn);
        mgr.suspend(qid).unwrap();

        // meta 应存 'STABLE'
        let md = MetaDao::new(&conn);
        let values =
            md.get_values_by_question_key(qid, "system.PreSuspendState").unwrap();
        assert_eq!(values, vec!["STABLE"]);

        // 状态应是 SUSPENDED
        let q = mgr.question_dao.get_by_id(qid).unwrap().unwrap();
        assert_eq!(q.state, QuestionState::SUSPENDED);
    }

    #[test]
    fn test_recover_restores_previous_state() {
        let conn = setup();
        let qid = insert_question(&conn, "题", QuestionState::STABLE);

        let mgr = ReviewManager::new(&conn);
        mgr.suspend(qid).unwrap();
        let recovered = mgr.recover(qid).unwrap();
        assert_eq!(recovered.state, QuestionState::STABLE);

        // meta 已被清掉
        let md = MetaDao::new(&conn);
        let values =
            md.get_values_by_question_key(qid, "system.PreSuspendState").unwrap();
        assert!(values.is_empty(), "pre-suspend meta should be cleared after recover");

        // due_at 应该是 now 附近
        let q = mgr.question_dao.get_by_id(qid).unwrap().unwrap();
        let now = now_ts();
        let delta = (q.due_at.unwrap().as_i64() - now.as_i64()).abs();
        assert!(delta < 5, "due_at should be close to now after recover, got delta={}", delta);
    }

    #[test]
    fn test_recover_falls_back_to_learning_when_meta_missing() {
        let conn = setup();
        let qid = insert_question(&conn, "题", QuestionState::LEARNING);

        // 直接把状态改成 SUSPENDED（模拟老库 / meta 缺失）
        conn.execute(
            "UPDATE question SET state = 'SUSPENDED' WHERE id = ?1",
            rusqlite::params![i64::from(qid)],
        )
        .unwrap();

        let mgr = ReviewManager::new(&conn);
        let recovered = mgr.recover(qid).unwrap();
        assert_eq!(recovered.state, QuestionState::LEARNING);
    }

    #[test]
    fn test_double_suspend_errors() {
        let conn = setup();
        let qid = insert_question(&conn, "题", QuestionState::STABLE);

        let mgr = ReviewManager::new(&conn);
        mgr.suspend(qid).unwrap();
        let err = mgr.suspend(qid).unwrap_err();
        assert!(err.contains("already suspended"), "got: {}", err);
    }

    #[test]
    fn test_recover_non_suspended_errors() {
        let conn = setup();
        let qid = insert_question(&conn, "题", QuestionState::STABLE);

        let mgr = ReviewManager::new(&conn);
        let err = mgr.recover(qid).unwrap_err();
        assert!(err.contains("only suspended"), "got: {}", err);
    }

    #[test]
    fn test_recommend_excludes_suspended() {
        let conn = setup();

        // 题 A: STABLE + 上次答错，应被推荐（step 1 last_result=WRONG）
        let qid_a = insert_question(&conn, "题A", QuestionState::STABLE);
        let now = now_ts();
        conn.execute(
            "UPDATE question SET last_result='WRONG', last_review_at=?1 WHERE id=?2",
            rusqlite::params![now.as_i64(), i64::from(qid_a)],
        )
        .unwrap();

        // 题 B: SUSPENDED（模拟：曾答错，被用户暂停），同样 last_result=WRONG
        let qid_b = insert_question(&conn, "题B", QuestionState::STABLE);
        conn.execute(
            "UPDATE question SET state='SUSPENDED', last_result='WRONG', last_review_at=?1 WHERE id=?2",
            rusqlite::params![now.as_i64(), i64::from(qid_b)],
        )
        .unwrap();

        let mgr = ReviewManager::new(&conn);
        let result = mgr.recommend(Some(10), None).unwrap();
        let ids: Vec<i64> = result
            .questions
            .iter()
            .map(|q| i64::from(q.id.clone()))
            .collect();

        assert!(
            ids.contains(&i64::from(qid_a)),
            "题 A 应被推荐: {:?}",
            ids
        );
        assert!(
            !ids.contains(&i64::from(qid_b)),
            "SUSPENDED 题 B 必须不出现在推荐列表里: {:?}",
            ids
        );
    }
}
