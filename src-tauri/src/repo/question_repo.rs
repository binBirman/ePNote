use crate::db::QuestionRow;
use crate::domain::enums::QuestionState;
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
    })
}

pub fn domain_to_row(domain: &Question) -> ConvertResult<QuestionRow> {
    Ok(QuestionRow {
        id: i64::from(domain.id),
        name: domain.name.clone(),
        state: String::from(domain.state.clone()),
        created_at: domain.created_at.as_i64(),
        deleted_at: domain.deleted_at.map(|t| t.as_i64()),
    })
}
