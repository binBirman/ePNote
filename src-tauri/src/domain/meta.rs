use crate::domain::enums::MetaKey;
use crate::domain::ids::{MetaId, QuestionId};

#[derive(Debug, Clone)]
pub struct Meta {
    pub id: MetaId,
    pub question_id: QuestionId,
    pub key: MetaKey,
    pub value: String,
}
