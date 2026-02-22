use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use crate::asset::store::AssetStore;

#[test]
fn save_many_moves_files() {
    let root = tempdir().expect("create root");
    let src = tempdir().expect("create src");

    let f1 = src.path().join("one.txt");
    let f2 = src.path().join("two.bin");

    fs::write(&f1, b"hello").expect("write f1");
    fs::write(&f2, b"world!!").expect("write f2");

    let store = AssetStore::new(root.path().to_path_buf());
    let metas = store
        .save_many(&[f1.clone(), f2.clone()])
        .expect("save many");

    assert_eq!(metas.len(), 2);

    for m in &metas {
        let abs = store.root().join(&m.relative_path);
        assert!(abs.exists(), "dest must exist: {:?}", abs);
        assert!(m.size > 0);
    }

    // original files should be moved
    assert!(!f1.exists());
    assert!(!f2.exists());
}

#[test]
fn move_to_recycle_moves_files() {
    let root = tempdir().expect("create root");
    let src = tempdir().expect("create src");

    let f = src.path().join("img.png");
    fs::write(&f, b"PNGDATA").expect("write img");

    let store = AssetStore::new(root.path().to_path_buf());
    let metas = store.save_many(&[f.clone()]).expect("save");
    assert_eq!(metas.len(), 1);

    let entries = store.move_to_recycle(&metas).expect("move to recycle");
    assert_eq!(entries.len(), 1);

    let (id, ext, recycle_relative) = &entries[0];
    let recycle_abs = store.root().join(recycle_relative);
    assert!(recycle_abs.exists(), "recycle file exists");

    // original should be gone (original relative path is in `metas`)
    let orig_abs = store.root().join(&metas[0].relative_path);
    assert!(!orig_abs.exists(), "original should be moved");

    // recycle path should include "garbages"
    let s = recycle_relative.to_string_lossy();
    assert!(s.contains("garbages"));
}
