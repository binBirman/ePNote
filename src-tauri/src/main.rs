// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod util;

use std::path::PathBuf;

fn main() {
    let data_root = PathBuf::from("./DataRoot");

    std::fs::create_dir_all(&data_root).unwrap();

    let conn = db::init_db(&data_root).unwrap();

    println!("DB initialized: {:?}", conn);

    let q1: Option<db::schema::QuestionRow> = db::schema::get_question_by_id(&conn, 1).unwrap();
    let q2: Option<db::schema::QuestionRow> =
        db::schema::get_question_by_name(&conn, "Example Question2").unwrap();

    //输出查询结果
    let q1c = util::ts_to_utc_datetime(q1.clone().unwrap().created_at);
    let q1d = util::ts_to_utc_datetime(q1.clone().unwrap().deleted_at.unwrap());

    let q2c = util::ts_to_utc_datetime(q2.clone().unwrap().created_at);

    println!(
        "Question 1:, created at: {:?}, deleted at: {:?}",
        q1c.clone(),
        q1d.clone()
    );
    println!("Question 2:, created at: {:?}", q2c.clone());

    println!(
        "Q1 created at {:?} days ago, deleted at: {:?} days ago.",
        util::time_util::days_from_now(q1.clone().unwrap().created_at),
        util::time_util::days_from_now(q1.clone().unwrap().deleted_at.unwrap())
    );
    println!(
        "Q2 created at {:?} days ago.",
        util::time_util::days_from_now(q2.clone().unwrap().created_at)
    );
}
