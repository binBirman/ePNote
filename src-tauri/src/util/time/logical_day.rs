use crate::util::time::timestamp::*;
use chrono::{Datelike, FixedOffset, TimeZone, Utc};
use tauri::webview::cookie::time::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalDay(pub i32);

/// 默认逻辑日边界：凌晨 3 点
const DAY_CUTOFF_HOUR: i64 = 3;

/// 默认时区：UTC+8（你可以改成用户配置）
fn default_offset() -> FixedOffset {
    FixedOffset::east_opt(8 * 3600).unwrap()
}

impl LogicalDay {
    pub fn to_string(&self) -> String {
        // num_days_from_ce() 返回公元以来的天数（从公元1年1月1日开始）
        // 需要用 chrono 的 from_num_days_from_ce_opt 转换回日期
        let naive_date = chrono::NaiveDate::from_num_days_from_ce_opt(self.0)
            .expect("Invalid logical day");
        // 直接用日期构造日期时间，然后应用时区偏移
        let dt = naive_date.and_hms_opt(0, 0, 0)
            .map(|ndt| default_offset().from_local_datetime(&ndt).single())
            .flatten()
            .expect("Invalid local datetime")
            + chrono::Duration::hours(DAY_CUTOFF_HOUR);
        dt.format("%Y-%m-%d").to_string()
    }
}

impl From<Timestamp> for LogicalDay {
    fn from(ts: Timestamp) -> Self {
        //from_timestamp(ts)
        let offset = default_offset();
        let dt = offset.timestamp_opt(ts.0, 0).unwrap();

        // 向前移动 3 小时
        let shifted = dt - chrono::Duration::hours(DAY_CUTOFF_HOUR);
        let day_index = shifted.date_naive().num_days_from_ce();

        LogicalDay(day_index)
    }
}

impl From<chrono::NaiveDateTime> for LogicalDay {
    fn from(dt: chrono::NaiveDateTime) -> Self {
        let offset = default_offset();

        let shifted =
            offset.from_local_datetime(&dt).unwrap() - chrono::Duration::hours(DAY_CUTOFF_HOUR);

        let day_index = shifted.date_naive().num_days_from_ce();

        LogicalDay(day_index)
    }
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
    let old_day = LogicalDay::from(old);
    let now_day = LogicalDay::from(now);

    now_day.0 - old_day.0
}
