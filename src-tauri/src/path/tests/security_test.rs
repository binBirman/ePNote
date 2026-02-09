use std::path::PathBuf;
use uuid::Uuid;

use crate::path::sanitize::sanitize_filename;
use crate::path::{PathBuilder, StorageLayout};

//
// ==============================
// sanitize 攻击测试
// ==============================
//

#[test]
fn sanitize_rejects_path_traversal() {
    let bad = [
        "..",
        "../a",
        "..\\a",
        "a/../b",
        "a\\..\\b",
        "/etc/passwd",
        "\\windows\\system32",
    ];

    for input in bad {
        assert!(sanitize_filename(input).is_err(), "should reject {}", input);
    }
}

#[test]
fn sanitize_rejects_illegal_chars() {
    let bad = ["a:b", "a*b", "a?", "a<", "a>", "a|", "a\"", "a\t", "a\n"];

    for input in bad {
        assert!(
            sanitize_filename(input).is_err(),
            "illegal char not rejected: {}",
            input
        );
    }
}

#[test]
fn sanitize_rejects_empty_and_hidden() {
    assert!(sanitize_filename("").is_err());
    assert!(sanitize_filename(".png").is_err());
    assert!(sanitize_filename("png.").is_err());
}

//
// ==============================
// layout 安全测试
// ==============================
//

#[test]
fn layout_never_escapes_root() {
    let root = PathBuf::from("/tmp/storage");
    let layout = StorageLayout::new(root.clone());

    let id = Uuid::from_bytes([0u8; 16]);
    let path = layout.asset_file(&id, "png");

    assert!(path.starts_with(&root), "path escaped root: {:?}", path);
}

//
// ==============================
// builder 安全测试
// ==============================
//

#[test]
fn builder_rejects_bad_ext() {
    let root = PathBuf::from("/tmp/storage");
    let layout = StorageLayout::new(root);
    let builder = PathBuilder::new(layout);

    let id = Uuid::from_bytes([0u8; 16]);

    let bad_ext = ["../png", "png/evil", "png\\evil", "p/ng", "p\\ng"];

    for ext in bad_ext {
        assert!(
            builder.build_asset_path(&id, ext).is_err(),
            "builder accepted bad ext: {}",
            ext
        );
    }
}

#[test]
fn builder_path_inside_root() {
    let root = PathBuf::from("/tmp/storage");
    let layout = StorageLayout::new(root.clone());
    let builder = PathBuilder::new(layout);

    let id = Uuid::from_bytes([0u8; 16]);
    let path = builder.build_asset_path(&id, "jpg").unwrap();

    assert!(path.as_path().starts_with(&root), "builder escaped root");
}

//
// ==============================
// Windows 特殊路径测试
// ==============================
//

#[test]
fn sanitize_windows_reserved() {
    let bad = ["CON", "NUL", "PRN", "AUX"];

    for name in bad {
        assert!(
            sanitize_filename(name).is_err(),
            "reserved name accepted: {}",
            name
        );
    }
}

//
// ==============================
// UUID 分桶稳定性
// ==============================
//

#[test]
fn layout_bucket_stability() {
    let root = PathBuf::from("/tmp/storage");
    let layout = StorageLayout::new(root);

    let id = Uuid::from_bytes([1u8; 16]);

    let dir1 = layout.asset_dir(&id);
    let dir2 = layout.asset_dir(&id);

    assert_eq!(dir1, dir2);
}
