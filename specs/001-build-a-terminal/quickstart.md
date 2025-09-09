# Quickstart Guide: Pomo-TUI

## Installation

### Via Cargo (Recommended)
```bash
cargo install pomo-tui
```

### Via Homebrew (macOS/Linux)
```bash
brew tap pomo-tui/tap
brew install pomo-tui
```

### Pre-built Binary
Download the latest release from GitHub:
```bash
# macOS (Apple Silicon)
curl -L https://github.com/pomo-tui/releases/latest/download/pomo-tui-darwin-aarch64 -o pomo-tui
# macOS (Intel)
curl -L https://github.com/pomo-tui/releases/latest/download/pomo-tui-darwin-x86_64 -o pomo-tui
# Linux
curl -L https://github.com/pomo-tui/releases/latest/download/pomo-tui-linux-x86_64 -o pomo-tui
# Windows
curl -L https://github.com/pomo-tui/releases/latest/download/pomo-tui-windows-x86_64.exe -o pomo-tui.exe

chmod +x pomo-tui
sudo mv pomo-tui /usr/local/bin/
```

## First Run

1. **Launch the application**:
   ```bash
   pomo-tui
   ```

2. **Initial setup** (automatic on first run):
   - Creates config directory at `~/.config/pomo-tui/`
   - Initializes SQLite database
   - Generates default configuration

## Basic Usage

### Quick Test: Your First Pomodoro

1. **Create a task** (press `n` in main screen):
   ```
   Title: Test task
   Priority: 2 (medium)
   Estimated pomodoros: 1
   ```

2. **Start a session** (press `s` with task selected):
   - Timer starts at 25:00
   - Press `p` to pause/resume
   - Press `q` to abandon
   - Wait for completion or press `c` to complete early

3. **Take a break** (automatic after session):
   - 5-minute break starts
   - Press `k` to skip
   - Press `b` for breathing exercise

4. **View statistics** (press `v`):
   - Shows today's progress
   - Press `w` for weekly view

## Keyboard Shortcuts

### Navigation
- `j`/`↓` - Move down
- `k`/`↑` - Move up
- `h`/`←` - Previous tab
- `l`/`→` - Next tab
- `g` - Go to top
- `G` - Go to bottom

### Task Management
- `n` - New task
- `e` - Edit task
- `d` - Delete task
- `Enter` - View task details
- `Space` - Toggle task completion
- `1-3` - Set priority (low/medium/high)

### Timer Control
- `s` - Start session
- `p` - Pause/Resume
- `c` - Complete session
- `q` - Abandon session
- `+` - Extend timer (5 min)
- `-` - Reduce timer (5 min)

### Views
- `t` - Tasks view
- `v` - Statistics view
- `o` - Settings view
- `?` - Help screen

### Global
- `Ctrl+C` - Quit application
- `/` - Search tasks
- `r` - Refresh display

## Configuration

### Edit Settings
Settings file location: `~/.config/pomo-tui/config.toml`

```toml
[timer]
session_duration = 1500  # 25 minutes
short_break = 300       # 5 minutes
long_break = 900        # 15 minutes
sessions_before_long_break = 4

[sound]
enabled = true
ambient = "brown_noise"
volume = 0.3

[ui]
theme = "dark"
vim_mode = true
show_seconds = true

[integrations]
git_commit_context = true
```

### Enable Website Blocking (requires admin)
```bash
# Grant permission for hosts file modification
sudo pomo-tui setup-blocking
```

## Integration Setup

### GitHub Integration
```bash
pomo-tui integrate github
# Follow OAuth flow in browser
# Tasks will sync with assigned issues
```

### Slack Status Updates
```bash
pomo-tui integrate slack
# Authorize in browser
# Status updates during focus sessions
```

### Git Commit Context
Automatically adds session context to commits:
```bash
# Just commit normally while timer is running
git commit -m "Fix navigation bug"
# Commit message becomes:
# "Fix navigation bug [25min session: Implement navigation]"
```

## Testing Core Features

### Test 1: Basic Timer Flow
```bash
# In one terminal, start the app
pomo-tui

# Create and start a task
# - Press 'n' to create "Test Timer"
# - Press 's' to start session
# - Wait 5 seconds
# - Press 'p' to pause
# - Press 'p' to resume
# - Press 'c' to complete
# Expected: Break starts automatically
```

### Test 2: Data Persistence
```bash
# Start a session
pomo-tui
# Create task "Persistence Test"
# Start session
# Press Ctrl+C to quit

# Restart application
pomo-tui
# Expected: See "Resume session?" prompt
# Press 'y' to resume
```

### Test 3: Statistics Tracking
```bash
# Complete 3 pomodoros
pomo-tui

# After completing sessions:
# Press 'v' for statistics
# Expected: Shows 3 completed sessions, total focus time
```

### Test 4: Export Data
```bash
# Export today's data
pomo-tui export --format json --output today.json

# Export weekly report
pomo-tui export --format markdown --period week

# Expected: Files created with session data
```

## Command-Line Interface

### Run without TUI
```bash
# Create task
pomo-tui task create "CLI task" --priority 2

# List tasks
pomo-tui task list

# Start session
pomo-tui session start --task-id 1

# Get status
pomo-tui session status

# Complete session
pomo-tui session complete
```

### Daemon Mode
```bash
# Run in background
pomo-tui daemon start

# Control via CLI
pomo-tui task create "Background task"
pomo-tui session start --task-id 1

# Check status
pomo-tui daemon status

# Stop daemon
pomo-tui daemon stop
```

## Troubleshooting

### Database Issues
```bash
# Backup and reset database
cp ~/.config/pomo-tui/pomo.db ~/.config/pomo-tui/pomo.db.bak
pomo-tui db reset
```

### Audio Not Working
```bash
# Test audio system
pomo-tui audio test

# Disable if causing issues
pomo-tui config set sound.enabled false
```

### Performance Issues
```bash
# Run diagnostics
pomo-tui diagnose

# Vacuum database
pomo-tui db vacuum
```

## Verification Checklist

- [ ] Application starts in <50ms
- [ ] Can create and edit tasks
- [ ] Timer counts down accurately
- [ ] Breaks trigger after sessions
- [ ] Statistics update correctly
- [ ] Data persists between restarts
- [ ] Keyboard shortcuts work
- [ ] Export generates valid files
- [ ] Settings changes take effect
- [ ] No memory leaks over extended use

## Next Steps

1. Customize your timer durations in settings
2. Set up integrations with your tools
3. Import existing tasks from GitHub/Jira
4. Configure ambient sounds for focus
5. Set daily session goals

For more information, see the full documentation at:
https://github.com/pomo-tui/docs