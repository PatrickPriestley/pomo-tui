use pomo_tui::core::{TaskManager, Timer};
use pomo_tui::database::models::{Task, Session, Statistics, Break};
use pomo_tui::tui::app::{App, AppState, Message};
use sqlx::SqlitePool;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// End-to-End test for a complete Pomodoro cycle:
/// 1. Create a task
/// 2. Start a session
/// 3. Complete the session  
/// 4. Take a break
/// 5. Verify statistics are updated correctly
#[tokio::test]
async fn test_complete_pomodoro_cycle() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_pomodoro_cycle.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Initialize components
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    // Step 1: Create a task
    let task_input = pomo_tui::database::models::TaskInput {
        title: "Test Pomodoro Task".to_string(),
        description: Some("End-to-end test task for complete cycle".to_string()),
        priority: 3,
        status: "pending".to_string(),
        tags: vec!["test".to_string(), "e2e".to_string()],
        estimated_sessions: Some(1),
        project: Some("testing".to_string()),
        due_date: None,
    };
    
    let created_task = task_manager.create_task(task_input).await?;
    assert_eq!(created_task.title, "Test Pomodoro Task");
    assert_eq!(created_task.status, "pending");
    
    // Step 2: Start a session
    app.send_message(Message::StartSession { 
        task_id: created_task.id,
        duration_minutes: 1, // Use 1 minute for testing
    })?;
    
    // Process the message
    app.process_messages().await?;
    
    // Verify session started
    let active_session = Session::get_active(&pool).await?;
    assert!(active_session.is_some());
    let session = active_session.unwrap();
    assert_eq!(session.task_id, created_task.id);
    assert_eq!(session.status, "active");
    assert_eq!(session.duration_minutes, 1);
    
    // Verify app state reflects active session
    assert!(matches!(app.state.current_mode, pomo_tui::tui::app::AppMode::Session));
    assert!(app.state.current_session.is_some());
    assert!(app.state.timer.is_some());
    
    // Step 3: Simulate session completion (wait for timer)
    // In a real test, we'd wait the full duration, but for testing we'll simulate
    sleep(Duration::from_millis(100)).await; // Brief wait to ensure timer started
    
    // Manually complete the session for testing
    app.send_message(Message::CompleteSession)?;
    app.process_messages().await?;
    
    // Verify session completed
    let completed_session = Session::find(&pool, session.id).await?;
    assert_eq!(completed_session.status, "completed");
    assert!(completed_session.end_time.is_some());
    
    // Verify task status updated
    let updated_task = Task::find(&pool, created_task.id).await?;
    assert_eq!(updated_task.status, "completed");
    
    // Step 4: Verify break is scheduled
    let scheduled_break = Break::find_by_session_id(&pool, session.id).await?;
    assert!(scheduled_break.is_some());
    let break_record = scheduled_break.unwrap();
    assert_eq!(break_record.after_session_id, session.id);
    assert_eq!(break_record.break_type, "short"); // First session should trigger short break
    
    // Start the break
    app.send_message(Message::StartBreak { break_id: break_record.id })?;
    app.process_messages().await?;
    
    // Verify app is in break mode
    assert!(matches!(app.state.current_mode, pomo_tui::tui::app::AppMode::Break));
    assert!(app.state.current_break.is_some());
    
    // Complete the break
    app.send_message(Message::CompleteBreak)?;
    app.process_messages().await?;
    
    // Verify break completed
    let completed_break = Break::find(&pool, break_record.id).await?;
    assert_eq!(completed_break.status, "completed");
    assert!(completed_break.end_time.is_some());
    
    // Step 5: Verify statistics updated
    let today = chrono::Utc::now().naive_utc().date();
    let stats = Statistics::calculate_for_date(&pool, today).await?;
    
    assert_eq!(stats.completed_sessions, 1);
    assert_eq!(stats.completed_breaks, 1);
    assert_eq!(stats.abandoned_sessions, 0);
    assert_eq!(stats.total_focus_time_minutes, 1);
    assert_eq!(stats.total_break_time_minutes, break_record.duration_minutes);
    
    // Verify productivity score calculation
    assert!(stats.productivity_score > 0.0);
    assert!(stats.productivity_score <= 100.0);
    
    // Verify session streak
    assert_eq!(stats.current_streak, 1);
    assert_eq!(stats.longest_streak, 1);
    
    Ok(())
}

