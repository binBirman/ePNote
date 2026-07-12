//! 状态机模块 - 实现错题本系统的状态转移逻辑
//!
//! 状态转移规则（当前实现）：
//! - NEW → LEARNING: 首次复习（任意结果）
//! - LEARNING → LEARNING: 复习结果 = 错误/模糊，或连续正确 < 3 次
//! - LEARNING → STABLE: 连续正确 ≥ 3 次
//! - STABLE → LEARNING: 任意一次复习结果 = 错误/模糊
//! - STABLE → STABLE: 复习结果 = 正确
//! - 任意状态 → SUSPENDED: 用户手动暂停
//! - SUSPENDED → 原状态: 用户手动恢复（保存于 meta `system.PreSuspendState`）

use crate::domain::enums::{QuestionState, ReviewResult};
use crate::domain::question::Question;
use crate::util::time::Timestamp;

/// 稳定状态所需的连续正确次数阈值
const STABLE_THRESHOLD: i64 = 3;

/// 状态转移结果
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// 转移后的状态
    pub new_state: QuestionState,
    /// 更新后的连续正确次数
    pub correct_streak: i64,
    /// 更新后的错误次数
    pub wrong_count: i64,
    /// 下次到期时间
    pub due_at: Option<Timestamp>,
}

/// 状态机
pub struct QuestionStateMachine;

impl QuestionStateMachine {
    /// 处理复习结果，返回新的状态和更新的字段
    ///
    /// # 参数
    /// - question: 当前的题目
    /// - result: 本次复习结果
    /// - now: 当前时间
    ///
    /// # 返回
    /// 返回状态转移结果
    pub fn process_review(
        question: &Question,
        result: ReviewResult,
        now: Timestamp,
    ) -> StateTransition {
        let current_state = &question.state;
        let current_streak = question.correct_streak;
        let current_wrong = question.wrong_count;

        match current_state {
            QuestionState::NEW => {
                // NEW → LEARNING: 首次复习（任意结果）
                // 新题一定是不稳定的，直接转到 LEARNING
                match result {
                    ReviewResult::CORRECT => {
                        let new_streak = 1;
                        StateTransition {
                            new_state: QuestionState::LEARNING,
                            correct_streak: new_streak,
                            wrong_count: current_wrong,
                            due_at: Some(Self::due_at_after_review(
                                now,
                                result,
                                new_streak,
                                current_wrong,
                            )),
                        }
                    }
                    ReviewResult::WRONG | ReviewResult::FUZZY => {
                        let new_streak = 0;
                        let new_wrong = current_wrong + 1;
                        StateTransition {
                            new_state: QuestionState::LEARNING,
                            correct_streak: new_streak,
                            wrong_count: new_wrong,
                            due_at: Some(Self::due_at_after_review(
                                now,
                                result,
                                new_streak,
                                new_wrong,
                            )),
                        }
                    }
                }
            }
            QuestionState::LEARNING => {
                // LEARNING 状态下的转移
                match result {
                    ReviewResult::CORRECT => {
                        // 连续正确，增加 streak
                        let new_streak = current_streak + 1;
                        let new_state = if new_streak >= STABLE_THRESHOLD {
                            QuestionState::STABLE
                        } else {
                            QuestionState::LEARNING
                        };
                        StateTransition {
                            new_state,
                            correct_streak: new_streak,
                            wrong_count: current_wrong,
                            due_at: Some(Self::due_at_after_review(
                                now,
                                result,
                                new_streak,
                                current_wrong,
                            )),
                        }
                    }
                    ReviewResult::WRONG | ReviewResult::FUZZY => {
                        // LEARNING: 错误清零；模糊减 1（与推荐系统一致）
                        let new_streak = match result {
                            ReviewResult::WRONG => 0,
                            _ => (current_streak - 1).max(0),
                        };
                        let new_wrong = current_wrong + 1;
                        StateTransition {
                            new_state: QuestionState::LEARNING,
                            correct_streak: new_streak,
                            wrong_count: new_wrong,
                            due_at: Some(Self::due_at_after_review(
                                now,
                                result,
                                new_streak,
                                new_wrong,
                            )),
                        }
                    }
                }
            }
            QuestionState::STABLE => {
                // STABLE 状态下的转移
                match result {
                    ReviewResult::CORRECT => {
                        // 保持 STABLE 状态，增加 streak
                        let new_streak = current_streak + 1;
                        StateTransition {
                            new_state: QuestionState::STABLE,
                            correct_streak: new_streak,
                            wrong_count: current_wrong,
                            due_at: Some(Self::due_at_after_review(
                                now,
                                result,
                                new_streak,
                                current_wrong,
                            )),
                        }
                    }
                    ReviewResult::WRONG | ReviewResult::FUZZY => {
                        // STABLE: 错误清零；模糊减 1（与推荐系统一致）
                        let new_streak = match result {
                            ReviewResult::WRONG => 0,
                            _ => (current_streak - 1).max(0),
                        };
                        let new_wrong = current_wrong + 1;
                        StateTransition {
                            new_state: QuestionState::LEARNING,
                            correct_streak: new_streak,
                            wrong_count: new_wrong,
                            due_at: Some(Self::due_at_after_review(
                                now,
                                result,
                                new_streak,
                                new_wrong,
                            )),
                        }
                    }
                }
            }
            QuestionState::SUSPENDED => {
                // SUSPENDED 状态：恢复后应该变成 DUE
                // 但这个方法不处理恢复，恢复由单独的 suspend/recover 方法处理
                // 这里保持 SUSPENDED 状态不变
                StateTransition {
                    new_state: QuestionState::SUSPENDED,
                    correct_streak: current_streak,
                    wrong_count: current_wrong,
                    due_at: None, // SUSPENDED 状态不更新 due_at
                }
            }
        }
    }

