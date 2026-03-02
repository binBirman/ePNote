use tauri::App;

use crate::app::appstate::{AppInner, AppState};
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

pub fn init_note(root: PathBuf) -> Result<AppInner, InitError> {
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

    match app_config::save_root(&ctx.root) {
        Ok(_) => {}
        Err(_) => return Err(InitError::InstanceError),
    }

    Ok(AppInner {
        db: conn,
        asset_store: _asset_store,
    })
}

pub fn tauri_init_note(root: String) -> Result<(), String> {
    println!("开始初始化: {:?}", root);
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
        println!("init_dataroot ok");
        return Ok(false);
    }

    // 检查 .instance.json 是否存在并合法
    let instance_file = root.join(".instance.json");
    if !instance_file.exists() {
        print!("instance file exist");
        return Ok(false);
    }

    // 尝试读取并验证
    let instance = load_instance(&instance_file)?;
    validate_instance(&instance)?;
    print!("validate instance ok");

    // 简单检查 DB 文件是否存在
    let db_file = root.join("db.sqlite");
    if !db_file.exists() {
        print!("db file exist");
        return Ok(false);
    }

    Ok(true)
}

pub fn tauri_check_init_default() -> Result<crate::app::types::InitStatus, String> {
    let root = app_config::load_root()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| PathBuf::from("data"));
    // 为了避免前端因为后端校验错误而被阻断，
    // 我们在出现检查错误时也返回 InitStatus.initialized = false，
    // 并把配置中读取到的 root 返回给前端用于预填与诊断，同时记录日志。
    match check_init(root.clone()) {
        Ok(true) => Ok(crate::app::types::InitStatus {
            initialized: true,
            root: Some(root.to_string_lossy().to_string()),
        }),
        Ok(false) => {
            // 未初始化但配置文件中存在 root，返回给前端以便预填
            println!(
                "tauri_check_init_default: root='{}' initialized=false (not initialized)",
                root.to_string_lossy()
            );
            Ok(crate::app::types::InitStatus {
                initialized: false,
                root: Some(root.to_string_lossy().to_string()),
            })
        }
        Err(e) => {
            // 出现检查错误（例如 .instance.json 解析失败或 validate 失败），记录并返回 initialized=false
            eprintln!("tauri_check_init_default: error checking init: {}", e);
            Ok(crate::app::types::InitStatus {
                initialized: false,
                root: Some(root.to_string_lossy().to_string()),
            })
        }
    }
}
