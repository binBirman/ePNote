use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "app_config.json";

/// 科目配置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubjectConfig {
    /// 是否归档（不参与推荐）
    #[serde(default)]
    pub archived: bool,
    /// 该科每日推荐题数限制，None=使用全局值，Some(0)=不推荐
    #[serde(default)]
    pub recommendation_limit: Option<u32>,
}

impl Default for SubjectConfig {
    fn default() -> Self {
        Self { archived: false, recommendation_limit: None }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    /// 默认复习题数
    #[serde(default = "default_review_limit")]
    pub default_review_limit: u32,
    /// 每科每日推荐默认题数
    #[serde(default = "default_per_subject_daily_limit")]
    pub per_subject_daily_limit: u32,
    /// 新题推荐比例（预留）
    #[serde(default = "default_new_question_ratio")]
    pub new_question_ratio: f64,
    /// 推荐随机性系数
    #[serde(default = "default_recommendation_randomness")]
    pub recommendation_randomness: f64,
    /// 显示推荐调试信息（开发用）
    #[serde(default = "default_show_debug_info")]
    pub show_debug_info: bool,
    /// 科目配置映射
    #[serde(default)]
    pub subjects: HashMap<String, SubjectConfig>,
}

fn default_review_limit() -> u32 { 10 }
fn default_per_subject_daily_limit() -> u32 { 10 }
fn default_new_question_ratio() -> f64 { 0.3 }
fn default_recommendation_randomness() -> f64 { 1.0 }
fn default_show_debug_info() -> bool { false }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_review_limit: default_review_limit(),
            per_subject_daily_limit: default_per_subject_daily_limit(),
            new_question_ratio: default_new_question_ratio(),
            recommendation_randomness: default_recommendation_randomness(),
            show_debug_info: default_show_debug_info(),
            subjects: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    root: String,
    #[serde(default)]
    settings: AppSettings,
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

/// 读取完整配置（不存在则返回 None）
fn load_config() -> Result<Option<AppConfig>, String> {
    let path = config_path();

    if !path.exists() {
        return Ok(None);
    }

    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| e.to_string())?;

    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(Some(config))
}

/// 保存完整配置
fn save_config(config: &AppConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;

    let mut file = fs::File::create(config_path()).map_err(|e| e.to_string())?;

    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

/// 保存 root 路径（合并写入，不覆盖 settings）
pub fn save_root(root: &Path) -> Result<(), String> {
    let mut config = load_config()?.unwrap_or(AppConfig {
        root: String::new(),
        settings: AppSettings::default(),
    });
    config.root = root.to_string_lossy().to_string();
    save_config(&config)?;

    println!("CONFIG PATH = {:?}", config_path());

    Ok(())
}

/// 读取 root 路径
pub fn load_root() -> Result<Option<PathBuf>, String> {
    match load_config() {
        Ok(Some(config)) => Ok(Some(PathBuf::from(config.root))),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 读取设置（不存在配置文件或缺少设置字段时返回默认值）
pub fn load_settings() -> AppSettings {
    match load_config() {
        Ok(Some(config)) => config.settings,
        Ok(None) => AppSettings::default(),
        Err(_) => AppSettings::default(),
    }
}

/// 保存设置（合并写入，不覆盖 root）
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let mut config = load_config()?.unwrap_or(AppConfig {
        root: String::new(),
        settings: AppSettings::default(),
    });
    config.settings = settings.clone();
    save_config(&config)
}

/// 获取完整配置（root + settings），用于命令层返回给前端
pub fn get_config() -> Result<(String, AppSettings), String> {
    let config = load_config()?.unwrap_or(AppConfig {
        root: String::new(),
        settings: AppSettings::default(),
    });
    Ok((config.root, config.settings))
}

/// 获取 AppSettings 的默认值
pub fn default_settings() -> AppSettings {
    AppSettings::default()
}
