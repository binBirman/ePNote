use chrono::{TimeZone, Utc};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

use crate::asset::garbage::{delete_entries, scan_recycle};
use crate::asset::path::{PathBuilder, StorageLayout};
use crate::asset::store::AssetStore;

fn setup_store_with_src() -> (TempDir, TempDir, PathBuf, AssetStore, PathBuilder) {
    let root = TempDir::new().expect("root tempdir");
    let rootp = root.path().to_path_buf();

    // create a source file to import; keep the src_dir alive by returning it
    let src_dir = TempDir::new().expect("src tempdir");
    let src_file = src_dir.path().join("photo.png");
    fs::write(&src_file, b"PNGDATA").expect("write src");

    let store = AssetStore::new(rootp.clone());
    let layout = StorageLayout::new(rootp.clone());
    let builder = PathBuilder::new(layout);

    (root, src_dir, src_file, store, builder)
}

#[test]
fn test_save_returns_relative_and_creates_file() {
    let (_root, _src_dir, src_file, store, _builder) = setup_store_with_src();

    let metas = store.save_many(&[src_file.clone()]).expect("save_many");
    assert_eq!(metas.len(), 1);
    let meta = &metas[0];

    // relative path should be non-empty and the physical file must exist
    assert!(!meta.relative_path.as_os_str().is_empty());
    let abs = store.root().join(&meta.relative_path);
    assert!(abs.exists());
}

#[test]
fn test_compute_physical_path_from_relative() {
    let (_root, _src_dir, src_file, store, _builder) = setup_store_with_src();

    let metas = store.save_many(&[src_file.clone()]).expect("save_many");
    let meta = &metas[0];

    // compute physical path from relative and verify it matches actual
    let computed = store.root().join(&meta.relative_path);
    let actual_exists = computed.exists();
    assert!(actual_exists);
}

#[test]
fn test_move_to_recycle_moves_file_and_returns_recycle_relative() {
    let (_root, _src_dir, src_file, store, _builder) = setup_store_with_src();

    let metas = store.save_many(&[src_file.clone()]).expect("save_many");
    let meta = &metas[0];

    let recycle_entries = store.move_to_recycle(&metas).expect("move_to_recycle");
    assert_eq!(recycle_entries.len(), 1);
    let (_id, _ext, recycle_relative) = &recycle_entries[0];

    let recycle_abs = store.root().join(recycle_relative);
    assert!(recycle_abs.exists());
    assert!(recycle_abs.starts_with(store.root().join("garbages")));
    // 原文件位置不再存在
    let original_abs = store.root().join(&meta.relative_path);
    assert!(!original_abs.exists());
}

#[test]
fn test_restore_from_recycle_moves_back_and_returns_meta() {
    let (_root, _src_dir, src_file, store, builder) = setup_store_with_src();

    let metas = store.save_many(&[src_file.clone()]).expect("save_many");
    let meta = &metas[0];

    let recycle_entries = store.move_to_recycle(&metas).expect("move_to_recycle");
    assert_eq!(recycle_entries.len(), 1);

    let restored = store.restore(&recycle_entries).expect("restore");
    assert_eq!(restored.len(), 1);
    let restored_meta = &restored[0];

    let restored_abs = store.root().join(&restored_meta.relative_path);
    assert!(restored_abs.exists());

    // 恢复后路径应当符合 PathBuilder 的月分桶规则
    let expected_dir = builder.layout().asset_dir_by_ts(
        restored_meta
            .created_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0),
    );
    let parent = restored_abs.parent().expect("has parent");
    // 这里只比较结尾目录是否匹配年月分桶中的年月（避免绝对路径差异）
    assert!(parent.ends_with(expected_dir));
}

#[test]
fn test_delete_physical_removes_from_recycle() {
    let (root_dir, _src_dir, src_file, store, _builder) = setup_store_with_src();

    let metas = store.save_many(&[src_file.clone()]).expect("save_many");

    let rec = store.move_to_recycle(&metas).expect("move to recycle");
    assert_eq!(rec.len(), 1);

    let recycle_abs = store.root().join(&rec[0].2);
    assert!(recycle_abs.exists());

    // physically delete using AssetStore API
    store.delete_physical(&rec).expect("delete physical");
    assert!(!recycle_abs.exists());

    // scan_recycle should no longer list the deleted entry
    let listed = scan_recycle(root_dir.path()).expect("scan recycle");
    assert!(listed.is_empty());
}
