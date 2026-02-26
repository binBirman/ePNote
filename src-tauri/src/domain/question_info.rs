use crate::domain::*;

pub struct QuestionInfo {
    question: Question,
    assets: Vec<Asset>,
    metas: Vec<Meta>,
    reviews: Vec<Review>,
}
