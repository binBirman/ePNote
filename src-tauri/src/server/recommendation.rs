//! 推荐系统模块
//!
//! 实现基于 Anki 改进的复习调度和推荐排序算法

use crate::dao::question_dao::QuestionDao;
use crate::dao::recommendation_dao::RecommendationDao;
use crate::dao::review_dao::ReviewDao;
use crate::db::error::DbError;
use crate::domain::enums::{QuestionState, ReviewResult};
use crate::domain::ids::QuestionId;
use crate::domain::question::Question;
use crate::util::time::{now_ts, LogicalDay, Timestamp};
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
    pub subject: Option<String>,       // 科目
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Vec<String>>,   // 推荐理由
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_detail: Option<ScoreDetail>, // 评分明细（仅 debug 构建填充）
}

/// 每日推荐结果
#[derive(Debug, Clone, Serialize)]
pub struct DailyRecommendation {
    pub day: i64,
    pub questions: Vec<RecommendedQuestion>,
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
    pub fn get_daily_recommendation(&self, target_count: i64) -> Result<DailyRecommendation, DbError> {
        let now = now_ts();
        let day = LogicalDay::from(now).0 as i64;

        // 尝试从数据库加载今日推荐
        if let Some(questions) = self.recommendation_dao.get_by_day(day)? {
            return Ok(DailyRecommendation { day, questions });
        }

        // 清理旧推荐记录，只保留当天的
        self.recommendation_dao.cleanup_old_recommendations(day)?;

        // 生成新推荐
        let questions = self.generate_recommendation(day, now, target_count)?;

        // 保存到数据库
        self.recommendation_dao.insert_batch(day, &questions)?;

        Ok(DailyRecommendation { day, questions })
    }

    /// 生成推荐列表，按科目均分 target_count 条
    fn generate_recommendation(
        &self,
        _day: i64,
        now: Timestamp,
        target_count: i64,
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
                detail.forget_risk,
                overdue_days,
                &last_result_str,
                error_rate,
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
            });
        }

        // 按科目分组，每科取10题
        let mut subject_groups: std::collections::HashMap<String, Vec<RecommendedQuestion>> = std::collections::HashMap::new();

        for q in scored_questions {
            let subject = q.subject.clone().unwrap_or_else(|| "未分类".to_string());
            subject_groups.entry(subject).or_insert_with(Vec::new).push(q);
        }

        // 按科目数均分 target_count，每科至少取 1 题
        let num_subjects = subject_groups.len().max(1);
        let per_subject_limit = ((target_count as usize) / num_subjects).max(1);
        let mut final_questions: Vec<RecommendedQuestion> = Vec::new();

        for (_, mut questions) in subject_groups {
            // 按分数降序排序
            questions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            // 取前10题
            questions.truncate(per_subject_limit);
            final_questions.extend(questions);
        }

        // 整体按分数排序
        final_questions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(final_questions)
    }

    /// 获取所有未删除的题目
    fn get_all_active_questions(&self) -> Result<Vec<Question>, DbError> {
        let mut questions = Vec::new();

        // 查询所有未删除的题目
        let rows = crate::db::select_all_active_questions(self.conn)?;

        for row in rows {
            let q = crate::repo::question_row_to_domain(&row)
                .map_err(|e| DbError::Migration(format!("convert error: {:?}", e)))?;
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
    fn generate_reason(
        review_count: i64,
        forget_risk: f64,
        overdue_days: f64,
        last_result: &Option<String>,
        error_rate: Option<f64>,
    ) -> Option<Vec<String>> {
        let mut reasons: Vec<(u8, String)> = Vec::new();

        if review_count == 0 {
            reasons.push((1, "新加入题目".to_string()));
        }
        if forget_risk >= 1.0 && overdue_days >= 0.0 {
            reasons.push((2, "已到复习时间".to_string()));
        }
        if overdue_days > 0.0 {
            reasons.push((3, format!("已超期{}天", overdue_days as i64)));
        }
        if let Some(r) = last_result {
            if r == "wrong" {
                reasons.push((4, "上次回答错误".to_string()));
            }
        }
        if let Some(rate) = error_rate {
            if rate >= 0.5 {
                reasons.push((5, format!("错误率{:.0}%", rate * 100.0)));
            }
        }

        if reasons.is_empty() {
            return None;
        }
        reasons.sort_by_key(|(priority, _)| *priority);
        Some(reasons.into_iter().map(|(_, text)| text).collect())
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
