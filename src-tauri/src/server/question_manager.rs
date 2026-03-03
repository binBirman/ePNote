//! question_manager.rs 题目管理器，负责题目的增删改查等业务逻辑。
//! 它调用 `QuestionDao`、`MetaDao`、`AssetDao` 等数据访问对象来操作数据库，并将数据库行转换为领域模型。
//! 同时操作asset模块，实现文件资源的物理存储和逻辑删除。
//! 题目管理器还负责维护题目的元信息（`Meta`）和资源（`Asset`）的关联关系，确保数据的一致性。
//! 题目管理器的设计目标是将题目的业务逻辑与数据访问层分离，使得代码更清晰、易于维护和测试。
//!
use crate::asset::store::AssetStore;
use crate::dao::{
    asset_dao::AssetDao, meta_dao::MetaDao, question_dao::QuestionDao, review_dao::ReviewDao,
};
use crate::db::connection::Connection;
use crate::domain::enums::{AssetType, MetaKey, QuestionState, SystemMetaKey};
use crate::domain::ids::{AssetId, MetaId, QuestionId};
use crate::domain::question_info::QuestionInfo;
use crate::error::AppError;
use crate::util::time::Timestamp;
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
) -> Result<QuestionId, AppError> {
    // Debug: Print received image paths
    println!("[DEBUG] create_question received:");
    println!("  question_image_paths count: {}", question_image_paths.len());
    for (i, p) in question_image_paths.iter().enumerate() {
        println!("    [{}]: {}", i, p);
    }

    // 1. 将图片移动到 AssetStore 并获取相对路径列表

    let q_srcs: Vec<PathBuf> = question_image_paths
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let a_srcs: Vec<PathBuf> = answer_image_paths.into_iter().map(PathBuf::from).collect();

    println!("  q_srcs count: {}", q_srcs.len());

    let q_metas = store.save_many(&q_srcs)?;
    let a_metas = store.save_many(&a_srcs)?;

    println!("  q_metas count: {}", q_metas.len());

    // 2. 创建题目记录
    let qd = QuestionDao::new(conn);

    let now = crate::util::time::now_ts();
    let qid = qd.insert(Some(&name), QuestionState::NEW, now)?;

    // 3. 插入资源记录
    let ad = AssetDao::new(conn);
    for meta in q_metas {
        let aid = ad.insert(
            qid.clone(),
            AssetType::QUESTION,
            meta.relative_path.clone(),
            Timestamp::from(meta.created_at.clone()),
        )?;
    }
    for meta in a_metas {
        let aid = ad.insert(
            qid.clone(),
            AssetType::ANSWER,
            meta.relative_path.clone(),
            Timestamp::from(meta.created_at.clone()),
        )?;
    }

    // 4. 插入元信息（科目与知识点）
    let md = MetaDao::new(conn);
    if let Some(s) = subject {
        let mid = md.insert(qid.clone(), MetaKey::System(SystemMetaKey::Subject), &s)?;
    }

    for kp in knowledge_points {
        let mid = md.insert(
            qid.clone(),
            MetaKey::System(SystemMetaKey::KnowledgePoint),
            &kp,
        )?;
    }

    Ok(qid)
}

/// 逻辑删除题目
/// 输入：题目ID
/// 向数据库中设置题目的删除时间戳（`deleted_at`），表示该题目已被删除，但数据仍保留在数据库中。
/// 输出：是否删除成功
///  - 成功返回 `Ok(true)`
///  - 失败返回 `Err(DbError)`
pub fn delete_question(conn: &Connection, qid: QuestionId) -> Result<bool, AppError> {
    let qd = QuestionDao::new(conn);
    let now = crate::util::time::now_ts();
    qd.update_deleted_at(qid, Some(now))?;
    Ok(true)
}

/// 恢复已删除的题目
/// 输入：题目ID
/// 将数据库中该题目的删除时间戳（`deleted_at`）设置为 `NULL`，表示该题目已被恢复。
/// 输出：是否恢复成功
///  - 成功返回 `Ok(true)`
/// - 失败返回 `Err(DbError)`
pub fn restore_question(conn: &Connection, qid: QuestionId) -> Result<bool, AppError> {
    let qd = QuestionDao::new(conn);
    qd.update_deleted_at(qid, None)?;
    Ok(true)
}

