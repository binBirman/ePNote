use crate::util::time::timestamp::*;
use chrono::{Datelike, FixedOffset, TimeZone, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalDay(pub i32);

/// 默认逻辑日边界：凌晨 3 点
const DAY_CUTOFF_HOUR: i64 = 3;

/// 默认时区：UTC+8（你可以改成用户配置）
fn default_offset() -> FixedOffset {
    FixedOffset::east_opt(8 * 3600).unwrap()
}

pub fn from_timestamp(ts: Timestamp) -> LogicalDay {
    let offset = default_offset();

    let dt = offset.timestamp_opt(ts.0, 0).unwrap();

    // 向前移动 3 小时
    let shifted = dt - chrono::Duration::hours(DAY_CUTOFF_HOUR);

    let day_index = shifted.date_naive().num_days_from_ce();

    LogicalDay(day_index)
}

pub fn from_datetime(dt: chrono::NaiveDateTime) -> LogicalDay {
    let offset = default_offset();

    let shifted =
        offset.from_local_datetime(&dt).unwrap() - chrono::Duration::hours(DAY_CUTOFF_HOUR);

    let day_index = shifted.date_naive().num_days_from_ce();

    LogicalDay(day_index)
}

/// 获取逻辑日对应的时间范围（UTC时间戳）
pub fn range_of_day(day: LogicalDay) -> (Timestamp, Timestamp) {
    let offset = default_offset();

    let base = chrono::NaiveDate::from_num_days_from_ce_opt(day.0)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let start =
        offset.from_local_datetime(&base).unwrap() + chrono::Duration::hours(DAY_CUTOFF_HOUR);

    let end = start + chrono::Duration::days(1);

    (Timestamp(start.timestamp()), Timestamp(end.timestamp()))
}

// 计算两个时间戳之间的逻辑日差
pub fn days_since(old: Timestamp, now: Timestamp) -> i32 {
    let old_day = from_timestamp(old);
    let now_day = from_timestamp(now);

    now_day.0 - old_day.0
}
