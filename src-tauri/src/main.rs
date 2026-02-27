// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ePNote::app::config as app_config;
use ePNote::app::AppState;
use ePNote::asset::store::AssetStore;
use ePNote::command::*;
use ePNote::dao::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::Mutex;
use tauri_plugin_dialog::DialogExt;

fn main() {
    // 准备资源（容错：读取失败时不崩溃，使用默认目录）
    let root = match app_config::load_root() {
        Ok(Some(p)) => {
            println!("使用配置中读取到的数据根路径: {:?}", p);
            p
        }
        Ok(None) => {
            println!("未在配置中读到数据路径，使用默认 'data' 目录");
            PathBuf::from("data")
        }
        Err(e) => {
            eprintln!("读取配置路径失败: {}，使用默认 'data' 目录", e);
            PathBuf::from("data")
        }
    };

    let conn = Connection::open(root.join("db.sqlite")).unwrap();
    let asset_store = AssetStore::new(root);

    let state = AppState::new(conn, asset_store);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            init_c::tauri_init_note,
            init_c::tauri_check_init_default,
            show_list_c::show_list_available_questions_page,
            show_list_c::show_list_deleted_questions_page,
            show_list_c::show_list_available_questions_by_state_page,
            show_list_c::show_list_available_questions_by_subject_page,
            question_c::create_question_comm,
            question_c::delete_question_comm,
            question_c::restore_question_comm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
