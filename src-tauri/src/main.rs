// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ePNote::app as applib;

#[tauri::command]
fn tauri_init_note(root: String) -> Result<(), String> {
    applib::tauri_init_note(root)
}

#[tauri::command]
fn tauri_check_init_default() -> Result<applib::InitStatus, String> {
    match applib::tauri_check_init_default() {
        Ok(s) => Ok(s),
        Err(e) => Err(e),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            tauri_init_note,
            tauri_check_init_default
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
