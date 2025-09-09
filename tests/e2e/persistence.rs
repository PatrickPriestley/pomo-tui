use pomo_tui::core::TaskManager;
use pomo_tui::database::models::{Task, Session, Preferences};
use pomo_tui::tui::app::{App, AppState, Message, AppMode, SessionRecoveryState};
use sqlx::SqlitePool;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// End-to-End test for data persistence and session recovery:
/// 1. Start a session
/// 2. Simulate application crash/restart
/// 3. Verify session recovery prompt is shown
/// 4. Test recovery options (resume, abandon, complete)
#[tokio::test]
async fn test_session_recovery_after_restart() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_persistence.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    // Setup database (this persists across "restarts")
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // First application instance
    {
        let task_manager = TaskManager::new(pool.clone()).await?;
        let mut app = App::new(pool.clone()).await?;
        
        // Create a task
        let task_input = pomo_tui::database::models::TaskInput {
            title: "Persistence Test Task".to_string(),
            description: Some("Task to test session recovery".to_string()),
            priority: 2,
            status: "pending".to_string(),
            tags: vec!["recovery".to_string()],
            estimated_sessions: Some(2),
            project: Some("testing".to_string()),
            due_date: None,
        };
        
        let task = task_manager.create_task(task_input).await?;
        
        // Start a session
        app.send_message(Message::StartSession { 
            task_id: task.id,
            duration_minutes: 25,
        })?;
        app.process_messages().await?;
        
        // Verify session is active
        let active_session = Session::get_active(&pool).await?;
        assert!(active_session.is_some());
        let session = active_session.unwrap();
        assert_eq!(session.status, "active");
        assert_eq!(session.task_id, task.id);
        
        // Simulate some work time
        sleep(Duration::from_millis(100)).await;
        
        // Simulate application crash (app goes out of scope)
        // Session remains "active" in database
    }
    
    // Second application instance (simulating restart)
    {
        let mut app_restarted = App::new(pool.clone()).await?;
        
        // App should detect unfinished session during initialization
        assert!(matches!(app_restarted.state.session_recovery, Some(SessionRecoveryState::PendingRecovery(_))));
        
        // Verify recovery state contains session info
        if let Some(SessionRecoveryState::PendingRecovery(recovery_session)) = &app_restarted.state.session_recovery {
            assert_eq!(recovery_session.status, "active");
            assert_eq!(recovery_session.duration_minutes, 25);
        } else {
            panic!("Expected pending recovery state");
        }
        
        // Test resume option
        app_restarted.send_message(Message::ResumeRecoveredSession)?;
        app_restarted.process_messages().await?;
        
        // Verify session resumed
        assert!(matches!(app_restarted.state.current_mode, AppMode::Session));
        assert!(app_restarted.state.current_session.is_some());
        assert!(app_restarted.state.timer.is_some());
        assert!(app_restarted.state.session_recovery.is_none());
        
        // Complete the session
        app_restarted.send_message(Message::CompleteSession)?;
        app_restarted.process_messages().await?;
        
        let completed_session = Session::get_latest(&pool).await?.unwrap();
        assert_eq!(completed_session.status, "completed");
    }
    
    Ok(())
}

/// Test session recovery with abandon option
#[tokio::test]
async fn test_session_recovery_with_abandon() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_abandon_recovery.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // First instance - start session then "crash"
    {
        let task_manager = TaskManager::new(pool.clone()).await?;
        let mut app = App::new(pool.clone()).await?;
        
        let task_input = pomo_tui::database::models::TaskInput {
            title: "Abandon Recovery Test".to_string(),
            description: None,
            priority: 1,
            status: "pending".to_string(),
            tags: vec![],
            estimated_sessions: None,
            project: None,
            due_date: None,
        };
        
        let task = task_manager.create_task(task_input).await?;
        
        app.send_message(Message::StartSession { 
            task_id: task.id,
            duration_minutes: 25,
        })?;
        app.process_messages().await?;
    }
    
    // Second instance - abandon recovered session
    {
        let mut app = App::new(pool.clone()).await?;
        
        // Verify recovery state
        assert!(app.state.session_recovery.is_some());
        
        // Abandon the recovered session
        app.send_message(Message::AbandonRecoveredSession)?;
        app.process_messages().await?;
        
        // Verify session was abandoned
        let abandoned_session = Session::get_latest(&pool).await?.unwrap();
        assert_eq!(abandoned_session.status, "abandoned");
        
        // Verify app state is clean
        assert!(app.state.current_session.is_none());
        assert!(app.state.timer.is_none());
        assert!(app.state.session_recovery.is_none());
        assert!(matches!(app.state.current_mode, AppMode::Tasks));
    }
    
    Ok(())
}

