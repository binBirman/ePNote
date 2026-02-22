use std::path::PathBuf;
use tempfile::tempdir;
use uuid::Uuid;

use crate::asset::error::PathError;
use crate::asset::path::{PathBuilder, StorageLayout};

#[test]
fn build_asset_path_structure() {
    let td = tempdir().expect("create tempdir");
    let root = td.path().to_path_buf();

    let layout = StorageLayout::new(root.clone());
    let builder = PathBuilder::new(layout);

    let id = Uuid::new_v4();
    let p = builder.build_asset_path(&id, "png").expect("build path");

    // 新方法返回相对路径：assets/{YYYY}/{MM}/{filename}
    assert!(!p.is_absolute());

    let mut comps = p.components();
    let first = comps.next().unwrap().as_os_str().to_str().unwrap();
    assert_eq!(first, "assets");

    // 年/月 两个段
    let year = comps.next().unwrap().as_os_str().to_str().unwrap();
    let month = comps.next().unwrap().as_os_str().to_str().unwrap();
    assert_eq!(year.len(), 4);
    assert_eq!(month.len(), 2);
    assert!(year.chars().all(|c| c.is_numeric()));
    assert!(month.chars().all(|c| c.is_numeric()));

    // 文件名应以 `{uuid}.png` 结尾（默认使用 timestamped 命名）
    let simple = id.simple().to_string();
    let file_name = p.file_name().unwrap().to_str().unwrap();
    assert!(file_name.ends_with(&format!("{}.png", simple)));
}

#[test]
fn build_physical_path_valid_and_invalid() {
    let td = tempdir().expect("create tempdir");
    let root = td.path().to_path_buf();

    let layout = StorageLayout::new(root.clone());
    let builder = PathBuilder::new(layout);

    // 有效的文件名
    let logical = PathBuf::from("document.txt");
    let res = builder.build_physical_path(&logical);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), root.join("document.txt"));

    // 带子目录，应该只取 file_name
    let logical2 = PathBuf::from("folder/document.txt");
    let res2 = builder.build_physical_path(&logical2);
    assert!(res2.is_ok());
    assert_eq!(res2.unwrap(), root.join("document.txt"));

    // 空路径 -> 错误
    let logical3 = PathBuf::from("");
    let res3 = builder.build_physical_path(&logical3);
    assert!(res3.is_err());

    // 隐藏文件（以 . 开头） -> 由 sanitize 拒绝
    let logical4 = PathBuf::from(".hidden");
    let res4 = builder.build_physical_path(&logical4);
    assert!(res4.is_err());

    // 文件名内部含有 ".." -> 由 sanitize 拒绝
    let logical5 = PathBuf::from("bad..name.txt");
    let res5 = builder.build_physical_path(&logical5);
    assert!(res5.is_err());
}

#[test]
fn build_asset_path_rejects_bad_ext() {
    let td = tempdir().expect("create tempdir");
    let root = td.path().to_path_buf();

    let layout = StorageLayout::new(root);
    let builder = PathBuilder::new(layout);

    let id = Uuid::new_v4();
    let res = builder.build_asset_path(&id, "../evil");
    assert!(res.is_err());

    match res.err().unwrap() {
        PathError::Sanitize(_) => {}
        other => panic!("expected sanitize error, got: {:?}", other),
    }
}
