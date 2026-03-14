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
}

impl<'a> RecommendationSystem<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            question_dao: QuestionDao::new(conn),
            review_dao: ReviewDao::new(conn),
            recommendation_dao: RecommendationDao::new(conn),
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

        // 生成新推荐
        let questions = self.generate_recommendation(day, now, target_count)?;

        // 保存到数据库
        self.recommendation_dao.insert_batch(day, &questions)?;

        Ok(DailyRecommendation { day, questions })
    }

    /// 生成推荐列表
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

        // 为每道题计算推荐分数
        let mut scored_questions: Vec<RecommendedQuestion> = Vec::new();

        for question in all_questions {
            let score = self.calculate_score(&question, now)?;
            scored_questions.push(RecommendedQuestion {
                question_id: i64::from(question.id),
                name: question.name,
                score,
                state: question.state.as_str().to_string(),
                due_at: question.due_at.map(|t| t.as_i64()),
                correct_streak: question.correct_streak,
                wrong_count: question.wrong_count,
                last_result: question.last_result.map(|r| r.as_str().to_string()),
                error_rate: None, // 稍后填充
            });
        }

        // 获取错误率
        self.enrich_error_rates(&mut scored_questions)?;

        // 按分数降序排序
        scored_questions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // 限制数量
        scored_questions.truncate(target_count as usize);

        Ok(scored_questions)
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
    /// score = overdue_score * last_wrong_bonus * new_question_bonus * error_rate_bonus * randomness
    fn calculate_score(&self, question: &Question, now: Timestamp) -> Result<f64, DbError> {
        let stability = (question.correct_streak + 1) as f64;
        let difficulty = 1.0 + (question.wrong_count as f64) * 0.2;

        // 1. 过期分数
        let overdue_score = if let Some(due_at) = question.due_at {
            let overdue_days = ((now.as_i64() - due_at.as_i64()) as f64 / DAY_SECONDS as f64).max(0.0);
            if overdue_days > 0.0 {
                overdue_days * 10.0
            } else {
                0.0
            }
        } else {
            // 从未复习过，视为过期
            10.0
        };

        // 2. 最近错误奖励 (last_result = WRONG)
        let last_wrong_bonus = if let Some(ReviewResult::WRONG) = question.last_result {
            5.0
        } else {
            1.0
        };

        // 3. 新题奖励 (从未复习过)
        let new_question_bonus = if question.last_review_at.is_none() {
            2.0
        } else {
            1.0
        };

        // 4. 错误率奖励 (从 review_summary 获取)
        let error_rate_bonus = 1.0; // 默认值，稍后通过 enrich_error_rates 填充

        // 5. 随机扰动 (每天略有不同)
        let qid_i64: i64 = question.id.clone().into();
        let day_seed = (qid_i64 * 31 + now.as_i64() / DAY_SECONDS) % 1000;
        let randomness = 0.9 + (day_seed as f64 / 5000.0); // 0.9 ~ 1.1

        // 计算总分
        let score = (1.0 + overdue_score)
            * last_wrong_bonus
            * new_question_bonus
            * error_rate_bonus
            * randomness;

        Ok(score)
    }

    /// 填充错误率信息
    fn enrich_error_rates(&self, questions: &mut Vec<RecommendedQuestion>) -> Result<(), DbError> {
        // 获取所有题目的错误率
        let error_rates = self.review_dao.get_all_error_rates()?;

        for q in questions.iter_mut() {
            if let Some(rate) = error_rates.get(&q.question_id) {
                q.error_rate = Some(*rate);
            }
        }

        // 重新计算带有错误率的分数
        for q in questions.iter_mut() {
            let error_rate_bonus = if let Some(rate) = q.error_rate {
                // 错误率越高，奖励越高
                1.0 + rate * 2.0
            } else {
                1.0
            };

            // 重新计算 (简化处理)
            q.score = q.score * error_rate_bonus;
        }

        Ok(())
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
