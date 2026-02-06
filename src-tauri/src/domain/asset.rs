use crate::domain::enums::*;
use crate::domain::ids::*;
use crate::util::time::*;

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: AssetId,
    pub question_id: QuestionId,
    pub asset_type: AssetType,
    pub path: String,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}
