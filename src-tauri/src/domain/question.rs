use crate::domain::enums::*;
use crate::domain::ids::*;
use crate::util::time::*;

#[derive(Debug, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub name: Option<String>,
    pub state: QuestionState,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}
