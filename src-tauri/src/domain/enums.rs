#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionState {
    NEW,       //新题，尚未复习
    LEARNING,  //学习中，理解不稳定
    STABLE,    //稳定掌握，低频复习
    DUE,       //已到建议复习时间
    SUSPENDED, //用户暂停复习
}

impl QuestionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestionState::NEW => "NEW",
            QuestionState::LEARNING => "LEARNING",
            QuestionState::STABLE => "STABLE",
            QuestionState::DUE => "DUE",
            QuestionState::SUSPENDED => "SUSPENDED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "NEW" | "new" => Some(QuestionState::NEW),
            "LEARNING" | "learning" => Some(QuestionState::LEARNING),
            "STABLE" | "stable" => Some(QuestionState::STABLE),
            "DUE" | "due" => Some(QuestionState::DUE),
            "SUSPENDED" | "suspended" => Some(QuestionState::SUSPENDED),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewResult {
    CORRECT, //正确，完全记住了
    WRONG,   //错误，完全忘记了
    FUZZY,   //模糊，记忆不清晰
}

impl ReviewResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewResult::CORRECT => "CORRECT",
            ReviewResult::WRONG => "WRONG",
            ReviewResult::FUZZY => "FUZZY",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CORRECT" | "correct" => Some(ReviewResult::CORRECT),
            "WRONG" | "wrong" => Some(ReviewResult::WRONG),
            "FUZZY" | "fuzzy" => Some(ReviewResult::FUZZY),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    QUESTION, //题干图片、音频等
    ANSWER,   //答案图片、音频等
    EXPLAIN,  //解析图片、音频等
    OTHER,    //其他与题目相关的资源
}

impl AssetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::QUESTION => "QUESTION",
            AssetType::ANSWER => "ANSWER",
            AssetType::EXPLAIN => "EXPLAIN",
            AssetType::OTHER => "OTHER",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "QUESTION" | "question" => Some(AssetType::QUESTION),
            "ANSWER" | "answer" => Some(AssetType::ANSWER),
            "EXPLAIN" | "explain" => Some(AssetType::EXPLAIN),
            "OTHER" | "other" => Some(AssetType::OTHER),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaKey {
    System(SystemMetaKey),       // 系统定义且必要的元信息
    Extension(ExtensionMetaKey), // 系统定义且可选的扩展元信息
    User(String),                // 用户自定义的元信息
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemMetaKey {
    Subject,        // 科目
    KnowledgePoint, // 知识点
}

impl MetaKey {
    /// Serialize to string form: `system.<key>` | `extension.<key>` | `user.<key>`
    pub fn as_str(&self) -> String {
        match self {
            MetaKey::System(k) => format!("system.{}", k.as_str()),
            MetaKey::Extension(k) => format!("extension.{}", k.as_str()),
            MetaKey::User(s) => format!("user.{}", s),
        }
    }

    /// Parse from the same string form produced by `as_str`.
    pub fn from_str(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix("system.") {
            SystemMetaKey::from_str(rest).map(MetaKey::System)
        } else if let Some(rest) = s.strip_prefix("extension.") {
            ExtensionMetaKey::from_str(rest).map(MetaKey::Extension)
        } else if let Some(rest) = s.strip_prefix("user.") {
            Some(MetaKey::User(rest.to_string()))
        } else {
            None
        }
    }
}

impl SystemMetaKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemMetaKey::Subject => "Subject",
            SystemMetaKey::KnowledgePoint => "KnowledgePoint",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Subject" | "subject" => Some(SystemMetaKey::Subject),
            "KnowledgePoint" | "knowledgepoint" | "knowledge_point" => {
                Some(SystemMetaKey::KnowledgePoint)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionMetaKey {
    SourcePaper, // 试卷来源
    Difficulty,  // 难度等级
}

impl ExtensionMetaKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtensionMetaKey::SourcePaper => "SourcePaper",
            ExtensionMetaKey::Difficulty => "Difficulty",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SourcePaper" | "sourcepaper" | "source_paper" | "source" => {
                Some(ExtensionMetaKey::SourcePaper)
            }
            "Difficulty" | "difficulty" => Some(ExtensionMetaKey::Difficulty),
            _ => None,
        }
    }
}
