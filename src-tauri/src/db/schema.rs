use rusqlite::Result;

use crate::db::Connection;

// question表
#[derive(Debug, Clone)]
pub struct QuestionRow {
    pub id: i64,
    pub name: Option<String>,
    pub state: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}

/* 增加一条记录 */
pub fn insert_question(
    conn: &Connection,
    name: Option<&str>,
    state: &str,
    created_at: i64,
) -> Result<i64> {
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

/* 修改题目名 */
pub fn update_question_name(
    conn: &Connection,
    question_id: i64,
    new_name: Option<&str>,
) -> Result<()> {
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

/* 修改题目删除日期 */
pub fn update_question_deleted_at(
    conn: &Connection,
    question_id: i64,
    new_deleted_at: Option<i64>,
) -> Result<()> {
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

/* 用ID查找题目 */
pub fn get_question_by_id(conn: &Connection, id: i64) -> Result<Option<QuestionRow>> {
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

pub fn get_question_by_name(conn: &Connection, name: &str) -> Result<Option<QuestionRow>> {
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

pub fn list_questions(conn: &Connection) -> Result<Vec<QuestionRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, state, created_at, deleted_at
        FROM question
        "#,
    )?;

    let question_iter = stmt.query_map((), |row| {
        Ok(QuestionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            created_at: row.get(3)?,
            deleted_at: row.get(4)?,
        })
    })?;

    let mut questions = Vec::new();
    for question in question_iter {
        questions.push(question?);
    }

    Ok(questions)
}

// review 不可删，不可改
#[derive(Debug, Clone)]
struct ReviewRow {
    pub id: i64,
    pub question_id: i64,
    pub content: String,
    pub created_at: i64,
}

/* 增加一条记录 */
pub fn insert_review(
    conn: &Connection,
    question_id: i64,
    content: &str,
    created_at: i64,
) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO review (question_id, content, created_at)
        VALUES (?1, ?2, ?3)
        "#,
        (question_id, content, created_at),
    )?;

    Ok(conn.last_insert_rowid())
}

/* 用ID查找复习记录 */
//pub fn get_review_by_id(conn: &Connection, id: i64) -> Result<Option<ReviewRow>> {}

/* 查找某题目的所有复习记录 */
//pub fn list_reviews_by_question_id(conn: &Connection, question_id: i64) -> Result<Vec<ReviewRow>> {}

// asset 以删代改
#[derive(Debug, Clone)]
struct AssetRow {
    pub id: i64,
    pub question_id: i64,
    pub type_: String,
    pub path: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}
/* 增加一条记录 */
/* 删除一条记录 */
/* 用ID查找资源 */
/* 查找某题目的所有资源 */

// meta
#[derive(Debug, Clone)]
struct MetaRow {
    pub id: i64,
    pub question_id: i64,
    pub key: String,
    pub value: String,
}
/* 增加一条记录 */
/* 删除一条记录 */
/* 用ID查找元信息 */
/* 查找某题目的所有元信息 */
