use pomo_tui::core::validation::{
    TaskValidator, SessionValidator, PreferencesValidator, ValidationResult, ValidationError
};
use pomo_tui::database::models::{TaskInput, SessionInput, PreferencesInput};
use chrono::{Utc, Duration as ChronoDuration};

#[test]
fn test_task_validation_valid_input() {
    let validator = TaskValidator::new();
    
    let valid_task = TaskInput {
        title: "Valid Task Title".to_string(),
        description: Some("A proper description".to_string()),
        priority: 3,
        status: "pending".to_string(),
        tags: vec!["work".to_string(), "urgent".to_string()],
        estimated_sessions: Some(5),
        project: Some("project-alpha".to_string()),
        due_date: Some(Utc::now().naive_utc().date() + ChronoDuration::days(7)),
    };
    
    let result = validator.validate(&valid_task);
    assert!(result.is_valid());
    assert!(result.errors().is_empty());
}

#[test]
fn test_task_validation_empty_title() {
    let validator = TaskValidator::new();
    
    let invalid_task = TaskInput {
        title: "".to_string(),
        description: None,
        priority: 3,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    
    let errors = result.errors();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::EmptyTitle)));
}

#[test]
fn test_task_validation_title_too_long() {
    let validator = TaskValidator::new();
    
    let long_title = "a".repeat(256); // Assuming 255 char limit
    let invalid_task = TaskInput {
        title: long_title,
        description: None,
        priority: 3,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    
    let errors = result.errors();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TitleTooLong(_))));
}

#[test]
fn test_task_validation_invalid_priority() {
    let validator = TaskValidator::new();
    
    // Test priority too high
    let invalid_task_high = TaskInput {
        title: "Valid Title".to_string(),
        description: None,
        priority: 6, // Max should be 5
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task_high);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidPriority(_))));
    
    // Test priority too low
    let invalid_task_low = TaskInput {
        title: "Valid Title".to_string(),
        description: None,
        priority: 0, // Min should be 1
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task_low);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidPriority(_))));
}

#[test]
fn test_task_validation_invalid_status() {
    let validator = TaskValidator::new();
    
    let invalid_task = TaskInput {
        title: "Valid Title".to_string(),
        description: None,
        priority: 3,
        status: "invalid_status".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidStatus(_))));
}

#[test]
fn test_task_validation_past_due_date() {
    let validator = TaskValidator::new();
    
    let invalid_task = TaskInput {
        title: "Valid Title".to_string(),
        description: None,
        priority: 3,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: Some(Utc::now().naive_utc().date() - ChronoDuration::days(1)), // Past date
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::PastDueDate(_))));
}

#[test]
fn test_task_validation_invalid_estimated_sessions() {
    let validator = TaskValidator::new();
    
    let invalid_task = TaskInput {
        title: "Valid Title".to_string(),
        description: None,
        priority: 3,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: Some(0), // Should be at least 1
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidEstimatedSessions(_))));
}

#[test]
fn test_task_validation_invalid_tags() {
    let validator = TaskValidator::new();
    
    let invalid_task = TaskInput {
        title: "Valid Title".to_string(),
        description: None,
        priority: 3,
        status: "pending".to_string(),
        tags: vec!["valid".to_string(), "".to_string(), "also-valid".to_string()], // Empty tag
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::EmptyTag)));
}

#[test]
fn test_session_validation_valid_input() {
    let validator = SessionValidator::new();
    
    let valid_session = SessionInput {
        task_id: 1,
        duration_minutes: 25,
        session_type: "pomodoro".to_string(),
        planned_start_time: Some(Utc::now().naive_utc()),
    };
    
    let result = validator.validate(&valid_session);
    assert!(result.is_valid());
}

#[test]
fn test_session_validation_invalid_duration() {
    let validator = SessionValidator::new();
    
    // Too short
    let invalid_session_short = SessionInput {
        task_id: 1,
        duration_minutes: 5, // Min should be 15
        session_type: "pomodoro".to_string(),
        planned_start_time: None,
    };
    
    let result = validator.validate(&invalid_session_short);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidSessionDuration(_))));
    
    // Too long
    let invalid_session_long = SessionInput {
        task_id: 1,
        duration_minutes: 120, // Max should be 90
        session_type: "pomodoro".to_string(),
        planned_start_time: None,
    };
    
    let result = validator.validate(&invalid_session_long);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidSessionDuration(_))));
}

