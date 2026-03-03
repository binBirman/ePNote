use crate::db::QuestionRow;
use crate::domain::enums::{QuestionState, ReviewResult};
use crate::domain::ids::QuestionId;
use crate::domain::question::Question;
use crate::repo::error::ConvertResult;
use crate::util::time::Timestamp;

pub fn row_to_domain(row: &QuestionRow) -> ConvertResult<Question> {
    Ok(Question {
        id: QuestionId::from(row.id),
        name: row.name.clone(),
        state: QuestionState::try_from(row.state.clone())?,
        created_at: Timestamp::from(row.created_at),
        deleted_at: row.deleted_at.map(Timestamp::from),
        last_review_at: row.last_review_at.map(Timestamp::from),
        last_result: row.last_result.as_ref().and_then(|s| ReviewResult::try_from(s.clone()).ok()),
        correct_streak: row.correct_streak,
        wrong_count: row.wrong_count,
        due_at: row.due_at.map(Timestamp::from),
    })
}

pub fn domain_to_row(domain: &Question) -> ConvertResult<QuestionRow> {
    Ok(QuestionRow {
        id: i64::from(domain.id),
        name: domain.name.clone(),
        state: String::from(domain.state.clone()),
        created_at: domain.created_at.as_i64(),
        deleted_at: domain.deleted_at.map(|t| t.as_i64()),
        last_review_at: domain.last_review_at.map(|t| t.as_i64()),
        last_result: domain.last_result.as_ref().map(|r| String::from(r.as_str())),
        correct_streak: domain.correct_streak,
        wrong_count: domain.wrong_count,
        due_at: domain.due_at.map(|t| t.as_i64()),
    })
}
