//! question_manager.rs 题目管理器，负责题目的增删改查等业务逻辑。
//! 它调用 `QuestionDao`、`MetaDao`、`AssetDao` 等数据访问对象来操作数据库，并将数据库行转换为领域模型。
//! 同时操作asset模块，实现文件资源的物理存储和逻辑删除。
//! 题目管理器还负责维护题目的元信息（`Meta`）和资源（`Asset`）的关联关系，确保数据的一致性。
//!
//! 目前为简化模型，server层直接调用了dao层的insert/update/delete方法，并没有实现复杂的业务逻辑。如果后续业务复杂度增加，可以在这里添加事务控制、数据验证、事件发布等功能，以保持领域模型和数据访问层的清晰分离。
//!
use crate::asset::store::AssetStore;
use crate::dao::{asset_dao::AssetDao, meta_dao::MetaDao, question_dao::QuestionDao};
use crate::db::connection::Connection;
use crate::db::schema::{asset_schema::*, meta_schema::*, question_schema::*};
use crate::domain::ids::QuestionId;
use crate::domain::question_info::QuestionInfo;
use crate::error::AppError;
use std::path::PathBuf;

/// 录入题目
/// 输入：题名，题目图路径列表，答案图路径列表，科目，知识点
/// 先将图片资源复制到对应位置，再把题目信息和资源信息写入数据库，最后返回新建题目的ID或错误信息。
/// 输出：新建题目的ID或错误信息
pub fn create_question(
    conn: &Connection,
    store: &AssetStore,
    name: String,
    question_image_paths: Vec<String>,
    answer_image_paths: Vec<String>,
    subject: Option<String>,
    knowledge_points: Vec<String>,
) -> Result<i64, AppError> {
    // 1. 将图片移动到 AssetStore 并获取相对路径列表

    let q_srcs: Vec<PathBuf> = question_image_paths
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let a_srcs: Vec<PathBuf> = answer_image_paths.into_iter().map(PathBuf::from).collect();

    let q_metas = store.save_many(&q_srcs)?;
    let a_metas = store.save_many(&a_srcs)?;

    // 2. 创建题目记录
    let now = crate::util::time::now_ts().as_i64();
    let qid = insert_question(conn, Some(&name), "NEW", now)?;

    // 3. 插入资源记录
    for meta in q_metas {
        let path = meta.relative_path.to_string_lossy().into_owned();
        let created_at = meta
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        insert_asset(conn, qid, "QUESTION", &path, created_at)?;
    }
    for meta in a_metas {
        let path = meta.relative_path.to_string_lossy().into_owned();
        let created_at = meta
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        insert_asset(conn, qid, "ANSWER", &path, created_at)?;
    }

    // 4. 插入元信息（科目与知识点）
    if let Some(s) = subject {
        let mid = insert_meta(conn, qid.clone(), "system.Subject", &s)?;
    }

    for kp in knowledge_points {
        let mid = insert_meta(conn, qid.clone(), "system.KnowledgePoint", &kp)?;
    }

    Ok(qid)
}

/// 逻辑删除题目
/// 输入：题目ID
/// 向数据库中设置题目的删除时间戳（`deleted_at`），表示该题目已被删除，但数据仍保留在数据库中。
/// 输出：是否删除成功
///  - 成功返回 `Ok(true)`
///  - 失败返回 `Err(DbError)`
pub fn delete_question(conn: &Connection, qid: i64) -> Result<bool, AppError> {
    let now = crate::util::time::now_ts().as_i64();
    update_question_deleted_at(conn, qid, Some(now))?;
    Ok(true)
}

/// 恢复已删除的题目
/// 输入：题目ID
/// 将数据库中该题目的删除时间戳（`deleted_at`）设置为 `NULL`，表示该题目已被恢复。
/// 输出：是否恢复成功
///  - 成功返回 `Ok(true)`
/// - 失败返回 `Err(DbError)`
pub fn restore_question(conn: &Connection, qid: i64) -> Result<bool, AppError> {
    update_question_deleted_at(conn, qid, None)?;
    Ok(true)
}

/// 题目改名
/// 输入：题目ID，新题目名
/// 输出：是否改名成功
pub fn rename_question(conn: &Connection, qid: i64, new_name: String) -> Result<bool, AppError> {
    update_question_name(conn, qid, Some(&new_name))?;
    Ok(true)
}

/// 提取单个题目的所有信息,返回domain结构（包括元信息和资源）
/// 输入：题目ID
/// 输出：对应question_info结构体，或错误信息
pub fn get_question_detail(conn: &Connection, qid: QuestionId) -> Result<QuestionInfo, AppError> {
    let qd = QuestionDao::new(conn);
    let md = MetaDao::new(conn);
    let ad = AssetDao::new(conn);
    let rd = crate::dao::review_dao::ReviewDao::new(conn);

    // 获取题目信息
    let question = qd
        .get_by_id(qid.clone())?
        .ok_or_else(|| AppError::NotFound("question not found".to_string()))?;

    // 获取元信息
    let metas = md.list_by_question(qid.clone())?;

    // 获取资源信息
    let assets = ad.list_by_question(qid.clone())?;

    // 获取复习记录信息
    let reviews = rd.list_by_question(qid.clone())?;

    let q_info = QuestionInfo::new(question, assets, metas, reviews);
    Ok(q_info)
}
