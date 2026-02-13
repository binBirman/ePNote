use crate::asset::{AssetPath, AssetStore};
use crate::domain::*;
use crate::util::time::*;
use chrono::TimeZone;
use chrono::Utc;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use uuid::Uuid;

fn create_test_file(dir: &Path, name: &str) -> PathBuf {
    let file_path = dir.join(name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test content").unwrap();
    file_path
}

fn logical_day_from_ymd(y: i32, m: u32, d: u32) -> crate::util::time::LogicalDay {
    let naive = chrono::NaiveDate::from_ymd_opt(y, m, d)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
    let dt = offset.from_local_datetime(&naive).unwrap();
    let ts = Timestamp::from(dt.timestamp());
    from_timestamp(ts)
}

#[test]
fn test_extract_extension() {
    assert_eq!(AssetStore::extract_extension(Path::new("file.jpg")), "jpg");
    assert_eq!(AssetStore::extract_extension(Path::new("file.JPG")), "jpg");
    assert_eq!(AssetStore::extract_extension(Path::new("file")), "");
    assert_eq!(
        AssetStore::extract_extension(Path::new("file.tar.gz")),
        "gz"
    );
}

#[test]
fn test_save_many() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    // 创建测试文件
    let file1 = create_test_file(test_dir, "test1.jpg");
    let file2 = create_test_file(test_dir, "test2.png");

    // 创建存储管理器
    let layout = crate::path::StorageLayout::new(test_dir.to_path_buf());
    let builder = crate::path::PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let store = AssetStore::new(path_manager);

    // 保存文件
    let (ids, paths) = store.save_many(&[file1.clone(), file2.clone()]).unwrap();

    assert_eq!(ids.len(), 2);
    assert_eq!(paths.len(), 2);

    // 验证文件已保存
    for id in &ids {
        assert!(store.asset_exists(*id, "jpg") || store.asset_exists(*id, "png"));
    }
}

#[test]
fn test_move_to_recycle() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    // 创建并保存测试文件
    let file_path = create_test_file(test_dir, "test.jpg");
    let layout = crate::path::StorageLayout::new(test_dir.to_path_buf());
    let builder = crate::path::PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let store = AssetStore::new(path_manager);

    let (ids, stored_paths) = store.save_many(&[file_path]).unwrap();
    let asset_id = ids[0];
    let ext = "jpg";

    // 确保文件已保存
    assert!(store.asset_exists(asset_id, ext));

    // 移动到回收区
    let day = logical_day_from_ymd(2024, 1, 1);
    let recycle_path = store.move_to_recycle_one(asset_id, ext, day).unwrap();

    // 验证文件已从原位置移除
    assert!(!store.asset_exists(asset_id, ext));

    // 验证文件已在回收区
    let recycle_full_path = test_dir.join(&recycle_path);
    assert!(recycle_full_path.exists());
}

#[test]
fn test_move_to_recycle_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = crate::path::StorageLayout::new(test_dir.to_path_buf());
    let builder = crate::path::PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let store = AssetStore::new(path_manager);

    let asset_id = AssetId(Uuid::new_v4());
    let day = logical_day_from_ymd(2024, 1, 1);

    // 尝试移动不存在的文件
    let result = store.move_to_recycle_one(asset_id, "jpg", day);
    assert!(result.is_err());
}

#[test]
fn test_read_asset() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    // 创建测试文件
    let file_path = create_test_file(test_dir, "test.jpg");

    let layout = crate::path::StorageLayout::new(test_dir.to_path_buf());
    let builder = crate::path::PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let store = AssetStore::new(path_manager);

    // 保存文件
    let (ids, _) = store.save_many(&[file_path]).unwrap();
    let asset_id = ids[0];

    // 读取文件
    let content = store.read_asset(asset_id, "jpg").unwrap();
    assert_eq!(content, b"test content");
}

#[test]
fn test_delete_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    // 创建测试文件
    let file_path = create_test_file(test_dir, "test.jpg");

    let layout = crate::path::StorageLayout::new(test_dir.to_path_buf());
    let builder = crate::path::PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let store = AssetStore::new(path_manager);

    // 保存文件
    let (ids, _) = store.save_many(&[file_path]).unwrap();
    let asset_id = ids[0];

    // 确保文件存在
    assert!(store.asset_exists(asset_id, "jpg"));

    // 删除文件
    store.delete_file(asset_id, "jpg").unwrap();

    // 验证文件已删除
    assert!(!store.asset_exists(asset_id, "jpg"));
}

#[test]
fn test_move_to_recycle_with_errors() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    // 创建并保存测试文件
    let file_path = create_test_file(test_dir, "test.jpg");
    let layout = crate::path::StorageLayout::new(test_dir.to_path_buf());
    let builder = crate::path::PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let store = AssetStore::new(path_manager);

    let (ids, stored_paths) = store.save_many(&[file_path]).unwrap();

    // 准备移动列表，包含一个不存在的 asset
    let assets = vec![
        (ids[0], stored_paths[0].clone()),
        (AssetId(Uuid::new_v4()), "nonexistent.jpg".to_string()),
    ];

    let day = logical_day_from_ymd(2024, 1, 1);
    let (success_paths, errors) = store.move_to_recycle(&assets, day).unwrap();

    // 验证结果
    assert_eq!(success_paths.len(), 1);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, assets[1].0);
}
