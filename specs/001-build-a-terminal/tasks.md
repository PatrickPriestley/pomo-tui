# Implementation Tasks: ADHD-Focused Pomodoro Terminal Application

**Feature Branch**: `001-build-a-terminal`
**Generated**: 2025-09-08
**Status**: Ready for implementation

## Overview
Implementation tasks for building a Rust-based terminal pomodoro timer with ADHD-focused features, following TDD principles with contract-first development.

## Prerequisites Checklist
- [ ] Rust 1.75+ installed (`rustup update stable`)
- [ ] SQLite3 installed (`brew install sqlite3` or system package manager)
- [ ] Git configured for the project
- [ ] Read `/specs/001-build-a-terminal/plan.md` for architecture overview
- [ ] Reviewed `/specs/001-build-a-terminal/data-model.md` for schema
- [ ] Checked `/specs/001-build-a-terminal/contracts/cli-api.yaml` for API

## Task Categories
- **Setup [S]**: Project initialization, dependencies, structure
- **Contract Tests [CT]**: API contract tests that must fail first
- **Data Layer [D]**: Database, models, migrations
- **Core Libraries [L]**: Business logic libraries  
- **Integration [I]**: External service connectors
- **UI Components [U]**: Terminal interface components
- **End-to-End [E]**: Full workflow tests
- **Polish [P]**: Documentation, performance, packaging

## Parallel Execution Groups

### Group A: Initial Setup (Sequential)
```bash
# Must be done in order
T001 → T002 → T003 → T004 → T005
```

### Group B: Contract Tests (Parallel)
```bash
# Can run simultaneously after setup
Task agents for: T006[P], T007[P], T008[P], T009[P], T010[P]
```

### Group C: Data Models (Parallel)
```bash
# Can run simultaneously after migrations
Task agents for: T016[P], T017[P], T018[P], T019[P], T020[P]
```

### Group D: Core Libraries (Parallel)
```bash
# Can run simultaneously after models
Task agents for: T021[P], T022[P], T023[P], T024[P]
```

---

## Setup Tasks

### T001 [S]: Initialize Rust Project
**File**: `Cargo.toml`
```toml
[package]
name = "pomo-tui"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.26"
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4", features = ["derive"] }
rodio = "0.17"
directories = "5"
toml = "0.8"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```
- Create project structure: `cargo init --name pomo-tui`
- Add dependencies listed above
- Set up workspace structure if needed

### T002 [S]: Configure Project Structure
**Files**: `src/lib.rs`, directory structure
```
src/
├── lib.rs
├── main.rs
├── cli/
│   └── mod.rs
├── core/
│   ├── mod.rs
│   ├── timer.rs
│   ├── task.rs
│   └── session.rs
├── database/
│   ├── mod.rs
│   ├── migrations.rs
│   └── models.rs
├── tui/
│   ├── mod.rs
│   ├── app.rs
│   └── widgets/
├── integrations/
│   ├── mod.rs
│   ├── git.rs
│   ├── github.rs
│   └── slack.rs
└── audio/
    └── mod.rs
```
- Create all directories
- Add module declarations in lib.rs
- Set up basic error types with thiserror

### T003 [S]: Set Up Logging and Error Handling
**Files**: `src/lib.rs`, `src/error.rs`
```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PomoError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}

// src/lib.rs - logging setup
use tracing_subscriber;

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("pomo_tui=debug")
        .init();
}
```

### T004 [S]: Create Database Migrations
**Files**: `migrations/001_initial_schema.sql`
```sql
-- Create all tables from data-model.md
-- Tasks, Sessions, Breaks, Preferences, Statistics, Integrations, AudioFiles
```
- Copy schema from `/specs/001-build-a-terminal/data-model.md`
- Create migrations directory
- Set up sqlx migrations

