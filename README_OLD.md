# pomo-tui

An ADHD-focused Pomodoro terminal application with comprehensive task management, statistics tracking, and integrations.

<p align="center">
  <img src="docs/demo.gif" alt="pomo-tui demo" width="800">
</p>

[![CI](https://github.com/pomo-tui/pomo-tui/workflows/CI/badge.svg)](https://github.com/pomo-tui/pomo-tui/actions)
[![Crates.io](https://img.shields.io/crates/v/pomo-tui.svg)](https://crates.io/crates/pomo-tui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/pomo-tui.svg)](https://crates.io/crates/pomo-tui)

## Features

### 🧠 ADHD-Focused Design
- **Low cognitive load**: Clean, distraction-free interface
- **Visual progress indicators**: Clear session and break timers  
- **Flexible session lengths**: Customizable 15-50 minute sessions
- **Gentle interruptions**: Optional ambient sounds and visual cues
- **Session recovery**: Resume interrupted sessions after crashes/restarts

### 📋 Task Management
- **Priority-based sorting**: Focus on what matters most
- **Project organization**: Group tasks by project
- **Tag system**: Flexible categorization and filtering
- **Progress tracking**: See completed sessions per task
- **Due date management**: Never miss deadlines

### 📊 Statistics & Analytics
- **Daily/weekly/monthly views**: Track your productivity patterns
- **Streak tracking**: Build and maintain focus habits  
- **Productivity scoring**: Understand your performance trends
- **Export capabilities**: JSON, CSV, and Markdown formats
- **Visual charts**: ASCII-based progress visualization

### 🔧 Integrations
- **GitHub**: Sync issues as tasks, enhance commit messages
- **Slack**: Update status during focus sessions
- **Git**: Automatic commit message enhancement with session context
- **Website blocking**: Temporary distraction blocking during sessions

### 🎵 Audio Support
- **Ambient sounds**: Brown noise, white noise, rain, forest
- **Session notifications**: Customizable completion sounds
- **Volume control**: Adjust audio levels per sound type
- **Silent mode**: Visual-only notifications

## Installation

### Homebrew (macOS/Linux)
```bash
# Add the tap
brew tap pomo-tui/tap

# Install pomo-tui
brew install pomo-tui
```

### Cargo (All platforms)
```bash
cargo install pomo-tui
```

### Pre-built Binaries
Download the latest release for your platform from [GitHub Releases](https://github.com/pomo-tui/pomo-tui/releases).

### From Source
```bash
git clone https://github.com/pomo-tui/pomo-tui.git
cd pomo-tui
cargo build --release
```

## Quick Start

1. **Create your first task:**
   ```bash
   pomo-tui task new "Complete project documentation" --priority 5 --project "work"
   ```

2. **Start a focus session:**
   ```bash
   pomo-tui session start
   ```

3. **Launch the TUI interface:**
   ```bash
   pomo-tui
   ```

4. **View your statistics:**
   ```bash
   pomo-tui stats --period week
   ```

## Usage

### Terminal Interface

Launch the interactive terminal interface:
```bash
pomo-tui
```

#### Keyboard Shortcuts
- `Tab` / `Shift+Tab`: Navigate between sections
- `↑↓` / `jk`: Move selection up/down
- `Enter`: Select/activate item
- `n`: Create new task
- `s`: Start session with selected task  
- `p`: Pause/resume current session
- `a`: Abandon current session
- `t`: Switch to timer view
- `k`: Switch to task list
- `d`: Switch to statistics dashboard
- `g`: Switch to settings
- `q`: Quit application

### Command Line Interface

#### Task Management
```bash
# Create tasks
pomo-tui task new "Task title" [options]
pomo-tui task new "Review code" --priority 4 --project "development"

# List tasks  
pomo-tui task list [--status pending] [--project work]

# Update tasks
pomo-tui task update <id> --status completed
pomo-tui task update <id> --priority 3

# Delete tasks
pomo-tui task delete <id>
```

#### Session Management
```bash
# Start sessions
pomo-tui session start [--task-id <id>] [--duration 25]

# Session controls  
pomo-tui session pause
pomo-tui session resume  
pomo-tui session complete
pomo-tui session abandon

# View current session
pomo-tui session status
```

#### Statistics
```bash
# View statistics
pomo-tui stats [--period day|week|month|year]
pomo-tui stats --from 2024-01-01 --to 2024-01-31

# Export data
pomo-tui export --format json --output data.json
pomo-tui export --format csv --output report.csv  
pomo-tui export --format markdown --output summary.md
```

#### Preferences
```bash
# View current settings
pomo-tui preferences show

# Update settings
pomo-tui preferences set session_duration 30
pomo-tui preferences set sound_enabled true
pomo-tui preferences set theme dark
```

#### Integrations
```bash
# Setup integrations
pomo-tui integration github setup
pomo-tui integration slack setup

# Test integrations  
pomo-tui integration github test
pomo-tui integration slack test
```

## Configuration

pomo-tui uses a TOML configuration file located at:
- **Linux**: `~/.config/pomo-tui/config.toml`
- **macOS**: `~/Library/Application Support/pomo-tui/config.toml`
- **Windows**: `%APPDATA%\pomo-tui\config.toml`

### Example Configuration
```toml
[session]
duration = 25                    # Default session length (minutes)
short_break_duration = 5         # Short break length  
long_break_duration = 15         # Long break after 4 sessions
sessions_until_long_break = 4    # Sessions before long break
auto_start_breaks = true         # Start breaks automatically
auto_start_sessions = false      # Require manual session start

[audio]  
sound_enabled = true
volume = 0.7
ambient_sound = "brown_noise"    # brown_noise, white_noise, rain, forest
notification_sound = "bell"      # bell, chime, soft_bell

[ui]
theme = "dark"                   # dark, light
show_seconds = true              # Show seconds in timer
show_session_count = true        # Show session number
show_daily_goal = true          # Show daily progress
daily_session_goal = 8          # Target sessions per day

[integrations]
github_enabled = false
slack_enabled = false  
git_commit_enhancement = true
website_blocking = false
blocked_websites = [
    "facebook.com",
    "twitter.com", 
    "reddit.com"
]

# GitHub integration (requires setup)  
[integrations.github]
token = "your_github_token"
repo_owner = "your-username"
repo_name = "your-repo"

# Slack integration (requires setup)
[integrations.slack]  
token = "your_slack_token"
user_id = "your_user_id"
```

## Integration Setup

### GitHub Integration
1. Create a GitHub Personal Access Token:
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Generate token with `repo` and `user` scopes
   
2. Configure pomo-tui:
   ```bash
   pomo-tui integration github setup --token YOUR_TOKEN
   ```

3. Sync issues as tasks:
   ```bash
   pomo-tui integration github sync
   ```

### Slack Integration  
1. Create a Slack App:
   - Go to https://api.slack.com/apps
   - Create new app and install to workspace
   - Copy the Bot User OAuth Token
   
2. Configure pomo-tui:
   ```bash
   pomo-tui integration slack setup --token YOUR_BOT_TOKEN
   ```

### Website Blocking
Requires administrator/root privileges to modify `/etc/hosts`:

1. Enable website blocking:
   ```bash
   pomo-tui preferences set website_blocking true
   ```

2. Add websites to block:
   ```bash  
   pomo-tui preferences set blocked_websites "facebook.com,twitter.com,reddit.com"
   ```

## Performance

pomo-tui is designed for optimal performance:

- **Startup time**: < 50ms (measured)
- **Memory usage**: < 50MB RSS
- **Timer precision**: < 100ms drift over 25 minutes
- **Database**: SQLite with optimized queries and caching
- **UI**: 60fps terminal rendering with efficient updates

## Development

### Prerequisites
- Rust 1.75+ (`rustup update stable`)  
- SQLite3 development libraries
- Audio system libraries (ALSA on Linux)

### Building
```bash
git clone https://github.com/pomo-tui/pomo-tui.git
cd pomo-tui
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test --all-features

# Run specific test categories
cargo test --test contract      # Contract tests
cargo test --test integration   # Integration tests  
cargo test --test e2e           # End-to-end tests

# Performance benchmarks
cargo test --release performance
```

### Database Migrations
```bash
# Install sqlx-cli
cargo install sqlx-cli

# Run migrations
sqlx migrate run --database-url sqlite:data.db
```

## Troubleshooting

### Audio Issues
**Linux**: Install ALSA development libraries:
```bash
sudo apt-get install libasound2-dev  # Ubuntu/Debian
sudo dnf install alsa-lib-devel       # Fedora
```

**macOS**: Audio should work out of the box. If not, check System Preferences → Sound.

**Windows**: Audio should work with default Windows audio drivers.

### Performance Issues  
Check performance with built-in diagnostics:
```bash
pomo-tui debug performance --duration 60  # Run 60-second benchmark
```

### Database Issues
Reset database (⚠️ **WARNING**: This deletes all data):
```bash
pomo-tui debug reset-database
```

Export data before resetting:
```bash
pomo-tui export --format json --output backup.json
```

### Integration Issues
Test integrations:
```bash
pomo-tui integration github test
pomo-tui integration slack test
```

Check configuration:
```bash
pomo-tui preferences show
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Process
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Run the test suite (`cargo test`)
5. Submit a pull request

### Code Standards
- Follow Rust idioms and conventions
- Add tests for new functionality  
- Update documentation for user-facing changes
- Use `cargo fmt` and `cargo clippy`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui) for the terminal interface
- Audio support provided by [rodio](https://github.com/RustAudio/rodio)
- Database functionality using [SQLx](https://github.com/launchbadge/sqlx)
- Inspired by the Pomodoro Technique® developed by Francesco Cirillo

## Support

- 📖 Documentation: [docs.rs/pomo-tui](https://docs.rs/pomo-tui)
- 🐛 Bug Reports: [GitHub Issues](https://github.com/pomo-tui/pomo-tui/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/pomo-tui/pomo-tui/discussions)
- 📧 Email: support@pomo-tui.dev

---

**Note**: The Pomodoro Technique® is a registered trademark of Francesco Cirillo. This application is not affiliated with, associated with, or endorsed by the Pomodoro Technique® or Francesco Cirillo.