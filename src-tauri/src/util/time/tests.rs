#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::time::{days_since, now_ts, range_of_day, to_unix_ts, ClockConfig, LogicalDay, Timestamp};
    use chrono::{TimeZone, Utc};

    fn cfg() -> ClockConfig {
        ClockConfig::default()
    }

    #[test]
    fn test_from_timestamp_and_range_of_day() {
        // 2026-02-09 04:00:00 UTC+8
        let dt = chrono::FixedOffset::east_opt(8 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 2, 9, 4, 0, 0)
            .unwrap();
        let ts = Timestamp(dt.timestamp());
        let logical_day = LogicalDay::from_timestamp(ts, &cfg());
        let (start, end) = range_of_day(logical_day, &cfg());
        assert!(start.0 < ts.0 && ts.0 < end.0);
    }

    #[test]
    fn test_days_since() {
        let ts1 = Timestamp(
            chrono::FixedOffset::east_opt(8 * 3600)
                .unwrap()
                .with_ymd_and_hms(2026, 2, 1, 4, 0, 0)
                .unwrap()
                .timestamp(),
        );
        let ts2 = Timestamp(
            chrono::FixedOffset::east_opt(8 * 3600)
                .unwrap()
                .with_ymd_and_hms(2026, 2, 9, 4, 0, 0)
                .unwrap()
                .timestamp(),
        );
        let diff = days_since(ts1, ts2, &cfg());
        assert_eq!(diff, 8);
    }

    #[test]
    fn test_now_ts_and_to_unix_ts() {
        let ts = now_ts();
        let unix = to_unix_ts(ts);
        assert!(unix > 0);
    }

    #[test]
    fn test_clock_config_default_matches_legacy_hardcoded_behavior() {
        // 默认 UTC+8 + 03:00 切日必须与之前硬编码行为一致。
        let cfg = ClockConfig::default();
        assert_eq!(cfg.offset_seconds, 8 * 3600);
        assert_eq!(cfg.cutoff_seconds, 3 * 3600);
    }

    /// 用 chrono 的公共 API（`from_num_days_from_ce_opt` 的反函数）求期望 day index。
    /// 这样测试与 chrono 实际实现走同一条路径，避免 off-by-one 反复。
    fn num_days_from_ce_via_chrono(target: chrono::NaiveDate) -> i64 {
        // 二分搜索 N：让 from_num_days_from_ce_opt(N) == target
        let mut lo: i32 = 0;
        let mut hi: i32 = 5_000_000;
        while lo < hi {
            let mid = (lo + hi) / 2;
            let d = chrono::NaiveDate::from_num_days_from_ce_opt(mid).unwrap();
            if d < target {
                lo = mid + 1;
            } else if d > target {
                hi = mid;
            } else {
                return mid as i64;
            }
        }
        lo as i64
    }

    #[test]
    fn test_clock_config_from_settings_zero_cutoff_midnight_aligned() {
        let cfg = ClockConfig::from_settings(0, 0);
        assert_eq!(cfg.offset_seconds, 0);
        assert_eq!(cfg.cutoff_seconds, 0);

        let dt_utc = chrono::Utc.with_ymd_and_hms(2026, 2, 9, 0, 0, 0).unwrap();
        let ts = Timestamp(dt_utc.timestamp());
        let day = LogicalDay::from_timestamp(ts, &cfg);

        let target = chrono::NaiveDate::from_ymd_opt(2026, 2, 9).unwrap();
        assert_eq!(day.0 as i64, num_days_from_ce_via_chrono(target));
    }

    #[test]
    fn test_clock_config_from_settings_utc_minus_5() {
        let cfg = ClockConfig::from_settings(-5, 0);
        let dt_utc = chrono::Utc.with_ymd_and_hms(2026, 2, 9, 0, 0, 0).unwrap();
        let ts = Timestamp(dt_utc.timestamp());
        let day = LogicalDay::from_timestamp(ts, &cfg);

        let target = chrono::NaiveDate::from_ymd_opt(2026, 2, 8).unwrap();
        assert_eq!(day.0 as i64, num_days_from_ce_via_chrono(target));
    }

    #[test]
    fn test_clock_config_from_settings_with_cutoff_8_hours() {
        let cfg = ClockConfig::from_settings(0, 8);
        let dt_utc = chrono::Utc.with_ymd_and_hms(2026, 2, 9, 6, 0, 0).unwrap();
        let ts = Timestamp(dt_utc.timestamp());
        let day = LogicalDay::from_timestamp(ts, &cfg);

        let target = chrono::NaiveDate::from_ymd_opt(2026, 2, 8).unwrap();
        assert_eq!(day.0 as i64, num_days_from_ce_via_chrono(target));
    }
}
