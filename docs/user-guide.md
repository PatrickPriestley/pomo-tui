# pomo-tui User Guide

Complete guide to using pomo-tui, the ADHD-focused Pomodoro terminal application.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Terminal Interface (TUI)](#terminal-interface-tui)
3. [Command Line Interface (CLI)](#command-line-interface-cli)
4. [Configuration](#configuration)
5. [Integrations](#integrations)
6. [Advanced Usage](#advanced-usage)
7. [Troubleshooting](#troubleshooting)

## Getting Started

### First Time Setup

After installation, pomo-tui will create its configuration directory and database on first run:

```bash
pomo-tui
```

This creates:
- Configuration directory with default settings
- SQLite database for your tasks and statistics
- Audio samples directory (if audio is enabled)

### Creating Your First Task

Before starting your first session, create a task to work on:

**Via TUI:**
1. Launch `pomo-tui`
2. Press `n` to create a new task
3. Fill in the task details
4. Press `Enter` to save

**Via CLI:**
```bash
pomo-tui task new "Complete project documentation" \
  --priority 5 \
  --project "work" \
  --tags "urgent,documentation" \
  --estimated-sessions 3
```

### Starting Your First Session

**Via TUI:**
1. Select a task with arrow keys or `j/k`
2. Press `s` to start a session
3. The timer will begin counting down

**Via CLI:**
```bash
pomo-tui session start --task-id 1 --duration 25
```

## Terminal Interface (TUI)

The TUI provides a full-screen terminal interface for managing your Pomodoro sessions.

### Main Layout

```
┌─ Timer ─────────────────────┬─ Tasks ─────────────────────┐
│                             │                             │
│     ██████  ██████          │ • [P5] Complete docs       │
│       ██ █    ██ █          │ • [P3] Review code         │
│       ██ █    ██ █          │ • [P1] Update README       │
│       ██ █    ██ █          │                             │
│     ██████  ██████          │ Status: 3 pending          │
│                             │ Today: 2/8 sessions        │
│   25:00 │█████████░░░░░│ 60% │                             │
│                             │                             │
│ Session 1 of estimated 3    │                             │
│ Working on: Complete docs   │                             │
└─────────────────────────────┴─────────────────────────────┘
┌─ Statistics ─────────────────────────────────────────────┐
│ Today: 2 completed │ This Week: 12 completed │ Streak: 3 │
│ Focus Time: 50min  │ Break Time: 15min       │ Score: 85%│
└─────────────────────────────────────────────────────────┘
[Tab] Switch sections │ [s] Start │ [p] Pause │ [n] New Task │ [q] Quit
```

### Navigation

#### Global Shortcuts
- `Tab` / `Shift+Tab`: Navigate between sections
- `q`: Quit application
- `?` / `F1`: Show help screen
- `Ctrl+C`: Force quit

#### Task List Section
- `↑↓` / `jk`: Move selection up/down
- `Enter`: View task details
- `s`: Start session with selected task
- `n`: Create new task
- `e`: Edit selected task
- `d`: Delete selected task
- `f`: Filter tasks
- `o`: Sort tasks (priority, due date, project)

#### Timer Section
- `s`: Start/Resume session
- `p`: Pause session
- `a`: Abandon session
- `Space`: Pause/Resume (when session active)
- `+/-`: Adjust timer (when paused)

#### Statistics Section
- `←→` / `hl`: Change time period (day/week/month)
- `Enter`: View detailed statistics
- `e`: Export statistics

#### Settings Section
- `↑↓`: Navigate settings categories
- `←→`: Navigate within category
- `Enter`: Edit setting value
- `r`: Reset to defaults
- `s`: Save changes

### Session Flow

1. **Task Selection**: Choose a task from the task list
2. **Session Start**: Press `s` or use session controls
3. **Focus Time**: Timer counts down, minimize distractions
4. **Session End**: 
   - Complete: Session finishes, break is scheduled
   - Pause: Timer stops, can be resumed
   - Abandon: Session is cancelled
5. **Break Time**: Take scheduled break (short/long)
6. **Repeat**: Continue with next session

### Visual Indicators

#### Priority Levels
- `[P5]`: ⚠️ Critical (red)
- `[P4]`: ‼️ High (orange) 
- `[P3]`: ❗ Medium (yellow)
- `[P2]`: ℹ️ Low (blue)
- `[P1]`: ➡️ Minimal (gray)

#### Task Status
- `●`: Pending (blue dot)
- `◐`: In Progress (half-filled)
- `✓`: Completed (green checkmark)
- `⚠`: Overdue (red warning)

#### Timer States
- `Running`: Green progress bar
- `Paused`: Yellow progress bar
- `Break`: Blue progress bar
- `Completed`: Full green bar

## Command Line Interface (CLI)

The CLI provides scriptable access to all functionality.

### Task Commands

#### Create Tasks
```bash
# Basic task
pomo-tui task new "Task title"

# Full options
pomo-tui task new "Complete documentation" \
  --description "Update README and user guide" \
  --priority 4 \
  --project "open-source" \
  --tags "docs,urgent" \
  --due-date 2024-12-31 \
  --estimated-sessions 2
```

#### List Tasks
```bash
# All tasks
pomo-tui task list

# Filter by status
pomo-tui task list --status pending
pomo-tui task list --status completed

# Filter by project  
pomo-tui task list --project work

# Filter by tags
pomo-tui task list --tags urgent,docs

# Sort options
pomo-tui task list --sort priority     # by priority (desc)
pomo-tui task list --sort due-date     # by due date
pomo-tui task list --sort created      # by creation date
```

#### Update Tasks
```bash
# Update status
pomo-tui task update 1 --status completed

# Update priority
pomo-tui task update 1 --priority 5

# Update multiple fields
pomo-tui task update 1 \
  --title "New title" \
  --description "New description" \
  --priority 3
```

#### Delete Tasks
```bash
# Delete single task
pomo-tui task delete 1

# Delete with confirmation
pomo-tui task delete 1 --confirm

# Delete multiple tasks
pomo-tui task delete 1,2,3
```

### Session Commands

#### Start Sessions
```bash
# Start with task selection prompt
pomo-tui session start

# Start with specific task
pomo-tui session start --task-id 1

# Start with custom duration
pomo-tui session start --task-id 1 --duration 30

# Start with next priority task
pomo-tui session start --auto
```

#### Session Control
```bash
# Check current session
pomo-tui session status

# Pause current session
pomo-tui session pause

# Resume paused session  
pomo-tui session resume

# Complete current session
pomo-tui session complete

# Abandon current session
pomo-tui session abandon
```

#### Session History
```bash
# Recent sessions
pomo-tui session history

# Sessions for specific date range
pomo-tui session history --from 2024-01-01 --to 2024-01-31

# Sessions for specific task
pomo-tui session history --task-id 1
```

### Statistics Commands

#### View Statistics
```bash
# Today's stats
pomo-tui stats

# Specific time period
pomo-tui stats --period day
pomo-tui stats --period week  
pomo-tui stats --period month
pomo-tui stats --period year

# Custom date range
pomo-tui stats --from 2024-01-01 --to 2024-01-31

# Specific metrics
pomo-tui stats --metric sessions
pomo-tui stats --metric focus-time
pomo-tui stats --metric productivity-score
```

#### Export Data
```bash
# Export all data as JSON
pomo-tui export --format json --output data.json

# Export as CSV
pomo-tui export --format csv --output report.csv

# Export as Markdown report  
pomo-tui export --format markdown --output summary.md

# Export specific date range
pomo-tui export --format json \
  --from 2024-01-01 \
  --to 2024-01-31 \
  --output january-data.json

# Include/exclude data types
pomo-tui export --format json \
  --include-preferences \
  --include-statistics \
  --output full-backup.json
```

### Preferences Commands

#### View Settings
```bash
# Show all preferences
pomo-tui preferences show

# Show specific category
pomo-tui preferences show session
pomo-tui preferences show audio
pomo-tui preferences show integrations
```

#### Update Settings
```bash
# Session settings
pomo-tui preferences set session_duration 30
pomo-tui preferences set auto_start_breaks true

# Audio settings  
pomo-tui preferences set sound_enabled true
pomo-tui preferences set volume 0.8
pomo-tui preferences set ambient_sound brown_noise

# UI settings
pomo-tui preferences set theme dark
pomo-tui preferences set show_seconds true

# Integration settings
pomo-tui preferences set github_integration true
pomo-tui preferences set blocked_websites "facebook.com,twitter.com"
```

#### Reset Settings
```bash
# Reset all to defaults
pomo-tui preferences reset

# Reset specific category
pomo-tui preferences reset session
pomo-tui preferences reset audio
```

## Configuration

### Configuration File Location

pomo-tui stores its configuration in a platform-specific location:

- **Linux**: `~/.config/pomo-tui/config.toml`
- **macOS**: `~/Library/Application Support/pomo-tui/config.toml`
- **Windows**: `%APPDATA%\pomo-tui\config.toml`

### Configuration Structure

The configuration file uses TOML format with the following sections:

#### Session Configuration
```toml
[session]
duration = 25                    # Default session length (15-50 minutes)
short_break_duration = 5         # Short break length (5-15 minutes)
long_break_duration = 15         # Long break length (15-30 minutes)  
sessions_until_long_break = 4    # Sessions before long break (3-6)
auto_start_breaks = true         # Automatically start breaks
auto_start_sessions = false      # Automatically start next session
allow_break_skip = true          # Allow skipping breaks
session_end_sound = true         # Play sound when session ends
```

#### Audio Configuration  
```toml
[audio]
sound_enabled = true             # Enable audio features
volume = 0.7                     # Master volume (0.0-1.0)
ambient_sound = "brown_noise"    # Background sound during sessions
notification_sound = "bell"      # Sound for notifications

# Available ambient sounds: brown_noise, white_noise, pink_noise, 
# rain, forest, ocean, coffee_shop, library, none

# Available notification sounds: bell, chime, soft_bell, ding, 
# completion, gentle, none
```

#### UI Configuration
```toml
[ui]
theme = "dark"                   # Theme: dark, light, auto
show_seconds = true              # Show seconds in timer display
show_session_count = true        # Show current session number
show_daily_goal = true           # Show daily progress indicator
daily_session_goal = 8           # Target sessions per day
compact_mode = false             # Use compact layout
ascii_art_timer = true           # Use large ASCII timer display
progress_bar_style = "blocks"    # blocks, smooth, minimal
```

#### Integration Configuration
```toml
[integrations]
github_enabled = false           # Enable GitHub integration
slack_enabled = false            # Enable Slack integration
git_commit_enhancement = true    # Enhance git commits with session info
website_blocking = false         # Enable website blocking during sessions

# Websites to block during sessions
blocked_websites = [
    "facebook.com",
    "twitter.com", 
    "reddit.com",
    "youtube.com",
    "instagram.com"
]

# GitHub integration settings
[integrations.github]
token = "your_personal_access_token"
repo_owner = "your-username"  
repo_name = "your-repository"
sync_issues = true               # Sync GitHub issues as tasks
label_filter = ["bug", "enhancement"]  # Only sync issues with these labels

# Slack integration settings  
[integrations.slack]
bot_token = "xoxb-your-bot-token"
user_token = "xoxp-your-user-token" 
update_status = true             # Update Slack status during sessions
status_emoji = ":tomato:"        # Emoji for focus status
focus_message = "In a focus session"
break_message = "On a break"
```

### Environment Variables

pomo-tui respects the following environment variables:

```bash
# Configuration directory override
export POMO_TUI_CONFIG_DIR="/custom/config/path"

# Database location override  
export POMO_TUI_DB_PATH="/custom/database/path/pomo.db"

# Disable audio (useful for headless environments)
export POMO_TUI_NO_AUDIO=1

# Enable debug logging
export POMO_TUI_LOG_LEVEL=debug
export RUST_LOG=pomo_tui=debug

# GitHub token (alternative to config file)
export GITHUB_TOKEN="your_github_token"

# Slack token (alternative to config file)  
export SLACK_BOT_TOKEN="your_slack_bot_token"
```

## Integrations

### GitHub Integration

#### Setup
1. **Create Personal Access Token**:
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Generate new token with scopes: `repo`, `user`, `notifications`
   - Copy the token

2. **Configure pomo-tui**:
   ```bash
   pomo-tui integration github setup --token YOUR_TOKEN
   ```

3. **Set repository** (optional):
   ```bash
   pomo-tui preferences set integrations.github.repo_owner your-username
   pomo-tui preferences set integrations.github.repo_name your-repo
   ```

#### Features
- **Issue Sync**: Import GitHub issues as tasks
- **Commit Enhancement**: Add session context to commit messages
- **Status Updates**: Update GitHub status during sessions

#### Usage
```bash
# Sync issues as tasks
pomo-tui integration github sync

# Sync specific labels only
pomo-tui integration github sync --labels bug,enhancement

# Test integration
pomo-tui integration github test

# Enhanced commit (automatic when git_commit_enhancement = true)
git commit -m "Fix authentication bug"
# Becomes: "Fix authentication bug [25min session: Authentication fixes]"
```

### Slack Integration

#### Setup  
1. **Create Slack App**:
   - Go to https://api.slack.com/apps
   - Create new app for your workspace
   - Add Bot Token Scopes: `users.profile:write`, `users:write`
   - Install app to workspace

2. **Configure pomo-tui**:
   ```bash
   pomo-tui integration slack setup --bot-token YOUR_BOT_TOKEN
   ```

#### Features
- **Status Updates**: Automatically update Slack status during sessions
- **Custom Messages**: Set focus and break status messages
- **Emoji Support**: Use custom emojis for status

#### Usage
```bash
# Test integration
pomo-tui integration slack test

# Manual status update
pomo-tui integration slack status "In deep focus" --emoji ":tomato:"

# Clear status
pomo-tui integration slack clear
```

### Website Blocking

#### Setup
Website blocking requires administrator privileges to modify `/etc/hosts`.

1. **Enable blocking**:
   ```bash
   pomo-tui preferences set website_blocking true
   ```

2. **Configure blocked sites**:
   ```bash
   pomo-tui preferences set blocked_websites "facebook.com,twitter.com,reddit.com"
   ```

3. **Grant permissions** (Linux/macOS):
   ```bash
   # Option 1: Run with sudo when needed (will prompt)
   sudo pomo-tui session start

   # Option 2: Grant permanent access (advanced users)
   sudo visudo
   # Add: your_username ALL=(ALL) NOPASSWD: /usr/bin/tee /etc/hosts
   ```

#### How It Works
- Backup original `/etc/hosts` file
- Add entries redirecting blocked sites to `127.0.0.1`
- Restore original file when session ends
- Safe fallback if application crashes

### Git Integration

#### Features
- **Automatic Commit Enhancement**: Add session context to commit messages
- **Branch-based Tasks**: Create tasks from branch names
- **Commit Statistics**: Track commits per session

#### Setup
```bash
pomo-tui preferences set git_commit_enhancement true
```

#### Enhanced Commit Messages
Original: `git commit -m "Fix authentication bug"`

Enhanced: `Fix authentication bug [25min session: Authentication fixes - 1/3]`

Information added:
- Session duration
- Task title
- Session progress (current/estimated)

## Advanced Usage

### Scripting and Automation

#### Bash Integration
Add to your `.bashrc` or `.zshrc`:

```bash
# Quick start function
pomo() {
    if [ $# -eq 0 ]; then
        pomo-tui
    else
        pomo-tui task new "$*" && pomo-tui session start --auto
    fi
}

# Focus mode with website blocking
focus() {
    local duration=${1:-25}
    pomo-tui preferences set website_blocking true
    pomo-tui session start --duration $duration --auto
}

# Work session with specific project
work() {
    local project=${1:-"default"}
    pomo-tui task list --project "$project" --status pending | head -1 | \
    xargs -I {} pomo-tui session start --task-id {}
}
```

#### Cron Jobs
```bash
# Daily statistics report
0 18 * * * pomo-tui stats --period day | mail -s "Daily Productivity" me@example.com

# Weekly data backup
0 0 * * 0 pomo-tui export --format json --output ~/backups/pomo-$(date +%Y%m%d).json

# GitHub sync every 4 hours
0 */4 * * * pomo-tui integration github sync
```

### Performance Monitoring

#### Built-in Diagnostics
```bash
# Performance benchmark
pomo-tui debug performance --duration 60

# Memory usage check  
pomo-tui debug memory

# Database analysis
pomo-tui debug database --analyze

# Startup time measurement
time pomo-tui --version
```

#### Custom Metrics
Monitor with external tools:

```bash
# Monitor memory usage
watch -n 1 'ps aux | grep pomo-tui'

# Monitor database size
watch -n 60 'du -h ~/.local/share/pomo-tui/pomo.db'

# Monitor audio latency  
pomo-tui debug audio --test-latency
```

### Data Management

#### Backup and Restore
```bash
# Full backup
pomo-tui export --format json \
  --include-preferences \
  --include-statistics \
  --output backup-$(date +%Y%m%d).json

# Restore from backup (manual process)
# 1. Export current data as safety backup
# 2. Reset database: pomo-tui debug reset-database  
# 3. Import tasks from backup JSON manually or via script
```

#### Database Maintenance
```bash
# Optimize database
pomo-tui debug database --optimize

# Vacuum database (reclaim space)
pomo-tui debug database --vacuum

# Check integrity
pomo-tui debug database --check

# View database stats
pomo-tui debug database --stats
```

### Customization

#### Custom Themes
Create custom theme in config:

```toml
[ui.themes.custom]
background = "#1e1e1e"
foreground = "#d4d4d4"  
primary = "#007acc"
secondary = "#ce9178"
accent = "#4ec9b0"
success = "#6a9955"
warning = "#d7ba7d"  
error = "#f44747"
```

#### Custom Sounds
Add custom audio files:

1. Place audio files in `~/.local/share/pomo-tui/audio/`
2. Supported formats: WAV, MP3, OGG, FLAC
3. Update config:
   ```toml
   [audio]
   ambient_sound = "custom/my-sound.wav"
   notification_sound = "custom/notification.wav"
   ```

#### Key Bindings
Currently key bindings are fixed, but you can create wrapper scripts:

```bash
#!/bin/bash
# pomo-vim.sh - Vim-style key bindings
case "$1" in
    "j") pomo-tui --key "down" ;;
    "k") pomo-tui --key "up" ;;
    "h") pomo-tui --key "left" ;;  
    "l") pomo-tui --key "right" ;;
    *) pomo-tui "$@" ;;
esac
```

## Troubleshooting

### Common Issues

#### Audio Problems

**No sound on Linux:**
```bash
# Install ALSA development libraries
sudo apt-get install libasound2-dev      # Ubuntu/Debian
sudo dnf install alsa-lib-devel           # Fedora
sudo pacman -S alsa-lib                   # Arch

# Check audio devices
aplay -l

# Test with different audio backend
pomo-tui preferences set audio.backend alsa  # or pulse
```

**Audio crackling/latency:**
```bash  
# Adjust buffer size
pomo-tui preferences set audio.buffer_size 2048

# Test audio latency
pomo-tui debug audio --test-latency
```

#### Database Issues

**Database locked error:**
```bash
# Check for other pomo-tui processes
ps aux | grep pomo-tui

# Kill hanging processes
pkill pomo-tui

# Reset database (⚠️ loses data)
pomo-tui debug reset-database
```

**Slow database queries:**
```bash
# Optimize database
pomo-tui debug database --optimize

# Check database size
du -h ~/.local/share/pomo-tui/pomo.db

# Vacuum if large
pomo-tui debug database --vacuum
```

#### Performance Issues

**Slow startup:**
```bash
# Check startup time
time pomo-tui --version

# Profile startup
RUST_LOG=debug pomo-tui --version 2>&1 | grep -i slow

# Clear cache
rm -rf ~/.cache/pomo-tui/
```

**High memory usage:**
```bash
# Check memory usage
pomo-tui debug memory

# Reduce cache size
pomo-tui preferences set performance.cache_size 10MB
```

#### Integration Issues

**GitHub sync fails:**
```bash
# Test GitHub connection
pomo-tui integration github test

# Check token permissions
curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user

# Reset integration  
pomo-tui integration github reset
```

**Slack status not updating:**
```bash
# Test Slack integration
pomo-tui integration slack test

# Check bot permissions in Slack admin
# Ensure bot has users.profile:write scope

# Manual status test
pomo-tui integration slack status "test"
```

**Website blocking not working:**
```bash
# Check hosts file backup
ls -la /etc/hosts*

# Test with sudo
sudo pomo-tui session start

# Check permissions
ls -la /etc/hosts

# Manual restore if needed
sudo cp /etc/hosts.pomo-backup /etc/hosts
```

### Debug Mode

Enable debug logging for detailed troubleshooting:

```bash
# Set log level
export RUST_LOG=debug
pomo-tui

# Or specific modules
export RUST_LOG=pomo_tui::database=debug,pomo_tui::audio=info
pomo-tui

# Save debug log to file
RUST_LOG=debug pomo-tui 2> debug.log
```

### Getting Help

If you encounter issues not covered here:

1. **Check logs**: `RUST_LOG=debug pomo-tui 2> debug.log`
2. **Search issues**: [GitHub Issues](https://github.com/pomo-tui/pomo-tui/issues)  
3. **Create issue**: Include debug log and system information
4. **Community support**: [GitHub Discussions](https://github.com/pomo-tui/pomo-tui/discussions)

#### System Information for Bug Reports
```bash
# Generate system info
pomo-tui debug system-info > system-info.txt

# Or manually collect:
echo "OS: $(uname -a)"
echo "Rust: $(rustc --version)"  
echo "pomo-tui: $(pomo-tui --version)"
echo "Audio: $(aplay -l 2>/dev/null || echo 'N/A')"
```

---

This completes the comprehensive user guide for pomo-tui. For additional help, see the [README](../README.md) or visit our [documentation site](https://docs.rs/pomo-tui).