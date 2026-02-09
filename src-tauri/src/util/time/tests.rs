#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::time::{
        days_since, from_timestamp, now_ts, range_of_day, to_unix_ts, LogicalDay, Timestamp,
    };
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_from_timestamp_and_range_of_day() {
        // 2026-02-09 04:00:00 UTC+8
        let dt = chrono::FixedOffset::east_opt(8 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 2, 9, 4, 0, 0)
            .unwrap();
        let ts = Timestamp(dt.timestamp());
        let logical_day = from_timestamp(ts);
        let (start, end) = range_of_day(logical_day);
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
        let diff = days_since(ts1, ts2);
        assert_eq!(diff, 8);
    }

    #[test]
    fn test_now_ts_and_to_unix_ts() {
        let ts = now_ts();
        let unix = to_unix_ts(ts);
        assert!(unix > 0);
    }
}
