use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "app_config.json";

#[derive(Serialize, Deserialize)]
struct AppConfig {
    root: String,
}

/// 返回 config 文件路径
fn config_path() -> PathBuf {
    let mut dir = dirs::config_dir().expect("无法获取配置目录");

    dir.push("ePNote");

    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }

    dir.join("app_config.json")
}

/// 保存 root 路径
pub fn save_root(root: &Path) -> Result<(), String> {
    let config = AppConfig {
        root: root.to_string_lossy().to_string(),
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

    let mut file = fs::File::create(config_path()).map_err(|e| e.to_string())?;

    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

    println!("CONFIG PATH = {:?}", config_path());

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
