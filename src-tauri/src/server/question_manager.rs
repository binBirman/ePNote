//! question_manager.rs 题目管理器，负责题目的增删改查等业务逻辑。
//! 它调用 `QuestionDao`、`MetaDao`、`AssetDao` 等数据访问对象来操作数据库，并将数据库行转换为领域模型。
//! 同时操作asset模块，实现文件资源的物理存储和逻辑删除。
//! 题目管理器还负责维护题目的元信息（`Meta`）和资源（`Asset`）的关联关系，确保数据的一致性。
//! 题目管理器的设计目标是将题目的业务逻辑与数据访问层分离，使得代码更清晰、易于维护和测试。
//!

/// 录入题目
/// 输入：题名，题目图路径列表，答案图路径列表，科目，知识点
/// 先将图片资源复制到对应位置，再把题目信息和资源信息写入数据库，最后返回新建题目的ID或错误信息。
/// 输出：新建题目的ID或错误信息
pub fn create_question(
    name: String,
    question_image_paths: Vec<String>,
    answer_image_paths: Vec<String>,
    subject: Option<String>,
    knowledge_points: Vec<String>,
) {
    // 1. 复制图片资源到指定位置，获取新的路径列表
    // 2. 创建题目记录，获取新题目的ID
    // 3. 创建资源记录，关联到新题目ID
    // 4. 创建元信息记录（如科目、知识点），关联到新题目ID
    // 5. 返回新题目的ID或错误信息
}

/// 逻辑删除题目
/// 输入：题目ID
/// 向数据库中设置题目的删除时间戳（`deleted_at`），表示该题目已被删除，但数据仍保留在数据库中。
/// 输出：是否删除成功
///  - 成功返回 `Ok(true)`
///  - 失败返回 `Err(DbError)`
pub fn delete_question() {}

/// 恢复已删除的题目
/// 输入：题目ID
/// 将数据库中该题目的删除时间戳（`deleted_at`）设置为 `NULL`，表示该题目已被恢复。
/// 输出：是否恢复成功
///  - 成功返回 `Ok(true)`
/// - 失败返回 `Err(DbError)`
pub fn restore_question() {}

/// 题目改名
/// 输入：题目ID，新题目名
/// 输出：是否改名成功
pub fn rename_question() {}

/// 提取单个题目的所有信息（包括元信息和资源）
/// 输入：题目ID
/// 输出：对应question_info结构体，或错误信息
pub fn get_question_detail() {}