/// 永久删除题目（物理删除）
/// 输入：题目ID, AssetStore
/// 删除数据库中的题目、元信息、资源、复习记录，并删除存储中的文件
/// 输出：是否删除成功
pub fn permanently_delete_question(
    conn: &Connection,
    store: &AssetStore,
    qid: QuestionId,
) -> Result<bool, AppError> {
    let qd = QuestionDao::new(conn);
    let md = MetaDao::new(conn);
    let ad = AssetDao::new(conn);

    // 1. 获取该题目的所有资源，用于后续删除文件
    let assets = ad.list_by_question(qid.clone())?;

    // 2. 删除存储中的文件
    let mut entries_to_delete: Vec<(uuid::Uuid, String, std::path::PathBuf)> = Vec::new();
    for asset in &assets {
        if let Ok(uuid) = uuid::Uuid::parse_str(&asset.id.0.to_string()) {
            let relative_path = asset.path.as_str();
            // 构造回收相对路径
            let recycle_relative = format!("recycle/{}/{}", uuid, relative_path);
            entries_to_delete.push((uuid, relative_path, std::path::PathBuf::from(recycle_relative)));
        }
    }
    if !entries_to_delete.is_empty() {
        store.delete_physical(&entries_to_delete)?;
    }

    // 3. 删除数据库中的复习记录
    let qid_i64: i64 = i64::from(qid.clone());
    crate::db::delete_reviews_by_question(conn, qid_i64)?;

    // 4. 删除数据库中的元信息
    let metas = md.list_by_question(qid.clone())?;
    for meta in metas {
        md.delete(meta.id)?;
    }

    // 5. 删除数据库中的资源记录
    for asset in &assets {
        ad.delete(asset.id)?;
    }

    // 6. 删除数据库中的题目记录
    let qid_i64: i64 = i64::from(qid);
    crate::db::delete_question_by_id(conn, qid_i64)?;

    Ok(true)
}

/// 清理回收站中超过指定天数的题目
/// 输入：天数阈值（默认30天）
/// 输出：删除的题目数量
pub fn cleanup_old_deleted_questions(
    conn: &Connection,
    store: &AssetStore,
    days_threshold: i64,
) -> Result<usize, AppError> {
    let qd = QuestionDao::new(conn);
    let now = crate::util::time::now_ts();

    // 计算阈值时间戳
    let threshold_ts = Timestamp(now.0 - (days_threshold * 24 * 3600));

    // 获取所有已删除且删除时间早于阈值的题目
    let old_questions = qd.list_deleted_before(threshold_ts)?;

    let mut deleted_count = 0;
    for question in old_questions {
        let qid = question.id;
        // 永久删除每个题目
        if permanently_delete_question(conn, store, qid).is_ok() {
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

/// 题目改名
/// 输入：题目ID，新题目名
/// 输出：是否改名成功
pub fn rename_question(
    conn: &Connection,
    qid: QuestionId,
    new_name: String,
) -> Result<bool, AppError> {
    let qd = QuestionDao::new(conn);
    qd.update_name(qid, Some(&new_name))?;
    Ok(true)
}

/// 更新题目的科目和知识点
/// 输入：题目ID，新科目（可选），新知识点列表（可选）
/// 输出：是否更新成功
pub fn update_question_meta(
    conn: &Connection,
    qid: QuestionId,
    new_subject: Option<String>,
    new_knowledge_points: Option<Vec<String>>,
) -> Result<bool, AppError> {
    let md = MetaDao::new(conn);

    // 更新科目
    if let Some(subject) = new_subject {
        // 删除旧的科目
        md.delete_by_question_and_key(qid.clone(), MetaKey::System(SystemMetaKey::Subject))?;
        // 添加新的科目
        md.insert(qid.clone(), MetaKey::System(SystemMetaKey::Subject), &subject)?;
    }

    // 更新知识点
    if let Some(kps) = new_knowledge_points {
        // 删除旧的知识点
        md.delete_by_question_and_key(qid.clone(), MetaKey::System(SystemMetaKey::KnowledgePoint))?;
        // 添加新的知识点
        for kp in kps {
            md.insert(qid.clone(), MetaKey::System(SystemMetaKey::KnowledgePoint), &kp)?;
        }
    }

    Ok(true)
}

/// 为题目添加图片
pub fn add_question_images(
    conn: &Connection,
    store: &AssetStore,
    qid: QuestionId,
    image_paths: Vec<String>,
    asset_type: AssetType,
) -> Result<bool, AppError> {
    let srcs: Vec<PathBuf> = image_paths.into_iter().map(PathBuf::from).collect();
    let metas = store.save_many(&srcs)?;

    let ad = AssetDao::new(conn);
    for meta in metas {
        ad.insert(
            qid.clone(),
            asset_type.clone(),
            meta.relative_path.clone(),
            Timestamp::from(meta.created_at.clone()),
        )?;
    }
    Ok(true)
}

/// 删除题目的图片资源（逻辑删除）
pub fn delete_question_image(conn: &Connection, asset_id: String) -> Result<bool, AppError> {
    let ad = AssetDao::new(conn);
    // 将 asset_id 转换为 UUID
    let uuid = uuid::Uuid::parse_str(&asset_id)
        .map_err(|e| AppError::NotFound(format!("Invalid asset id: {}", e)))?;
    let aid = AssetId(uuid);
    ad.delete(aid)?;
    Ok(true)
}

/// 提取单个题目的所有信息（包括元信息和资源）
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