#[test]
fn test_session_validation_invalid_type() {
    let validator = SessionValidator::new();
    
    let invalid_session = SessionInput {
        task_id: 1,
        duration_minutes: 25,
        session_type: "invalid_type".to_string(),
        planned_start_time: None,
    };
    
    let result = validator.validate(&invalid_session);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidSessionType(_))));
}

#[test]
fn test_session_validation_future_start_time() {
    let validator = SessionValidator::new();
    
    let invalid_session = SessionInput {
        task_id: 1,
        duration_minutes: 25,
        session_type: "pomodoro".to_string(),
        planned_start_time: Some(Utc::now().naive_utc() + ChronoDuration::days(1)),
    };
    
    let result = validator.validate(&invalid_session);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::FutureStartTime(_))));
}

#[test]
fn test_preferences_validation_valid_input() {
    let validator = PreferencesValidator::new();
    
    let valid_prefs = PreferencesInput {
        session_duration: 25,
        short_break_duration: 5,
        long_break_duration: 15,
        sessions_until_long_break: 4,
        volume: 0.7,
        daily_session_goal: 8,
        theme: "dark".to_string(),
        blocked_websites: vec!["facebook.com".to_string(), "twitter.com".to_string()],
    };
    
    let result = validator.validate(&valid_prefs);
    assert!(result.is_valid());
}

#[test]
fn test_preferences_validation_invalid_durations() {
    let validator = PreferencesValidator::new();
    
    let invalid_prefs = PreferencesInput {
        session_duration: 10, // Too short (min 15)
        short_break_duration: 2, // Too short (min 3)
        long_break_duration: 50, // Too long (max 45)
        sessions_until_long_break: 8, // Too many (max 6)
        volume: 0.7,
        daily_session_goal: 8,
        theme: "dark".to_string(),
        blocked_websites: vec![],
    };
    
    let result = validator.validate(&invalid_prefs);
    assert!(!result.is_valid());
    
    let errors = result.errors();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidSessionDuration(_))));
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidBreakDuration(_))));
}

#[test]
fn test_preferences_validation_invalid_volume() {
    let validator = PreferencesValidator::new();
    
    let invalid_prefs = PreferencesInput {
        session_duration: 25,
        short_break_duration: 5,
        long_break_duration: 15,
        sessions_until_long_break: 4,
        volume: 1.5, // Invalid (must be 0.0-1.0)
        daily_session_goal: 8,
        theme: "dark".to_string(),
        blocked_websites: vec![],
    };
    
    let result = validator.validate(&invalid_prefs);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidVolume(_))));
}

#[test]
fn test_preferences_validation_invalid_theme() {
    let validator = PreferencesValidator::new();
    
    let invalid_prefs = PreferencesInput {
        session_duration: 25,
        short_break_duration: 5,
        long_break_duration: 15,
        sessions_until_long_break: 4,
        volume: 0.7,
        daily_session_goal: 8,
        theme: "rainbow".to_string(), // Invalid theme
        blocked_websites: vec![],
    };
    
    let result = validator.validate(&invalid_prefs);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidTheme(_))));
}

#[test]
fn test_preferences_validation_invalid_websites() {
    let validator = PreferencesValidator::new();
    
    let invalid_prefs = PreferencesInput {
        session_duration: 25,
        short_break_duration: 5,
        long_break_duration: 15,
        sessions_until_long_break: 4,
        volume: 0.7,
        daily_session_goal: 8,
        theme: "dark".to_string(),
        blocked_websites: vec![
            "facebook.com".to_string(),
            "not_a_domain".to_string(), // Invalid domain
            "".to_string(), // Empty string
        ],
    };
    
    let result = validator.validate(&invalid_prefs);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidWebsite(_))));
}