### T005 [S]: Configure CLI Argument Parser
**File**: `src/cli/mod.rs`
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pomo-tui")]
#[command(about = "ADHD-focused Pomodoro timer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Task(TaskCommand),
    Session(SessionCommand),
    // ... other commands
}
```

---

## Contract Test Tasks (Must Fail First!)

### T006 [CT][P]: Task CRUD Contract Tests
**File**: `tests/contract/task_api.rs`
```rust
// Test createTask, getTask, updateTask, deleteTask, listTasks
// Must fail initially (no implementation)
#[test]
fn test_create_task_contract() {
    // POST /cli/task
    panic!("Not implemented");
}
```

### T007 [CT][P]: Session Management Contract Tests
**File**: `tests/contract/session_api.rs`
```rust
// Test startSession, pauseSession, resumeSession, completeSession, abandonSession
#[test]
fn test_start_session_contract() {
    // POST /cli/session
    panic!("Not implemented");
}
```

### T008 [CT][P]: Break Management Contract Tests
**File**: `tests/contract/break_api.rs`
```rust
// Test startBreak, getCurrentBreak, skipBreak
#[test]
fn test_start_break_contract() {
    // POST /cli/break
    panic!("Not implemented");
}
```

### T009 [CT][P]: Statistics Contract Tests
**File**: `tests/contract/statistics_api.rs`
```rust
// Test getStatistics with different periods
#[test]
fn test_get_statistics_contract() {
    // GET /cli/statistics
    panic!("Not implemented");
}
```

### T010 [CT][P]: Preferences Contract Tests
**File**: `tests/contract/preferences_api.rs`
```rust
// Test getPreferences, updatePreferences
#[test]
fn test_get_preferences_contract() {
    // GET /cli/preferences
    panic!("Not implemented");
}
```

### T011 [CT][P]: Export Contract Tests
**File**: `tests/contract/export_api.rs`
```rust
// Test exportData with JSON, CSV, Markdown formats
#[test]
fn test_export_json_contract() {
    // GET /cli/export?format=json
    panic!("Not implemented");
}
```

---

## Integration Test Tasks

### T012 [I]: Database Connection Integration Test
**File**: `tests/integration/database_test.rs`
```rust
// Test database connection, migrations, basic queries
#[tokio::test]
async fn test_database_connection() {
    let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    // Verify tables exist
}
```

### T013 [I]: Timer Precision Integration Test
**File**: `tests/integration/timer_test.rs`
```rust
// Test timer accuracy over 25 minutes
// Verify <100ms drift requirement
#[test]
fn test_timer_precision() {
    // Create timer, run for simulated time, check drift
}
```

### T014 [I]: Session State Machine Integration Test
**File**: `tests/integration/session_state_test.rs`
```rust
// Test state transitions: active → paused → active → completed
#[test]
fn test_session_state_transitions() {
    // Verify all valid state transitions
}
```

### T015 [I]: Break Scheduling Integration Test
**File**: `tests/integration/break_scheduling_test.rs`
```rust
// Test automatic break scheduling after sessions
// Test long break after 4 sessions
#[test]
fn test_break_after_session() {
    // Complete session, verify break scheduled
}
```

---

## Data Layer Tasks

### T016 [D][P]: Implement Task Model
**File**: `src/database/models/task.rs`
```rust
use sqlx::FromRow;
use serde::{Serialize, Deserialize};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub status: String,
    // ... all fields from schema
}

impl Task {
    pub async fn create(pool: &SqlitePool, input: TaskInput) -> Result<Self> {
        // INSERT INTO tasks ...
    }
    
    pub async fn find(pool: &SqlitePool, id: i64) -> Result<Self> {
        // SELECT * FROM tasks WHERE id = ?
    }
    
    // update, delete, list methods
}
```

### T017 [D][P]: Implement Session Model
**File**: `src/database/models/session.rs`
```rust
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub task_id: i64,
    pub start_time: DateTime<Utc>,
    pub status: String,
    // ... all fields from schema
}

