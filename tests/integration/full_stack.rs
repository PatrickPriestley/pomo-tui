use pomo_tui::tui::app::App;
use pomo_tui::core::{TaskManager, Timer};
use pomo_tui::database::models::{Task, Session, Preferences, Statistics};
use pomo_tui::core::performance::{PerformanceMonitor, MemoryMonitor, PerformanceBenchmark};
use sqlx::SqlitePool;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;

/// Full stack integration test that verifies all acceptance criteria
/// from the quickstart.md scenarios are working correctly.
#[tokio::test]
async fn test_quickstart_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("quickstart_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Scenario 1: Create first task and start session
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    // Create task as in quickstart
    let task = task_manager.create_task(pomo_tui::database::models::TaskInput {
        title: "Complete project documentation".to_string(),
        description: Some("Write comprehensive docs".to_string()),
        priority: 5,
        status: "pending".to_string(),
        tags: vec!["docs".to_string(), "urgent".to_string()],
        estimated_sessions: Some(3),
        project: Some("work".to_string()),
        due_date: None,
    }).await?;
    
    assert_eq!(task.title, "Complete project documentation");
    assert_eq!(task.priority, 5);
    assert_eq!(task.status, "pending");
    
    // Start session as in quickstart
    app.send_message(pomo_tui::tui::app::Message::StartSession {
        task_id: task.id,
        duration_minutes: 25,
    })?;
    app.process_messages().await?;
    
    // Verify session is active
    let active_session = Session::get_active(&pool).await?;
    assert!(active_session.is_some());
    let session = active_session.unwrap();
    assert_eq!(session.task_id, task.id);
    assert_eq!(session.duration_minutes, 25);
    
    // Scenario 2: Complete session and verify statistics
    app.send_message(pomo_tui::tui::app::Message::CompleteSession)?;
    app.process_messages().await?;
    
    let completed_session = Session::find(&pool, session.id).await?;
    assert_eq!(completed_session.status, "completed");
    
    // Verify statistics updated
    let today = chrono::Utc::now().naive_utc().date();
    let stats = Statistics::calculate_for_date(&pool, today).await?;
    assert_eq!(stats.completed_sessions, 1);
    assert!(stats.productivity_score > 0.0);
    
    // Scenario 3: Test TUI navigation (simulated)
    // In a real TUI test, we'd send keyboard events
    // Here we test the state transitions
    use pomo_tui::tui::app::{AppMode, CurrentTab};
    
    app.send_message(pomo_tui::tui::app::Message::SwitchTab(CurrentTab::Tasks))?;
    app.process_messages().await?;
    assert_eq!(app.state.current_tab, CurrentTab::Tasks);
    
    app.send_message(pomo_tui::tui::app::Message::SwitchTab(CurrentTab::Statistics))?;
    app.process_messages().await?;
    assert_eq!(app.state.current_tab, CurrentTab::Statistics);
    
    // Scenario 4: Test preferences management
    let original_prefs = Preferences::get_or_create(&pool).await?;
    
    app.send_message(pomo_tui::tui::app::Message::UpdatePreferences {
        session_duration: 30,
        short_break_duration: 7,
        long_break_duration: 20,
        sessions_until_long_break: 3,
        auto_start_breaks: true,
        auto_start_sessions: false,
        sound_enabled: true,
        volume: 0.8,
        ambient_sound: Some("brown_noise".to_string()),
        notification_sound: Some("chime".to_string()),
        visual_notifications: true,
        desktop_notifications: false,
        github_integration: false,
        slack_integration: false,
        git_commit_enhancement: true,
        website_blocking: false,
        theme: "dark".to_string(),
        show_seconds: true,
        show_session_count: true,
        show_daily_goal: true,
        daily_session_goal: 8,
        blocked_websites: vec!["facebook.com".to_string()],
    })?;
    app.process_messages().await?;
    
    let updated_prefs = Preferences::get_or_create(&pool).await?;
    assert_eq!(updated_prefs.session_duration, 30);
    assert_eq!(updated_prefs.volume, 0.8);
    assert_eq!(updated_prefs.theme, "dark");
    
    Ok(())
}

