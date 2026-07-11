use crate::util::time::timestamp::*;
use chrono::{Datelike, FixedOffset, TimeZone};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalDay(pub i32);

/// 用户视角的"作息时钟"。
///
/// - `offset_seconds`：本地时间相对 UTC 的偏移（秒）。例如 UTC+8 = 28800。
///   实际值由用户设置中的"时区"决定；负数表示西时区。
/// - `cutoff_seconds`：逻辑日"切日"分界秒。例如 UTC+8 的"凌晨 03:00 切日"
///   = 10800；纯按日历 0:00 切日则 = 0。
///
/// 新功能（如 Stats 折线图）应从用户设置读取 `timezone_offset_hours` 和
/// `day_cutoff_hour` 后构造本类型传入；旧有调用方（review / recommendation）
/// 暂时传 `ClockConfig::default()`（与历史 UTC+8 + 03:00 行为等价）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockConfig {
    pub offset_seconds: i64,
    pub cutoff_seconds: i64,
}

impl Default for ClockConfig {
    /// 默认 UTC+8 + 03:00 切日（与历史硬编码行为一致）。
    fn default() -> Self {
        Self {
            offset_seconds: 8 * 3600,
            cutoff_seconds: 3 * 3600,
        }
    }
}

impl ClockConfig {
    /// 从用户设置构造。`offset_hours` 可为负数；`cutoff_hour` 必须 0..24。
    pub fn from_settings(offset_hours: i32, cutoff_hour: i32) -> Self {
        Self {
            offset_seconds: offset_hours as i64 * 3600,
            cutoff_seconds: cutoff_hour as i64 * 3600,
        }
    }
}

impl LogicalDay {
    /// 把 unix 时间戳按配置投影成 LogicalDay 日号。
    pub fn from_timestamp(ts: Timestamp, cfg: &ClockConfig) -> Self {
        let offset_sec = cfg.offset_seconds;
        let cutoff_sec = cfg.cutoff_seconds;
        // ts → 本地时间戳 → 减去 cutoff → 那天的 unix 0 点 → day index
        let offset = FixedOffset::east_opt(offset_sec as i32).unwrap();
        let dt = offset.timestamp_opt(ts.0, 0).unwrap();
        let shifted = dt - chrono::Duration::seconds(cutoff_sec);
        let day_index = shifted.date_naive().num_days_from_ce();
        LogicalDay(day_index)
    }

    /// LogicalDay → "YYYY-MM-DD" 字符串（本地日历日）。
    pub fn to_string(&self, cfg: &ClockConfig) -> String {
        let offset_sec = cfg.offset_seconds;
        let cutoff_sec = cfg.cutoff_seconds;
        let naive_date = chrono::NaiveDate::from_num_days_from_ce_opt(self.0)
            .expect("Invalid logical day");
        let offset = FixedOffset::east_opt(offset_sec as i32).unwrap();
        let dt = naive_date.and_hms_opt(0, 0, 0)
            .map(|ndt| offset.from_local_datetime(&ndt).single())
            .flatten()
            .expect("Invalid local datetime")
            + chrono::Duration::seconds(cutoff_sec);
        dt.format("%Y-%m-%d").to_string()
    }
}

/// 逻辑日对应的 UTC 时间戳范围 `[start, end)`（左闭右开）。
pub fn range_of_day(day: LogicalDay, cfg: &ClockConfig) -> (Timestamp, Timestamp) {
    let offset_sec = cfg.offset_seconds;
    let cutoff_sec = cfg.cutoff_seconds;
    let day_start_offset = chrono::Duration::seconds(cutoff_sec - offset_sec);
    let naive_date =
        chrono::NaiveDate::from_num_days_from_ce_opt(day.0).expect("Invalid logical day");
    let start_local = naive_date.and_hms_opt(0, 0, 0).unwrap() + day_start_offset;
    let end_local = start_local + chrono::Duration::days(1);
    (
        Timestamp(start_local.and_utc().timestamp()),
        Timestamp(end_local.and_utc().timestamp()),
    )
}

/// 两个时间戳之间的逻辑日差（按配置时区 + 切日秒数）。
pub fn days_since(old: Timestamp, now: Timestamp, cfg: &ClockConfig) -> i32 {
    let old_day = LogicalDay::from_timestamp(old, cfg);
    let now_day = LogicalDay::from_timestamp(now, cfg);
    now_day.0 - old_day.0
}
