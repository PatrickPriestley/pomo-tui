# Research Findings: ADHD-Focused Pomodoro Terminal Application

**Date**: 2025-09-08
**Feature**: 001-build-a-terminal

## Executive Summary
Research conducted to resolve technical unknowns and establish best practices for building a Rust-based terminal pomodoro timer with ADHD-focused features.

## Research Areas & Decisions

### 1. Website Blocking Mechanism
**Decision**: Use `/etc/hosts` file modification with elevated permissions
**Rationale**: 
- Cross-platform compatible (works on macOS, Linux, Windows)
- Simple and reliable mechanism
- Can be automated with sudo/UAC prompts
- Easy to reverse when session ends
**Alternatives Considered**:
- Firewall rules: Too complex, platform-specific
- Browser extensions: Requires browser-specific integration
- Network proxy: Overly complex for the use case
**Implementation**: Create a hosts manager module that backs up original hosts file and modifies it during sessions

### 2. Audio Playback in Rust
**Decision**: Use `rodio` crate for audio playback
**Rationale**:
- Pure Rust implementation
- Cross-platform support
- Simple API for playing audio files
- Supports common formats (WAV, MP3, OGG)
**Alternatives Considered**:
- `cpal`: Lower-level, more complex than needed
- `SDL2`: Requires external dependencies
- System commands: Platform-specific, less reliable
**Implementation**: Pre-load audio files at startup for instant playback

### 3. GitHub Issues Integration
**Decision**: Read-only sync with manual task import
**Rationale**:
- Simpler implementation
- Avoids complex conflict resolution
- User maintains control over task management
- Can mark tasks as "linked" to GitHub issues
**Alternatives Considered**:
- Bidirectional sync: Complex conflict resolution needed
- Write-only: Limited utility for users
**Implementation**: Use `octocrab` crate for GitHub API, store issue IDs with tasks

### 4. Jira Integration
**Decision**: API key authentication with read-only sync
**Rationale**:
- API keys simpler than OAuth for CLI
- Most Jira instances support API keys
- Read-only avoids permission complexities
**Alternatives Considered**:
- OAuth 2.0: Complex flow for CLI apps
- Basic auth: Deprecated by Atlassian
- Personal Access Tokens: Not universally available
**Implementation**: Store encrypted API key in config, use `reqwest` for REST calls

### 5. Slack Status Updates
**Decision**: Use Slack App with OAuth 2.0 token stored locally
**Rationale**:
- Official supported method
- Can update status and presence
- Token can be refreshed programmatically
**Alternatives Considered**:
- Webhook: Cannot update user status
- Legacy tokens: Deprecated
- xoxp tokens: Security concerns
**Implementation**: One-time OAuth flow via browser, store refresh token encrypted

### 6. Data Export Formats
**Decision**: Support JSON, CSV, and Markdown formats
**Rationale**:
- JSON: Machine-readable, preserves structure
- CSV: Excel/spreadsheet compatible
- Markdown: Human-readable reports
**Alternatives Considered**:
- XML: Overly verbose
- SQLite dump: Not user-friendly
- PDF: Requires heavy dependencies
**Implementation**: Use `serde_json`, `csv` crate, and template engine for Markdown

### 7. Session Recovery
**Decision**: Write state to SQLite every 30 seconds during active sessions
**Rationale**:
- SQLite transactions ensure consistency
- 30-second interval balances performance and safety
- Can detect incomplete sessions on startup
**Alternatives Considered**:
- Continuous writes: Performance impact
- Memory-only with crash dump: Complex signal handling
- File-based checkpoints: Less reliable than DB
**Implementation**: Background thread for periodic state persistence, recovery prompt on startup

## Technical Best Practices

### Ratatui Architecture
**Decision**: Event-driven architecture with state machines
**Key Patterns**:
- Separate UI state from business logic
- Use channels for inter-component communication
- Implement Command pattern for user actions
- Virtual DOM-like rendering for efficiency

### SQLite Schema Migrations
**Decision**: Use `sqlx` with embedded migrations
**Rationale**:
- Compile-time checked queries
- Built-in migration support
- Async support for future needs
**Migration Strategy**:
- Numbered migration files
- Up/down migrations for rollback
- Automatic migration on startup

### Error Handling
**Decision**: Use `anyhow` for application errors, `thiserror` for library errors
**Rationale**:
- Clear separation between library and application concerns
- Good error chaining and context
- Integrates well with Result type

### Configuration Management
**Decision**: TOML configuration with XDG Base Directory compliance
**Rationale**:
- TOML is human-readable and editable
- XDG ensures proper file locations
- Layered config (defaults → system → user)

## Performance Considerations

### Timer Precision
**Approach**: Use `std::time::Instant` for monotonic time
- Not affected by system clock changes
- Sub-millisecond precision
- Check timer every 100ms for UI updates

### UI Rendering
**Approach**: Differential rendering with dirty flags
- Only redraw changed components
- Batch updates in 16ms windows (60fps)
- Use double-buffering to prevent flicker

### Database Performance
**Approach**: 
- WAL mode for concurrent reads
- Prepared statements for repeated queries
- Indexes on frequently queried columns
- Vacuum on application exit

## Security Considerations

### Credential Storage
**Decision**: Use OS keychain where available, encrypted file fallback
**Implementation**:
- macOS: Keychain Services
- Linux: Secret Service API
- Windows: Credential Manager
- Fallback: AES-256 encrypted SQLite

### Integration Security
- Never log credentials
- Use environment variables for CI/CD
- Implement credential rotation reminders
- Clear sensitive data from memory after use

## Accessibility Features

### Keyboard Navigation
- Vim-style keys for power users
- Arrow keys for standard navigation
- Tab/Shift-Tab for focus management
- Single-key shortcuts for common actions

### Screen Reader Support
- Implement alternate text mode output
- Structured logging for action feedback
- Status announcements for timer changes

## Distribution Strategy

### Cargo Installation
- Publish to crates.io
- Include all required assets
- Feature flags for optional integrations

### Homebrew Formula
- Tap for initial distribution
- Move to core after stability
- Include shell completion scripts

### Pre-built Binaries
- GitHub Releases with CI/CD
- Support macOS (Intel + Apple Silicon), Linux (x64), Windows (x64)
- Include man pages and completions

## Unresolved Questions
None - all NEEDS CLARIFICATION items have been resolved.

## Next Steps
1. Create data model based on research decisions
2. Design API contracts for integrations
3. Implement failing tests following TDD
4. Begin modular implementation