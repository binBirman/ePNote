use rusqlite::{Connection, Result};

//use crate::db::connection;

// 单条迁移描述
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

// 所有迁移，按 version 升序排列
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "init_schema",
        sql: r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );

        INSERT INTO schema_version (version)
        SELECT 0
        WHERE NOT EXISTS (SELECT 1 FROM schema_version);

        CREATE TABLE question (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            state TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            deleted_at INTEGER
        );

        CREATE TABLE review (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            reviewed_at INTEGER NOT NULL,
            result TEXT NOT NULL,
            FOREIGN KEY(question_id) REFERENCES question(id)
        );

        CREATE TABLE asset (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            deleted_at INTEGER,
            FOREIGN KEY(question_id) REFERENCES question(id)
        );

        CREATE TABLE meta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            FOREIGN KEY(question_id) REFERENCES question(id)
        );
        "#,
    },
    Migration {
        version: 2,
        name: "show_view",
        sql: r#"
        CREATE VIEW IF NOT EXISTS show_view AS
            SELECT
                q.id,
                q.name,
                q.state,
                q.created_at,
                q.deleted_at,
                m.value AS subject,
                r.last_reviewed_at AS last_reviewed_at
            FROM question q
            LEFT JOIN meta m
            ON m.question_id = q.id
            AND m.key = 'system.Subject'
            LEFT JOIN (
                SELECT question_id, MAX(reviewed_at) AS last_reviewed_at
                FROM review
                GROUP BY question_id
            ) r ON r.question_id = q.id
        "#,
    },
    Migration {
        version: 3,
        name: "add_review_fields",
        sql: r#"
        ALTER TABLE question ADD COLUMN last_review_at INTEGER;
        ALTER TABLE question ADD COLUMN last_result TEXT;
        ALTER TABLE question ADD COLUMN correct_streak INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE question ADD COLUMN wrong_count INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE question ADD COLUMN due_at INTEGER;
        "#,
    },
    Migration {
        version: 4,
        name: "recommendation_engine",
        sql: r#"
        CREATE TABLE IF NOT EXISTS recommendation (
            day INTEGER NOT NULL,
            question_id INTEGER NOT NULL,
            score REAL NOT NULL,
            PRIMARY KEY(day, question_id),
            FOREIGN KEY(question_id) REFERENCES question(id)
        );

        CREATE VIEW IF NOT EXISTS review_summary AS
            SELECT
                question_id,
                COUNT(*) AS review_count,
                MAX(reviewed_at) AS last_reviewed_at,
                SUM(CASE WHEN result != 'correct' THEN 1 ELSE 0 END) * 1.0 / COUNT(*) AS error_rate
            FROM review
            GROUP BY question_id;
        "#,
    },
    Migration {
        version: 5,
        name: "add_subject_to_recommendation",
        sql: r#"
        ALTER TABLE recommendation ADD COLUMN subject TEXT;
        "#,
    },
    Migration {
        version: 6,
        name: "add_asset_sort_order",
        sql: r#"
        ALTER TABLE asset ADD COLUMN sort_order INTEGER;
        "#,
    },
    Migration {
        version: 7,
        name: "set_asset_sort_order_from_created_at",
        sql: r#"
        UPDATE asset SET sort_order = (
            SELECT COUNT(*) FROM asset AS a
            WHERE a.question_id = asset.question_id
            AND a.type = asset.type
            AND a.created_at < asset.created_at
        ) + (
            SELECT COUNT(*) FROM asset AS a
            WHERE a.question_id = asset.question_id
            AND a.type = asset.type
            AND a.created_at = asset.created_at
            AND a.id < asset.id
        ) + 1;
        "#,
    },
    Migration {
        version: 8,
        name: "add_reason_to_recommendation",
        sql: r#"
        ALTER TABLE recommendation ADD COLUMN reason TEXT;
        "#,
    },
    Migration {
        version: 9,
        name: "fix_review_summary_case_sensitivity",
        sql: r#"
        DROP VIEW IF EXISTS review_summary;
        CREATE VIEW review_summary AS
            SELECT
                question_id,
                COUNT(*) AS review_count,
                MAX(reviewed_at) AS last_reviewed_at,
                SUM(CASE WHEN LOWER(result) != 'correct' THEN 1 ELSE 0 END) * 1.0 / COUNT(*) AS error_rate
            FROM review
            GROUP BY question_id;
        "#,
    },
    Migration {
        version: 10,
        name: "drop_due_state",
        sql: r#"
        -- DUE 状态被废弃：推荐算法按 score 选题，不再依赖 state=DUE。
        -- 将存量 state='DUE' 转为 'STABLE'，与新行为保持一致。
        UPDATE question SET state = 'STABLE' WHERE state = 'DUE';
        "#,
    },
    Migration {
        version: 11,
        name: "show_view_add_wrong_count_and_error_rate",
        sql: r#"
        -- 为支持"高频错题"过滤，给 show_view 加 q.wrong_count 与
        -- 来自 review_summary 视图的 error_rate。
        -- 不能 ALTER VIEW，先 DROP 后 CREATE。
        DROP VIEW IF EXISTS show_view;
        CREATE VIEW show_view AS
            SELECT
                q.id,
                q.name,
                q.state,
                q.created_at,
                q.deleted_at,
                m.value AS subject,
                r.last_reviewed_at AS last_reviewed_at,
                q.wrong_count AS wrong_count,
                rs.error_rate AS error_rate
            FROM question q
            LEFT JOIN meta m
                ON m.question_id = q.id
                AND m.key = 'system.Subject'
            LEFT JOIN (
                SELECT question_id, MAX(reviewed_at) AS last_reviewed_at
                FROM review
                GROUP BY question_id
            ) r ON r.question_id = q.id
            LEFT JOIN review_summary rs ON rs.question_id = q.id;
        "#,
    },
];

/*
    启动时调用，检查当前 schema 版本，依次应用未应用的迁移
    输入：
    - conn: 可变数据库连接引用
*/
pub fn migrate(conn: &mut Connection) -> Result<()> {
    ensure_schema_version_table(conn)?;

    let current_version = get_current_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version > current_version {
            apply_migration(conn, migration)?;
        }
    }

    Ok(())
}

// 工具函数，确保 schema_version 表存在
fn ensure_schema_version_table(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );
        "#,
        [],
    )?;
    Ok(())
}

// 工具函数，读取当前 schema version
fn get_current_version(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT version FROM schema_version LIMIT 1")?;

    let version = stmt.query_row([], |row| row.get(0)).unwrap_or(0);

    Ok(version)
}

// 工具函数，执行单条迁移
fn apply_migration(conn: &mut Connection, migration: &Migration) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute_batch(migration.sql)?;

    tx.execute(
        "UPDATE schema_version SET version = ?1",
        [migration.version],
    )?;

    tx.commit()?;

    Ok(())
}
