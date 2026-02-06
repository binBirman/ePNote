use crate::domain::enums::*;
use crate::domain::ids::*;
use crate::util::time::*;

#[derive(Debug, Clone)]
pub struct Meta {
    pub id: MetaId,
    pub question_id: QuestionId,
    pub key: MetaKey,
    pub value: String,
}
