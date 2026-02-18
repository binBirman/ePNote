use crate::db::AssetRow;
use crate::domain::asset::Asset;
use crate::domain::enums::AssetType;
use crate::domain::ids::{AssetId, QuestionId};
use crate::repo::error::{ConvertError, ConvertResult};
use crate::util::path::LogicalPath;
use crate::util::time::Timestamp;

pub fn row_to_domain(row: &AssetRow) -> ConvertResult<Asset> {
    let path = match LogicalPath::try_from(row.path.as_str()) {
        Ok(p) => p,
        Err(_) => return Err(ConvertError::InvalidLogicalPath(row.path.clone())),
    };

    Ok(Asset {
        id: AssetId::from(row.id),
        question_id: QuestionId::from(row.question_id),
        asset_type: AssetType::try_from(row.type_.clone())?,
        path,
        created_at: Timestamp::from(row.created_at),
        deleted_at: row.deleted_at.map(Timestamp::from),
    })
}

pub fn domain_to_row(domain: &Asset) -> ConvertResult<AssetRow> {
    Ok(AssetRow {
        id: i64::from(domain.id),
        question_id: i64::from(domain.question_id),
        type_: domain.asset_type.as_str().to_string(),
        path: domain.path.as_str(),
        created_at: domain.created_at.as_i64(),
        deleted_at: domain.deleted_at.map(|t| t.as_i64()),
    })
}
