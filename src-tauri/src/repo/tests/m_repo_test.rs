use crate::db::MetaRow;
use crate::domain::enums::MetaKey;
use crate::domain::ids::{MetaId, QuestionId};
use crate::domain::meta::Meta;
use crate::repo::error::ConvertError;
use crate::repo::*;

#[test]
fn meta_row_domain_roundtrip() {
    let row = MetaRow {
        id: 11,
        question_id: 2,
        key: "system.Subject".to_string(),
        value: "math".to_string(),
    };

    let domain = meta_repo::row_to_domain(&row).expect("row_to_domain failed");
    assert_eq!(i64::from(domain.id), 11);
    assert_eq!(i64::from(domain.question_id), 2);
    assert_eq!(domain.value, "math");

    // key roundtrip as string
    let row2 = meta_repo::domain_to_row(&domain).expect("domain_to_row failed");
    assert_eq!(row2.key, domain.key.as_str());
}

#[test]
fn meta_row_invalid_key_returns_err() {
    let row = MetaRow {
        id: 12,
        question_id: 3,
        key: "invalid.key".to_string(),
        value: "v".to_string(),
    };

    let res = meta_repo::row_to_domain(&row);
    assert!(matches!(res, Err(ConvertError::InvalidMetaKey(_))));
}

#[test]
fn domain_to_row_preserves_fields() {
    let meta = Meta {
        id: MetaId::from(21_i64),
        question_id: QuestionId::from(22_i64),
        key: MetaKey::System(crate::domain::enums::SystemMetaKey::Subject),
        value: "science".to_string(),
    };

    let row = meta_repo::domain_to_row(&meta).expect("domain_to_row failed");
    assert_eq!(row.id, i64::from(meta.id));
    assert_eq!(row.question_id, i64::from(meta.question_id));
    assert_eq!(row.value, meta.value);
    assert_eq!(row.key, meta.key.as_str());
}
