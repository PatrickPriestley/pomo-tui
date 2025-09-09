use pomo_tui::core::TaskManager;
use pomo_tui::database::models::{Task, Session, Statistics, Break, Preferences};
use pomo_tui::tui::app::{App, Message};
use pomo_tui::core::export::{ExportManager, ExportFormat, ExportOptions};
use sqlx::SqlitePool;
use std::fs;
use tempfile::TempDir;
use serde_json;

/// End-to-End test for export functionality:
/// 1. Create sample data (tasks, sessions, breaks, statistics)
/// 2. Export data in JSON format
/// 3. Export data in CSV format
/// 4. Export data in Markdown format
/// 5. Verify exported data integrity and completeness
#[tokio::test]
async fn test_export_all_formats() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_export.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create sample data
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    let export_manager = ExportManager::new(pool.clone());
    
    // Create multiple tasks with different properties
    let task_inputs = vec![
        pomo_tui::database::models::TaskInput {
            title: "High Priority Design Task".to_string(),
            description: Some("Complete the UI mockups for the new feature".to_string()),
            priority: 5,
            status: "pending".to_string(),
            tags: vec!["design".to_string(), "ui".to_string(), "urgent".to_string()],
            estimated_sessions: Some(3),
            project: Some("Project Alpha".to_string()),
            due_date: Some(chrono::Utc::now().naive_utc().date() + chrono::Duration::days(3)),
        },
        pomo_tui::database::models::TaskInput {
            title: "Code Review Session".to_string(),
            description: Some("Review pull requests from team members".to_string()),
            priority: 3,
            status: "pending".to_string(),
            tags: vec!["code-review".to_string(), "team".to_string()],
            estimated_sessions: Some(2),
            project: Some("Project Beta".to_string()),
            due_date: None,
        },
        pomo_tui::database::models::TaskInput {
            title: "Documentation Update".to_string(),
            description: None,
            priority: 2,
            status: "pending".to_string(),
            tags: vec!["documentation".to_string()],
            estimated_sessions: Some(1),
            project: Some("Internal".to_string()),
            due_date: Some(chrono::Utc::now().naive_utc().date() + chrono::Duration::days(7)),
        },
    ];
    
    let mut created_tasks = Vec::new();
    for task_input in task_inputs {
        let task = task_manager.create_task(task_input).await?;
        created_tasks.push(task);
    }
    
    // Complete sessions for the first two tasks
    for (i, task) in created_tasks.iter().take(2).enumerate() {
        // Start and complete a session
        app.send_message(Message::StartSession { 
            task_id: task.id,
            duration_minutes: if i == 0 { 25 } else { 15 }, // Different durations
        })?;
        app.process_messages().await?;
        
        // Simulate pause/resume for first task
        if i == 0 {
            app.send_message(Message::PauseSession)?;
            app.process_messages().await?;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            app.send_message(Message::ResumeSession)?;
            app.process_messages().await?;
        }
        
        app.send_message(Message::CompleteSession)?;
        app.process_messages().await?;
        
        // Complete the scheduled break
        let session = Session::get_latest_for_task(&pool, task.id).await?.unwrap();
        let break_record = Break::find_by_session_id(&pool, session.id).await?.unwrap();
        
        app.send_message(Message::StartBreak { break_id: break_record.id })?;
        app.process_messages().await?;
        app.send_message(Message::CompleteBreak)?;
        app.process_messages().await?;
    }
    
    // Abandon a session for the third task to test varied data
    app.send_message(Message::StartSession { 
        task_id: created_tasks[2].id,
        duration_minutes: 10,
    })?;
    app.process_messages().await?;
    
    app.send_message(Message::AbandonSession)?;
    app.process_messages().await?;
    
    // Update preferences to have non-default values
    app.send_message(Message::UpdatePreferences {
        session_duration: 30,
        short_break_duration: 7,
        long_break_duration: 18,
        sessions_until_long_break: 3,
        auto_start_breaks: true,
        auto_start_sessions: false,
        sound_enabled: true,
        volume: 0.8,
        ambient_sound: Some("rain".to_string()),
        notification_sound: Some("chime".to_string()),
        visual_notifications: true,
        desktop_notifications: true,
        github_integration: true,
        slack_integration: false,
        git_commit_enhancement: true,
        website_blocking: true,
        theme: "light".to_string(),
        show_seconds: true,
        show_session_count: true,
        show_daily_goal: true,
        daily_session_goal: 6,
        blocked_websites: vec!["facebook.com".to_string(), "twitter.com".to_string()],
    })?;
    app.process_messages().await?;
    
    // Test JSON Export
    let json_export_path = temp_dir.path().join("export_test.json");
    let json_options = ExportOptions {
        format: ExportFormat::Json,
        output_path: json_export_path.clone(),
        date_range: None, // Export all data
        include_preferences: true,
        include_statistics: true,
        pretty_format: true,
    };
    
    export_manager.export_data(json_options).await?;
    
    // Verify JSON file exists and contains valid JSON
    assert!(json_export_path.exists());
    let json_content = fs::read_to_string(&json_export_path)?;
    let json_data: serde_json::Value = serde_json::from_str(&json_content)?;
    
    // Verify JSON structure and content
    assert!(json_data.get("tasks").is_some());
    assert!(json_data.get("sessions").is_some());
    assert!(json_data.get("breaks").is_some());
    assert!(json_data.get("statistics").is_some());
    assert!(json_data.get("preferences").is_some());
    assert!(json_data.get("export_metadata").is_some());
    
    // Verify task data
    let tasks_array = json_data["tasks"].as_array().unwrap();
    assert_eq!(tasks_array.len(), 3);
    
    let first_task = &tasks_array[0];
    assert_eq!(first_task["title"], "High Priority Design Task");
    assert_eq!(first_task["priority"], 5);
    assert_eq!(first_task["status"], "completed");
    assert!(first_task["tags"].as_array().unwrap().len() == 3);
    
    // Verify session data
    let sessions_array = json_data["sessions"].as_array().unwrap();
    assert_eq!(sessions_array.len(), 3); // 2 completed, 1 abandoned
    
    let completed_sessions: Vec<_> = sessions_array.iter()
        .filter(|s| s["status"] == "completed")
        .collect();
    assert_eq!(completed_sessions.len(), 2);
    
    let abandoned_sessions: Vec<_> = sessions_array.iter()
        .filter(|s| s["status"] == "abandoned")
        .collect();
    assert_eq!(abandoned_sessions.len(), 1);
    
    // Test CSV Export
    let csv_export_path = temp_dir.path().join("export_test.csv");
    let csv_options = ExportOptions {
        format: ExportFormat::Csv,
        output_path: csv_export_path.clone(),
        date_range: Some((
            chrono::Utc::now().naive_utc().date() - chrono::Duration::days(1),
            chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1),
        )),
        include_preferences: false, // CSV doesn't include preferences
        include_statistics: true,
        pretty_format: false,
    };
    
    export_manager.export_data(csv_options).await?;
    
    // Verify CSV file exists and has proper structure
    assert!(csv_export_path.exists());
    let csv_content = fs::read_to_string(&csv_export_path)?;
    
    // CSV should have multiple sections separated by blank lines
    assert!(csv_content.contains("## Tasks"));
    assert!(csv_content.contains("## Sessions"));
    assert!(csv_content.contains("## Breaks"));
    assert!(csv_content.contains("## Statistics"));
    
    // Verify CSV headers and data rows
    let lines: Vec<&str> = csv_content.lines().collect();
    let tasks_header_idx = lines.iter().position(|&line| line == "## Tasks").unwrap();
    let tasks_csv_header = lines[tasks_header_idx + 1];
    assert!(tasks_csv_header.contains("id,title,description,priority,status"));
    
    // Should have 3 task data rows
    let task_data_rows: Vec<_> = lines.iter()
        .skip(tasks_header_idx + 2)
        .take_while(|&&line| !line.is_empty() && !line.starts_with("##"))
        .collect();
    assert_eq!(task_data_rows.len(), 3);
    
    // Test Markdown Export
    let md_export_path = temp_dir.path().join("export_test.md");
    let md_options = ExportOptions {
        format: ExportFormat::Markdown,
        output_path: md_export_path.clone(),
        date_range: None,
        include_preferences: true,
        include_statistics: true,
        pretty_format: true,
    };
    
    export_manager.export_data(md_options).await?;
    
    // Verify Markdown file exists and has proper structure
    assert!(md_export_path.exists());
    let md_content = fs::read_to_string(&md_export_path)?;
    
    // Verify Markdown structure
    assert!(md_content.contains("# Pomodoro Timer Export"));
    assert!(md_content.contains("## Tasks"));
    assert!(md_content.contains("## Sessions"));
    assert!(md_content.contains("## Breaks"));
    assert!(md_content.contains("## Statistics"));
    assert!(md_content.contains("## Preferences"));
    
    // Verify task table formatting
    assert!(md_content.contains("| Title | Status | Priority |"));
    assert!(md_content.contains("|-------|--------|----------|"));
    assert!(md_content.contains("| High Priority Design Task | completed | 5 |"));
    
    // Verify statistics formatting
    assert!(md_content.contains("**Completed Sessions:** 2"));
    assert!(md_content.contains("**Abandoned Sessions:** 1"));
    
    Ok(())
}

