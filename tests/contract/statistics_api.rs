use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::process::Command;
use tempfile::TempDir;

/// Contract tests for Statistics operations
/// These tests verify the CLI API matches the OpenAPI specification
/// All tests MUST fail initially (no implementation exists yet)

#[test]
fn test_get_statistics_today_contract() {
    // Test: GET /cli/statistics?period=today (getStatistics)
    // Contract: Get today's productivity statistics

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["statistics", "get", "--period", "today"])
    .assert()
    .success();

    panic!("Contract test must fail - getStatistics today not implemented");
}

#[test]
fn test_get_statistics_json_output() {
    // Test JSON format output for statistics

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get", "--period", "today"])
        .output()
        .expect("Failed to execute command");

    // Should return JSON with statistics structure
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Check required fields according to OpenAPI schema
    assert!(json.get("date").is_some());
    assert!(json.get("completed_sessions").is_some());
    assert!(json.get("abandoned_sessions").is_some());
    assert!(json.get("total_focus_time").is_some());
    assert!(json.get("total_break_time").is_some());
    assert!(json.get("tasks_completed").is_some());
    assert!(json.get("tasks_created").is_some());

    // Optional fields
    if json.get("average_session_duration").is_some() {
        assert!(json["average_session_duration"].is_number());
    }
    if json.get("longest_focus_streak").is_some() {
        assert!(json["longest_focus_streak"].is_number());
    }
    if json.get("most_productive_hour").is_some() {
        let hour = json["most_productive_hour"].as_i64().unwrap();
        assert!(hour >= 0 && hour <= 23);
    }

    panic!("Contract test must fail - statistics JSON output not implemented");
}

#[test]
fn test_get_statistics_week_period() {
    // Test weekly statistics aggregation

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get", "--period", "week"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Week period should return array of daily statistics
    if let Some(days) = json.as_array() {
        assert_eq!(days.len(), 7); // Full week
        for day in days {
            assert!(day.get("date").is_some());
            assert!(day.get("completed_sessions").is_some());
        }
    } else {
        // Or aggregated weekly totals
        assert!(json.get("completed_sessions").is_some());
        assert!(json.get("total_focus_time").is_some());
    }

    panic!("Contract test must fail - weekly statistics not implemented");
}

#[test]
fn test_get_statistics_month_period() {
    // Test monthly statistics aggregation

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["statistics", "get", "--period", "month"])
    .assert()
    .success();

    panic!("Contract test must fail - monthly statistics not implemented");
}

#[test]
fn test_get_statistics_all_period() {
    // Test all-time statistics aggregation

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["statistics", "get", "--period", "all"])
    .assert()
    .success();

    panic!("Contract test must fail - all-time statistics not implemented");
}

#[test]
fn test_statistics_default_period() {
    // Test default period (should be today)

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Should default to today's statistics
    assert!(json.get("date").is_some());
    let date_str = json["date"].as_str().unwrap();

    // Should be today's date (YYYY-MM-DD format)
    use chrono::{Local, NaiveDate};
    let today = Local::now().date_naive();
    let expected_date = today.format("%Y-%m-%d").to_string();
    assert_eq!(date_str, expected_date);

    panic!("Contract test must fail - default period not implemented");
}

#[test]
fn test_statistics_calculation_accuracy() {
    // Test that statistics calculations are accurate

    let temp_dir = TempDir::new().unwrap();

    // Create test data: 2 completed sessions, 1 abandoned, 3 tasks
    // This would normally be set up through the API, but we're testing contracts

    let output = Command::cargo_bin("pomo-tui")
        .unwrap()
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Verify calculations match expected values
    assert_eq!(json["completed_sessions"], 2);
    assert_eq!(json["abandoned_sessions"], 1);
    assert_eq!(json["tasks_completed"], 3);

    // Total focus time should be sum of completed session durations
    let total_focus = json["total_focus_time"].as_i64().unwrap();
    assert!(total_focus > 0);

    // Average session duration = total_focus_time / completed_sessions
    if json.get("average_session_duration").is_some() {
        let avg_duration = json["average_session_duration"].as_f64().unwrap();
        let expected_avg = total_focus as f64 / 2.0; // 2 completed sessions
        assert!((avg_duration - expected_avg).abs() < 0.1);
    }

    panic!("Contract test must fail - statistics calculations not implemented");
}

#[test]
fn test_statistics_focus_streak_calculation() {
    // Test longest focus streak calculation

    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("pomo-tui")
        .unwrap()
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Should calculate longest consecutive completed sessions
    if json.get("longest_focus_streak").is_some() {
        let streak = json["longest_focus_streak"].as_i64().unwrap();
        assert!(streak >= 0);
    }

    panic!("Contract test must fail - focus streak calculation not implemented");
}

#[test]
fn test_statistics_productive_hour_calculation() {
    // Test most productive hour calculation

    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("pomo-tui")
        .unwrap()
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Should identify hour with most completed sessions
    if json.get("most_productive_hour").is_some() {
        let hour = json["most_productive_hour"].as_i64().unwrap();
        assert!(hour >= 0 && hour <= 23);
    }

    panic!("Contract test must fail - productive hour calculation not implemented");
}

#[test]
fn test_statistics_empty_data() {
    // Test statistics when no data exists (fresh database)

    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("pomo-tui")
        .unwrap()
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Empty data should return zeros, not fail
    assert_eq!(json["completed_sessions"], 0);
    assert_eq!(json["abandoned_sessions"], 0);
    assert_eq!(json["total_focus_time"], 0);
    assert_eq!(json["total_break_time"], 0);
    assert_eq!(json["tasks_completed"], 0);
    assert_eq!(json["tasks_created"], 0);

    // Optional fields should be null or 0
    if json.get("average_session_duration").is_some() {
        assert!(
            json["average_session_duration"].is_null() || json["average_session_duration"] == 0
        );
    }

    panic!("Contract test must fail - empty statistics handling not implemented");
}

#[test]
fn test_statistics_invalid_period() {
    // Test error handling for invalid period parameter

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["statistics", "get", "--period", "invalid"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Invalid period"));

    panic!("Contract test must fail - period validation not implemented");
}

#[test]
fn test_statistics_time_zones() {
    // Test that statistics respect local time zones
    // Contract: Day boundaries should be based on user's local time

    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("pomo-tui")
        .unwrap()
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .env("TZ", "UTC")
        .args(&["--format", "json", "statistics", "get", "--period", "today"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Date should reflect UTC timezone
    assert!(json.get("date").is_some());

    // Test with different timezone
    let output = Command::cargo_bin("pomo-tui")
        .unwrap()
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .env("TZ", "America/New_York")
        .args(&["--format", "json", "statistics", "get", "--period", "today"])
        .output()
        .expect("Failed to execute command");

    let json2: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Dates might differ if crossing day boundary
    assert!(json2.get("date").is_some());

    panic!("Contract test must fail - timezone handling not implemented");
}

#[test]
fn test_statistics_performance() {
    // Test that statistics queries perform well with large datasets
    // Contract: Should handle reasonable amounts of historical data efficiently

    let temp_dir = TempDir::new().unwrap();

    // This test would ideally create a large dataset first
    // For contract testing, we just verify the query completes reasonably fast
    let start = std::time::Instant::now();

    let _output = Command::cargo_bin("pomo-tui")
        .unwrap()
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "statistics", "get", "--period", "all"])
        .output()
        .expect("Failed to execute command");

    let duration = start.elapsed();

    // Should complete within reasonable time (1 second for contract test)
    assert!(duration.as_secs() < 1);

    panic!("Contract test must fail - statistics performance not implemented");
}
