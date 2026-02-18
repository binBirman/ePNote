use crate::domain::enums::QuestionState;
use crate::domain::ids::QuestionId;
use crate::util::time::Timestamp;

#[derive(Debug, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub name: Option<String>,
    pub state: QuestionState,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}
