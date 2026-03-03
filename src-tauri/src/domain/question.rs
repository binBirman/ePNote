use crate::domain::enums::{QuestionState, ReviewResult};
use crate::domain::ids::QuestionId;
use crate::util::time::Timestamp;

#[derive(Debug, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub name: Option<String>,
    pub state: QuestionState,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
    // 复习相关字段
    pub last_review_at: Option<Timestamp>,
    pub last_result: Option<ReviewResult>,
    pub correct_streak: i64,
    pub wrong_count: i64,
    pub due_at: Option<Timestamp>,
}
