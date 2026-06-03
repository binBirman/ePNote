//! 设置命令层
//!
//! 提供设置项的读取、保存以及数据目录管理命令。

use crate::app::config::{self, AppSettings};
use serde::{Deserialize, Serialize};

/// 获取完整配置（返回给前端的格式）
#[derive(Serialize, Deserialize)]
pub struct AppConfigResponse {
    pub root: String,
    pub settings: AppSettings,
}

/// 获取完整配置（root 路径 + 设置项）
#[tauri::command]
pub fn get_app_settings_comm() -> Result<AppConfigResponse, String> {
    let (root, settings) = config::get_config()?;
    Ok(AppConfigResponse { root, settings })
}

/// 保存设置项（仅更新 settings，不影响 root）
#[tauri::command]
pub fn save_app_settings_comm(settings: AppSettings) -> Result<(), String> {
    config::save_settings(&settings)
}

/// 打开数据目录（调用操作系统文件管理器）
#[tauri::command]
pub fn open_data_directory_comm() -> Result<(), String> {
    let (root, _) = config::get_config()?;

    if root.is_empty() {
        return Err("数据目录未设置".to_string());
    }

    let path = std::path::Path::new(&root);
    if !path.exists() {
        return Err(format!("数据目录不存在: {}", root));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开数据目录失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开数据目录失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开数据目录失败: {}", e))?;
    }

    Ok(())
}
