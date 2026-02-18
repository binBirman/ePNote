use crate::db::MetaRow;
use crate::domain::enums::MetaKey;
use crate::domain::ids::{MetaId, QuestionId};
use crate::domain::meta::Meta;
use crate::repo::error::ConvertResult;

pub fn row_to_domain(row: &MetaRow) -> ConvertResult<Meta> {
    Ok(Meta {
        id: MetaId::from(row.id),
        question_id: QuestionId::from(row.question_id),
        key: MetaKey::try_from(row.key.clone())?,
        value: row.value.clone(),
    })
}

pub fn domain_to_row(domain: &Meta) -> ConvertResult<MetaRow> {
    Ok(MetaRow {
        id: i64::from(domain.id),
        question_id: i64::from(domain.question_id),
        key: domain.key.as_str(),
        value: domain.value.clone(),
    })
}
