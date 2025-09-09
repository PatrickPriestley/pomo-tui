# Claude Code Context: Pomo-TUI

## Project Overview
ADHD-focused Pomodoro terminal application built with Rust and Ratatui for high-performance, distraction-free productivity.

## Tech Stack
- **Language**: Rust 1.75+
- **UI Framework**: Ratatui (terminal UI)
- **Database**: SQLite with sqlx
- **Audio**: rodio for sound playback
- **Testing**: cargo test with TDD approach
- **Logging**: tracing crate

## Project Structure
```
src/
├── lib/           # Core libraries
│   ├── timer/     # Timer logic, Duration-based precision
│   ├── tasks/     # Task CRUD operations
│   ├── database/  # SQLite persistence layer
│   ├── tui/       # Ratatui UI components
│   ├── audio/     # Sound playback (rodio)
│   └── integrations/ # Git, GitHub, Jira, Slack
├── cli/           # CLI argument parsing
└── main.rs        # Application entry point

tests/
├── contract/      # API contract tests
├── integration/   # Cross-module tests
└── unit/          # Module-specific tests
```

## Key Patterns
- **Event-driven architecture** with state machines
- **Command pattern** for user actions
- **TDD mandatory**: Tests written first, must fail before implementation
- **Library-first**: Every feature as standalone library with CLI exposure

## Database Schema
- Tasks: id, title, priority, status, estimated_pomodoros
- Sessions: id, task_id, start_time, duration, status
- Breaks: id, session_id, type, duration
- Preferences: Single-row config table
- Statistics: Materialized daily aggregates

## Performance Requirements
- Startup: <50ms
- Timer precision: <100ms drift over 25 minutes
- UI updates: 60fps
- Memory: <50MB usage

## Testing Approach
1. Contract tests first (OpenAPI specs)
2. Integration tests for library boundaries
3. Unit tests last
4. Real dependencies (no mocks)
5. Tests must fail before implementation

## Current Phase
- Phase 1: Design complete
- Phase 2: Task planning ready
- Constitution compliance verified

## Integration Points
- Git: Automatic commit message enhancement
- GitHub: Read-only issue sync via octocrab
- Jira: API key auth, read-only sync
- Slack: OAuth 2.0 for status updates
- Website blocking: /etc/hosts modification

## Commands to Run
```bash
# Build and test
cargo build --release
cargo test

# Run application
cargo run

# Format and lint
cargo fmt
cargo clippy
```

## Development Workflow
1. Create failing test
2. Implement minimal code to pass
3. Refactor while keeping tests green
4. Commit with descriptive message
5. Update this file with significant changes

## Recent Changes
- Initial project setup with Rust/Ratatui
- SQLite schema designed
- OpenAPI contracts defined