use super::asset_schema::*;
use super::meta_schema::*;
use super::migrate;
use super::question_schema::*;
use super::review_schema::*;
use super::Connection;
use chrono::Utc;

fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();

    migrate(&mut conn).unwrap();

    conn
}

#[test]
fn test_migrate_idempotent() {
    let mut conn = Connection::open_in_memory().unwrap();

    migrate(&mut conn).unwrap();
    migrate(&mut conn).unwrap();
    migrate(&mut conn).unwrap();
}

#[test]
fn test_insert_and_select_question() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let name = Some("测试题目");
    let state = "active";
    let id = insert_question(&conn, name, state, now).unwrap();
    assert!(id > 0);
    let row = select_question_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(row.name.as_deref(), name);
    assert_eq!(row.state, state);
    assert_eq!(row.created_at, now);
    assert!(row.deleted_at.is_none());
}

#[test]
fn test_update_question_name() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let id = insert_question(&conn, Some("old name"), "active", now).unwrap();
    update_question_name(&conn, id, Some("new name")).unwrap();
    let row = select_question_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(row.name.as_deref(), Some("new name"));
}

#[test]
fn test_update_question_deleted_at() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let id = insert_question(&conn, Some("题目"), "active", now).unwrap();
    let del_time = now + 1000;
    update_question_deleted_at(&conn, id, Some(del_time)).unwrap();
    let row = select_question_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(row.deleted_at, Some(del_time));
}

#[test]
fn test_select_question_by_name() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let name = "唯一题目";
    let id = insert_question(&conn, Some(name), "active", now).unwrap();
    let row = select_question_by_name(&conn, name).unwrap().unwrap();
    assert_eq!(row.id, id);
}

#[test]
fn test_select_questions_page() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    for i in 0..5 {
        let name = format!("题目{}", i);
        insert_question(&conn, Some(&name), "active", now + i).unwrap();
    }
    let page = select_questions_page(&conn, 3, 0).unwrap();
    assert_eq!(page.len(), 3);
    // 检查排序
    assert!(page[0].created_at > page[1].created_at);
}

#[test]
fn test_insert_and_select_review() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    // 先插入一个题目
    let qid = insert_question(&conn, Some("题目A"), "active", now).unwrap();
    // 插入review
    let result = "correct";
    let rid = insert_review(&conn, qid, result, now).unwrap();
    assert!(rid > 0);
    let row = select_review_by_id(&conn, rid).unwrap().unwrap();
    assert_eq!(row.question_id, qid);
    assert_eq!(row.result, result);
    assert_eq!(row.reviewed_at, now);
}

#[test]
fn test_select_reviews_by_question_id() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目B"), "active", now).unwrap();
    for i in 0..3 {
        let result = format!("result{}", i);
        insert_review(&conn, qid, &result, now + i).unwrap();
    }
    let reviews = select_reviews_by_question_id(&conn, qid).unwrap();
    assert_eq!(reviews.len(), 3);
    assert!(reviews.iter().any(|r| r.result == "result0"));
}

#[test]
fn test_select_reviews_by_time_range() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目C"), "active", now).unwrap();
    let r1 = insert_review(&conn, qid, "A", now).unwrap();
    let r2 = insert_review(&conn, qid, "B", now + 10).unwrap();
    let r3 = insert_review(&conn, qid, "C", now + 20).unwrap();
    let reviews = select_reviews_by_time_range(&conn, now + 5, now + 15).unwrap();
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews[0].id, r2);
}

#[test]
fn test_insert_and_select_asset() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目A"), "active", now).unwrap();
    let type_ = "image";
    let path = "/tmp/test.png";
    let aid = insert_asset(&conn, qid, type_, path, now).unwrap();
    assert!(aid > 0);
    let asset = select_asset_by_id(&conn, aid).unwrap().unwrap();
    assert_eq!(asset.question_id, qid);
    assert_eq!(asset.type_, type_);
    assert_eq!(asset.path, path);
    assert_eq!(asset.created_at, now);
    assert!(asset.deleted_at.is_none());
}

#[test]
fn test_delete_asset() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目B"), "active", now).unwrap();
    let aid = insert_asset(&conn, qid, "image", "/tmp/test2.png", now).unwrap();
    let del_time = now + 1000;
    delete_asset(&conn, aid, del_time).unwrap();
    let asset = select_asset_by_id(&conn, aid).unwrap();
    assert!(asset.is_none());
}

#[test]
fn test_select_asset_by_question() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目C"), "active", now).unwrap();
    for i in 0..3 {
        let path = format!("/tmp/asset{}.png", i);
        insert_asset(&conn, qid, "image", &path, now + i).unwrap();
    }
    let assets = select_asset_by_question(&conn, qid).unwrap();
    assert_eq!(assets.len(), 3);
    assert!(assets.iter().any(|a| a.path.ends_with("asset0.png")));
}

#[test]
fn test_insert_and_select_meta() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目D"), "active", now).unwrap();
    let key = "author";
    let value = "张三";
    let mid = insert_meta(&conn, qid, key, value).unwrap();
    assert!(mid > 0);
    let meta = select_meta_by_id(&conn, mid).unwrap().unwrap();
    assert_eq!(meta.question_id, qid);
    assert_eq!(meta.key, key);
    assert_eq!(meta.value, value);
}

#[test]
fn test_delete_meta() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目E"), "active", now).unwrap();
    let mid = insert_meta(&conn, qid, "tag", "test").unwrap();
    delete_meta(&conn, mid).unwrap();
    let meta = select_meta_by_id(&conn, mid).unwrap();
    assert!(meta.is_none());
}

#[test]
fn test_select_meta_by_question() {
    let conn = setup_test_db();
    let now = Utc::now().timestamp();
    let qid = insert_question(&conn, Some("题目F"), "active", now).unwrap();
    for i in 0..2 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        insert_meta(&conn, qid, &key, &value).unwrap();
    }
    let metas = select_meta_by_question(&conn, qid).unwrap();
    assert_eq!(metas.len(), 2);
    assert!(metas.iter().any(|m| m.key == "key0"));
}
