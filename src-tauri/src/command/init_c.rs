use crate::app;

#[tauri::command]
pub fn tauri_init_note(root: String) -> Result<(), String> {
    app::tauri_init_note(root)
}

#[tauri::command]
pub fn tauri_check_init_default() -> Result<app::InitStatus, String> {
    match app::tauri_check_init_default() {
        Ok(s) => Ok(s),
        Err(e) => Err(e),
    }
}