#[test]
fn test_validation_result_aggregation() {
    let validator = TaskValidator::new();
    
    let invalid_task = TaskInput {
        title: "".to_string(), // Empty title
        description: None,
        priority: 0, // Invalid priority
        status: "invalid".to_string(), // Invalid status
        tags: vec!["".to_string()], // Empty tag
        estimated_sessions: Some(0), // Invalid estimate
        project: None,
        due_date: Some(Utc::now().naive_utc().date() - ChronoDuration::days(1)), // Past due
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    
    // Should have multiple errors
    let errors = result.errors();
    assert!(errors.len() >= 5); // At least 5 validation errors
    
    // Check specific error types are present
    assert!(errors.iter().any(|e| matches!(e, ValidationError::EmptyTitle)));
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidPriority(_))));
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidStatus(_))));
    assert!(errors.iter().any(|e| matches!(e, ValidationError::EmptyTag)));
    assert!(errors.iter().any(|e| matches!(e, ValidationError::PastDueDate(_))));
}

#[test]
fn test_validation_warnings() {
    let validator = TaskValidator::new();
    
    let task_with_warnings = TaskInput {
        title: "Valid Task".to_string(),
        description: Some("A".repeat(500)), // Very long description (warning)
        priority: 5, // Very high priority (warning)
        status: "pending".to_string(),
        tags: vec!["tag1".to_string(); 10], // Too many tags (warning)
        estimated_sessions: Some(20), // Very high estimate (warning)
        project: None,
        due_date: Some(Utc::now().naive_utc().date() + ChronoDuration::days(1)), // Due tomorrow (warning)
    };
    
    let result = validator.validate_with_warnings(&task_with_warnings);
    assert!(result.is_valid()); // Should be valid but have warnings
    assert!(!result.warnings().is_empty());
    
    let warnings = result.warnings();
    assert!(warnings.iter().any(|w| w.contains("description is very long")));
    assert!(warnings.iter().any(|w| w.contains("high priority")));
    assert!(warnings.iter().any(|w| w.contains("many tags")));
    assert!(warnings.iter().any(|w| w.contains("due soon")));
}

#[test]
fn test_custom_validation_rules() {
    let mut validator = TaskValidator::new();
    
    // Add custom validation rule
    validator.add_custom_rule(Box::new(|task| {
        if task.title.to_lowercase().contains("test") && task.priority > 3 {
            Some(ValidationError::Custom("Test tasks should not have high priority".to_string()))
        } else {
            None
        }
    }));
    
    let test_task = TaskInput {
        title: "Test Task".to_string(),
        description: None,
        priority: 4, // High priority test task
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&test_task);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| matches!(e, ValidationError::Custom(_))));
}

#[test]
fn test_validation_performance() {
    use std::time::Instant;
    
    let validator = TaskValidator::new();
    
    // Create many tasks to validate
    let tasks: Vec<TaskInput> = (0..1000).map(|i| TaskInput {
        title: format!("Task {}", i),
        description: Some(format!("Description for task {}", i)),
        priority: (i % 5) + 1,
        status: "pending".to_string(),
        tags: vec![format!("tag{}", i % 10)],
        estimated_sessions: Some((i % 10) + 1),
        project: Some(format!("project{}", i % 5)),
        due_date: Some(Utc::now().naive_utc().date() + ChronoDuration::days(i % 30)),
    }).collect();
    
    let start = Instant::now();
    
    for task in &tasks {
        let _ = validator.validate(task);
    }
    
    let duration = start.elapsed();
    
    // Should validate 1000 tasks in under 100ms
    assert!(duration.as_millis() < 100, 
        "Validation took {}ms for 1000 tasks, should be under 100ms", 
        duration.as_millis());
}

#[test]
fn test_validation_context() {
    let validator = TaskValidator::with_context("test_context");
    
    let invalid_task = TaskInput {
        title: "".to_string(),
        description: None,
        priority: 3,
        status: "pending".to_string(),
        tags: vec![],
        estimated_sessions: None,
        project: None,
        due_date: None,
    };
    
    let result = validator.validate(&invalid_task);
    assert!(!result.is_valid());
    
    // Context should be included in error reporting
    let error_message = result.format_errors();
    assert!(error_message.contains("test_context"));
}