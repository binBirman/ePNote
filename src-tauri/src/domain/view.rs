use crate::domain::enums::QuestionState;
use crate::domain::ids::QuestionId;
use crate::util::time::Timestamp;

#[derive(Debug, Clone)]
pub struct View {
    pub id: QuestionId,
    pub name: Option<String>,
    pub state: QuestionState,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
    pub subject: Option<String>,
    pub last_reviewed_at: Timestamp,
}
