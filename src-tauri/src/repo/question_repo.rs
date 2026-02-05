pub enum QuestionState {
    NEW,       //新题，尚未复习
    LEARNING,  //学习中，理解不稳定
    STABLE,    //稳定掌握，低频复习
    DUE,       //已到建议复习时间
    SUSPENDED, //用户暂停复习
}

pub struct QuestionRepo {
    pub id: i64,
    pub name: Option<String>,
    pub state: QuestionState,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}
