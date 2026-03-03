// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ePNote::app::appstate::{AppInner, AppState};
use ePNote::app::config as app_config;
use ePNote::app::init::{check_init, init_note};
use ePNote::asset::store::AssetStore;
use ePNote::command::*;
use ePNote::dao::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::Mutex;
use tauri_plugin_dialog::DialogExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .setup(|app| {
            use tauri::Manager;

            let state: tauri::State<AppState> = app.state();

            // 启动时尝试加载
            if let Ok(Some(root)) = app_config::load_root() {
                if let Ok(true) = check_init(root.clone()) {
                    println!("启动时自动加载 AppInner");

                    let conn = ePNote::db::init_db(&root).expect("open db failed");

                    let asset_store = AssetStore::new(root.clone());

                    let mut guard = state.inner.lock().unwrap();
                    *guard = Some(AppInner {
                        db: conn,
                        asset_store,
                    });
                }
            }

            print!("当前 DB 路径 = {}\n", state.get_db_path());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_c::tauri_init_note,
            init_c::tauri_check_init_default,
            show_list_c::show_list_available_questions_page,
            show_list_c::show_questions_with_filters,
            show_list_c::show_list_deleted_questions_page,
            show_list_c::show_list_available_questions_by_state_page,
            show_list_c::show_list_available_questions_by_subject_page,
            show_list_c::show_list_available_questions_by_subject_and_state_page,
            show_list_c::show_subjects,
            show_list_c::show_states,
            question_c::create_question_comm,
            question_c::delete_question_comm,
            question_c::restore_question_comm,
            question_c::permanently_delete_question_comm,
            question_c::cleanup_recycle_bin_comm,
            question_c::update_question_comm,
            question_c::get_question_detail_comm,
            question_c::get_image_base64,
            question_c::add_question_images_comm,
            question_c::delete_question_image_comm,
            // 复习相关命令
            review_c::process_review_comm,
            review_c::recommend_questions_comm,
            review_c::suspend_question_comm,
            review_c::recover_question_comm,
            review_c::list_subjects_comm,
            review_c::get_stats_comm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
