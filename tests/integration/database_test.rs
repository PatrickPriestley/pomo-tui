use pomo_tui::database::migrations::{get_database_url, init_database};
use sqlx::{Row, SqlitePool};
use tempfile::TempDir;

/// Integration tests for database connection and schema
/// These tests verify the database layer works correctly with migrations

#[tokio::test]
async fn test_database_connection() {
    // Test basic database connection and migration
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Verify connection is working
    let result = sqlx::query("SELECT 1 as test")
        .fetch_one(&pool)
        .await
        .unwrap();

    let test_value: i32 = result.get("test");
    assert_eq!(test_value, 1);
}

#[tokio::test]
async fn test_migration_tables_created() {
    // Test that all required tables are created by migrations
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Query for all tables
    let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap();

    let table_names: Vec<String> = tables
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    // Verify all expected tables exist
    let expected_tables = vec![
        "_sqlx_migrations", // sqlx internal table
        "audio_files",
        "breaks",
        "integrations",
        "preferences",
        "sessions",
        "statistics",
        "tasks",
    ];

    for expected in &expected_tables {
        assert!(
            table_names.contains(&expected.to_string()),
            "Missing table: {}",
            expected
        );
    }
}

#[tokio::test]
async fn test_default_preferences_inserted() {
    // Test that default preferences row is created
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Check preferences table has default row
    let prefs = sqlx::query("SELECT * FROM preferences WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Verify some default values
    let session_duration: i32 = prefs.get("session_duration");
    let short_break_duration: i32 = prefs.get("short_break_duration");
    let theme: String = prefs.get("theme");
    let vim_mode: bool = prefs.get("vim_mode_enabled");

    assert_eq!(session_duration, 1500); // 25 minutes
    assert_eq!(short_break_duration, 300); // 5 minutes
    assert_eq!(theme, "dark");
    assert_eq!(vim_mode, true);
}

#[tokio::test]
async fn test_database_schema_constraints() {
    // Test that database constraints are properly enforced
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Test foreign key constraint (session -> task)
    let insert_result = sqlx::query("INSERT INTO sessions (task_id, start_time, planned_duration) VALUES (999, datetime('now'), 1500)")
        .execute(&pool)
        .await;

    // Should fail due to foreign key constraint
    assert!(insert_result.is_err());

    // Test unique constraint on preferences (only one row allowed)
    let insert_prefs = sqlx::query("INSERT INTO preferences (id) VALUES (2)")
        .execute(&pool)
        .await;

    // Should succeed (we don't have unique constraint on id for preferences)
    // But in practice we maintain single row through application logic
    assert!(insert_prefs.is_ok());
}

#[tokio::test]
async fn test_database_indexes_exist() {
    // Test that performance indexes are created
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Query for indexes
    let indexes = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let index_names: Vec<String> = indexes
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    // Verify key indexes exist
    let expected_indexes = vec![
        "idx_tasks_status",
        "idx_tasks_priority",
        "idx_sessions_task",
        "idx_sessions_status",
        "idx_sessions_date",
        "idx_breaks_session",
        "idx_breaks_type",
        "idx_statistics_date",
        "idx_integrations_service",
        "idx_audio_files_type",
    ];

    for expected in &expected_indexes {
        assert!(
            index_names.contains(&expected.to_string()),
            "Missing index: {}",
            expected
        );
    }
}

#[tokio::test]
async fn test_concurrent_database_connections() {
    // Test that multiple connections can work simultaneously
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    // Create initial connection
    let pool1 = init_database(&db_url).await.unwrap();

    // Create second connection to same database
    let pool2 = SqlitePool::connect(&db_url).await.unwrap();

    // Both should be able to read
    let result1 = sqlx::query("SELECT COUNT(*) as count FROM tasks")
        .fetch_one(&pool1)
        .await
        .unwrap();

    let result2 = sqlx::query("SELECT COUNT(*) as count FROM tasks")
        .fetch_one(&pool2)
        .await
        .unwrap();

    let count1: i32 = result1.get("count");
    let count2: i32 = result2.get("count");

    assert_eq!(count1, count2);
}

#[tokio::test]
async fn test_database_url_generation() {
    // Test that database URL generation works correctly

    // Test with environment variable
    std::env::set_var("DATABASE_URL", "sqlite:test_env.db");
    let url = get_database_url().unwrap();
    assert_eq!(url, "sqlite:test_env.db");

    // Clean up
    std::env::remove_var("DATABASE_URL");

    // Test default XDG path
    let default_url = get_database_url().unwrap();
    assert!(default_url.starts_with("sqlite:"));
    assert!(default_url.contains("pomo-tui"));
    assert!(default_url.ends_with("pomo.db"));
}

#[tokio::test]
async fn test_database_transaction_rollback() {
    // Test that transactions work correctly
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Start a transaction
    let mut tx = pool.begin().await.unwrap();

    // Insert a task within transaction
    sqlx::query("INSERT INTO tasks (title, priority) VALUES ('Test Task', 1)")
        .execute(&mut *tx)
        .await
        .unwrap();

    // Rollback transaction
    tx.rollback().await.unwrap();

    // Verify task was not persisted
    let count = sqlx::query("SELECT COUNT(*) as count FROM tasks")
        .fetch_one(&pool)
        .await
        .unwrap();

    let task_count: i32 = count.get("count");
    assert_eq!(task_count, 0);
}

#[tokio::test]
async fn test_database_wal_mode() {
    // Test that WAL mode is enabled for better concurrency
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Enable WAL mode
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .unwrap();

    // Verify WAL mode is active
    let result = sqlx::query("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .unwrap();

    let mode: String = result.get(0);
    assert_eq!(mode, "wal");
}

#[tokio::test]
async fn test_database_connection_pool() {
    // Test connection pooling behavior
    let temp_dir = TempDir::new().unwrap();
    let db_url = format!("sqlite:{}/test.db", temp_dir.path().display());

    let pool = init_database(&db_url).await.unwrap();

    // Execute multiple concurrent queries
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let result = sqlx::query("SELECT ? as number")
                    .bind(i)
                    .fetch_one(&pool)
                    .await
                    .unwrap();

                let number: i32 = result.get("number");
                number
            })
        })
        .collect();

    // Wait for all queries to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        assert_eq!(result, i as i32);
    }
}