/// Test all performance requirements are met
#[tokio::test]
async fn test_performance_requirements() -> Result<(), Box<dyn std::error::Error>> {
    // Test 1: Startup time < 50ms
    let startup_start = Instant::now();
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("perf_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let mut performance_monitor = PerformanceMonitor::new();
    let _app = App::new(pool.clone()).await?;
    let startup_time = performance_monitor.mark_startup_complete().await;
    
    // Requirement: startup time < 50ms
    assert!(
        startup_time < Duration::from_millis(50),
        "Startup time {}ms exceeds 50ms requirement",
        startup_time.as_millis()
    );
    
    // Test 2: Memory usage < 50MB
    let memory_check = MemoryMonitor::check_memory_limits();
    assert!(
        memory_check.within_limits,
        "Memory usage {:.1}MB exceeds {:.1}MB limit",
        memory_check.current_mb,
        memory_check.limit_mb
    );
    
    // Test 3: Timer precision < 100ms drift over 25 minutes
    // We'll test with a shorter duration but equivalent precision
    let precision_result = PerformanceBenchmark::benchmark_timer_precision(1).await;
    assert!(
        precision_result.within_tolerance,
        "Timer drift {}ms exceeds 100ms requirement", 
        precision_result.drift.as_millis()
    );
    
    // Test 4: Database performance
    let db_benchmark = PerformanceBenchmark::benchmark_database_ops(&pool).await;
    let avg_duration = db_benchmark.average_duration();
    assert!(
        avg_duration < Duration::from_millis(10),
        "Average database operation {}ms too slow",
        avg_duration.as_millis()
    );
    
    Ok(())
}

/// Test all keyboard shortcuts work correctly
#[tokio::test]
async fn test_keyboard_shortcuts() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("keyboard_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    // Create test task
    let task = task_manager.create_task(pomo_tui::database::models::TaskInput {
        title: "Keyboard Test Task".to_string(),
        description: None,
        priority: 3,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    }).await?;
    
    // Test task selection and session start (simulated 's' key)
    app.send_message(pomo_tui::tui::app::Message::SelectTask(task.id))?;
    app.process_messages().await?;
    
    app.send_message(pomo_tui::tui::app::Message::StartSession {
        task_id: task.id,
        duration_minutes: 25,
    })?;
    app.process_messages().await?;
    
    // Verify session started
    let active_session = Session::get_active(&pool).await?;
    assert!(active_session.is_some());
    
    // Test pause/resume (simulated 'p' key)
    app.send_message(pomo_tui::tui::app::Message::PauseSession)?;
    app.process_messages().await?;
    
    let paused_session = Session::find(&pool, active_session.unwrap().id).await?;
    assert_eq!(paused_session.status, "paused");
    
    app.send_message(pomo_tui::tui::app::Message::ResumeSession)?;
    app.process_messages().await?;
    
    let resumed_session = Session::find(&pool, paused_session.id).await?;
    assert_eq!(resumed_session.status, "active");
    
    // Test tab navigation (simulated Tab key)
    use pomo_tui::tui::app::CurrentTab;
    
    app.send_message(pomo_tui::tui::app::Message::SwitchTab(CurrentTab::Timer))?;
    app.process_messages().await?;
    assert_eq!(app.state.current_tab, CurrentTab::Timer);
    
    app.send_message(pomo_tui::tui::app::Message::SwitchTab(CurrentTab::Tasks))?;
    app.process_messages().await?;
    assert_eq!(app.state.current_tab, CurrentTab::Tasks);
    
    app.send_message(pomo_tui::tui::app::Message::SwitchTab(CurrentTab::Statistics))?;
    app.process_messages().await?;
    assert_eq!(app.state.current_tab, CurrentTab::Statistics);
    
    app.send_message(pomo_tui::tui::app::Message::SwitchTab(CurrentTab::Settings))?;
    app.process_messages().await?;
    assert_eq!(app.state.current_tab, CurrentTab::Settings);
    
    // Test session abandon (simulated 'a' key)
    app.send_message(pomo_tui::tui::app::Message::AbandonSession)?;
    app.process_messages().await?;
    
    let abandoned_session = Session::find(&pool, resumed_session.id).await?;
    assert_eq!(abandoned_session.status, "abandoned");
    
    Ok(())
}

/// Test export functionality works correctly
#[tokio::test]
async fn test_export_formats_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("export_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create test data
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    let task = task_manager.create_task(pomo_tui::database::models::TaskInput {
        title: "Export Test Task".to_string(),
        description: Some("Task for export testing".to_string()),
        priority: 4,
        status: "pending".to_string(),
        tags: vec!["export".to_string(), "test".to_string()],
        estimated_sessions: Some(2),
        project: Some("testing".to_string()),
        due_date: None,
    }).await?;
    
    // Complete a session to have data to export
    app.send_message(pomo_tui::tui::app::Message::StartSession {
        task_id: task.id,
        duration_minutes: 25,
    })?;
    app.process_messages().await?;
    
    app.send_message(pomo_tui::tui::app::Message::CompleteSession)?;
    app.process_messages().await?;
    
    // Test JSON export
    let json_export_path = temp_dir.path().join("test_export.json");
    app.send_message(pomo_tui::tui::app::Message::ExportData {
        format: pomo_tui::core::export::ExportFormat::Json,
        path: json_export_path.clone(),
        date_range: None,
        include_preferences: true,
    })?;
    app.process_messages().await?;
    
    // Verify JSON export exists and is valid
    assert!(json_export_path.exists());
    let json_content = std::fs::read_to_string(&json_export_path)?;
    let json_data: serde_json::Value = serde_json::from_str(&json_content)?;
    
    assert!(json_data.get("tasks").is_some());
    assert!(json_data.get("sessions").is_some());
    assert!(json_data.get("preferences").is_some());
    
    // Test CSV export
    let csv_export_path = temp_dir.path().join("test_export.csv");
    app.send_message(pomo_tui::tui::app::Message::ExportData {
        format: pomo_tui::core::export::ExportFormat::Csv,
        path: csv_export_path.clone(),
        date_range: None,
        include_preferences: false,
    })?;
    app.process_messages().await?;
    
    // Verify CSV export exists
    assert!(csv_export_path.exists());
    let csv_content = std::fs::read_to_string(&csv_export_path)?;
    assert!(csv_content.contains("## Tasks"));
    assert!(csv_content.contains("## Sessions"));
    
    Ok(())
}

/// Test integration system works correctly
#[tokio::test]
async fn test_integration_system() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("integration_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let mut app = App::new(pool.clone()).await?;
    
    // Test git commit enhancement (mock)
    app.send_message(pomo_tui::tui::app::Message::UpdatePreferences {
        session_duration: 25,
        short_break_duration: 5,
        long_break_duration: 15,
        sessions_until_long_break: 4,
        auto_start_breaks: true,
        auto_start_sessions: false,
        sound_enabled: true,
        volume: 0.7,
        ambient_sound: None,
        notification_sound: None,
        visual_notifications: true,
        desktop_notifications: false,
        github_integration: false,
        slack_integration: false,
        git_commit_enhancement: true, // Enable git integration
        website_blocking: false,
        theme: "dark".to_string(),
        show_seconds: true,
        show_session_count: true,
        show_daily_goal: true,
        daily_session_goal: 8,
        blocked_websites: vec![],
    })?;
    app.process_messages().await?;
    
    let prefs = Preferences::get_or_create(&pool).await?;
    assert!(prefs.git_commit_enhancement);
    
    // Test website blocking configuration (without actually modifying hosts file)
    app.send_message(pomo_tui::tui::app::Message::UpdatePreferences {
        session_duration: 25,
        short_break_duration: 5,
        long_break_duration: 15,
        sessions_until_long_break: 4,
        auto_start_breaks: true,
        auto_start_sessions: false,
        sound_enabled: true,
        volume: 0.7,
        ambient_sound: None,
        notification_sound: None,
        visual_notifications: true,
        desktop_notifications: false,
        github_integration: false,
        slack_integration: false,
        git_commit_enhancement: true,
        website_blocking: true, // Enable website blocking
        theme: "dark".to_string(),
        show_seconds: true,
        show_session_count: true,
        show_daily_goal: true,
        daily_session_goal: 8,
        blocked_websites: vec!["facebook.com".to_string(), "twitter.com".to_string()],
    })?;
    app.process_messages().await?;
    
    let updated_prefs = Preferences::get_or_create(&pool).await?;
    assert!(updated_prefs.website_blocking);
    assert_eq!(updated_prefs.blocked_websites, vec!["facebook.com", "twitter.com"]);
    
    Ok(())
}

/// Test error handling and recovery scenarios
#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("error_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let mut app = App::new(pool.clone()).await?;
    
    // Test starting session with invalid task ID
    let result = app.send_message(pomo_tui::tui::app::Message::StartSession {
        task_id: 99999, // Non-existent task
        duration_minutes: 25,
    });
    
    // Should handle gracefully (either return error or ignore)
    if result.is_ok() {
        app.process_messages().await?;
        // Should not have created a session
        let active_session = Session::get_active(&pool).await?;
        assert!(active_session.is_none());
    }
    
    // Test invalid preferences
    let invalid_prefs_result = app.send_message(pomo_tui::tui::app::Message::UpdatePreferences {
        session_duration: 0, // Invalid duration
        short_break_duration: 5,
        long_break_duration: 15,
        sessions_until_long_break: 4,
        auto_start_breaks: true,
        auto_start_sessions: false,
        sound_enabled: true,
        volume: 2.0, // Invalid volume (> 1.0)
        ambient_sound: None,
        notification_sound: None,
        visual_notifications: true,
        desktop_notifications: false,
        github_integration: false,
        slack_integration: false,
        git_commit_enhancement: false,
        website_blocking: false,
        theme: "invalid_theme".to_string(), // Invalid theme
        show_seconds: true,
        show_session_count: true,
        show_daily_goal: true,
        daily_session_goal: 8,
        blocked_websites: vec![],
    });
    
    // Should handle validation errors gracefully
    if invalid_prefs_result.is_ok() {
        app.process_messages().await?;
        
        // Preferences should remain unchanged or use defaults
        let prefs = Preferences::get_or_create(&pool).await?;
        assert!(prefs.session_duration >= 15); // Should use valid default
        assert!(prefs.volume >= 0.0 && prefs.volume <= 1.0); // Should use valid default
    }
    
    Ok(())
}

