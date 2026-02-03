use chrono::{TimeZone, Utc};

pub fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}