impl Session {
    // CRUD methods
    pub async fn get_active(pool: &SqlitePool) -> Result<Option<Self>> {
        // SELECT * FROM sessions WHERE status = 'active'
    }
}
```

### T018 [D][P]: Implement Break Model
**File**: `src/database/models/break.rs`
```rust
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Break {
    pub id: i64,
    pub after_session_id: i64,
    pub break_type: String,
    // ... all fields
}

impl Break {
    // CRUD methods
    pub async fn schedule_after_session(pool: &SqlitePool, session_id: i64) -> Result<Self> {
        // Determine break type based on session count
    }
}
```

### T019 [D][P]: Implement Preferences Model
**File**: `src/database/models/preferences.rs`
```rust
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Preferences {
    pub session_duration: i32,
    pub short_break_duration: i32,
    // ... all fields
}

impl Preferences {
    pub async fn get_or_create(pool: &SqlitePool) -> Result<Self> {
        // Single row table logic
    }
}
```

### T020 [D][P]: Implement Statistics Model
**File**: `src/database/models/statistics.rs`
```rust
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Statistics {
    pub date: NaiveDate,
    pub completed_sessions: i32,
    // ... all fields
}

impl Statistics {
    pub async fn calculate_for_date(pool: &SqlitePool, date: NaiveDate) -> Result<Self> {
        // Aggregate data for given date
    }
}
```

---

## Core Library Tasks

### T021 [L][P]: Timer Core Library
**File**: `src/core/timer.rs`
```rust
use std::time::{Duration, Instant};

pub struct Timer {
    start: Instant,
    duration: Duration,
    paused_at: Option<Instant>,
    total_pause_duration: Duration,
}

impl Timer {
    pub fn new(seconds: u64) -> Self {
        Timer {
            start: Instant::now(),
            duration: Duration::from_secs(seconds),
            paused_at: None,
            total_pause_duration: Duration::ZERO,
        }
    }
    
    pub fn remaining(&self) -> Duration {
        // Calculate remaining time accounting for pauses
    }
    
    pub fn pause(&mut self) {
        // Pause timer
    }
    
    pub fn resume(&mut self) {
        // Resume timer
    }
}

// CLI exposure
pub fn timer_cli() {
    // --help, --version, --format support
}
```

### T022 [L][P]: Task Manager Library
**File**: `src/core/task_manager.rs`
```rust
pub struct TaskManager {
    pool: SqlitePool,
}

impl TaskManager {
    pub async fn create_task(&self, input: TaskInput) -> Result<Task> {
        Task::create(&self.pool, input).await
    }
    
    pub async fn prioritize_tasks(&self) -> Result<Vec<Task>> {
        // Get tasks ordered by priority
    }
    
    // Other task operations
}

// CLI exposure
pub fn task_cli() {
    // Task subcommands
}
```

### T023 [L][P]: Session State Machine Library
**File**: `src/core/session_state.rs`
```rust
pub enum SessionState {
    Active,
    Paused,
    Completed,
    Abandoned,
}

pub struct SessionStateMachine {
    current_state: SessionState,
    session: Session,
}

impl SessionStateMachine {
    pub fn transition(&mut self, event: SessionEvent) -> Result<()> {
        // Handle state transitions with validation
    }
}
```

### T024 [L][P]: Audio Playback Library
**File**: `src/audio/player.rs`
```rust
use rodio::{Decoder, OutputStream, Sink};

pub struct AudioPlayer {
    sink: Sink,
}

impl AudioPlayer {
    pub fn play_ambient(&self, sound_type: &str) {
        // Play brown_noise, white_noise, etc.
    }
    
    pub fn play_notification(&self) {
        // Play completion sound
    }
}

// CLI exposure
pub fn audio_cli() {
    // Audio test commands
}
```

---

## Integration Tasks

### T025 [I]: Git Integration
**File**: `src/integrations/git.rs`
```rust
pub struct GitIntegration;

impl GitIntegration {
    pub fn enhance_commit_message(&self, msg: &str, session: &Session) -> String {
        // Add session context to commit message
        format!("{} [{}min: {}]", msg, session.duration_minutes(), session.task_title())
    }
}
```

### T026 [I]: GitHub Integration
**File**: `src/integrations/github.rs`
```rust
use octocrab;

