#[derive(Debug, Clone)]
pub struct Meta {
    pub id: i64,
    pub question_id: i64,
    pub key: String, // "sys.subject" | "ext.difficulty" | "user.tag" 等
    pub value: String,
}
