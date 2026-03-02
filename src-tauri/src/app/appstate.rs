use crate::asset::store::AssetStore;
use rusqlite::Connection;
use std::sync::Mutex;

// pub struct AppState {
//     pub db: Mutex<rusqlite::Connection>,
//     pub asset: AssetStore,
// }

pub struct AppState {
    pub inner: Mutex<Option<AppInner>>,
}

pub struct AppInner {
    pub db: Connection,
    pub asset_store: AssetStore,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn get_db_path(&self) -> String {
        let guard = self.inner.lock().unwrap();
        if let Some(inner) = &*guard {
            inner.db.path().unwrap().to_string()
        } else {
            "not initialized".to_string()
        }
    }
}
