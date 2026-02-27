use crate::app::config as app_config;
use crate::app::instance::{init_dataroot, load_instance, validate_instance};
use crate::app::types::*;
use crate::app::{self, error::*};
use crate::asset::store::AssetStore;
use crate::db::migrate;
//use crate::db::migrate::migrate;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn init_note(root: PathBuf) -> Result<(), InitError> {
    // 初始化数据根目录，执行instance校验，获取上下文
    let ctx = init_dataroot(root.clone()).expect("初始化数据目录失败");

    // 初始化 AssetStore，使用数据根目录作为存储根（AssetStore 在内部使用相对路径：assets/ 和 garbages/）
    let _asset_store = AssetStore::new(ctx.root.clone());

    // 打开数据库
    let conn = rusqlite::Connection::open(&ctx.db_path).expect("无法打开数据库");

    // 执行迁移
    let mut conn = conn;
    migrate(&mut conn).expect("数据库迁移失败");

    // 将 root 写入到程序根目录下的 app_config.json，便于下次启动直接读取
    match app_config::save_root(&root) {
        Ok(_) => {}
        Err(_) => return Err(InitError::InstanceError),
    }

    Ok(())
}

pub fn tauri_init_note(root: String) -> Result<(), String> {
    let path = PathBuf::from(root);
    match init_note(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 检查给定 root 是否已完成初始化（不创建或修改任何文件）
pub fn check_init(root: PathBuf) -> Result<bool, InitError> {
    // 检查根目录是否存在
    if !root.exists() {
        return Ok(false);
    }

    // 检查 .instance.json 是否存在并合法
    let instance_file = root.join(".instance.json");
    if !instance_file.exists() {
        return Ok(false);
    }

    // 尝试读取并验证
    let instance = load_instance(&instance_file)?;
    validate_instance(&instance)?;

    // 简单检查 DB 文件是否存在
    let db_file = root.join("db.sqlite");
    if !db_file.exists() {
        return Ok(false);
    }

    Ok(true)
}

pub fn tauri_check_init_default() -> Result<crate::app::types::InitStatus, String> {
    let root = app_config::load_root()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| PathBuf::from("data"));

    match check_init(root.clone()) {
        Ok(true) => Ok(crate::app::types::InitStatus {
            initialized: true,
            root: Some(root.to_string_lossy().to_string()),
        }),
        Ok(false) => Ok(crate::app::types::InitStatus {
            initialized: false,
            root: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}