    /// 暂停题目（任意状态 → SUSPENDED）
    pub fn suspend(question: &Question) -> StateTransition {
        StateTransition {
            new_state: QuestionState::SUSPENDED,
            correct_streak: question.correct_streak,
            wrong_count: question.wrong_count,
            due_at: None, // SUSPENDED 状态不更新 due_at
        }
    }

    /// 恢复题目（SUSPENDED → `target_state`）。
    /// `target_state` 由调用方从 `system.PreSuspendState` meta 读取；
    /// meta 缺失时调用方应回退到 `LEARNING`。
    /// 恢复后 `due_at` 设为 `now`，表示立即可被推荐。
    pub fn recover(
        question: &Question,
        target_state: QuestionState,
        now: Timestamp,
    ) -> StateTransition {
        StateTransition {
            new_state: target_state,
            correct_streak: question.correct_streak,
            wrong_count: question.wrong_count,
            due_at: Some(now),
        }
    }

    /// 根据复习结果与更新后的 streak / wrong_count 计算下次复习间隔天数。
    /// 公式与推荐系统 (server/recommendation.rs) 保持一致，用于 due_at 字段。
    /// - CORRECT: ceil(stability² / difficulty)，stability = streak_after + 1
    /// - FUZZY:   上述一半
    /// - WRONG:   1 天
    ///   其中 difficulty = 1 + wrong_count × 0.2
    pub fn calculate_interval_days(
        result: ReviewResult,
        streak_after: i64,
        wrong_count_after: i64,
    ) -> i64 {
        let stability = (streak_after + 1) as f64;
        let difficulty = 1.0 + (wrong_count_after as f64) * 0.2;
        let raw_days = match result {
            ReviewResult::CORRECT => stability * stability / difficulty,
            ReviewResult::FUZZY => (stability * stability / difficulty) * 0.5,
            ReviewResult::WRONG => 1.0,
        };
        raw_days.ceil().max(1.0) as i64
    }

