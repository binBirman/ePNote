use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "app_config.json";

#[derive(Serialize, Deserialize)]
struct AppConfig {
    root: String,
}

/// 返回 config 文件路径（当前程序目录下）
fn config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE)
}

/// 保存 root 路径
pub fn save_root(root: &Path) -> Result<(), String> {
    let config = AppConfig {
        root: root.to_string_lossy().to_string(),
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

    let mut file = fs::File::create(config_path()).map_err(|e| e.to_string())?;

    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

/// 读取 root 路径
pub fn load_root() -> Result<Option<PathBuf>, String> {
    let path = config_path();

    if !path.exists() {
        return Ok(None);
    }

    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| e.to_string())?;

    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(Some(PathBuf::from(config.root)))
}
