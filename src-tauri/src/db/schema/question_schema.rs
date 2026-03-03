use rusqlite::Result;

use crate::db::error::DbError;
use crate::db::Connection;

#[derive(Debug, Clone)]
pub struct QuestionRow {
    pub id: i64,
    pub name: Option<String>,
    pub state: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
    // 复习相关字段
    pub last_review_at: Option<i64>,
    pub last_result: Option<String>,
    pub correct_streak: i64,
    pub wrong_count: i64,
    pub due_at: Option<i64>,
}

/*
    增加一条题目记录
    输入：
        name: 题目名称，可选
        state: 题目状态，必填
        created_at: 创建时间戳，必填
    输出：
        若插入成功，返回新增记录的ID
        若插入失败，返回错误信息
*/
pub fn insert_question(
    conn: &Connection,
    name: Option<&str>,
    state: &str,
    created_at: i64,
) -> Result<i64, DbError> {
    conn.execute(
        r#"
        INSERT INTO question (name, state, created_at)
        VALUES (?1, ?2, ?3)
        "#,
        (name, state, created_at),
    )?;
    Ok(conn.last_insert_rowid())
}

/*
    修改题目名
    输入：
        question_id: 题目ID，必填
        new_name: 新题目名称，可选
    输出：
        若修改成功，返回空值
*/
pub fn update_question_name(
    conn: &Connection,
    question_id: i64,
    new_name: Option<&str>,
) -> Result<(), DbError> {
    conn.execute(
        r#"
        UPDATE question
        SET name = ?1
        WHERE id = ?2
        "#,
        (new_name, question_id),
    )?;
    Ok(())
}

/*
    修改题删除日期
    输入：
        question_id: 题目ID，必填
        new_deleted_at: 新题目删除日期，可选，可为空
    输出：
        若修改成功，返回空值
*/
pub fn update_question_deleted_at(
    conn: &Connection,
    question_id: i64,
    new_deleted_at: Option<i64>,
) -> Result<(), DbError> {
    conn.execute(
        r#"
        UPDATE question
        SET deleted_at = ?1
        WHERE id = ?2
        "#,
        (new_deleted_at, question_id),
    )?;
    Ok(())
}

/*
    用ID查找题目
    输入：
        id: 题目ID，必填
    输出：
        若找到，返回Some(QuestionRow)
*/
pub fn select_question_by_id(conn: &Connection, id: i64) -> Result<Option<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE id = ?1
        "#,
    )?;

    let question_iter = stmt.query_map((id,), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    for question in question_iter {
        return Ok(Some(question?));
    }

    Ok(None)
}

/*
    用名称查找题目
    输入：
        name: 题目名称，必填
    输出：
        若找到，返回Some(QuestionRow)
*/
pub fn select_question_by_name(
    conn: &Connection,
    name: &str,
) -> Result<Option<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE name = ?1
        "#,
    )?;

    let question_iter = stmt.query_map((name,), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    for question in question_iter {
        return Ok(Some(question?));
    }

    Ok(None)
}

/*
    分页列出未删除题目
    输入：
        limit: 每页记录数
        offset: 偏移量
    输出：
        返回未删除题目列表
*/
/// 参数顺序为 `(offset, limit)` 以配合上层 DAO 的调用习惯；内部按 SQL 需要传入 `(limit, offset)`。
pub fn select_questions_page(
    conn: &Connection,
    offset: i64,
    limit: i64,
) -> Result<Vec<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map((limit, offset), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    列出时间早于某时间戳的已删除题目
    输入：
        时间戳
    输出：
        返回已删除题目列表
*/
pub fn select_deleted_questions_before_timestamp(
    conn: &Connection,
    timestamp: i64,
) -> Result<Vec<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE deleted_at IS NOT NULL AND deleted_at < ?1
        ORDER BY deleted_at DESC
        "#,
    )?;

    let iter = stmt.query_map((timestamp,), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    更新题目复习相关字段（状态转移后调用）
    输入：
        question_id: 题目ID
        last_review_at: 上次复习时间
        last_result: 上次复习结果
        correct_streak: 连续正确次数
        wrong_count: 错误次数
        due_at: 下次到期时间
    输出：
        若更新成功，返回空值
*/
pub fn update_question_review_fields(
    conn: &Connection,
    question_id: i64,
    last_review_at: Option<i64>,
    last_result: Option<&str>,
    correct_streak: i64,
    wrong_count: i64,
    due_at: Option<i64>,
) -> Result<(), DbError> {
    conn.execute(
        r#"
        UPDATE question
        SET last_review_at = ?1,
            last_result = ?2,
            correct_streak = ?3,
            wrong_count = ?4,
            due_at = ?5
        WHERE id = ?6
        "#,
        (last_review_at, last_result, correct_streak, wrong_count, due_at, question_id),
    )?;
    Ok(())
}

/*
    更新题目状态
    输入：
        question_id: 题目ID
        state: 新状态
    输出：
        若更新成功，返回空值
*/
pub fn update_question_state(
    conn: &Connection,
    question_id: i64,
    state: &str,
) -> Result<(), DbError> {
    conn.execute(
        r#"
        UPDATE question
        SET state = ?1
        WHERE id = ?2
        "#,
        (state, question_id),
    )?;
    Ok(())
}

/*
    查询指定状态的题目列表
    输入：
        state: 题目状态
    输出：
        返回题目列表
*/
pub fn select_questions_by_state(
    conn: &Connection,
    state: &str,
) -> Result<Vec<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE state = ?1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )?;

    let iter = stmt.query_map((state,), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    查询指定复习结果的题目列表
    输入：
        result: 上次复习结果 (WRONG / FUZZY / CORRECT)
    输出：
        返回题目列表
*/
pub fn select_questions_by_last_result(
    conn: &Connection,
    result: &str,
) -> Result<Vec<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE last_result = ?1 AND deleted_at IS NULL
        ORDER BY last_review_at ASC
        "#,
    )?;

    let iter = stmt.query_map((result,), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    查询已到期的题目列表
    输入：
        now: 当前时间戳
    输出：
        返回已到期题目列表
*/
pub fn select_due_questions(
    conn: &Connection,
    now: i64,
) -> Result<Vec<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE due_at IS NOT NULL AND due_at <= ?1
              AND deleted_at IS NULL
              AND state != 'SUSPENDED'
        ORDER BY due_at ASC
        "#,
    )?;

    let iter = stmt.query_map((now,), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    查询长期未复习的题目
    输入：
        days_threshold: 天数阈值
    输出：
        返回长期未复习的题目列表
*/
pub fn select_stale_questions(
    conn: &Connection,
    days_threshold: i64,
) -> Result<Vec<QuestionRow>, DbError> {
    let threshold = days_threshold * 24 * 60 * 60;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at,
               last_review_at, last_result, correct_streak, wrong_count, due_at
        FROM question
        WHERE last_review_at IS NOT NULL
              AND last_review_at < ?1
              AND deleted_at IS NULL
              AND state != 'SUSPENDED'
              AND state != 'NEW'
        ORDER BY last_review_at ASC
        "#,
    )?;

    let iter = stmt.query_map((threshold,), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
            last_review_at: row.get(5)?,
            last_result: row.get(6)?,
            correct_streak: row.get(7)?,
            wrong_count: row.get(8)?,
            due_at: row.get(9)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
