use crate::asset::store::AssetStore;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub asset: AssetStore,
}

impl AppState {
    pub fn new(conn: Connection, asset: AssetStore) -> Self {
        Self {
            db: Mutex::new(conn),
            asset,
        }
    }
}
