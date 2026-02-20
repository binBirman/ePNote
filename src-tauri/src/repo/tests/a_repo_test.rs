use crate::db::AssetRow;
use crate::domain::asset::Asset;
use crate::domain::enums::AssetType;
use crate::domain::ids::{AssetId, QuestionId};
use crate::repo::error::ConvertError;
use crate::repo::*;
use crate::util::path::LogicalPath;
use crate::util::time::Timestamp;
use std::path::PathBuf;

#[test]
fn asset_row_domain_roundtrip() {
    let row = AssetRow {
        id: 3,
        question_id: 1,
        type_: "QUESTION".to_string(),
        path: "answer/1.png".to_string(),
        created_at: 1_600_000_200,
        deleted_at: None,
    };

    let domain = asset_repo::row_to_domain(&row).expect("row_to_domain failed");
    assert_eq!(i64::from(domain.id), 3);
    assert_eq!(i64::from(domain.question_id), 1);
    assert_eq!(domain.asset_type.as_str(), "QUESTION");
    assert_eq!(domain.path.as_str(), "answer/1.png");

    let row2 = asset_repo::domain_to_row(&domain).expect("domain_to_row failed");
    assert_eq!(row2.type_, "QUESTION");
    assert_eq!(row2.path, "answer/1.png");
}

#[test]
fn asset_row_invalid_type_returns_err() {
    let row = AssetRow {
        id: 4,
        question_id: 1,
        type_: "UNKNOWN".to_string(),
        path: "answer/2.png".to_string(),
        created_at: 1_600_000_300,
        deleted_at: None,
    };

    let res = asset_repo::row_to_domain(&row);
    assert!(matches!(res, Err(ConvertError::InvalidAssetType(_))));
}

#[test]
fn asset_row_invalid_path_returns_err() {
    let row = AssetRow {
        id: 5,
        question_id: 2,
        type_: "QUESTION".to_string(),
        // path contains parent dir -> should be invalid
        path: "../etc/passwd".to_string(),
        created_at: 1_600_000_400,
        deleted_at: None,
    };

    let res = asset_repo::row_to_domain(&row);
    assert!(matches!(res, Err(ConvertError::InvalidLogicalPath(_))));
}

#[test]
fn domain_to_row_preserves_fields() {
    let lp = LogicalPath::new(PathBuf::from("answer/3.png"));
    let asset = Asset {
        id: AssetId::from(7_i64),
        question_id: QuestionId::from(8_i64),
        asset_type: AssetType::QUESTION,
        path: lp,
        created_at: Timestamp::from(1_600_000_500),
        deleted_at: Some(Timestamp::from(1_600_000_600)),
    };

    let row = asset_repo::domain_to_row(&asset).expect("domain_to_row failed");
    assert_eq!(row.id, i64::from(asset.id));
    assert_eq!(row.question_id, i64::from(asset.question_id));
    assert_eq!(row.type_, asset.asset_type.as_str());
    assert_eq!(row.path, asset.path.as_str());
    assert_eq!(row.created_at, asset.created_at.as_i64());
    assert_eq!(row.deleted_at, Some(asset.deleted_at.unwrap().as_i64()));
}
