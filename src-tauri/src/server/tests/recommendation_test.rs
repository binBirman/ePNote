#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;
    use crate::server::recommendation::{DailyRecommendation, RecommendationSystem};
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        conn
    }

    #[test]
    fn test_get_daily_recommendation_empty() {
        let conn = setup_test_db();
        let rs = RecommendationSystem::new(&conn);

        let result = rs.get_daily_recommendation(10).unwrap();
        assert_eq!(result.questions.len(), 0);
    }

    #[test]
    fn test_daily_recommendation_stability() {
        let conn = setup_test_db();
        let rs = RecommendationSystem::new(&conn);

        // 第一次获取推荐
        let result1 = rs.get_daily_recommendation(10).unwrap();

        // 第二次获取推荐应该是相同的（从数据库加载）
        let result2 = rs.get_daily_recommendation(10).unwrap();

        assert_eq!(result1.day, result2.day);
        assert_eq!(result1.questions.len(), result2.questions.len());
    }

    #[test]
    fn test_interval_calculation() {
        let conn = setup_test_db();

        // 插入测试题目
        let now = chrono::Utc::now().timestamp();
        crate::db::insert_question(&conn, Some("Test Q1"), "LEARNING", now).unwrap();

        let rs = RecommendationSystem::new(&conn);
        let result = rs.get_daily_recommendation(10).unwrap();

        // 应该有题目
        assert!(result.questions.len() >= 1);
    }
}
