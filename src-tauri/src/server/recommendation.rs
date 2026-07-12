//! 推荐系统模块
//!
//! 实现基于 Anki 改进的复习调度和推荐排序算法

use std::collections::HashMap;

use crate::app::config::SubjectConfig;
use crate::dao::question_dao::QuestionDao;
use crate::dao::recommendation_dao::RecommendationDao;
use crate::dao::review_dao::ReviewDao;
use crate::db::error::DbError;
use crate::domain::enums::{QuestionState, ReviewResult};
use crate::domain::ids::QuestionId;
use crate::domain::question::Question;
use crate::util::time::{now_ts, ClockConfig, LogicalDay, Timestamp};
use rusqlite::Connection;
use serde::Serialize;

/// 一天对应的秒数
const DAY_SECONDS: i64 = 24 * 60 * 60;

/// 评分明细（调试用）
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ScoreDetail {
    pub forget_risk: f64,
    pub freshness_bonus: f64,
    pub last_wrong_bonus: f64,
    pub error_rate_bonus: f64,
    pub randomness: f64,
    pub final_score: f64,
}

/// 推荐结果项
#[derive(Debug, Clone, Serialize)]
pub struct RecommendedQuestion {
    pub question_id: i64,
    pub name: Option<String>,
    pub score: f64,
    pub state: String,
    pub due_at: Option<i64>,
    pub correct_streak: i64,
    pub wrong_count: i64,
    pub last_result: Option<String>,
    pub error_rate: Option<f64>,
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_detail: Option<ScoreDetail>,
    /// 复习次数（内部使用，不序列化）
    #[serde(skip)]
    pub review_count: i64,
    /// 创建时间戳（内部使用，不序列化）
    #[serde(skip)]
    pub created_at: i64,
}

/// 每日推荐结果
#[derive(Debug, Clone, Serialize)]
pub struct DailyRecommendation {
    pub day: i64,
    pub questions: Vec<RecommendedQuestion>,
}

/// 预览推荐项（展示全部题目的评分和入选状态）
#[derive(Debug, Clone, Serialize)]
pub struct PreviewRecommendationItem {
    pub question_id: i64,
    pub name: String,
    pub subject: Option<String>,
    pub score: f64,
    pub selected: bool,
    pub reason: Vec<String>,
    pub exclusion_reason: Vec<String>,
    pub score_detail: Option<ScoreDetail>,
    pub subject_rank: usize,
    pub subject_limit: usize,
}

/// 推荐统计信息
#[derive(Debug, Clone, Serialize)]
pub struct RecommendationStats {
    pub total_questions: usize,
    pub participating_questions: usize,
    pub archived_subjects: Vec<String>,
    pub recommended_count: usize,
    pub new_questions: usize,
    pub pending_review: usize,
}

/// 推荐系统
pub struct RecommendationSystem<'a> {
    conn: &'a Connection,
    question_dao: QuestionDao<'a>,
    review_dao: ReviewDao<'a>,
    recommendation_dao: RecommendationDao<'a>,
    meta_dao: crate::dao::MetaDao<'a>,
}

