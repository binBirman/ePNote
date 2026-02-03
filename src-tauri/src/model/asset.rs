#[derive(Debug, Clone)]
pub struct Asset {
    pub id: i64,
    pub question_id: i64,
    pub asset_type: String, // "question" | "answer" | "explain" | "other"
    pub path: String,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}