pub struct GitHubIntegration {
    client: octocrab::Octocrab,
}

impl GitHubIntegration {
    pub async fn sync_issues(&self) -> Result<Vec<GitHubIssue>> {
        // Fetch assigned issues
    }
}
```

### T027 [I]: Slack Integration
**File**: `src/integrations/slack.rs`
```rust
pub struct SlackIntegration {
    token: String,
}

impl SlackIntegration {
    pub async fn update_status(&self, status: &str) -> Result<()> {
        // Update Slack status via API
    }
}
```

### T028 [I]: Website Blocking
**File**: `src/integrations/website_blocker.rs`
```rust
use std::fs;

pub struct WebsiteBlocker {
    hosts_backup: Option<String>,
}

impl WebsiteBlocker {
    pub fn block_websites(&mut self, domains: &[String]) -> Result<()> {
        // Backup and modify /etc/hosts
    }
    
    pub fn unblock_websites(&mut self) -> Result<()> {
        // Restore original /etc/hosts
    }
}
```

---

## UI Component Tasks

### T029 [U]: Main Application State
**File**: `src/tui/app.rs`
```rust
use ratatui::prelude::*;

pub struct App {
    current_tab: Tab,
    tasks: Vec<Task>,
    current_session: Option<Session>,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        // Initialize app state
    }
    
    pub fn update(&mut self, msg: Message) -> Result<()> {
        // Handle state updates
    }
    
    pub fn draw(&self, frame: &mut Frame) {
        // Render UI
    }
}
```

### T030 [U]: Task List Widget
**File**: `src/tui/widgets/task_list.rs`
```rust
pub struct TaskListWidget<'a> {
    tasks: &'a [Task],
    selected: Option<usize>,
}

impl Widget for TaskListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render task list with priorities
    }
}
```

### T031 [U]: Timer Display Widget
**File**: `src/tui/widgets/timer_display.rs`
```rust
pub struct TimerWidget {
    remaining: Duration,
    total: Duration,
}

impl Widget for TimerWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render timer with progress bar
    }
}
```

### T032 [U]: Statistics Dashboard Widget
**File**: `src/tui/widgets/statistics.rs`
```rust
pub struct StatisticsWidget {
    stats: Statistics,
}

impl Widget for StatisticsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render daily/weekly statistics
    }
}
```

### T033 [U]: Keyboard Input Handler
**File**: `src/tui/input.rs`
```rust
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('n') => app.new_task(),
        KeyCode::Char('s') => app.start_session(),
        KeyCode::Char('p') => app.pause_resume(),
        KeyCode::Char('q') => app.should_quit = true,
        // ... other shortcuts
    }
}
```

### T034 [U]: Settings Screen
**File**: `src/tui/widgets/settings.rs`
```rust
pub struct SettingsWidget {
    preferences: Preferences,
}

impl Widget for SettingsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render settings form
    }
}
```

---

## End-to-End Test Tasks

### T035 [E]: Complete Pomodoro Cycle Test
**File**: `tests/e2e/pomodoro_cycle.rs`
```rust
#[test]
fn test_complete_pomodoro_cycle() {
    // Create task
    // Start session
    // Complete session
    // Take break
    // Verify statistics updated
}
```

### T036 [E]: Data Persistence Test
**File**: `tests/e2e/persistence.rs`
```rust
#[test]
fn test_session_recovery() {
    // Start session
    // Simulate crash
    // Restart app
    // Verify session recovery prompt
}
```

### T037 [E]: Export Functionality Test
**File**: `tests/e2e/export.rs`
```rust
#[test]
fn test_export_formats() {
    // Create data
    // Export as JSON
    // Export as CSV  
    // Export as Markdown
    // Verify formats
}
```

---

## Polish Tasks

### T038 [P][P]: Performance Optimization
**File**: `src/core/performance.rs`
- Profile startup time
- Optimize database queries
- Implement query caching
- Verify <50ms startup requirement

### T039 [P][P]: Package for Distribution
**File**: `Cargo.toml`, CI/CD scripts
- Configure release builds
- Create GitHub Actions workflow
- Generate shell completions
- Create man pages

### T040 [P][P]: Homebrew Formula
**File**: `homebrew/pomo-tui.rb`
```ruby
class PomoTui < Formula
  desc "ADHD-focused Pomodoro terminal application"
  homepage "https://github.com/pomo-tui/pomo-tui"
  url "https://github.com/pomo-tui/pomo-tui/archive/v0.1.0.tar.gz"
  sha256 "..."
  
  depends_on "rust" => :build
  
  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end