impl<'a> RecommendationSystem<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            question_dao: QuestionDao::new(conn),
            review_dao: ReviewDao::new(conn),
            recommendation_dao: RecommendationDao::new(conn),
            meta_dao: crate::dao::MetaDao::new(conn),
        }
    }

    /// 获取或生成每日推荐
    pub fn get_daily_recommendation(
        &self,
        subject_configs: &HashMap<String, SubjectConfig>,
        per_subject_default_limit: u32,
        new_question_guarantee_ratio: f64,
    ) -> Result<DailyRecommendation, DbError> {
        let now = now_ts();
        let cfg = ClockConfig::default();
        let day = LogicalDay::from_timestamp(now, &cfg).0 as i64;

        // 尝试从数据库加载今日推荐
        if let Some(questions) = self.recommendation_dao.get_by_day(day)? {
            return Ok(DailyRecommendation { day, questions });
        }

        // 清理旧推荐记录，只保留当天的
        self.recommendation_dao.cleanup_old_recommendations(day)?;

        // 生成新推荐
        let questions = self.generate_recommendation(now, subject_configs, per_subject_default_limit, new_question_guarantee_ratio)?;

        // 保存到数据库
        self.recommendation_dao.insert_batch(day, &questions)?;

        Ok(DailyRecommendation { day, questions })
    }

    /// 预览推荐：对全部题目评分，标记入选/落选状态，不写库
    pub fn preview_recommendation(
        &self,
        show_score_detail: bool,
        show_exclusion_reason: bool,
        subject_configs: &HashMap<String, SubjectConfig>,
        per_subject_default_limit: u32,
        new_question_guarantee_ratio: f64,
    ) -> Result<Vec<PreviewRecommendationItem>, DbError> {
        let now = now_ts();
        let all_questions = self.get_all_active_questions()?;

        if all_questions.is_empty() {
            return Ok(vec![]);
        }

        let review_summaries = self.review_dao.get_all_error_rates()?;
        let subject_key = "system.Subject";

        // Step 1: 为每道题计算评分（复用现有逻辑）
        let mut scored_questions: Vec<RecommendedQuestion> = Vec::new();

        for question in all_questions {
            let qid = i64::from(question.id.clone());

            let (review_count, error_rate) = review_summaries
                .get(&qid)
                .map(|&(err_rate, cnt)| (cnt, Some(err_rate)))
                .unwrap_or((0, None));

            let detail = self.calculate_score(&question, now, review_count, error_rate);

            let overdue_days = question.due_at
                .map(|d| ((now.as_i64() - d.as_i64()) as f64 / DAY_SECONDS as f64).max(0.0))
                .unwrap_or(0.0);

            let last_result_str = question.last_result.map(|r| r.as_str().to_string());

            let reason = Self::generate_reason(
                review_count,
                question.due_at.map(|d| d.as_i64()),
                now.as_i64(),
                &last_result_str,
            );

            let subject = self.meta_dao
                .get_by_question_key(question.id.clone(), subject_key)
                .ok()
                .flatten()
                .map(|m| m.value);

            let score_detail = if show_score_detail {
                Some(detail)
            } else {
                None
            };

            scored_questions.push(RecommendedQuestion {
                question_id: qid,
                name: question.name,
                score: detail.final_score,
                state: question.state.as_str().to_string(),
                due_at: question.due_at.map(|t| t.as_i64()),
                correct_streak: question.correct_streak,
                wrong_count: question.wrong_count,
                last_result: last_result_str,
                error_rate,
                subject,
                reason,
                score_detail,
                review_count,
                created_at: question.created_at.as_i64(),
            });
        }

        // Step 2: 按科目分组
        let mut subject_groups: HashMap<String, Vec<RecommendedQuestion>> = HashMap::new();
        for q in scored_questions {
            let subject = q.subject.clone().unwrap_or_else(|| "未分类".to_string());
            subject_groups.entry(subject).or_default().push(q);
        }

        // Step 3: 每组内使用分池逻辑标记入选/落选
        let mut results: Vec<PreviewRecommendationItem> = Vec::new();

        for (subject, questions) in subject_groups {
            let (archived, limit) = if subject == "未分类" {
                (false, per_subject_default_limit as usize)
            } else {
                match subject_configs.get(&subject) {
                    None => (false, per_subject_default_limit as usize),
                    Some(cfg) => {
                        if cfg.archived {
                            (true, 0)
                        } else {
                            match cfg.recommendation_limit {
                                Some(0) => (true, 0),
                                Some(n) => (false, n.max(1) as usize),
                                None => (false, per_subject_default_limit as usize),
                            }
                        }
                    }
                }
            };

            if archived {
                // 已归档科目：全部标记为落选
                for q in questions {
                    let exclusion_reason: Vec<String> = if show_exclusion_reason {
                        vec!["科目已归档".to_string()]
                    } else {
                        vec![]
                    };
                    results.push(PreviewRecommendationItem {
                        question_id: q.question_id,
                        name: q.name.unwrap_or_default(),
                        subject: q.subject,
                        score: q.score,
                        selected: false,
                        reason: q.reason.unwrap_or_default(),
                        exclusion_reason,
                        score_detail: q.score_detail,
                        subject_rank: 0,
                        subject_limit: 0,
                    });
                }
                continue;
            }

            // 分池
            let (mut new_questions, other_questions): (Vec<_>, Vec<_>) =
                questions.into_iter().partition(|q| q.review_count == 0);

            let guarantee = (limit as f64 * new_question_guarantee_ratio).ceil() as usize;

            // Pool A: new questions sorted by created_at ascending
            new_questions.sort_by_key(|q| q.created_at);
            let take_new_count = new_questions.len().min(guarantee);

            // Consume new_questions into pool_a items and remaining
            let mut new_iter = new_questions.into_iter();
            let mut new_rank = 0usize;
            // Phase A: push pool A selected items as results
            for _ in 0..take_new_count {
                new_rank += 1;
                if let Some(q) = new_iter.next() {
                    results.push(PreviewRecommendationItem {
                        question_id: q.question_id,
                        name: q.name.unwrap_or_default(),
                        subject: q.subject,
                        score: q.score,
                        selected: true,
                        reason: vec!["新题保送".to_string()],
                        exclusion_reason: vec![],
                        score_detail: q.score_detail,
                        subject_rank: new_rank,
                        subject_limit: take_new_count,
                    });
                }
            }

            // Pool B: pool A rejected new questions + other questions
            let mut pool_b: Vec<RecommendedQuestion> = new_iter.collect();
            pool_b.extend(other_questions);
            pool_b.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            let pool_b_limit = limit - take_new_count;

            // Collect pool B selected question_ids
            let pool_b_selected_ids: std::collections::HashSet<i64> = pool_b
                .iter()
                .take(pool_b_limit)
                .map(|q| q.question_id)
                .collect();

            // Build preview items: pass through pool_b in score-sorted order for rank
            let total = pool_b.len();
            let mut b_rank = 0usize;
            for q in pool_b {
                b_rank += 1;
                // pool B items can never be pool A selected (those were consumed above),
                // but new questions in pool B might have been pool A rejects
                let selected = pool_b_selected_ids.contains(&q.question_id);

                let reason: Vec<String> = q.reason.clone().unwrap_or_default();
                let score_detail = q.score_detail;
                let exclusion_reason: Vec<String> = if !selected && show_exclusion_reason {
                    Self::generate_exclusion_reason(&q, now.as_i64(), b_rank)
                } else {
                    vec![]
                };

                results.push(PreviewRecommendationItem {
                    question_id: q.question_id,
                    name: q.name.unwrap_or_default(),
                    subject: q.subject,
                    score: q.score,
                    selected,
                    reason,
                    exclusion_reason,
                    score_detail: q.score_detail,
                    subject_rank: b_rank,
                    subject_limit: limit,
                });
            }
        }

        // 整体按分数降序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// 获取推荐统计概览
    pub fn get_recommendation_stats(
        &self,
        subject_configs: &HashMap<String, SubjectConfig>,
    ) -> Result<RecommendationStats, DbError> {
        let all_questions = self.get_all_active_questions()?;
        let review_summaries = self.review_dao.get_all_error_rates()?;
        let subject_key = "system.Subject";
        let now = now_ts();
        let cfg = ClockConfig::default();

        let total_questions = all_questions.len();
        let mut participating = 0usize;
        let mut new_count = 0usize;
        let mut pending_review = 0usize;

        // 收集已归档科目
        let mut archived_subjects: Vec<String> = Vec::new();
        for (name, cfg) in subject_configs {
            if cfg.archived {
                archived_subjects.push(name.clone());
            }
        }
        archived_subjects.sort();

        for question in &all_questions {
            let qid = i64::from(question.id.clone());

            let review_count = review_summaries.get(&qid).map(|&(_, cnt)| cnt).unwrap_or(0);
            if review_count == 0 {
                new_count += 1;
            }

            let subject = self.meta_dao
                .get_by_question_key(question.id.clone(), subject_key)
                .ok()
                .flatten()
                .map(|m| m.value);

            let is_archived = subject
                .as_ref()
                .and_then(|s| subject_configs.get(s))
                .map(|cfg| cfg.archived)
                .unwrap_or(false);

            if !is_archived {
                participating += 1;
            }

            if let Some(due) = question.due_at {
                if due.as_i64() <= now.as_i64() {
                    pending_review += 1;
                }
            }
        }

        // 今日推荐题数
        let day = LogicalDay::from_timestamp(now, &cfg).0 as i64;
        let recommended_count = self
            .recommendation_dao
            .get_by_day(day)?
            .map(|qs| qs.len())
            .unwrap_or(0);

        Ok(RecommendationStats {
            total_questions,
            participating_questions: participating,
            archived_subjects,
            recommended_count,
            new_questions: new_count,
            pending_review,
        })
    }

    /// 生成推荐列表，使用分池算法：池A（新题保送）+ 池B（评分竞争）
    fn generate_recommendation(
        &self,
        now: Timestamp,
        subject_configs: &HashMap<String, SubjectConfig>,
        per_subject_default_limit: u32,
        new_question_guarantee_ratio: f64,
    ) -> Result<Vec<RecommendedQuestion>, DbError> {
        // 获取所有未删除的题目
        let all_questions = self.get_all_active_questions()?;

        if all_questions.is_empty() {
            return Ok(vec![]);
        }

        // 获取所有题目的复习摘要（错误率 + 复习次数）
        let review_summaries = self.review_dao.get_all_error_rates()?;

        // 科目元信息key
        let subject_key = "system.Subject";

        // 为每道题计算推荐分数并获取科目
        let mut scored_questions: Vec<RecommendedQuestion> = Vec::new();

        for question in all_questions {
            let qid = i64::from(question.id.clone());

            // 从摘要中获取 review_count 和 error_rate
            let (review_count, error_rate) = review_summaries
                .get(&qid)
                .map(|&(err_rate, cnt)| (cnt, Some(err_rate)))
                .unwrap_or((0, None));

            let detail = self.calculate_score(&question, now, review_count, error_rate);

            // 计算超期天数
            let overdue_days = question.due_at
                .map(|d| ((now.as_i64() - d.as_i64()) as f64 / DAY_SECONDS as f64).max(0.0))
                .unwrap_or(0.0);

            // 提取 last_result 字符串
            let last_result_str = question.last_result.map(|r| r.as_str().to_string());

            // 生成推荐理由
            let reason = Self::generate_reason(
                review_count,
                question.due_at.map(|d| d.as_i64()),
                now.as_i64(),
                &last_result_str,
            );

            // 获取科目
            let subject = self.meta_dao
                .get_by_question_key(question.id.clone(), subject_key)
                .ok()
                .flatten()
                .map(|m| m.value);

            // 仅 debug 构建填充评分明细
            let score_detail = if cfg!(debug_assertions) {
                Some(detail)
            } else {
                None
            };

            scored_questions.push(RecommendedQuestion {
                question_id: qid,
                name: question.name,
                score: detail.final_score,
                state: question.state.as_str().to_string(),
                due_at: question.due_at.map(|t| t.as_i64()),
                correct_streak: question.correct_streak,
                wrong_count: question.wrong_count,
                last_result: last_result_str,
                error_rate,
                subject,
                reason,
                score_detail,
                review_count,
                created_at: question.created_at.as_i64(),
            });
        }

        // 按科目分组（无科目的归入 "未分类"）
        let mut subject_groups: HashMap<String, Vec<RecommendedQuestion>> = HashMap::new();

        for q in scored_questions {
            let subject = q.subject.clone().unwrap_or_else(|| "未分类".to_string());
            subject_groups.entry(subject).or_insert_with(Vec::new).push(q);
        }

        let mut final_questions: Vec<RecommendedQuestion> = Vec::new();

        for (subject, questions) in subject_groups {
            // 按科目配置确定该科题数上限
            let limit = if subject == "未分类" {
                per_subject_default_limit
            } else {
                match subject_configs.get(&subject) {
                    None => per_subject_default_limit,
                    Some(cfg) => {
                        if cfg.archived {
                            continue;
                        }
                        match cfg.recommendation_limit {
                            Some(0) => continue,
                            Some(n) => n.max(1),
                            None => per_subject_default_limit,
                        }
                    }
                }
            } as usize;

            // 分池
            let (mut new_questions, other_questions): (Vec<_>, Vec<_>) =
                questions.into_iter().partition(|q| q.review_count == 0);

            let guarantee = (limit as f64 * new_question_guarantee_ratio).ceil() as usize;

            // 池A：新题按创建时间升序，取前 guarantee 题，标记"新题保送"
            new_questions.sort_by_key(|q| q.created_at);
            let take_new_count = new_questions.len().min(guarantee);

            let mut selected: Vec<RecommendedQuestion> = Vec::new();
            let mut new_iter = new_questions.into_iter();
            for _ in 0..take_new_count {
                if let Some(mut q) = new_iter.next() {
                    q.reason = Some(vec!["新题保送".to_string()]);
                    selected.push(q);
                }
            }
            let new_remaining: Vec<RecommendedQuestion> = new_iter.collect();

            // 池B：池A落选的新题 + 其他题，按分数降序
            let mut pool_b: Vec<RecommendedQuestion> = Vec::new();
            pool_b.extend(new_remaining);
            pool_b.extend(other_questions);
            pool_b.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

            let remaining_slots = limit - selected.len();
            pool_b.truncate(remaining_slots);
            selected.extend(pool_b);

            final_questions.extend(selected);
        }

        // 整体按分数排序
        final_questions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(final_questions)
    }

    /// 获取所有未删除、未暂停的题目（SUSPENDED 不参与推荐打分池）
    fn get_all_active_questions(&self) -> Result<Vec<Question>, DbError> {
        let mut questions = Vec::new();

        // 查询所有未删除、未暂停的题目
        let rows = crate::db::select_all_active_questions(self.conn)?;

        for row in rows {
            let q = crate::repo::question_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
            if q.state == QuestionState::SUSPENDED {
                continue;
            }
            questions.push(q);
        }

        Ok(questions)
    }

    /// 计算推荐分数
    /// score = (1 + forget_risk) * freshness_bonus * last_wrong_bonus * error_rate_bonus * randomness
    fn calculate_score(
        &self,
        question: &Question,
        now: Timestamp,
        review_count: i64,
        error_rate: Option<f64>,
    ) -> ScoreDetail {
        // 1. 遗忘风险 (软对数上限)
        let forget_risk = match (question.last_review_at, question.due_at) {
            (Some(last_review), Some(due)) => {
                let expected = ((due.as_i64() - last_review.as_i64()) as f64 / DAY_SECONDS as f64).max(1.0);
                let passed = ((now.as_i64() - last_review.as_i64()) as f64 / DAY_SECONDS as f64).max(0.0);
                (passed / expected + 1.0).log2()
            }
            _ => 0.0, // 从未复习过的题，遗忘风险为 0
        };

        // 2. 新鲜度奖励 (仅从未复习过的新题)
        let freshness_bonus = if review_count == 0 {
            let days = ((now.as_i64() - question.created_at.as_i64()) as f64 / DAY_SECONDS as f64).max(0.0);
            (5.0 - days * 0.25).max(1.0)
        } else {
            1.0
        };

        // 3. 上次错误奖励
        let last_wrong_bonus = if let Some(ReviewResult::WRONG) = question.last_result {
            3.0
        } else {
            1.0
        };

        // 4. 错误率奖励 (带样本量平滑)
        let error_rate_bonus = match error_rate {
            Some(rate) => 1.0 + rate * ((review_count + 1) as f64).log2(),
            None => 1.0,
        };

        // 5. 随机扰动
        let qid_i64: i64 = question.id.clone().into();
        let day_seed = (qid_i64 * 31 + now.as_i64() / DAY_SECONDS) % 1000;
        let randomness = 0.95 + (day_seed as f64 / 10000.0); // 0.95 ~ 1.05

        // 计算总分
        let final_score = (1.0 + forget_risk)
            * freshness_bonus
            * last_wrong_bonus
            * error_rate_bonus
            * randomness;

        ScoreDetail {
            forget_risk,
            freshness_bonus,
            last_wrong_bonus,
            error_rate_bonus,
            randomness,
            final_score,
        }
    }

    /// 生成推荐理由，按固定优先级排序
    /// 生成"被推荐"理由标签（按优先级排序）。
    ///
    /// 词条：
    /// - "新题"        review_count ≤ 3
    /// - "到期"        due_at 存在且 overdue_days = 0（且不是新题）
    /// - "超期 N 天"   due_at 存在且 overdue_days > 0
    /// - "上次出错"    last_result == "wrong" 或 "fuzzy"
    ///
    /// `due_at = None` 时不输出"到期"/"超期"（新题未设置 due_at 属正常）。
    fn generate_reason(
        review_count: i64,
        due_at: Option<i64>,
        now: i64,
        last_result: &Option<String>,
    ) -> Option<Vec<String>> {
        let mut reasons: Vec<(u8, String)> = Vec::new();

        if review_count <= 3 {
            reasons.push((1, "新题".to_string()));
        }
        if let Some(d) = due_at {
            let overdue_days = (now - d) / DAY_SECONDS;
            if overdue_days > 0 {
                reasons.push((2, format!("超期{overdue_days}天")));
            } else if review_count > 0 {
                reasons.push((2, "到期".to_string()));
            }
        }
        if let Some(r) = last_result {
            if r == "wrong" || r == "fuzzy" {
                reasons.push((3, "上次出错".to_string()));
            }
        }

        if reasons.is_empty() {
            return None;
        }
        reasons.sort_by_key(|(priority, _)| *priority);
        Some(reasons.into_iter().map(|(_, text)| text).collect())
    }

    /// "遗忘风险低" / "错误率低" 阈值：评分超过阈值时算"高"，不输出该词条。
    const EXCLUSION_THRESHOLD: f64 = 1.5;

    /// 生成"落选"原因标签。
    ///
    /// 规则：
    /// - SUSPENDED → ["暂停复习"]
    /// - 已掌握（STABLE 且未到期）→ ["已掌握"]
    /// - 其他：
    ///   - forget_risk < 阈值 → "遗忘风险低"
    ///   - error_rate_bonus < 阈值 → "错误率低"
    ///   - 都 ≥ 阈值 → "同科排名低于 N"（N = subject_rank）
    fn generate_exclusion_reason(
        q: &RecommendedQuestion,
        now: i64,
        subject_rank: usize,
    ) -> Vec<String> {
        // 1. 暂停复习
        if q.state == "SUSPENDED" {
            return vec!["暂停复习".to_string()];
        }
        // 2. 已掌握
        let overdue_days = q.due_at
            .map(|d| (now - d) / DAY_SECONDS)
            .unwrap_or(0);
        if q.state == "STABLE" && overdue_days == 0 {
            return vec!["已掌握".to_string()];
        }
        // 3. 按评分给分
        let detail = q.score_detail.unwrap_or(ScoreDetail {
            forget_risk: 0.0,
            freshness_bonus: 1.0,
            last_wrong_bonus: 1.0,
            error_rate_bonus: 1.0,
            randomness: 1.0,
            final_score: 0.0,
        });
        let mut reasons: Vec<String> = Vec::new();
        if detail.forget_risk < Self::EXCLUSION_THRESHOLD {
            reasons.push("遗忘风险低".to_string());
        }
        if detail.error_rate_bonus < Self::EXCLUSION_THRESHOLD {
            reasons.push("错误率低".to_string());
        }
        if reasons.is_empty() {
            reasons.push(format!("同科排名低于{subject_rank}"));
        }
        reasons
    }

    /// 处理复习结果 - 更新题目复习状态
    pub fn process_review(
        &self,
        question_id: i64,
        result: &str,
    ) -> Result<Question, DbError> {
        let qid = QuestionId::from(question_id);

        // 获取题目
        let question = self.question_dao.get_by_id(qid)?
            .ok_or_else(|| DbError::NotFound)?;

        // 计算新的 correct_streak, wrong_count, due_at
        let (new_streak, new_wrong, due_at) = self.calculate_next_review(
            &question,
            result,
            now_ts(),
        )?;

        // 更新题目
        self.question_dao.update_review_fields(
            qid,
            Some(now_ts()),
            Some(result),
            new_streak,
            new_wrong,
            Some(due_at),
        )?;

        // 插入复习记录 (使用 insert_str 接受字符串)
        self.review_dao.insert_str(
            qid,
            result,
            now_ts(),
        )?;

        // 返回更新后的题目
        self.question_dao.get_by_id(qid)?
            .ok_or_else(|| DbError::NotFound)
    }

    /// 计算下一次复习的时间
    fn calculate_next_review(
        &self,
        question: &Question,
        result: &str,
        now: Timestamp,
    ) -> Result<(i64, i64, Timestamp), DbError> {
        let stability = (question.correct_streak + 1) as f64;
        let difficulty = 1.0 + (question.wrong_count as f64) * 0.2;

        let (new_streak, new_wrong, interval_days) = match result {
            "correct" => {
                let new_streak = question.correct_streak + 1;
                let interval = (stability * stability) / difficulty;
                (new_streak, question.wrong_count, interval)
            }
            "fuzzy" => {
                let new_streak = (question.correct_streak - 1).max(0);
                let interval = ((stability * stability) / difficulty) * 0.5;
                (new_streak, question.wrong_count, interval)
            }
            "wrong" => {
                let new_wrong = question.wrong_count + 1;
                let interval = 1.0; // 错误题第二天必须复习
                (0, new_wrong, interval)
            }
            _ => {
                return Err(DbError::Migration(format!("Invalid result: {}", result)));
            }
        };

        let due_at_seconds = (now.as_i64() as f64 + interval_days * DAY_SECONDS as f64) as i64;
        let due_at = Timestamp::from(due_at_seconds);

        Ok((new_streak, new_wrong, due_at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_recommended_question(
        state: &str,
        review_count: i64,
        wrong_count: i64,
        due_at: Option<i64>,
        last_result: Option<&str>,
        forget_risk: f64,
        error_rate_bonus: f64,
    ) -> RecommendedQuestion {
        RecommendedQuestion {
            question_id: 1,
            name: Some("Q".to_string()),
            score: 1.0,
            state: state.to_string(),
            due_at,
            correct_streak: 0,
            wrong_count,
            last_result: last_result.map(|s| s.to_string()),
            error_rate: Some(0.5),
            subject: Some("数学".to_string()),
            reason: None,
            score_detail: Some(ScoreDetail {
                forget_risk,
                freshness_bonus: 1.0,
                last_wrong_bonus: 1.0,
                error_rate_bonus,
                randomness: 1.0,
                final_score: 1.0,
            }),
            review_count,
            created_at: 0,
        }
    }

    // ===== generate_reason =====

    #[test]
    fn test_reason_new_q() {
        // 新题: review_count = 0
        let r = RecommendationSystem::generate_reason(0, None, 0, &None);
        assert_eq!(r, Some(vec!["新题".to_string()]));
    }

    #[test]
    fn test_reason_review_count_3_still_new_q() {
        // review_count = 3 仍标"新题"
        let r = RecommendationSystem::generate_reason(3, None, 100, &None);
        assert_eq!(r, Some(vec!["新题".to_string()]));
    }

    #[test]
    fn test_reason_review_count_4_not_new_q() {
        // review_count = 4 不再标"新题"
        let overdue_secs = 50 * 86400;
        let r = RecommendationSystem::generate_reason(
            4,
            Some(1_000_000 - overdue_secs),
            1_000_000,
            &None,
        );
        // 4 次复习, due=now-50天, now → overdue 50 天 → "超期 50 天"
        assert_eq!(r, Some(vec!["超期50天".to_string()]));
    }

    #[test]
    fn test_reason_due_now_no_overdue_reviewed() {
        // due_at = now, review_count > 0 → "到期"
        let r = RecommendationSystem::generate_reason(5, Some(1000), 1000, &None);
        assert_eq!(r, Some(vec!["到期".to_string()]));
    }

    #[test]
    fn test_reason_due_in_future_not_overdue() {
        // due_at 在未来, overdue = 0, review_count > 0 → "到期"
        let r = RecommendationSystem::generate_reason(5, Some(2000), 1000, &None);
        assert_eq!(r, Some(vec!["到期".to_string()]));
    }

    #[test]
    fn test_reason_due_in_past_overdue_days() {
        // due_at 过去 7 天 → "超期 7 天"
        let r = RecommendationSystem::generate_reason(5, Some(1000 - 7 * 86400), 1000, &None);
        assert_eq!(r, Some(vec!["超期7天".to_string()]));
    }

    #[test]
    fn test_reason_new_q_due_in_past() {
        // 新题 (review_count=0) 即便 due_at 在过去也不标"超期" — 新题没有 due_at
        // 测试 due_at = None 路径
        let r = RecommendationSystem::generate_reason(0, None, 1000, &None);
        assert_eq!(r, Some(vec!["新题".to_string()]));
    }

    #[test]
    fn test_reason_last_wrong() {
        let r = RecommendationSystem::generate_reason(5, Some(1000), 1000, &Some("wrong".to_string()));
        assert_eq!(r, Some(vec!["到期".to_string(), "上次出错".to_string()]));
    }

    #[test]
    fn test_reason_last_fuzzy() {
        let r = RecommendationSystem::generate_reason(5, Some(1000), 1000, &Some("fuzzy".to_string()));
        assert_eq!(r, Some(vec!["到期".to_string(), "上次出错".to_string()]));
    }

    #[test]
    fn test_reason_last_correct_not_included() {
        let r = RecommendationSystem::generate_reason(5, Some(1000), 1000, &Some("correct".to_string()));
        assert_eq!(r, Some(vec!["到期".to_string()]));
    }

    #[test]
    fn test_reason_combined_new_q_overdue() {
        // 新题 due_at 在过去: 只显示"新题"
        let r = RecommendationSystem::generate_reason(2, Some(1000 - 3 * 86400), 1000, &Some("wrong".to_string()));
        // 新题: 优先级 1
        // 超期: 优先级 2
        // 上次出错: 优先级 3
        assert_eq!(r, Some(vec!["新题".to_string(), "超期3天".to_string(), "上次出错".to_string()]));
    }

    #[test]
    fn test_reason_no_conditions_returns_none() {
        // review_count > 3, due_at None, last_result = correct
        // 没有"到期"标签(因 due_at = None), 没"超期", 没"上次出错"
        // 只有 review_count = 4 没有"新题"标签
        // 所以没标签 → None
        let r = RecommendationSystem::generate_reason(4, None, 1000, &Some("correct".to_string()));
        assert_eq!(r, None);
    }

    // ===== generate_exclusion_reason =====

    #[test]
    fn test_exclusion_suspended() {
        let q = dummy_recommended_question("SUSPENDED", 5, 0, Some(1000), None, 1.0, 1.0);
        assert_eq!(RecommendationSystem::generate_exclusion_reason(&q, 1000, 5), vec!["暂停复习".to_string()]);
    }

    #[test]
    fn test_exclusion_stable_no_overdue() {
        // STABLE + due_at 未来 → "已掌握"
        let q = dummy_recommended_question("STABLE", 10, 0, Some(2000), None, 1.0, 1.0);
        assert_eq!(RecommendationSystem::generate_exclusion_reason(&q, 1000, 5), vec!["已掌握".to_string()]);
    }

    #[test]
    fn test_exclusion_stable_due_now() {
        // STABLE + due_at = now → "已掌握" (overdue = 0)
        let q = dummy_recommended_question("STABLE", 10, 0, Some(1000), None, 1.0, 1.0);
        assert_eq!(RecommendationSystem::generate_exclusion_reason(&q, 1000, 5), vec!["已掌握".to_string()]);
    }

    #[test]
    fn test_exclusion_stable_overdue_goes_to_score() {
        // STABLE + overdue 7 天 → 走评分路径 (不再算"已掌握")
        let q = dummy_recommended_question("STABLE", 10, 0, Some(1000 - 7 * 86400), None, 0.5, 1.0);
        // forget_risk = 0.5 < 1.5 → "遗忘风险低"
        let res = RecommendationSystem::generate_exclusion_reason(&q, 1000, 5);
        assert!(res.contains(&"遗忘风险低".to_string()));
    }

    #[test]
    fn test_exclusion_low_forget_only() {
        // LEARNING, forget_risk 低, error_rate_bonus 高
        let q = dummy_recommended_question(
            "LEARNING",
            10,
            0,
            Some(1000 - 86400),  // overdue 1 天
            None,
            1.0,  // forget_risk 低 (< 1.5)
            2.0,  // error_rate_bonus 高
        );
        let res = RecommendationSystem::generate_exclusion_reason(&q, 1000, 5);
        assert_eq!(res, vec!["遗忘风险低".to_string()]);
    }

    #[test]
    fn test_exclusion_low_error_rate_only() {
        let q = dummy_recommended_question(
            "LEARNING",
            10,
            0,
            Some(1000 - 86400),
            None,
            2.0,  // forget_risk 高
            1.0,  // error_rate_bonus 低
        );
        let res = RecommendationSystem::generate_exclusion_reason(&q, 1000, 5);
        assert_eq!(res, vec!["错误率低".to_string()]);
    }

    #[test]
    fn test_exclusion_both_low() {
        let q = dummy_recommended_question(
            "LEARNING",
            10,
            0,
            Some(1000 - 86400),
            None,
            1.0,
            1.0,
        );
        let res = RecommendationSystem::generate_exclusion_reason(&q, 1000, 5);
        assert_eq!(res, vec!["遗忘风险低".to_string(), "错误率低".to_string()]);
    }

    #[test]
    fn test_exclusion_both_high_falls_back_to_rank() {
        let q = dummy_recommended_question(
            "LEARNING",
            10,
            0,
            Some(1000 - 86400),
            None,
            2.0,
            2.0,
        );
        let res = RecommendationSystem::generate_exclusion_reason(&q, 1000, 7);
        assert_eq!(res, vec!["同科排名低于7".to_string()]);
    }

    #[test]
    fn test_exclusion_no_score_detail_uses_zero_defaults() {
        // 没 score_detail 时, forget_risk = 0, error_rate_bonus = 0 都 < 1.5
        let mut q = dummy_recommended_question("LEARNING", 10, 0, Some(1000 - 86400), None, 0.0, 0.0);
        q.score_detail = None;
        let res = RecommendationSystem::generate_exclusion_reason(&q, 1000, 5);
        // forget_risk = 0 < 1.5 → "遗忘风险低"
        // error_rate_bonus = 0 < 1.5 → "错误率低"
        assert_eq!(res, vec!["遗忘风险低".to_string(), "错误率低".to_string()]);
    }
}
