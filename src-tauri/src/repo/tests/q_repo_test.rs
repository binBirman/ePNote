use crate::db::QuestionRow;
use crate::repo::*;

#[test]
fn question_row_domain_roundtrip() {
    let row = QuestionRow {
        id: 1,
        name: Some("Test".to_string()),
        state: "NEW".to_string(),
        created_at: 1_600_000_000,
        deleted_at: None,
    };

    let domain = question_repo::row_to_domain(&row).expect("row_to_domain failed");
    assert_eq!(i64::from(domain.id), 1);
    assert_eq!(domain.name.as_deref(), Some("Test"));
    assert_eq!(domain.state.as_str(), "NEW");

    let row2 = question_repo::domain_to_row(&domain).expect("domain_to_row failed");
    assert_eq!(row2.id, 1);
    assert_eq!(row2.state, "NEW");
}

#[test]
fn question_row_domain_roundtrip_with_deleted_at() {
    let row = QuestionRow {
        id: 42,
        name: Some("Deleted".to_string()),
        state: "SUSPENDED".to_string(),
        created_at: 1_600_000_100,
        deleted_at: Some(1_700_000_000),
    };

    let domain = question_repo::row_to_domain(&row).expect("row_to_domain failed");
    assert_eq!(i64::from(domain.id), 42);
    assert_eq!(domain.name.as_deref(), Some("Deleted"));
    assert_eq!(domain.state.as_str(), "SUSPENDED");
    assert_eq!(domain.deleted_at.map(|t| t.as_i64()), Some(1_700_000_000));

    let row2 = question_repo::domain_to_row(&domain).expect("domain_to_row failed");
    assert_eq!(row2.id, 42);
    assert_eq!(row2.state, "SUSPENDED");
    assert_eq!(row2.deleted_at, Some(1_700_000_000));
}

#[test]
fn question_row_invalid_state_errors() {
    let row = QuestionRow {
        id: 7,
        name: Some("BadState".to_string()),
        state: "NOT_A_STATE".to_string(),
        created_at: 1_600_000_200,
        deleted_at: None,
    };

    let res = question_repo::row_to_domain(&row);
    assert!(
        res.is_err(),
        "expected conversion to fail for invalid state"
    );
}