/// Test concurrent access scenarios
#[tokio::test]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("concurrent_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create multiple app instances to simulate concurrent access
    let mut app1 = App::new(pool.clone()).await?;
    let mut app2 = App::new(pool.clone()).await?;
    let task_manager1 = TaskManager::new(pool.clone()).await?;
    let task_manager2 = TaskManager::new(pool.clone()).await?;
    
    // Create tasks concurrently
    let (task1, task2) = tokio::try_join!(
        task_manager1.create_task(pomo_tui::database::models::TaskInput {
            title: "Concurrent Task 1".to_string(),
            description: None,
            priority: 1,
            status: "pending".to_string(),
            tags: vec!["concurrent".to_string()],
            estimated_sessions: None,
            project: None,
            due_date: None,
        }),
        task_manager2.create_task(pomo_tui::database::models::TaskInput {
            title: "Concurrent Task 2".to_string(),
            description: None,
            priority: 2,
            status: "pending".to_string(),
            tags: vec!["concurrent".to_string()],
            estimated_sessions: None,
            project: None,
            due_date: None,
        })
    )?;
    
    assert_ne!(task1.id, task2.id);
    assert_eq!(task1.title, "Concurrent Task 1");
    assert_eq!(task2.title, "Concurrent Task 2");
    
    // Attempt to start sessions simultaneously (should handle gracefully)
    let session1_future = async {
        app1.send_message(pomo_tui::tui::app::Message::StartSession {
            task_id: task1.id,
            duration_minutes: 25,
        })?;
        app1.process_messages().await
    };
    
    let session2_future = async {
        // Small delay to avoid exact simultaneity
        tokio::time::sleep(Duration::from_millis(1)).await;
        app2.send_message(pomo_tui::tui::app::Message::StartSession {
            task_id: task2.id,
            duration_minutes: 25,
        })?;
        app2.process_messages().await
    };
    
    // Both should complete without error
    tokio::try_join!(session1_future, session2_future)?;
    
    // Only one session should be active (business rule enforcement)
    let active_sessions = sqlx::query!("SELECT COUNT(*) as count FROM sessions WHERE status = 'active'")
        .fetch_one(&pool)
        .await?;
    
    assert!(active_sessions.count <= 1, "Multiple active sessions detected");
    
    Ok(())
}

