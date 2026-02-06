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

/* 删除一条记录 */
// pub fn delete_question(conn: &Connection, question_id: i64) -> Result<()> {
//     conn.execute(
//         r#"
//         DELETE FROM question
//         WHERE id = ?1
//         "#,
//         (question_id,),
//     )?;
//     Ok(())
// }

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
        SELECT id, name, state, created_at, deleted_at
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
        SELECT id, name, state, created_at, deleted_at
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
pub fn select_questions_page(
    conn: &Connection,
    limit: i64,
    offset: i64,
) -> Result<Vec<QuestionRow>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at
        FROM question
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;

    let iter = stmt.query_map([limit, offset], |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
        })
    })?;

    iter.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/*
    列出所有未删除题目（直接搜索全库，慎用！慎用！）
    输入：
        无
    输出：
        返回未删除题目列表
*/
// pub fn select_questions_active(conn: &Connection) -> Result<Vec<QuestionRow>, DbError> {
//     let mut stmt = conn.prepare(
//         r#"
//         SELECT id, name, state, created_at, deleted_at
//         FROM question
//         WHERE deleted_at IS NULL
//         ORDER BY created_at DESC
//         "#,
//     )?;

//     let question_iter = stmt.query_map([], |row| {
//         Ok(QuestionRow {
//             id: row.get(0)?,
//             name: row.get(1)?,
//             state: row.get(2)?,
//             created_at: row.get(3)?,
//             deleted_at: row.get(4)?,
//         })
//     })?;

//     question_iter
//         .collect::<Result<Vec<_>, _>>()
//         .map_err(Into::into)
// }
