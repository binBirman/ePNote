use crate::app::error::*;
use crate::app::instance::init_dataroot;
use crate::app::types::*;
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

    Ok(())
}
