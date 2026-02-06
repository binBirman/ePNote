pub mod connection;
pub mod error;

pub mod migrate;
pub mod schema {
    pub mod asset_schema;
    pub mod meta_schema;
    pub mod question_schema;
    pub mod review_schema;
}

pub use connection::*;
pub use migrate::*;
pub use schema::asset_schema::*;
pub use schema::meta_schema::*;
pub use schema::question_schema::*;
pub use schema::review_schema::*;

use rusqlite::Connection;
use std::path::Path;

mod tests;

pub fn init_db(data_root: &Path) -> rusqlite::Result<Connection> {
    let mut conn = connection::open_db(data_root)?;
    migrate::migrate(&mut conn)?;
    Ok(conn)
}
