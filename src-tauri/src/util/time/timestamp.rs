use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for Timestamp {
    fn from(value: i64) -> Self {
        Self(value)
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