/// Test memory leak detection over extended use
#[tokio::test] 
async fn test_memory_leak_detection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("memory_test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let initial_memory = MemoryMonitor::get_memory_usage();
    
    // Perform many operations to detect potential leaks
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    // Create and complete many sessions
    for i in 0..50 {
        let task = task_manager.create_task(pomo_tui::database::models::TaskInput {
            title: format!("Memory Test Task {}", i),
            description: None,
            priority: 1,
            status: "pending".to_string(),
            tags: vec![],
            estimated_sessions: None,
            project: None,
            due_date: None,
        }).await?;
        
        app.send_message(pomo_tui::tui::app::Message::StartSession {
            task_id: task.id,
            duration_minutes: 1, // Short session for testing
        })?;
        app.process_messages().await?;
        
        app.send_message(pomo_tui::tui::app::Message::CompleteSession)?;
        app.process_messages().await?;
        
        // Periodically check memory growth
        if i % 10 == 0 {
            let current_memory = MemoryMonitor::get_memory_usage();
            let memory_growth = current_memory.current_rss.saturating_sub(initial_memory.current_rss);
            
            // Memory growth should be reasonable (< 10MB for 50 operations)
            assert!(
                memory_growth < 10 * 1024 * 1024,
                "Excessive memory growth detected: {} bytes after {} operations",
                memory_growth,
                i + 1
            );
        }
    }
    
    let final_memory = MemoryMonitor::get_memory_usage();
    let total_growth = final_memory.current_rss.saturating_sub(initial_memory.current_rss);
    
    // Total memory growth should be reasonable
    assert!(
        total_growth < 20 * 1024 * 1024,
        "Total memory growth {} MB exceeds 20MB limit",
        total_growth / (1024 * 1024)
    );
    
    Ok(())
}

/// Integration test timeout wrapper
async fn with_timeout<F, T>(future: F, duration: Duration) -> Result<T, Box<dyn std::error::Error>>
where
    F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    match timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err("Test timed out".into()),
    }
}

/// Test all quickstart scenarios with timeout protection
#[tokio::test]
async fn test_full_stack_with_timeout() {
    let result = with_timeout(
        test_quickstart_scenarios(),
        Duration::from_secs(30)
    ).await;
    
    assert!(result.is_ok(), "Full stack test failed or timed out: {:?}", result.err());
}

/// Comprehensive performance validation
#[tokio::test]
async fn test_performance_with_timeout() {
    let result = with_timeout(
        test_performance_requirements(),
        Duration::from_secs(10)
    ).await;
    
    assert!(result.is_ok(), "Performance test failed or timed out: {:?}", result.err());
}