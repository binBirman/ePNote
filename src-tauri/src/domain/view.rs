use crate::domain::enums::QuestionState;
use crate::domain::ids::QuestionId;
use crate::util::time::Timestamp;

#[derive(Debug, Clone)]
pub struct View {
    pub id: QuestionId,
    pub name: Option<String>,
    pub state: QuestionState,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
    pub subject: String,
    pub knowledge_points: Vec<String>,
    pub last_reviewed_at: Timestamp,
    /// 累计答错次数（来自 `question.wrong_count`）。
    pub wrong_count: i64,
    /// 错误率（来自 `review_summary.error_rate`），新题可能为 `None`。
    pub error_rate: Option<f64>,
}
