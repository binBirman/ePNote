use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};

const DAY_BOUNDARY_HOUR: i64 = 3;

const LOGICAL_DAY_EPOCH: NaiveDate = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();

/* 获取当前时间戳 */
pub fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

// /* 时间戳转换为UTC时间 */
// pub fn ts_to_utc_datetime(ts: i64) -> DateTime<Utc> {
//     Utc.timestamp_opt(ts, 0).single().unwrap()
// }

// /* UTC时间转换为时间戳 */
// pub fn utc_datetime_to_ts(dt: DateTime<Utc>) -> i64 {
//     dt.timestamp()
// }

/* 时间戳转换为本地时间 */
pub fn ts_to_local_datetime(ts: i64) -> DateTime<Local> {
    chrono::Local.timestamp_opt(ts, 0).single().unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalDay(i64);

pub fn logical_day_from_ts(ts_utc: i64) -> LogicalDay {
    // UTC -> Local
    let utc_dt = Utc.timestamp_opt(ts_utc, 0).unwrap();
    let local_dt: DateTime<Local> = utc_dt.with_timezone(&Local);

    // 分界点平移
    let shifted = local_dt - Duration::hours(DAY_BOUNDARY_HOUR);

    let date = shifted.date_naive();

    // ⭐ 关键：用 signed_duration_since
    let day_index = date.signed_duration_since(LOGICAL_DAY_EPOCH).num_days();

    LogicalDay(day_index)
}

/* 计算从指定时间戳到今天经过了多少个逻辑天 */
pub fn days_since(ts_utc: i64) -> i64 {
    let today = logical_day_from_ts(now_ts());
    let target = logical_day_from_ts(ts_utc);
    today.0 - target.0
}

/*  */
