use chrono::{NaiveDateTime, TimeZone, Utc};

/* 获取当前时间戳 */
pub fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

/* 时间戳转换为UTC时间 */
pub fn ts_to_utc_datetime(ts: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(ts, 0).single().unwrap()
}

/* UTC时间转换为时间戳 */
pub fn utc_datetime_to_ts(dt: chrono::DateTime<Utc>) -> i64 {
    dt.timestamp()
}

// 计算距今天数
pub fn days_from_now(ts: i64) -> i64 {
    let now_ts = now_ts();
    let days = (now_ts - ts) / 86400;
    days
}