/// Test partial session with pause/resume functionality
#[tokio::test]
async fn test_session_with_pause_resume() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_pause_resume.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    // Create task
    let task_input = pomo_tui::database::models::TaskInput {
        title: "Pause Resume Test Task".to_string(),
        description: None,
        priority: 2,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let task = task_manager.create_task(task_input).await?;
    
    // Start session
    app.send_message(Message::StartSession { 
        task_id: task.id,
        duration_minutes: 2,
    })?;
    app.process_messages().await?;
    
    let session = Session::get_active(&pool).await?.unwrap();
    
    // Pause session
    app.send_message(Message::PauseSession)?;
    app.process_messages().await?;
    
    let paused_session = Session::find(&pool, session.id).await?;
    assert_eq!(paused_session.status, "paused");
    assert!(paused_session.paused_at.is_some());
    
    // Wait a bit while paused
    sleep(Duration::from_millis(50)).await;
    
    // Resume session
    app.send_message(Message::ResumeSession)?;
    app.process_messages().await?;
    
    let resumed_session = Session::find(&pool, session.id).await?;
    assert_eq!(resumed_session.status, "active");
    assert!(resumed_session.resumed_at.is_some());
    
    // Complete session
    app.send_message(Message::CompleteSession)?;
    app.process_messages().await?;
    
    let completed_session = Session::find(&pool, session.id).await?;
    assert_eq!(completed_session.status, "completed");
    
    // Verify statistics include the pause/resume cycle
    let today = chrono::Utc::now().naive_utc().date();
    let stats = Statistics::calculate_for_date(&pool, today).await?;
    assert_eq!(stats.completed_sessions, 1);
    
    Ok(())
}

/// Test session abandonment and its impact on statistics
#[tokio::test]
async fn test_session_abandonment() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_abandonment.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    // Create task
    let task_input = pomo_tui::database::models::TaskInput {
        title: "Abandonment Test Task".to_string(),
        description: None,
        priority: 1,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let task = task_manager.create_task(task_input).await?;
    
    // Start session
    app.send_message(Message::StartSession { 
        task_id: task.id,
        duration_minutes: 1,
    })?;
    app.process_messages().await?;
    
    let session = Session::get_active(&pool).await?.unwrap();
    
    // Abandon session
    app.send_message(Message::AbandonSession)?;
    app.process_messages().await?;
    
    let abandoned_session = Session::find(&pool, session.id).await?;
    assert_eq!(abandoned_session.status, "abandoned");
    assert!(abandoned_session.end_time.is_some());
    
    // Verify task remains in original status
    let task_after_abandon = Task::find(&pool, task.id).await?;
    assert_eq!(task_after_abandon.status, "pending");
    
    // Verify no break is scheduled after abandonment
    let break_record = Break::find_by_session_id(&pool, session.id).await?;
    assert!(break_record.is_none());
    
    // Verify statistics reflect abandonment
    let today = chrono::Utc::now().naive_utc().date();
    let stats = Statistics::calculate_for_date(&pool, today).await?;
    assert_eq!(stats.completed_sessions, 0);
    assert_eq!(stats.abandoned_sessions, 1);
    assert_eq!(stats.current_streak, 0);
    
    // Productivity score should be affected by abandonment
    assert_eq!(stats.productivity_score, 0.0);
    
    Ok(())
}

/// Test multiple sessions and long break scheduling
#[tokio::test]
async fn test_long_break_after_four_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_long_break.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    // Create multiple tasks
    let mut task_ids = Vec::new();
    for i in 1..=4 {
        let task_input = pomo_tui::database::models::TaskInput {
            title: format!("Long Break Test Task {}", i),
            description: None,
            priority: 1,
            status: "pending".to_string(),
            tags: vec![],
            estimated_sessions: None,
            project: None,
            due_date: None,
        };
        let task = task_manager.create_task(task_input).await?;
        task_ids.push(task.id);
    }
    
    // Complete 4 sessions
    for (i, task_id) in task_ids.iter().enumerate() {
        // Start session
        app.send_message(Message::StartSession { 
            task_id: *task_id,
            duration_minutes: 1,
        })?;
        app.process_messages().await?;
        
        // Complete session
        app.send_message(Message::CompleteSession)?;
        app.process_messages().await?;
        
        let session = Session::get_latest_for_task(&pool, *task_id).await?.unwrap();
        let break_record = Break::find_by_session_id(&pool, session.id).await?.unwrap();
        
        if i < 3 {
            // First 3 sessions should trigger short breaks
            assert_eq!(break_record.break_type, "short");
            assert_eq!(break_record.duration_minutes, 5); // Default short break
        } else {
            // 4th session should trigger long break
            assert_eq!(break_record.break_type, "long");
            assert_eq!(break_record.duration_minutes, 15); // Default long break
        }
        
        // Complete the break
        app.send_message(Message::StartBreak { break_id: break_record.id })?;
        app.process_messages().await?;
        app.send_message(Message::CompleteBreak)?;
        app.process_messages().await?;
    }
    
    // Verify final statistics
    let today = chrono::Utc::now().naive_utc().date();
    let stats = Statistics::calculate_for_date(&pool, today).await?;
    assert_eq!(stats.completed_sessions, 4);
    assert_eq!(stats.completed_breaks, 4);
    assert_eq!(stats.current_streak, 4);
    
    // Verify break time calculation (3 short + 1 long)
    let expected_break_time = (3 * 5) + (1 * 15); // 30 minutes total
    assert_eq!(stats.total_break_time_minutes, expected_break_time);
    
    Ok(())
}