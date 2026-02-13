use crate::asset::AssetPath;
use crate::domain::ids::AssetId;
use crate::util::time::*;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_asset_subdir() {
    let layout = crate::path::StorageLayout::new(PathBuf::from("/data"));
    let builder = crate::path::PathBuilder::new(layout.clone());
    let asset_path = AssetPath::new(builder);

    // UUID: 12345678-1234-5678-1234-567812345678
    let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());

    let subdir = asset_path.asset_subdir(id);
    assert_eq!(subdir, PathBuf::from("12").join("34"));
}

#[test]
fn test_asset_storage_path() {
    let layout = crate::path::StorageLayout::new(PathBuf::from("/data"));
    let builder = crate::path::PathBuilder::new(layout.clone());
    let asset_path = AssetPath::new(builder);

    let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());

    let path = asset_path.asset_storage_path(id);
    let expected = PathBuf::from("/data").join("assets").join("12").join("34");
    assert_eq!(path.as_path(), expected.as_path());
}

#[test]
fn test_asset_file_path() {
    let layout = crate::path::StorageLayout::new(PathBuf::from("/data"));
    let builder = crate::path::PathBuilder::new(layout.clone());
    let asset_path = AssetPath::new(builder);

    let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());

    let path = asset_path.asset_file_path(id, "jpg").unwrap();
    let expected = PathBuf::from("/data")
        .join("assets")
        .join("12")
        .join("34")
        .join("12345678123456781234567812345678.jpg");
    assert_eq!(path.as_path(), expected.as_path());
}

#[test]
fn test_garbage_subdir() {
    let layout = crate::path::StorageLayout::new(PathBuf::from("/data"));
    let builder = crate::path::PathBuilder::new(layout.clone());
    let asset_path = AssetPath::new(builder);

    // Use util::time helper to construct logical day for 2024-01-01
    fn logical_day_from_ymd(y: i32, m: u32, d: u32) -> crate::util::time::LogicalDay {
        let naive = chrono::NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        from_datetime(naive)
    }
    let day = logical_day_from_ymd(2024, 1, 1);

    let subdir = asset_path.garbage_subdir(day);
    let expected = PathBuf::from("/data").join("garbages").join("738156");
    assert_eq!(subdir.as_path(), expected.as_path());
}

#[test]
fn test_garbage_file_path() {
    let layout = crate::path::StorageLayout::new(PathBuf::from("/data"));
    let builder = crate::path::PathBuilder::new(layout.clone());
    let asset_path = AssetPath::new(builder);

    let id = AssetId(Uuid::parse_str("12345678123456781234567812345678").unwrap());
    let day = logical_day::LogicalDay(738156);

    let path = asset_path.garbage_file_path(id, "jpg", day);
    let expected = PathBuf::from("/data")
        .join("garbages")
        .join("738156")
        .join("12345678123456781234567812345678.jpg");
    assert_eq!(path.as_path(), expected.as_path());
}

#[test]
fn test_temp_path() {
    let layout = crate::path::StorageLayout::new(PathBuf::from("/data"));
    let builder = crate::path::PathBuilder::new(layout.clone());
    let asset_path = AssetPath::new(builder);

    let path = asset_path.temp_path("upload_123");
    assert_eq!(
        path.as_path(),
        PathBuf::from("/data/temp/upload_123").as_path()
    );
}
