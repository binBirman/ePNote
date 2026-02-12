//! 异步数据库操作
//!
//! 使用 sqlx 异步 API，复用现有 schema 模块的 SQL 语句

use sqlx::{Error as SqlxError, Pool, Sqlite};

use crate::db::error::DbError;

// 异步数据库连接
#[derive(Debug, Clone)]
pub struct AsyncDb {
    pool: Pool<Sqlite>,
}

impl AsyncDb {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// 获取数据库连接池
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}

// ============ Question 相关操作 ============

/// 异步 Question 行结构（返回 QuestionRow）
#[derive(Debug)]
pub struct QuestionRow {
    pub id: i64,
    pub name: Option<String>,
    pub state: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}

/// 插入新的 Question
pub async fn insert_question(
    pool: &Pool<Sqlite>,
    name: Option<String>,
    state: &str,
    created_at: i64,
) -> Result<i64, DbError> {
    let result = sqlx::query(
        r#"
        INSERT INTO question (name, state, created_at)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(name)
    .bind(state)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// 更新 Question 的名称
pub async fn update_question_name(
    pool: &Pool<Sqlite>,
    question_id: i64,
    new_name: Option<String>,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        UPDATE question
        SET name = ?1
        WHERE id = ?2
        "#,
    )
    .bind(new_name)
    .bind(question_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 逻辑删除 Question（设置 deleted_at）
pub async fn soft_delete_question(
    pool: &Pool<Sqlite>,
    question_id: i64,
    deleted_at: i64,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        UPDATE question
        SET deleted_at = ?1
        WHERE id = ?2
        "#,
    )
    .bind(deleted_at)
    .bind(question_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 根据 ID 查找 Question
pub async fn select_question_by_id(
    pool: &Pool<Sqlite>,
    id: i64,
) -> Result<Option<QuestionRow>, DbError> {
    let result = sqlx::query_as::<QuestionRow>(
        r#"
        SELECT id, name, state, created_at, deleted_at
        FROM question
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// 根据名称查找 Question
pub async fn select_question_by_name(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<Option<QuestionRow>, DbError> {
    let result = sqlx::query_as::<QuestionRow>(
        r#"
        SELECT id, name, state, created_at, deleted_at
        FROM question
        WHERE name = ?1
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// 分页查询未删除的 Questions
pub async fn select_questions_page(
    pool: &Pool<Sqlite>,
    limit: i64,
    offset: i64,
) -> Result<Vec<QuestionRow>, DbError> {
    let results = sqlx::query_as::<QuestionRow>(
        r#"
        SELECT id, name, state, created_at, deleted_at
        FROM question
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 查询所有未删除的 Questions
pub async fn select_questions_active(pool: &Pool<Sqlite>) -> Result<Vec<QuestionRow>, DbError> {
    let results = sqlx::query_as::<QuestionRow>(
        r#"
        SELECT id, name, state, created_at, deleted_at
        FROM question
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 根据 state 查询 Questions
pub async fn select_questions_by_state(
    pool: &Pool<Sqlite>,
    state: &str,
) -> Result<Vec<QuestionRow>, DbError> {
    let results = sqlx::query_as::<QuestionRow>(
        r#"
        SELECT id, name, state, created_at, deleted_at
        FROM question
        WHERE state = ?1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(state)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 统计未删除的 Questions 数量
pub async fn count_questions_active(pool: &Pool<Sqlite>) -> Result<i64, DbError> {
    let result = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM question
        WHERE deleted_at IS NULL
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(result.map(|r| r.count.unwrap_or(0)).unwrap_or(0))
}

// ============ Review 相关操作 ============

/// 异步 Review 行结构
#[derive(Debug)]
pub struct ReviewRow {
    pub id: i64,
    pub question_id: i64,
    pub result: String,
    pub reviewed_at: i64,
}

/// 插入新的 Review
pub async fn insert_review(
    pool: &Pool<Sqlite>,
    question_id: i64,
    result: &str,
    reviewed_at: i64,
) -> Result<i64, DbError> {
    let db_result = sqlx::query(
        r#"
        INSERT INTO review (question_id, result, reviewed_at)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(question_id)
    .bind(result)
    .bind(reviewed_at)
    .execute(pool)
    .await?;

    Ok(db_result.last_insert_rowid())
}

/// 根据 ID 查找 Review
pub async fn select_review_by_id(
    pool: &Pool<Sqlite>,
    id: i64,
) -> Result<Option<ReviewRow>, DbError> {
    let result = sqlx::query_as::<ReviewRow>(
        r#"
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// 查找某 Question 的所有 Reviews
pub async fn select_reviews_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<Vec<ReviewRow>, DbError> {
    let results = sqlx::query_as::<ReviewRow>(
        r#"
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE question_id = ?1
        ORDER BY reviewed_at DESC
        "#,
    )
    .bind(question_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 查找指定时间范围内的所有 Reviews
pub async fn select_reviews_by_time_range(
    pool: &Pool<Sqlite>,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<ReviewRow>, DbError> {
    let results = sqlx::query_as::<ReviewRow>(
        r#"
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE reviewed_at BETWEEN ?1 AND ?2
        ORDER BY reviewed_at DESC
        "#,
    )
    .bind(start_ts)
    .bind(end_ts)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 查找指定 LogicalDay 范围内的所有 Reviews
pub async fn select_reviews_by_logical_day_range(
    pool: &Pool<Sqlite>,
    start_day: crate::util::time::logical_day::LogicalDay,
    end_day: crate::util::time::logical_day::LogicalDay,
) -> Result<Vec<ReviewRow>, DbError> {
    use crate::util::time::logical_day;

    let (start_ts, end_ts) = logical_day::range_of_day(end_day);
    let (start_day_ts, _) = logical_day::range_of_day(start_day);

    let results = sqlx::query_as::<ReviewRow>(
        r#"
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE reviewed_at >= ?1 AND reviewed_at < ?2
        ORDER BY reviewed_at DESC
        "#,
    )
    .bind(start_day_ts.as_i64())
    .bind(end_ts.as_i64())
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 统计某 Question 的 Review 次数
pub async fn count_reviews_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<i64, DbError> {
    let result = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM review
        WHERE question_id = ?1
        "#,
    )
    .bind(question_id)
    .fetch_one(pool)
    .await?;

    Ok(result.map(|r| r.count.unwrap_or(0)).unwrap_or(0))
}

/// 获取某 Question 的最后一次 Review
pub async fn select_last_review_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<Option<ReviewRow>, DbError> {
    let result = sqlx::query_as::<ReviewRow>(
        r#"
        SELECT id, question_id, result, reviewed_at
        FROM review
        WHERE question_id = ?1
        ORDER BY reviewed_at DESC
        LIMIT 1
        "#,
    )
    .bind(question_id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

// ============ Asset 相关操作 ============

/// 异步 Asset 行结构
#[derive(Debug)]
pub struct AssetRow {
    pub id: i64,
    pub question_id: i64,
    pub type_: String,
    pub path: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}

/// 插入新的 Asset
pub async fn insert_asset(
    pool: &Pool<Sqlite>,
    question_id: i64,
    type_: &str,
    path: &str,
    created_at: i64,
) -> Result<i64, DbError> {
    let result = sqlx::query(
        r#"
        INSERT INTO asset (question_id, type, path, created_at)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(question_id)
    .bind(type_)
    .bind(path)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// 删除 Asset（设置 deleted_at）
pub async fn soft_delete_asset(
    pool: &Pool<Sqlite>,
    asset_id: i64,
    deleted_at: i64,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        UPDATE asset
        SET deleted_at = ?1
        WHERE id = ?2
        "#,
    )
    .bind(deleted_at)
    .bind(asset_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 根据 ID 查找 Asset
pub async fn select_asset_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<AssetRow>, DbError> {
    let result = sqlx::query_as::<AssetRow>(
        r#"
        SELECT id, question_id, type, path, created_at, deleted_at
        FROM asset
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// 查找某 Question 的所有未删除 Assets
pub async fn select_assets_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<Vec<AssetRow>, DbError> {
    let results = sqlx::query_as::<AssetRow>(
        r#"
        SELECT id, question_id, type, path, created_at, deleted_at
        FROM asset
        WHERE question_id = ?1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(question_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 根据 AssetType 查找 Assets
pub async fn select_assets_by_type(
    pool: &Pool<Sqlite>,
    question_id: i64,
    type_: &str,
) -> Result<Vec<AssetRow>, DbError> {
    let results = sqlx::query_as::<AssetRow>(
        r#"
        SELECT id, question_id, type, path, created_at, deleted_at
        FROM asset
        WHERE question_id = ?1 AND type = ?2 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(question_id)
    .bind(type_)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 查找某 Question 的所有 Assets（包含已删除）
pub async fn select_all_assets_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<Vec<AssetRow>, DbError> {
    let results = sqlx::query_as::<AssetRow>(
        r#"
        SELECT id, question_id, type, path, created_at, deleted_at
        FROM asset
        WHERE question_id = ?1
        ORDER BY created_at DESC
        "#,
    )
    .bind(question_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 统计某 Question 的 Asset 数量
pub async fn count_assets_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<i64, DbError> {
    let result = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM asset
        WHERE question_id = ?1 AND deleted_at IS NULL
        "#,
    )
    .bind(question_id)
    .fetch_one(pool)
    .await?;

    Ok(result.map(|r| r.count.unwrap_or(0)).unwrap_or(0))
}

// ============ Meta 相关操作 ============

/// 异步 Meta 行结构
#[derive(Debug)]
pub struct MetaRow {
    pub id: i64,
    pub question_id: i64,
    pub key: String,
    pub value: String,
}

/// 插入新的 Meta
pub async fn insert_meta(
    pool: &Pool<Sqlite>,
    question_id: i64,
    key: &str,
    value: &str,
) -> Result<i64, DbError> {
    let result = sqlx::query(
        r#"
        INSERT INTO meta (question_id, key, value)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(question_id)
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// 删除 Meta
pub async fn delete_meta(pool: &Pool<Sqlite>, meta_id: i64) -> Result<(), DbError> {
    sqlx::query(
        r#"
        DELETE FROM meta
        WHERE id = ?1
        "#,
    )
    .bind(meta_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量删除 Meta（按 question_id）
pub async fn delete_meta_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        DELETE FROM meta
        WHERE question_id = ?1
        "#,
    )
    .bind(question_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 根据 ID 查找 Meta
pub async fn select_meta_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<MetaRow>, DbError> {
    let result = sqlx::query_as::<MetaRow>(
        r#"
        SELECT id, question_id, key, value
        FROM meta
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// 查找某 Question 的所有 Meta
pub async fn select_meta_by_question_id(
    pool: &Pool<Sqlite>,
    question_id: i64,
) -> Result<Vec<MetaRow>, DbError> {
    let results = sqlx::query_as::<MetaRow>(
        r#"
        SELECT id, question_id, key, value
        FROM meta
        WHERE question_id = ?1
        ORDER BY id ASC
        "#,
    )
    .bind(question_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

/// 根据键名查找 Meta
pub async fn select_meta_by_key(
    pool: &Pool<Sqlite>,
    question_id: i64,
    key: &str,
) -> Result<Option<MetaRow>, DbError> {
    let result = sqlx::query_as::<MetaRow>(
        r#"
        SELECT id, question_id, key, value
        FROM meta
        WHERE question_id = ?1 AND key = ?2
        "#,
    )
    .bind(question_id)
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// 更新 Meta 的值
pub async fn update_meta(
    pool: &Pool<Sqlite>,
    meta_id: i64,
    new_value: &str,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        UPDATE meta
        SET value = ?1
        WHERE id = ?2
        "#,
    )
    .bind(new_value)
    .bind(meta_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量插入或更新 Meta
///
/// 对于已存在的 key，更新值；对于不存在的 key，插入新记录
pub async fn upsert_meta_many(
    pool: &Pool<Sqlite>,
    question_id: i64,
    entries: &[(String, String)],
) -> Result<(), DbError> {
    for (key, value) in entries {
        // 先尝试更新
        let update_result = sqlx::query(
            r#"
                UPDATE meta
                SET value = ?1
                WHERE question_id = ?2 AND key = ?3
                "#,
        )
        .bind(value)
        .bind(question_id)
        .bind(key)
        .execute(pool)
        .await?;

        // 如果没有更新任何行，则插入
        if update_result.rows_affected() == 0 {
            sqlx::query(
                r#"
                INSERT INTO meta (question_id, key, value)
                VALUES (?1, ?2, ?3)
                "#,
            )
            .bind(question_id)
            .bind(key)
            .bind(value)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

// 错误转换
impl From<SqlxError> for DbError {
    fn from(err: SqlxError) -> Self {
        DbError::Sqlx(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_db_creation() {
        let pool = sqlx::SqlitePool::connect("sqlite:::memory:").await.unwrap();
        let db = AsyncDb::new(pool);

        assert_eq!(db.pool().as_str(), "sqlite:::memory:");
    }
}
