//! 状态机模块 - 实现错题本系统的状态转移逻辑
//!
//! 状态转移规则（根据设计文档）：
//! - NEW → LEARNING: 首次复习（任意结果）
//! - LEARNING → LEARNING: 复习结果 = 错误/模糊
//! - LEARNING → STABLE: 连续正确 ≥ 3 次
//! - STABLE → LEARNING: 任意一次复习结果 = 错误/模糊
//! - STABLE → DUE: 当前时间 ≥ due_at
//! - DUE → LEARNING/STABLE: 复习结果决定
//! - 任意状态 → SUSPENDED: 用户手动暂停
//! - SUSPENDED → DUE: 用户手动恢复

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
                    ReviewResult::CORRECT => StateTransition {
                        new_state: QuestionState::LEARNING,
                        correct_streak: 1,
                        wrong_count: current_wrong,
                        due_at: Some(Self::calculate_next_due(now, QuestionState::LEARNING)),
                    },
                    ReviewResult::WRONG | ReviewResult::FUZZY => StateTransition {
                        new_state: QuestionState::LEARNING,
                        correct_streak: 0,
                        wrong_count: current_wrong + 1,
                        due_at: Some(Self::calculate_next_due(now, QuestionState::LEARNING)),
                    },
                }
            }
            QuestionState::LEARNING => {
                // LEARNING 状态下的转移
                match result {
                    ReviewResult::CORRECT => {
                        // 连续正确，增加 streak
                        let new_streak = current_streak + 1;
                        if new_streak >= STABLE_THRESHOLD {
                            // LEARNING → STABLE: 连续正确 ≥ 3 次
                            StateTransition {
                                new_state: QuestionState::STABLE,
                                correct_streak: new_streak,
                                wrong_count: current_wrong,
                                due_at: Some(Self::calculate_next_due(now, QuestionState::STABLE)),
                            }
                        } else {
                            // 保持 LEARNING 状态
                            StateTransition {
                                new_state: QuestionState::LEARNING,
                                correct_streak: new_streak,
                                wrong_count: current_wrong,
                                due_at: Some(Self::calculate_next_due(now, QuestionState::LEARNING)),
                            }
                        }
                    }
                    ReviewResult::WRONG | ReviewResult::FUZZY => {
                        // LEARNING → LEARNING: 错误/模糊，streak 清零
                        StateTransition {
                            new_state: QuestionState::LEARNING,
                            correct_streak: 0,
                            wrong_count: current_wrong + 1,
                            due_at: Some(Self::calculate_next_due(now, QuestionState::LEARNING)),
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
                            due_at: Some(Self::calculate_next_due(now, QuestionState::STABLE)),
                        }
                    }
                    ReviewResult::WRONG | ReviewResult::FUZZY => {
                        // STABLE → LEARNING: 错误/模糊，回到 LEARNING
                        StateTransition {
                            new_state: QuestionState::LEARNING,
                            correct_streak: 0,
                            wrong_count: current_wrong + 1,
                            due_at: Some(Self::calculate_next_due(now, QuestionState::LEARNING)),
                        }
                    }
                }
            }
            QuestionState::DUE => {
                // DUE 状态下的转移
                match result {
                    ReviewResult::CORRECT => {
                        // DUE → STABLE: 正确
                        let new_streak = current_streak + 1;
                        StateTransition {
                            new_state: QuestionState::STABLE,
                            correct_streak: new_streak,
                            wrong_count: current_wrong,
                            due_at: Some(Self::calculate_next_due(now, QuestionState::STABLE)),
                        }
                    }
                    ReviewResult::WRONG | ReviewResult::FUZZY => {
                        // DUE → LEARNING: 错误/模糊
                        StateTransition {
                            new_state: QuestionState::LEARNING,
                            correct_streak: 0,
                            wrong_count: current_wrong + 1,
                            due_at: Some(Self::calculate_next_due(now, QuestionState::LEARNING)),
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

    /// 恢复题目（SUSPENDED → DUE）
    pub fn recover(question: &Question, now: Timestamp) -> StateTransition {
        StateTransition {
            new_state: QuestionState::DUE,
            correct_streak: question.correct_streak,
            wrong_count: question.wrong_count,
            due_at: Some(now), // 恢复时设置为当前时间，表示立即到期
        }
    }

    /// 检查题目是否应该从 STABLE 转移到 DUE（到期检查）
    pub fn check_due_transition(question: &Question, now: Timestamp) -> Option<QuestionState> {
        if question.state == QuestionState::STABLE {
            if let Some(due_at) = question.due_at {
                if now >= due_at {
                    return Some(QuestionState::DUE);
                }
            }
        }
        None
    }

    /// 根据状态计算下次复习间隔
    fn calculate_next_due(now: Timestamp, state: QuestionState) -> Timestamp {
        let interval_seconds = match state {
            QuestionState::LEARNING => 1 * 24 * 60 * 60, // 1天后
            QuestionState::STABLE => 7 * 24 * 60 * 60,   // 7天后
            _ => 1 * 24 * 60 * 60,                       // 默认1天
        };
        Timestamp::from(now.as_i64() + interval_seconds)
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
    fn test_stable_to_learning_on_wrong() {
        let mut question = create_question(QuestionState::STABLE);
        question.correct_streak = 5;
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::process_review(&question, ReviewResult::FUZZY, now);

        assert_eq!(result.new_state, QuestionState::LEARNING);
        assert_eq!(result.correct_streak, 0);
    }

    #[test]
    fn test_due_to_stable_on_correct() {
        let question = create_question(QuestionState::DUE);
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::process_review(&question, ReviewResult::CORRECT, now);

        assert_eq!(result.new_state, QuestionState::STABLE);
    }

    #[test]
    fn test_suspend() {
        let question = create_question(QuestionState::LEARNING);

        let result = QuestionStateMachine::suspend(&question);

        assert_eq!(result.new_state, QuestionState::SUSPENDED);
        assert!(result.due_at.is_none());
    }

    #[test]
    fn test_recover() {
        let question = create_question(QuestionState::SUSPENDED);
        let now = Timestamp::from(1000);

        let result = QuestionStateMachine::recover(&question, now);

        assert_eq!(result.new_state, QuestionState::DUE);
    }
}
