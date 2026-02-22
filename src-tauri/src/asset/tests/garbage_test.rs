use std::fs;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

use uuid::Uuid;

use crate::asset::garbage::{delete_entries, list_deletable, scan_recycle};

#[test]
fn scan_list_and_delete_recycle() {
    let root = tempdir().expect("root");
    let rootp = root.path();

    // create two files under garbages/<subdir>/
    let sub = rootp.join("garbages").join("subdir");
    fs::create_dir_all(&sub).expect("create subdir");

    let id = Uuid::new_v4();
    let f1 = sub.join(format!("{}.txt", id));
    let f2 = sub.join("other.bin");

    fs::write(&f1, b"one").expect("write f1");
    fs::write(&f2, b"two").expect("write f2");

    // scan
    let all = scan_recycle(rootp).expect("scan");
    // should contain at least these two
    assert!(all.iter().any(|e| e
        .recycle_relative
        .to_string_lossy()
        .contains(&id.to_string())));
    assert!(all
        .iter()
        .any(|e| e.recycle_relative.to_string_lossy().contains("other.bin")));

    // list deletable with cutoff in future -> both included
    let cutoff_future = SystemTime::now() + Duration::from_secs(60);
    let deletable = list_deletable(rootp, cutoff_future).expect("list deletable");
    assert!(deletable.len() >= 2);

    // list deletable with cutoff in past -> none
    let cutoff_past = SystemTime::now() - Duration::from_secs(60);
    let none = list_deletable(rootp, cutoff_past).expect("list past");
    assert!(none.is_empty());

    // delete entries explicitly
    delete_entries(rootp, &deletable).expect("delete entries");

    // files removed
    assert!(!f1.exists());
    assert!(!f2.exists());
}