/// Test export with date range filtering
#[tokio::test]
async fn test_export_with_date_range() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_date_range_export.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    let export_manager = ExportManager::new(pool.clone());
    
    // Create tasks with different dates (simulated by manipulating timestamps)
    let task_input = pomo_tui::database::models::TaskInput {
        title: "Today's Task".to_string(),
        description: None,
        priority: 1,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let task = task_manager.create_task(task_input).await?;
    
    // Complete a session today
    app.send_message(Message::StartSession { 
        task_id: task.id,
        duration_minutes: 25,
    })?;
    app.process_messages().await?;
    app.send_message(Message::CompleteSession)?;
    app.process_messages().await?;
    
    // Manually create an "old" session by inserting directly into database
    let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
    let old_session_id = sqlx::query!(
        "INSERT INTO sessions (task_id, start_time, end_time, duration_minutes, status, session_type)
         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
        task.id,
        yesterday,
        yesterday + chrono::Duration::minutes(25),
        25,
        "completed",
        "pomodoro"
    )
    .fetch_one(&pool)
    .await?
    .id;
    
    // Export only today's data
    let today = chrono::Utc::now().naive_utc().date();
    let json_export_path = temp_dir.path().join("today_export.json");
    let today_options = ExportOptions {
        format: ExportFormat::Json,
        output_path: json_export_path.clone(),
        date_range: Some((today, today)),
        include_preferences: false,
        include_statistics: false,
        pretty_format: true,
    };
    
    export_manager.export_data(today_options).await?;
    
    // Verify only today's session is included
    let json_content = fs::read_to_string(&json_export_path)?;
    let json_data: serde_json::Value = serde_json::from_str(&json_content)?;
    
    let sessions_array = json_data["sessions"].as_array().unwrap();
    
    // Should only have 1 session (today's), not the old one
    assert_eq!(sessions_array.len(), 1);
    
    let session = &sessions_array[0];
    let session_date = chrono::DateTime::parse_from_rfc3339(session["start_time"].as_str().unwrap())?
        .naive_utc()
        .date();
    assert_eq!(session_date, today);
    
    // Export all data (no date range)
    let all_export_path = temp_dir.path().join("all_export.json");
    let all_options = ExportOptions {
        format: ExportFormat::Json,
        output_path: all_export_path.clone(),
        date_range: None,
        include_preferences: false,
        include_statistics: false,
        pretty_format: true,
    };
    
    export_manager.export_data(all_options).await?;
    
    // Verify all sessions are included
    let all_json_content = fs::read_to_string(&all_export_path)?;
    let all_json_data: serde_json::Value = serde_json::from_str(&all_json_content)?;
    
    let all_sessions_array = all_json_data["sessions"].as_array().unwrap();
    assert_eq!(all_sessions_array.len(), 2); // Today's + yesterday's
    
    Ok(())
}

