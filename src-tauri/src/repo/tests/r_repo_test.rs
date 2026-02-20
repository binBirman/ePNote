use crate::db::ReviewRow;
use crate::domain::enums::ReviewResult;
use crate::domain::ids::{QuestionId, ReviewId};
use crate::domain::review::Review;
use crate::repo::*;
use crate::util::time::Timestamp;

#[test]
fn review_row_domain_roundtrip() {
    let row = ReviewRow {
        id: 2,
        question_id: 1,
        result: "CORRECT".to_string(),
        reviewed_at: 1_600_000_100,
    };

    let domain = review_repo::row_to_domain(&row).expect("row_to_domain failed");
    assert_eq!(i64::from(domain.id), 2);
    assert_eq!(i64::from(domain.question_id), 1);
    assert_eq!(domain.result.as_str(), "CORRECT");

    let row2 = review_repo::domain_to_row(&domain).expect("domain_to_row failed");
    assert_eq!(row2.question_id, 1);
    assert_eq!(row2.result, "CORRECT");
    // also ensure id and timestamp roundtrip
    assert_eq!(row2.id, 2);
    assert_eq!(row2.reviewed_at, 1_600_000_100);
}

#[test]
fn review_row_invalid_result_returns_err() {
    let row = ReviewRow {
        id: 3,
        question_id: 1,
        result: "INVALID".to_string(),
        reviewed_at: 1_600_000_200,
    };

    // invalid result string should fail conversion
    assert!(review_repo::row_to_domain(&row).is_err());
}

#[test]
fn review_row_variant_wrong_and_fuzzy_roundtrip() {
    let row_wrong = ReviewRow {
        id: 4,
        question_id: 2,
        result: "WRONG".to_string(),
        reviewed_at: 1_600_000_300,
    };

    let domain_wrong =
        review_repo::row_to_domain(&row_wrong).expect("row_to_domain failed for WRONG");
    assert_eq!(domain_wrong.result.as_str(), "WRONG");

    let row2 = review_repo::domain_to_row(&domain_wrong).expect("domain_to_row failed for WRONG");
    assert_eq!(row2.result, "WRONG");

    let row_fuzzy = ReviewRow {
        id: 5,
        question_id: 2,
        result: "FUZZY".to_string(),
        reviewed_at: 1_600_000_400,
    };

    let domain_fuzzy =
        review_repo::row_to_domain(&row_fuzzy).expect("row_to_domain failed for FUZZY");
    assert_eq!(domain_fuzzy.result.as_str(), "FUZZY");

    let row2f = review_repo::domain_to_row(&domain_fuzzy).expect("domain_to_row failed for FUZZY");
    assert_eq!(row2f.result, "FUZZY");
}

#[test]
fn domain_to_row_preserves_fields_for_all_results() {
    let cases = vec![
        ReviewResult::CORRECT,
        ReviewResult::WRONG,
        ReviewResult::FUZZY,
    ];
    let mut id_base = 10;
    for res in cases {
        let review = Review {
            id: ReviewId::from(id_base),
            question_id: QuestionId::from(id_base + 1),
            result: res.clone(),
            reviewed_at: Timestamp::from(1_700_000_000 + id_base),
        };

        let row = review_repo::domain_to_row(&review).expect("domain_to_row failed");
        assert_eq!(row.id, i64::from(review.id));
        assert_eq!(row.question_id, i64::from(review.question_id));
        assert_eq!(row.reviewed_at, review.reviewed_at.as_i64());
        assert_eq!(row.result, review.result.as_str());

        id_base += 2;
    }
}
