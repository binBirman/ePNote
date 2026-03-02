use crate::app;
use std::path::PathBuf;

// #[tauri::command]
// pub fn tauri_init_note(root: String) -> Result<(), String> {
//     app::tauri_init_note(root)
// }

#[tauri::command]
pub fn tauri_check_init_default() -> Result<app::InitStatus, String> {
    match app::tauri_check_init_default() {
        Ok(s) => Ok(s),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn tauri_init_note(state: tauri::State<app::AppState>, root: String) -> Result<(), String> {
    let root_path = PathBuf::from(root);

    let inner = app::init_note(root_path).map_err(|e| e.to_string())?;

    let mut guard = state.inner.lock().unwrap();
    *guard = Some(inner);

    Ok(())
}