/// Test export error handling and validation
#[tokio::test]
async fn test_export_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_export_errors.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let export_manager = ExportManager::new(pool.clone());
    
    // Test invalid output path (directory doesn't exist)
    let invalid_path = temp_dir.path().join("nonexistent_dir").join("export.json");
    let invalid_options = ExportOptions {
        format: ExportFormat::Json,
        output_path: invalid_path,
        date_range: None,
        include_preferences: true,
        include_statistics: true,
        pretty_format: true,
    };
    
    let result = export_manager.export_data(invalid_options).await;
    assert!(result.is_err());
    
    // Test invalid date range (end before start)
    let today = chrono::Utc::now().naive_utc().date();
    let invalid_date_path = temp_dir.path().join("invalid_date_export.json");
    let invalid_date_options = ExportOptions {
        format: ExportFormat::Json,
        output_path: invalid_date_path,
        date_range: Some((today, today - chrono::Duration::days(1))), // Invalid range
        include_preferences: true,
        include_statistics: true,
        pretty_format: true,
    };
    
    let date_result = export_manager.export_data(invalid_date_options).await;
    assert!(date_result.is_err());
    
    // Test export with no data (should succeed but produce minimal output)
    let empty_export_path = temp_dir.path().join("empty_export.json");
    let empty_options = ExportOptions {
        format: ExportFormat::Json,
        output_path: empty_export_path.clone(),
        date_range: None,
        include_preferences: false,
        include_statistics: false,
        pretty_format: true,
    };
    
    export_manager.export_data(empty_options).await?;
    
    // Verify empty export has proper structure but empty arrays
    let empty_json_content = fs::read_to_string(&empty_export_path)?;
    let empty_json_data: serde_json::Value = serde_json::from_str(&empty_json_content)?;
    
    assert_eq!(empty_json_data["tasks"].as_array().unwrap().len(), 0);
    assert_eq!(empty_json_data["sessions"].as_array().unwrap().len(), 0);
    assert_eq!(empty_json_data["breaks"].as_array().unwrap().len(), 0);
    assert!(empty_json_data.get("preferences").is_none());
    assert!(empty_json_data.get("statistics").is_none());
    
    Ok(())
}

