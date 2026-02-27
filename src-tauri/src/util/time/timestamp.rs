use std::alloc::System;
use std::time::SystemTime;

use chrono::{FixedOffset, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn as_i64(self) -> i64 {
        self.0
    }

    pub fn as_system_time(self) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.0 as u64)
    }
}

impl From<i64> for Timestamp {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<SystemTime> for Timestamp {
    fn from(value: SystemTime) -> Self {
        let duration = value
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        Self(duration.as_secs() as i64)
    }
}

impl From<Timestamp> for i64 {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

pub fn now_ts() -> Timestamp {
    Timestamp(Utc::now().timestamp())
}

pub fn to_unix_ts(ts: Timestamp) -> i64 {
    ts.0
}
