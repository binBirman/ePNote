use crate::db::ReviewRow;
use crate::domain::enums::ReviewResult;
use crate::domain::ids::{QuestionId, ReviewId};
use crate::domain::review::Review;
use crate::repo::error::ConvertResult;
use crate::util::time::Timestamp;

pub fn row_to_domain(row: &ReviewRow) -> ConvertResult<Review> {
    Ok(Review {
        id: ReviewId::from(row.id),
        question_id: QuestionId::from(row.question_id),
        result: ReviewResult::try_from(row.result.clone())?,
        reviewed_at: Timestamp::from(row.reviewed_at),
    })
}

pub fn domain_to_row(domain: &Review) -> ConvertResult<ReviewRow> {
    Ok(ReviewRow {
        id: i64::from(domain.id),
        question_id: i64::from(domain.question_id),
        result: String::from(domain.result.clone()),
        reviewed_at: domain.reviewed_at.as_i64(),
    })
}