/// Test preferences persistence across restarts
#[tokio::test]
async fn test_preferences_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_preferences.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // First instance - modify preferences
    {
        let mut app = App::new(pool.clone()).await?;
        
        // Update preferences
        app.send_message(Message::UpdatePreferences {
            session_duration: 30,
            short_break_duration: 10,
            long_break_duration: 20,
            sessions_until_long_break: 3,
            auto_start_breaks: false,
            auto_start_sessions: true,
            sound_enabled: false,
            volume: 0.5,
            ambient_sound: Some("brown_noise".to_string()),
            notification_sound: Some("bell".to_string()),
            visual_notifications: true,
            desktop_notifications: false,
            github_integration: true,
            slack_integration: false,
            git_commit_enhancement: true,
            website_blocking: false,
            theme: "dark".to_string(),
            show_seconds: false,
            show_session_count: true,
            show_daily_goal: true,
            daily_session_goal: 8,
            blocked_websites: vec!["social.com".to_string()],
        })?;
        app.process_messages().await?;
        
        // Verify preferences saved
        let prefs = Preferences::get_or_create(&pool).await?;
        assert_eq!(prefs.session_duration, 30);
        assert_eq!(prefs.short_break_duration, 10);
        assert_eq!(prefs.sessions_until_long_break, 3);
        assert!(!prefs.auto_start_breaks);
        assert!(prefs.auto_start_sessions);
    }
    
    // Second instance - verify preferences loaded
    {
        let app = App::new(pool.clone()).await?;
        
        // Check that app loaded the custom preferences
        let loaded_prefs = Preferences::get_or_create(&pool).await?;
        assert_eq!(loaded_prefs.session_duration, 30);
        assert_eq!(loaded_prefs.short_break_duration, 10);
        assert_eq!(loaded_prefs.sessions_until_long_break, 3);
        assert!(!loaded_prefs.auto_start_breaks);
        assert!(loaded_prefs.auto_start_sessions);
        assert!(!loaded_prefs.sound_enabled);
        assert_eq!(loaded_prefs.volume, 0.5);
        assert_eq!(loaded_prefs.ambient_sound.as_deref(), Some("brown_noise"));
        assert!(loaded_prefs.github_integration);
        assert!(!loaded_prefs.slack_integration);
        assert_eq!(loaded_prefs.daily_session_goal, 8);
        assert_eq!(loaded_prefs.blocked_websites, vec!["social.com"]);
    }
    
    Ok(())
}

/// Test task persistence and state consistency
#[tokio::test]
async fn test_task_persistence_across_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_task_persistence.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let task_id;
    
    // First instance - create and partially work on task
    {
        let task_manager = TaskManager::new(pool.clone()).await?;
        let mut app = App::new(pool.clone()).await?;
        
        // Create task
        let task_input = pomo_tui::database::models::TaskInput {
            title: "Multi-session Task".to_string(),
            description: Some("Task that spans multiple app sessions".to_string()),
            priority: 3,
            status: "pending".to_string(),
            tags: vec!["persistence".to_string(), "multi-session".to_string()],
            estimated_sessions: Some(3),
            project: Some("test-project".to_string()),
            due_date: Some(chrono::Utc::now().naive_utc().date() + chrono::Duration::days(7)),
        };
        
        let task = task_manager.create_task(task_input).await?;
        task_id = task.id;
        
        // Complete one session
        app.send_message(Message::StartSession { 
            task_id: task.id,
            duration_minutes: 1,
        })?;
        app.process_messages().await?;
        
        app.send_message(Message::CompleteSession)?;
        app.process_messages().await?;
        
        // Verify task is now "in_progress"
        let task_after_session = Task::find(&pool, task.id).await?;
        assert_eq!(task_after_session.status, "in_progress");
        assert!(task_after_session.started_at.is_some());
        assert_eq!(task_after_session.completed_sessions, 1);
    }
    
    // Second instance - continue working on same task
    {
        let task_manager = TaskManager::new(pool.clone()).await?;
        let mut app = App::new(pool.clone()).await?;
        
        // Load task and verify its state
        let task = Task::find(&pool, task_id).await?;
        assert_eq!(task.status, "in_progress");
        assert_eq!(task.completed_sessions, 1);
        assert_eq!(task.estimated_sessions, Some(3));
        
        // Complete another session
        app.send_message(Message::StartSession { 
            task_id: task.id,
            duration_minutes: 1,
        })?;
        app.process_messages().await?;
        
        app.send_message(Message::CompleteSession)?;
        app.process_messages().await?;
        
        // Verify progress
        let task_after_second = Task::find(&pool, task.id).await?;
        assert_eq!(task_after_second.completed_sessions, 2);
        assert_eq!(task_after_second.status, "in_progress"); // Still not complete
    }
    
    // Third instance - complete the task
    {
        let task_manager = TaskManager::new(pool.clone()).await?;
        let mut app = App::new(pool.clone()).await?;
        
        // Complete final session
        app.send_message(Message::StartSession { 
            task_id,
            duration_minutes: 1,
        })?;
        app.process_messages().await?;
        
        app.send_message(Message::CompleteSession)?;
        app.process_messages().await?;
        
        // Verify task is complete
        let final_task = Task::find(&pool, task_id).await?;
        assert_eq!(final_task.status, "completed");
        assert_eq!(final_task.completed_sessions, 3);
        assert!(final_task.completed_at.is_some());
    }
    
    Ok(())
}