end
```

### T041 [P][P]: User Documentation
**File**: `README.md`, `docs/`
- Write comprehensive README
- Create user guide
- Document keyboard shortcuts
- Add contribution guidelines

### T042 [P][P]: Integration Documentation
**File**: `docs/integrations.md`
- Document GitHub setup
- Document Slack OAuth flow
- Document website blocking setup
- Create troubleshooting guide

### T043 [P][P]: Unit Tests for Utilities
**File**: `tests/unit/`
- Test timer precision
- Test state machines
- Test data validators
- Achieve 80% code coverage

### T044 [P][P]: Configuration Templates
**File**: `config/`
- Create default config.toml
- Add example blocked websites list
- Create systemd service file
- Add launchd plist for macOS

### T045 [P][P]: Final Integration Verification
**File**: `tests/integration/full_stack.rs`
- Run quickstart.md scenarios
- Verify all acceptance criteria
- Performance benchmarks
- Memory leak detection

---

## Execution Order Summary

### Phase 1: Foundation (Sequential)
```
T001 → T002 → T003 → T004 → T005
```

### Phase 2: Contract Tests (Parallel)
```
T006[P] + T007[P] + T008[P] + T009[P] + T010[P] + T011[P]
```

### Phase 3: Integration Tests (Mixed)
```
T012 → (T013[P] + T014[P] + T015[P])
```

### Phase 4: Data Layer (Parallel)
```
T016[P] + T017[P] + T018[P] + T019[P] + T020[P]
```

### Phase 5: Core Libraries (Parallel)
```
T021[P] + T022[P] + T023[P] + T024[P]
```

### Phase 6: Integrations (Parallel)
```
T025[P] + T026[P] + T027[P] + T028[P]
```

### Phase 7: UI Components (Sequential with some parallel)
```
T029 → (T030[P] + T031[P] + T032[P]) → T033 → T034
```

### Phase 8: End-to-End Tests (Parallel)
```
T035[P] + T036[P] + T037[P]
```

### Phase 9: Polish (Parallel)
```
T038[P] + T039[P] + T040[P] + T041[P] + T042[P] + T043[P] + T044[P] → T045
```

---

## Validation Checklist

Before marking the feature complete:

- [ ] All contract tests pass
- [ ] All integration tests pass
- [ ] All end-to-end tests pass
- [ ] Startup time <50ms verified
- [ ] Timer precision <100ms drift verified
- [ ] Memory usage <50MB verified
- [ ] All keyboard shortcuts working
- [ ] Export formats validated
- [ ] Integrations tested
- [ ] Documentation complete
- [ ] Package builds for all platforms
- [ ] Quickstart guide scenarios pass

---

## Notes for Implementers

1. **TDD is mandatory**: Every implementation task must have a failing test first
2. **Parallel execution**: Tasks marked [P] can be run simultaneously
3. **Dependencies**: Ensure database migrations (T004) complete before data layer tasks
4. **Testing order**: Contract → Integration → Implementation → Unit
5. **Git commits**: Show test implementation before feature implementation
6. **Performance**: Profile regularly, especially startup time
7. **Cross-platform**: Test on macOS, Linux, and Windows
8. **Accessibility**: Ensure keyboard navigation works throughout

---

*Generated from `/specs/001-build-a-terminal/plan.md` following Constitutional principles*