    /// 给定更新后的 streak / wrong_count 与结果，把 due_at 设为 now + interval 天
    fn due_at_after_review(
        now: Timestamp,
        result: ReviewResult,
        streak_after: i64,
        wrong_count_after: i64,
    ) -> Timestamp {
        let interval = Self::calculate_interval_days(result, streak_after, wrong_count_after);
        Timestamp::from(now.as_i64() + interval * 24 * 60 * 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::QuestionId;
    use crate::util::time::Timestamp;

    fn create_question(state: QuestionState) -> Question {
        Question {
            id: QuestionId::from(1i64),
            name: Some("Test Question".to_string()),
            state,
            created_at: Timestamp::from(0),
            deleted_at: None,
            last_review_at: None,
            last_result: None,
            correct_streak: 0,
            wrong_count: 0,
            due_at: None,
        }
    }

    #[test]
    fn test_new_to_learning() {
        let question = create_question(QuestionState::NEW);
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::process_review(&question, ReviewResult::CORRECT, now);

        assert_eq!(result.new_state, QuestionState::LEARNING);
        assert_eq!(result.correct_streak, 1);
    }

    #[test]
    fn test_learning_to_stable() {
        let mut question = create_question(QuestionState::LEARNING);
        question.correct_streak = 2; // 已经连续正确2次
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::process_review(&question, ReviewResult::CORRECT, now);

        assert_eq!(result.new_state, QuestionState::STABLE);
        assert_eq!(result.correct_streak, 3);
    }

    #[test]
    fn test_learning_wrong_resets_streak() {
        let mut question = create_question(QuestionState::LEARNING);
        question.correct_streak = 2;
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::process_review(&question, ReviewResult::WRONG, now);

        assert_eq!(result.new_state, QuestionState::LEARNING);
        assert_eq!(result.correct_streak, 0);
        assert_eq!(result.wrong_count, 1);
    }

    #[test]
    fn test_stable_to_learning_on_fuzzy_decreases_streak() {
        let mut question = create_question(QuestionState::STABLE);
        question.correct_streak = 5;
        let now = Timestamp::from(1000);

        // FUZZY: streak 减 1（与 recommendation.rs::calculate_next_review 一致）
        let result = QuestionStateMachine::process_review(&question, ReviewResult::FUZZY, now);

        assert_eq!(result.new_state, QuestionState::LEARNING);
        assert_eq!(result.correct_streak, 4);
    }

    #[test]
    fn test_suspend() {
        let question = create_question(QuestionState::LEARNING);

        let result = QuestionStateMachine::suspend(&question);

        assert_eq!(result.new_state, QuestionState::SUSPENDED);
        assert!(result.due_at.is_none());
    }

    #[test]
    fn test_recover_to_stable() {
        let question = create_question(QuestionState::SUSPENDED);
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::recover(&question, QuestionState::STABLE, now);

        assert_eq!(result.new_state, QuestionState::STABLE);
        assert_eq!(result.due_at, Some(now));
    }

    #[test]
    fn test_recover_to_learning() {
        let question = create_question(QuestionState::SUSPENDED);
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::recover(&question, QuestionState::LEARNING, now);

        assert_eq!(result.new_state, QuestionState::LEARNING);
        assert_eq!(result.due_at, Some(now));
    }

    // ===== due_at 公式测试（业务规则 5.3：连续答对 → 间隔逐渐拉长）=====
    //
    // 公式: stability = (streak_after + 1)
    //       difficulty = 1 + wrong_count × 0.2
    //       CORRECT: ceil(stability² / difficulty)
    //       FUZZY:   ceil(stability² / difficulty × 0.5)
    //       WRONG:   1 天

    fn due_at_days(transition: &StateTransition, now: Timestamp) -> i64 {
        let due = transition.due_at.expect("due_at should be set");
        (due.as_i64() - now.as_i64()) / 86_400
    }

    #[test]
    fn test_due_at_correct_streak_0() {
        // NEW 答对 → streak=1, wrong=0, stability=2, difficulty=1
        // interval = 4/1 = 4 天
        let question = create_question(QuestionState::NEW);
        let now = Timestamp::from(1000);
        let t = QuestionStateMachine::process_review(&question, ReviewResult::CORRECT, now);
        assert_eq!(t.correct_streak, 1);
        assert_eq!(due_at_days(&t, now), 4);
    }

    #[test]
    fn test_due_at_correct_streak_2() {
        // LEARNING streak=2 答对 → streak=3, stability=4
        // interval = 16/1 = 16 天，同时升 STABLE
        let mut question = create_question(QuestionState::LEARNING);
        question.correct_streak = 2;
        let now = Timestamp::from(1000);
        let t = QuestionStateMachine::process_review(&question, ReviewResult::CORRECT, now);
        assert_eq!(t.correct_streak, 3);
        assert_eq!(t.new_state, QuestionState::STABLE);
        assert_eq!(due_at_days(&t, now), 16);
    }

    #[test]
    fn test_due_at_correct_streak_4_keeps_growing() {
        // STABLE streak=4 答对 → streak=5, stability=6
        // interval = 36/1 = 36 天
        let mut question = create_question(QuestionState::STABLE);
        question.correct_streak = 4;
        let now = Timestamp::from(1000);
        let t = QuestionStateMachine::process_review(&question, ReviewResult::CORRECT, now);
        assert_eq!(t.correct_streak, 5);
        assert_eq!(t.new_state, QuestionState::STABLE);
        assert_eq!(due_at_days(&t, now), 36);
    }

    #[test]
    fn test_due_at_fuzzy_streak_3_is_half() {
        // LEARNING streak=3 fuzzy → streak=2, wrong=1, difficulty=1.2
        // interval = 9/1.2 × 0.5 = 3.75 → ceil = 4 天
        let mut question = create_question(QuestionState::LEARNING);
        question.correct_streak = 3;
        let now = Timestamp::from(1000);
        let t = QuestionStateMachine::process_review(&question, ReviewResult::FUZZY, now);
        assert_eq!(t.correct_streak, 2);
        assert_eq!(t.new_state, QuestionState::LEARNING);
        assert_eq!(due_at_days(&t, now), 4);
    }

    #[test]
    fn test_due_at_wrong_is_one_day() {
        // STABLE streak=5 答错 → streak=0, wrong=1
        // interval 公式: WRONG 直接返回 1.0 → 1 天
        let mut question = create_question(QuestionState::STABLE);
        question.correct_streak = 5;
        question.wrong_count = 0;
        let now = Timestamp::from(1000);
        let t = QuestionStateMachine::process_review(&question, ReviewResult::WRONG, now);
        assert_eq!(t.correct_streak, 0);
        assert_eq!(t.wrong_count, 1);
        assert_eq!(due_at_days(&t, now), 1);
    }

    #[test]
    fn test_due_at_difficulty_with_wrong_count() {
        // STABLE streak=0 答对 → streak=1, wrong=3, difficulty=1.6
        // stability=2, interval = 4/1.6 = 2.5 → ceil = 3 天
        let mut question = create_question(QuestionState::STABLE);
        question.correct_streak = 0;
        question.wrong_count = 3;
        let now = Timestamp::from(1000);
        let t = QuestionStateMachine::process_review(&question, ReviewResult::CORRECT, now);
        assert_eq!(t.correct_streak, 1);
        assert_eq!(due_at_days(&t, now), 3);
    }

    #[test]
    fn test_due_at_correct_streak_2_no_wrong() {
        // 隔离验证 calculate_interval_days: streak_after=2, wrong=0, CORRECT
        // stability=3, difficulty=1, raw=9, ceil=9
        let r = QuestionStateMachine::calculate_interval_days(ReviewResult::CORRECT, 2, 0);
        assert_eq!(r, 9);
        // 同样输入下 FUZZY 应是 4.5 → ceil 5
        let r2 = QuestionStateMachine::calculate_interval_days(ReviewResult::FUZZY, 2, 0);
        assert_eq!(r2, 5);
    }

    #[test]
    fn test_due_at_growth_monotonic_for_correct_streak() {
        // 验证业务规则 5.3：连续答对时间隔逐次增长
        let now = Timestamp::from(1000);
        let mut q = create_question(QuestionState::NEW);
        let mut prev = 0;
        for _ in 0..6 {
            let t = QuestionStateMachine::process_review(&q, ReviewResult::CORRECT, now);
            let d = due_at_days(&t, now);
            assert!(d > prev, "interval must grow: was {prev}, now {d}");
            prev = d;
            // 让下一题用 transition 后的状态/streak
            q.state = t.new_state;
            q.correct_streak = t.correct_streak;
            q.wrong_count = t.wrong_count;
        }
    }
}
