use crate::domain::enums::*;
use crate::domain::ids::*;
use crate::util::time::*;

#[derive(Debug, Clone)]
pub struct Review {
    pub id: ReviewId,
    pub question_id: QuestionId,
    pub result: ReviewResult,
    pub reviewed_at: Timestamp,
}
