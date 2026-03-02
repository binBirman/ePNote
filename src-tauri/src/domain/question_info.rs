use crate::domain::*;

pub struct QuestionInfo {
    pub question: Question,
    pub assets: Vec<Asset>,
    pub metas: Vec<Meta>,
    pub reviews: Vec<Review>,
}

impl QuestionInfo {
    pub fn new(
        question: Question,
        assets: Vec<Asset>,
        metas: Vec<Meta>,
        reviews: Vec<Review>,
    ) -> Self {
        Self {
            question,
            assets,
            metas,
            reviews,
        }
    }
}
