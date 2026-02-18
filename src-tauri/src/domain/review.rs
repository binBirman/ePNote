use crate::domain::enums::ReviewResult;
use crate::domain::ids::{QuestionId, ReviewId};
use crate::util::time::Timestamp;

#[derive(Debug, Clone)]
pub struct Review {
    pub id: ReviewId,
    pub question_id: QuestionId,
    pub result: ReviewResult,
    pub reviewed_at: Timestamp,
}
