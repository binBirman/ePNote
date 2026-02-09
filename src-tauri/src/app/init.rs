use crate::app::error::*;
use crate::app::types::*;
use crate::db::migrate;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn ensure_dir(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn init_dataroot(root: PathBuf) -> Result<DataRootContext, InitError> {
    // 1 创建根目录
    ensure_dir(&root)?;

    // 2 定义结构
    let assets_dir = root.join("assets");
    let trash_dir = root.join("trash");
    let exports_dir = root.join("exports");
    let backups_dir = root.join("backups");

    let question_dir = assets_dir.join("question");
    let answer_dir = assets_dir.join("answer");
    let explain_dir = assets_dir.join("explain");
    let other_dir = assets_dir.join("other");

    // 3 创建目录
    ensure_dir(&assets_dir)?;
    ensure_dir(&trash_dir)?;
    ensure_dir(&exports_dir)?;
    ensure_dir(&backups_dir)?;

    ensure_dir(&question_dir)?;
    ensure_dir(&answer_dir)?;
    ensure_dir(&explain_dir)?;
    ensure_dir(&other_dir)?;

    // 4 instance.json
    init_instance(&root)?;

    // 5 DB 路径
    let db_path = root.join("db.sqlite");

    Ok(DataRootContext {
        root,
        assets_dir,
        trash_dir,
        exports_dir,
        backups_dir,
        db_path,
    })
}

fn init_instance(root: &Path) -> Result<(), InitError> {
    let instance_file = root.join(".instance.json");

    if instance_file.exists() {
        // TODO: 读取 + 校验
        return Ok(());
    }

    let instance = InstanceFile::default();

    let json = serde_json::to_string_pretty(&instance).map_err(|_| InitError::InstanceError)?;

    std::fs::write(instance_file, json)?;

    Ok(())
}

pub fn run_migrate() {}
