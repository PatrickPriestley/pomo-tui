# Claude Code Context: Pomo-TUI

## Project Overview
ADHD-focused Pomodoro terminal application built with Rust and Ratatui for high-performance, distraction-free productivity.

## Tech Stack
- **Language**: Rust 1.75+
- **UI Framework**: Ratatui (terminal UI)
- **Testing**: cargo test with TDD approach
- **Async Runtime**: tokio
- **Cross-platform**: crossterm for terminal handling

## Project Structure
```
src/
├── core/          # Core logic
│   ├── timer.rs   # Timer logic, Duration-based precision
│   ├── breathing.rs # Breathing exercises for breaks
│   └── mod.rs     # Core module exports
├── tui/           # Terminal UI components
│   ├── app.rs     # Main application state and logic
│   ├── ui.rs      # UI rendering and layout
│   └── mod.rs     # TUI module exports
├── integrations/  # Platform integrations
│   ├── macos_dnd.rs # macOS Focus mode integration
│   └── mod.rs     # Integration module exports
├── lib.rs         # Library root
└── main.rs        # Application entry point

tests/
├── contract/      # API contract tests
├── integration/   # Cross-module tests
└── unit/          # Module-specific tests
```

## Key Features
- **25-minute Pomodoro sessions** with automatic break management
- **ADHD-focused design** with breathing exercises during breaks
- **macOS Focus mode integration** for distraction-free sessions
- **Terminal-based interface** using Ratatui for high performance
- **Break customization** with shorten/extend options
- **Session tracking** with automatic long breaks after 4 sessions

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