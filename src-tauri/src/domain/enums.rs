#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionState {
    NEW,       //新题，尚未复习
    LEARNING,  //学习中，理解不稳定
    STABLE,    //稳定掌握，低频复习
    DUE,       //已到建议复习时间
    SUSPENDED, //用户暂停复习
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewResult {
    CORRECT, //正确，完全记住了
    WRONG,   //错误，完全忘记了
    FUZZY,   //模糊，记忆不清晰
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    QUESTION, //题干图片、音频等
    ANSWER,   //答案图片、音频等
    EXPLAIN,  //解析图片、音频等
    OTHER,    //其他与题目相关的资源
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaKey {
    System(SystemMetaKey),
    Extension(ExtensionMetaKey),
    User(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemMetaKey {
    SourcePaper,
    Subject,
    KnowledgePoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionMetaKey {
    Difficulty,
    TeacherComment,
}
