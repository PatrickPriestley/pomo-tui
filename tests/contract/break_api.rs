use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::process::Command;
use tempfile::TempDir;

/// Contract tests for Break Management operations
/// These tests verify the CLI API matches the OpenAPI specification
/// All tests MUST fail initially (no implementation exists yet)

#[test]
fn test_start_break_contract() {
    // Test: POST /cli/break (startBreak)
    // Contract: Start a break after completing a session

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&[
        "break",
        "start",
        "--session-id",
        "1",
        "--break-type",
        "short",
    ])
    .assert()
    .success();

    panic!("Contract test must fail - startBreak not implemented");
}

#[test]
fn test_start_break_json_output() {
    // Test JSON format output for break creation

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&[
            "--format",
            "json",
            "break",
            "start",
            "--session-id",
            "1",
            "--break-type",
            "long",
        ])
        .output()
        .expect("Failed to execute command");

    // Should return JSON with break structure
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    assert!(json.get("id").is_some());
    assert_eq!(json["after_session_id"], 1);
    assert_eq!(json["break_type"], "long");
    assert_eq!(json["planned_duration"], 900); // 15 minutes for long break
    assert!(json.get("start_time").is_some());

    panic!("Contract test must fail - break JSON output not implemented");
}

#[test]
fn test_start_short_break_duration() {
    // Test short break has correct default duration (5 minutes = 300 seconds)

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&[
            "--format",
            "json",
            "break",
            "start",
            "--session-id",
            "1",
            "--break-type",
            "short",
        ])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    assert_eq!(json["planned_duration"], 300); // 5 minutes

    panic!("Contract test must fail - short break duration not implemented");
}

#[test]
fn test_start_long_break_duration() {
    // Test long break has correct default duration (15 minutes = 900 seconds)

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&[
            "--format",
            "json",
            "break",
            "start",
            "--session-id",
            "1",
            "--break-type",
            "long",
        ])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    assert_eq!(json["planned_duration"], 900); // 15 minutes

    panic!("Contract test must fail - long break duration not implemented");
}

#[test]
fn test_start_break_with_activity() {
    // Test starting break with activity type

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&[
            "--format",
            "json",
            "break",
            "start",
            "--session-id",
            "1",
            "--activity",
            "breathing",
        ])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    assert_eq!(json["activity_type"], "breathing");

    panic!("Contract test must fail - break activities not implemented");
}

#[test]
fn test_get_current_break_contract() {
    // Test: GET /cli/break (getCurrentBreak)
    // Contract: Get currently active break

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["break", "current"])
    .assert()
    .success();

    panic!("Contract test must fail - getCurrentBreak not implemented");
}

#[test]
fn test_get_current_break_not_found() {
    // Test 404 when no active break exists

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["break", "current"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("No active break"));

    panic!("Contract test must fail - no break error handling not implemented");
}

#[test]
fn test_skip_break_contract() {
    // Test: POST /cli/break/{id}/skip (skipBreak)
    // Contract: Skip an active break

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["break", "skip", "1"])
    .assert()
    .success();

    panic!("Contract test must fail - skipBreak not implemented");
}

#[test]
fn test_skip_nonexistent_break() {
    // Test 404 when trying to skip non-existent break

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["break", "skip", "999"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Break not found"));

    panic!("Contract test must fail - skip error handling not implemented");
}

#[test]
fn test_break_automatic_scheduling() {
    // Test that breaks are automatically scheduled after sessions
    // Contract: Short break after each session, long break after 4 sessions

    let temp_dir = TempDir::new().unwrap();

    // Complete 3 sessions - should suggest short breaks
    for i in 1..=3 {
        let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
        let output = cmd
            .env(
                "DATABASE_URL",
                format!("sqlite:{}/test.db", temp_dir.path().display()),
            )
            .args(&["--format", "json", "session", "complete", &i.to_string()])
            .output()
            .expect("Failed to execute command");

        let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
            .expect("Invalid JSON output");

        // Should suggest short break
        assert_eq!(json["suggested_break_type"], "short");
    }

    // Complete 4th session - should suggest long break
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "session", "complete", "4"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Should suggest long break after 4 sessions
    assert_eq!(json["suggested_break_type"], "long");

    panic!("Contract test must fail - automatic break scheduling not implemented");
}

#[test]
fn test_break_activity_types() {
    // Test all valid activity types: breathing, movement, rest, custom

    let temp_dir = TempDir::new().unwrap();
    let activities = ["breathing", "movement", "rest", "custom"];

    for activity in &activities {
        let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
        cmd.env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&[
            "break",
            "start",
            "--session-id",
            "1",
            "--activity",
            activity,
        ])
        .assert()
        .success();
    }

    panic!("Contract test must fail - activity types not implemented");
}

#[test]
fn test_break_completion_tracking() {
    // Test that breaks track actual duration vs planned duration

    let temp_dir = TempDir::new().unwrap();

    // Start break
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["break", "start", "--session-id", "1"])
    .assert()
    .success();

    // Skip break (should record actual_duration)
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "break", "skip", "1"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    assert_eq!(json["skipped"], true);
    assert!(json.get("actual_duration").is_some());
    assert!(json.get("end_time").is_some());

    panic!("Contract test must fail - break completion tracking not implemented");
}

#[test]
fn test_break_validation() {
    // Test input validation for break creation

    let temp_dir = TempDir::new().unwrap();

    // Test invalid break type
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&[
        "break",
        "start",
        "--session-id",
        "1",
        "--break-type",
        "invalid",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Invalid break type"));

    // Test invalid session ID
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["break", "start", "--session-id", "0"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Invalid session"));

    panic!("Contract test must fail - break validation not implemented");
}

#[test]
fn test_break_timer_precision() {
    // Test that break timer maintains accuracy
    // Contract: Break timing should be accurate for user experience

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    // Start a short break (for testing purposes)
    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&[
        "break",
        "start",
        "--session-id",
        "1",
        "--break-type",
        "short",
    ])
    .assert()
    .success();

    // Check remaining time is accurate
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "break", "current"])
        .output()
        .expect("Failed to execute command");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    // Should have remaining time close to planned duration
    let remaining = json["remaining_seconds"].as_i64().unwrap();
    assert!(remaining > 290 && remaining <= 300); // Within 10 seconds of 5 minutes

    panic!("Contract test must fail - break timer not implemented");
}