/// Test export data integrity and roundtrip capability
#[tokio::test]
async fn test_export_data_integrity() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_integrity_export.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    let export_manager = ExportManager::new(pool.clone());
    
    // Create comprehensive test data
    let task_input = pomo_tui::database::models::TaskInput {
        title: "Integrity Test Task".to_string(),
        description: Some("Task for testing data integrity".to_string()),
        priority: 4,
        status: "pending".to_string(),
        tags: vec!["integrity".to_string(), "test".to_string()],
        estimated_sessions: Some(2),
        project: Some("QA".to_string()),
        due_date: Some(chrono::Utc::now().naive_utc().date() + chrono::Duration::days(5)),
    };
    
    let task = task_manager.create_task(task_input).await?;
    
    // Complete a session with specific characteristics
    app.send_message(Message::StartSession { 
        task_id: task.id,
        duration_minutes: 25,
    })?;
    app.process_messages().await?;
    
    // Add pause/resume cycle
    app.send_message(Message::PauseSession)?;
    app.process_messages().await?;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    app.send_message(Message::ResumeSession)?;
    app.process_messages().await?;
    
    app.send_message(Message::CompleteSession)?;
    app.process_messages().await?;
    
    // Get original data for comparison
    let original_task = Task::find(&pool, task.id).await?;
    let original_session = Session::get_latest_for_task(&pool, task.id).await?.unwrap();
    let original_break = Break::find_by_session_id(&pool, original_session.id).await?.unwrap();
    
    // Export data
    let export_path = temp_dir.path().join("integrity_test.json");
    let options = ExportOptions {
        format: ExportFormat::Json,
        output_path: export_path.clone(),
        date_range: None,
        include_preferences: true,
        include_statistics: true,
        pretty_format: true,
    };
    
    export_manager.export_data(options).await?;
    
    // Read and parse exported data
    let json_content = fs::read_to_string(&export_path)?;
    let json_data: serde_json::Value = serde_json::from_str(&json_content)?;
    
    // Verify task data integrity
    let exported_tasks = json_data["tasks"].as_array().unwrap();
    let exported_task = &exported_tasks[0];
    
    assert_eq!(exported_task["id"], original_task.id);
    assert_eq!(exported_task["title"], original_task.title);
    assert_eq!(exported_task["description"], original_task.description);
    assert_eq!(exported_task["priority"], original_task.priority);
    assert_eq!(exported_task["status"], original_task.status);
    assert_eq!(exported_task["estimated_sessions"], original_task.estimated_sessions);
    assert_eq!(exported_task["project"], original_task.project);
    
    // Verify session data integrity
    let exported_sessions = json_data["sessions"].as_array().unwrap();
    let exported_session = &exported_sessions[0];
    
    assert_eq!(exported_session["id"], original_session.id);
    assert_eq!(exported_session["task_id"], original_session.task_id);
    assert_eq!(exported_session["duration_minutes"], original_session.duration_minutes);
    assert_eq!(exported_session["status"], original_session.status);
    assert_eq!(exported_session["session_type"], original_session.session_type);
    
    // Verify timestamp formats are valid ISO 8601
    let start_time_str = exported_session["start_time"].as_str().unwrap();
    let end_time_str = exported_session["end_time"].as_str().unwrap();
    
    chrono::DateTime::parse_from_rfc3339(start_time_str)?;
    chrono::DateTime::parse_from_rfc3339(end_time_str)?;
    
    // Verify break data integrity
    let exported_breaks = json_data["breaks"].as_array().unwrap();
    let exported_break = &exported_breaks[0];
    
    assert_eq!(exported_break["id"], original_break.id);
    assert_eq!(exported_break["after_session_id"], original_break.after_session_id);
    assert_eq!(exported_break["break_type"], original_break.break_type);
    assert_eq!(exported_break["duration_minutes"], original_break.duration_minutes);
    
    // Verify export metadata
    let metadata = json_data["export_metadata"].as_object().unwrap();
    assert!(metadata.contains_key("export_date"));
    assert!(metadata.contains_key("version"));
    assert!(metadata.contains_key("total_tasks"));
    assert!(metadata.contains_key("total_sessions"));
    
    assert_eq!(metadata["total_tasks"], 1);
    assert_eq!(metadata["total_sessions"], 1);
    
    Ok(())
}

/// Test CLI export command integration
#[tokio::test]
async fn test_cli_export_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_cli_export.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create test data
    let task_manager = TaskManager::new(pool.clone()).await?;
    let mut app = App::new(pool.clone()).await?;
    
    let task_input = pomo_tui::database::models::TaskInput {
        title: "CLI Export Test".to_string(),
        description: None,
        priority: 1,
        status: "pending".to_string(),
        tags: vec!["cli".to_string()],
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
    app.send_message(Message::CompleteSession)?;
    app.process_messages().await?;
    
    // Test CLI export command through app
    let export_path = temp_dir.path().join("cli_export.json");
    app.send_message(Message::ExportData {
        format: ExportFormat::Json,
        path: export_path.clone(),
        date_range: None,
        include_preferences: true,
    })?;
    app.process_messages().await?;
    
    // Verify export was created
    assert!(export_path.exists());
    
    let json_content = fs::read_to_string(&export_path)?;
    let json_data: serde_json::Value = serde_json::from_str(&json_content)?;
    
    // Verify basic structure
    assert!(json_data.get("tasks").is_some());
    assert!(json_data.get("sessions").is_some());
    assert_eq!(json_data["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(json_data["sessions"].as_array().unwrap().len(), 1);
    
    Ok(())
}