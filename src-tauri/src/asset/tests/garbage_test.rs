use crate::asset::{AssetPath, GarbageManager};
//use crate::util::time::LogicalDay as LD;
use crate::util::path::*;
use crate::util::time::*;
use chrono::Utc;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

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
    from_datetime(naive)
}

#[test]
fn test_scan_garbage_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = StorageLayout::new(test_dir.to_path_buf());
    let builder = PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let manager = GarbageManager::new(path_manager);

    // 扫描不存在的回收区
    let entries = manager.scan_garbage().unwrap();
    assert_eq!(entries.len(), 0);
}

#[test]
fn test_scan_garbage_with_files() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = StorageLayout::new(test_dir.to_path_buf());
    let builder = PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let manager = GarbageManager::new(path_manager);

    // 创建回收区目录和文件
    let garbage_dir = test_dir.join("garbages").join("738156");
    fs::create_dir_all(&garbage_dir).unwrap();

    let file_path = create_test_file(&garbage_dir, "asset1.jpg");
    let file_path2 = create_test_file(&garbage_dir, "asset2.png");

    let entries = manager.scan_garbage().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].logical_day.0, logical_day_from_ymd(2024, 1, 1).0);
    assert_eq!(entries[1].logical_day.0, logical_day_from_ymd(2024, 1, 1).0);
}

#[test]
fn test_get_stats() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = StorageLayout::new(test_dir.to_path_buf());
    let builder = PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let manager = GarbageManager::new(path_manager);

    // 创建回收区文件
    let garbage_dir = test_dir.join("garbages").join("738156");
    fs::create_dir_all(&garbage_dir).unwrap();

    create_test_file(&garbage_dir, "asset1.jpg");
    create_test_file(&garbage_dir, "asset2.png");

    // 创建另一天的文件
    let garbage_dir2 = test_dir.join("garbages").join("738157");
    fs::create_dir_all(&garbage_dir2).unwrap();
    create_test_file(&garbage_dir2, "asset3.jpg");

    let stats = manager.get_stats().unwrap();
    assert_eq!(stats.file_count, 3);
    assert_eq!(stats.count_by_day.len(), 2);
    assert_eq!(
        stats.count_by_day[0].0 .0,
        logical_day_from_ymd(2024, 1, 1).0
    );
    assert_eq!(stats.count_by_day[0].1, 2);
    assert_eq!(
        stats.count_by_day[1].0 .0,
        logical_day_from_ymd(2024, 1, 2).0
    );
    assert_eq!(stats.count_by_day[1].1, 1);
}

#[test]
fn test_cleanup_before() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = StorageLayout::new(test_dir.to_path_buf());
    let builder = PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let manager = GarbageManager::new(path_manager);

    // 创建多天的文件
    let day1_dir = test_dir.join("garbages").join("738150");
    fs::create_dir_all(&day1_dir).unwrap();
    create_test_file(&day1_dir, "asset1.jpg");

    let day2_dir = test_dir.join("garbages").join("738155");
    fs::create_dir_all(&day2_dir).unwrap();
    create_test_file(&day2_dir, "asset2.jpg");

    let day3_dir = test_dir.join("garbages").join("738160");
    fs::create_dir_all(&day3_dir).unwrap();
    create_test_file(&day3_dir, "asset3.jpg");

    // 清理 738155 之前的文件
    let deleted = manager
        .cleanup_before(logical_day_from_ymd(2024, 1, 2))
        .unwrap();
    assert_eq!(deleted.len(), 1);

    // 验证文件状态
    let entries = manager.scan_garbage().unwrap();
    assert_eq!(entries.len(), 2);

    // 验证目录
    assert!(!day1_dir.exists());
    assert!(day2_dir.exists());
    assert!(day3_dir.exists());
}

#[test]
fn test_cleanup_day() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = StorageLayout::new(test_dir.to_path_buf());
    let builder = PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let manager = GarbageManager::from_ref(&path_manager);

    // 创建文件
    let day_dir = path_manager.garbage_subdir(logical_day_from_ymd(2024, 1, 1));
    fs::create_dir_all(&day_dir).unwrap();
    create_test_file(&day_dir, "asset1.jpg");
    create_test_file(&day_dir, "asset2.png");

    // 清理该日
    let deleted = manager
        .cleanup_day(logical_day_from_ymd(2024, 1, 1))
        .unwrap();
    assert_eq!(deleted.len(), 2);

    // 验证目录已删除
    assert!(!day_dir.exists());
}

#[test]
fn test_check_expiration() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = StorageLayout::new(test_dir.to_path_buf());
    let builder = PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let manager = GarbageManager::new(path_manager);

    // 创建不同天的文件
    let day1_dir = test_dir.join("garbages").join("738150");
    fs::create_dir_all(&day1_dir).unwrap();
    create_test_file(&day1_dir, "asset1.jpg");

    let day2_dir = test_dir.join("garbages").join("738155");
    fs::create_dir_all(&day2_dir).unwrap();
    create_test_file(&day2_dir, "asset2.jpg");

    // 检查过期（保留5天，从 738160 开始）
    // 阈值 = 738160 - 5 = 738155
    // 738150 < 738155 过期
    // 738155 >= 738155 不过期
    let expired = manager
        .check_expiration(5, logical_day_from_ymd(2024, 1, 6))
        .unwrap();
    assert_eq!(expired.len(), 1);
}

#[test]
fn test_expired_size() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    let layout = StorageLayout::new(test_dir.to_path_buf());
    let builder = PathBuilder::new(layout.clone());
    let path_manager = AssetPath::new(builder);
    let manager = GarbageManager::new(path_manager);

    // 创建文件
    let day_dir = test_dir.join("garbages").join("738150");
    fs::create_dir_all(&day_dir).unwrap();
    create_test_file(&day_dir, "asset1.jpg");
    create_test_file(&day_dir, "asset2.jpg");

    // 计算过期大小
    let size = manager
        .expired_size(5, logical_day_from_ymd(2024, 1, 6))
        .unwrap();
    // 每个文件 12 字节
    assert_eq!(size, 24);
}
