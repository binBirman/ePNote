pub mod connection;
pub mod migrate;
pub mod schema;

pub use connection::*;
pub use migrate::*;
pub use schema::*;

use rusqlite::Connection;
use std::path::Path;

pub fn init_db(data_root: &Path) -> rusqlite::Result<Connection> {
    let mut conn = connection::open_db(data_root)?;
    migrate::migrate(&mut conn)?;
    Ok(conn)
}
