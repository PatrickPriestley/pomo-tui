use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::process::Command;
use tempfile::TempDir;

/// Contract tests for Task CRUD operations
/// These tests verify the CLI API matches the OpenAPI specification
/// All tests MUST fail initially (no implementation exists yet)

#[test]
fn test_create_task_contract() {
    // Test: POST /cli/task (createTask)
    // Contract: Create a new task with required title

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&[
        "task",
        "create",
        "Test Task",
        "--priority",
        "2",
        "--estimated",
        "3",
    ])
    .assert()
    .success(); // This WILL FAIL - no implementation yet

    panic!("Contract test must fail - createTask not implemented");
}

#[test]
fn test_create_task_json_output() {
    // Test JSON format output for task creation

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    let output = cmd
        .env(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", temp_dir.path().display()),
        )
        .args(&["--format", "json", "task", "create", "JSON Task"])
        .output()
        .expect("Failed to execute command");

    // Should return JSON with task structure
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("Invalid JSON output");

    assert!(json.get("id").is_some());
    assert_eq!(json["title"], "JSON Task");
    assert_eq!(json["status"], "pending");

    panic!("Contract test must fail - JSON output not implemented");
}

#[test]
fn test_list_tasks_contract() {
    // Test: GET /cli/task (listTasks)
    // Contract: List all tasks with optional filtering

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "list"])
    .assert()
    .success();

    panic!("Contract test must fail - listTasks not implemented");
}

#[test]
fn test_list_tasks_with_filters() {
    // Test filtering by status and priority

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "list", "--status", "pending", "--priority", "2"])
    .assert()
    .success();

    panic!("Contract test must fail - task filtering not implemented");
}

#[test]
fn test_get_task_contract() {
    // Test: GET /cli/task/{id} (getTask)
    // Contract: Get specific task by ID

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "get", "1"])
    .assert()
    .success();

    panic!("Contract test must fail - getTask not implemented");
}

#[test]
fn test_get_nonexistent_task() {
    // Test 404 behavior for non-existent task

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "get", "999"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Task not found"));

    panic!("Contract test must fail - error handling not implemented");
}

#[test]
fn test_update_task_contract() {
    // Test: PUT /cli/task/{id} (updateTask)
    // Contract: Update existing task fields

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&[
        "task",
        "update",
        "1",
        "--title",
        "Updated Task",
        "--priority",
        "3",
    ])
    .assert()
    .success();

    panic!("Contract test must fail - updateTask not implemented");
}

#[test]
fn test_update_partial_task() {
    // Test partial updates (only some fields)

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&[
        "task",
        "update",
        "1",
        "--description",
        "New description only",
    ])
    .assert()
    .success();

    panic!("Contract test must fail - partial updates not implemented");
}

#[test]
fn test_delete_task_contract() {
    // Test: DELETE /cli/task/{id} (deleteTask)
    // Contract: Delete task by ID

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "delete", "1"])
    .assert()
    .success();

    panic!("Contract test must fail - deleteTask not implemented");
}

#[test]
fn test_delete_nonexistent_task() {
    // Test 404 behavior for deleting non-existent task

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();

    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "delete", "999"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Task not found"));

    panic!("Contract test must fail - delete error handling not implemented");
}

#[test]
fn test_task_validation_contract() {
    // Test input validation according to schema

    let temp_dir = TempDir::new().unwrap();

    // Test empty title (should fail)
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "create", ""])
    .assert()
    .failure()
    .stderr(predicate::str::contains("title"));

    // Test invalid priority (should fail)
    let mut cmd = Command::cargo_bin("pomo-tui").unwrap();
    cmd.env(
        "DATABASE_URL",
        format!("sqlite:{}/test.db", temp_dir.path().display()),
    )
    .args(&["task", "create", "Test", "--priority", "5"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("priority"));

    panic!("Contract test must fail - validation not implemented");
}