/// Test database corruption recovery
#[tokio::test]
async fn test_database_integrity_and_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_integrity.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create some data
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    let task_input = pomo_tui::database::models::TaskInput {
        title: "Integrity Test Task".to_string(),
        description: None,
        priority: 1,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let task = task_manager.create_task(task_input).await?;
    
    // Start and complete a session
    app.send_message(Message::StartSession { 
        task_id: task.id,
        duration_minutes: 1,
    })?;
    app.process_messages().await?;
    
    app.send_message(Message::CompleteSession)?;
    app.process_messages().await?;
    
    // Verify data integrity
    let integrity_check = sqlx::query("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await?;
    
    // SQLite returns "ok" if database is intact
    let result: String = integrity_check.try_get(0)?;
    assert_eq!(result, "ok");
    
    // Verify referential integrity
    let orphaned_sessions = sqlx::query!(
        "SELECT COUNT(*) as count FROM sessions s 
         LEFT JOIN tasks t ON s.task_id = t.id 
         WHERE t.id IS NULL"
    )
    .fetch_one(&pool)
    .await?;
    
    assert_eq!(orphaned_sessions.count, 0);
    
    // Verify statistics consistency
    let session_count = sqlx::query!("SELECT COUNT(*) as count FROM sessions WHERE status = 'completed'")
        .fetch_one(&pool)
        .await?;
    
    let today = chrono::Utc::now().naive_utc().date();
    let stats = Statistics::calculate_for_date(&pool, today).await?;
    
    assert_eq!(session_count.count, stats.completed_sessions as i64);
    
    Ok(())
}

/// Test concurrent access and data consistency
#[tokio::test]
async fn test_concurrent_data_access() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_concurrent.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create multiple app instances to simulate concurrent access
    let task_manager1 = TaskManager::new(pool.clone()).await?;
    let task_manager2 = TaskManager::new(pool.clone()).await?;
    let mut app1 = App::new(pool.clone()).await?;
    let mut app2 = App::new(pool.clone()).await?;
    
    // Create tasks concurrently
    let task_input1 = pomo_tui::database::models::TaskInput {
        title: "Concurrent Task 1".to_string(),
        description: None,
        priority: 1,
        status: "pending".to_string(),
        tags: vec!["concurrent".to_string()],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let task_input2 = pomo_tui::database::models::TaskInput {
        title: "Concurrent Task 2".to_string(),
        description: None,
        priority: 2,
        status: "pending".to_string(),
        tags: vec!["concurrent".to_string()],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    // Create tasks simultaneously
    let (task1, task2) = tokio::try_join!(
        task_manager1.create_task(task_input1),
        task_manager2.create_task(task_input2)
    )?;
    
    // Start sessions on both tasks simultaneously
    tokio::try_join!(
        async {
            app1.send_message(Message::StartSession { 
                task_id: task1.id,
                duration_minutes: 1,
            })?;
            app1.process_messages().await
        },
        async {
            app2.send_message(Message::StartSession { 
                task_id: task2.id,
                duration_minutes: 1,
            })?;
            app2.process_messages().await
        }
    )?;
    
    // Only one session should be active at a time (business rule)
    let active_sessions = sqlx::query!("SELECT COUNT(*) as count FROM sessions WHERE status = 'active'")
        .fetch_one(&pool)
        .await?;
    
    // Based on business rules, only the first session should be active
    // The second should either be queued or rejected
    assert!(active_sessions.count <= 1);
    
    // Complete both sessions
    tokio::try_join!(
        async {
            app1.send_message(Message::CompleteSession)?;
            app1.process_messages().await
        },
        async {
            app2.send_message(Message::CompleteSession)?;
            app2.process_messages().await
        }
    )?;
    
    // Verify final state consistency
    let final_sessions = sqlx::query!("SELECT COUNT(*) as count FROM sessions WHERE status IN ('completed', 'abandoned')")
        .fetch_one(&pool)
        .await?;
    
    assert!(final_sessions.count >= 1); // At least one session should have completed
    
    Ok(())
}