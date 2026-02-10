// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod util;

use ePNote::app::*;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from("D:\\chb\\APPData\\ePNote\\DataRoot"); // 暂时固定路径 ，后续可改为配置项或命令行参数

    // 初始化数据根目录，执行instance校验，获取上下文
    let ctx = init_dataroot(root.clone()).expect("初始化数据目录失败");

    // 打开数据库
    let conn = rusqlite::Connection::open(&ctx.db_path).expect("无法打开数据库");

    // 执行迁移
    let mut conn = conn;
    ePNote::db::migrate::migrate(&mut conn).expect("数据库迁移失败");
}
