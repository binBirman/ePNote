use crate::domain::enums::AssetType;
use crate::domain::ids::{AssetId, QuestionId};
use crate::util::path::LogicalPath;
use crate::util::time::timestamp::Timestamp;

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: AssetId,
    pub question_id: QuestionId,
    pub asset_type: AssetType,
    pub path: LogicalPath,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}
