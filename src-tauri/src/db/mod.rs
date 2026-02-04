pub mod asset_schema;
pub mod connection;
pub mod error;
pub mod meta_schema;
pub mod migrate;
pub mod question_schema;
pub mod review_schema;

pub use asset_schema::*;
pub use connection::*;
pub use meta_schema::*;
pub use migrate::*;
pub use question_schema::*;
pub use review_schema::*;

use rusqlite::Connection;
use std::path::Path;

pub fn init_db(data_root: &Path) -> rusqlite::Result<Connection> {
    let mut conn = connection::open_db(data_root)?;
    migrate::migrate(&mut conn)?;
    Ok(conn)
}
