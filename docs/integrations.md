# pomo-tui Integration Guide

Complete guide to setting up and using integrations with pomo-tui.

## Table of Contents

1. [GitHub Integration](#github-integration)
2. [Slack Integration](#slack-integration)  
3. [Git Integration](#git-integration)
4. [Website Blocking](#website-blocking)
5. [Audio System Integration](#audio-system-integration)
6. [Custom Integrations](#custom-integrations)
7. [Troubleshooting](#troubleshooting)

## GitHub Integration

The GitHub integration allows you to sync issues as tasks, enhance commit messages with session context, and update your GitHub status.

### Setup

#### Step 1: Create Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Set expiration and select these scopes:
   - `repo` - Access to repositories
   - `user` - Access to user profile information  
   - `notifications` - Access to notifications (optional)
4. Generate token and copy it immediately

#### Step 2: Configure pomo-tui

**Option 1: Interactive setup**
```bash
pomo-tui integration github setup
# Follow prompts to enter token and repository information
```

**Option 2: Command line**
```bash
pomo-tui integration github setup \
  --token "ghp_your_token_here" \
  --repo-owner "your-username" \
  --repo-name "your-repository"
```

**Option 3: Configuration file**
Add to your `config.toml`:
```toml
[integrations.github]
token = "ghp_your_token_here"
repo_owner = "your-username"
repo_name = "your-repository"  
sync_issues = true
label_filter = ["bug", "enhancement", "task"]
auto_close_on_complete = true
```

**Option 4: Environment variable**
```bash
export GITHUB_TOKEN="ghp_your_token_here"
pomo-tui integration github setup --use-env-token
```

#### Step 3: Test Integration
```bash
pomo-tui integration github test
```

### Features

#### Issue Synchronization

**Sync all issues:**
```bash
pomo-tui integration github sync
```

**Sync with filters:**
```bash
# Only sync issues with specific labels
pomo-tui integration github sync --labels "bug,enhancement"

# Only sync assigned issues
pomo-tui integration github sync --assigned-only

# Sync from specific milestone
pomo-tui integration github sync --milestone "v1.0"

# Sync open issues only (default)
pomo-tui integration github sync --state open
```

**Automatic sync:**
Set up automatic synchronization in config:
```toml
[integrations.github]
auto_sync = true
sync_interval_minutes = 60  # Sync every hour
```

#### Commit Message Enhancement

When `git_commit_enhancement = true`, pomo-tui automatically enhances your commit messages:

**Original commit:**
```bash
git commit -m "Fix authentication bug"
```

**Enhanced commit:**
```
Fix authentication bug

[pomo-tui] Session: 25min working on "Authentication fixes" (2/3)
Task: #123 Fix OAuth login issues
```

**Manual enhancement:**
```bash
pomo-tui integration github enhance-commit "Fix authentication bug"
```

#### Status Updates

Update your GitHub status during sessions:

```toml
[integrations.github]
update_status = true
status_message = "🍅 In a focus session"
```

**Manual status updates:**
```bash
# Set focus status
pomo-tui integration github status "In deep focus" --busy

# Clear status  
pomo-tui integration github status --clear
```

### Advanced Configuration

```toml
[integrations.github]
# Authentication
token = "ghp_your_token"

# Repository settings
repo_owner = "your-username"
repo_name = "your-repo"
multiple_repos = ["owner1/repo1", "owner2/repo2"]  # Multiple repositories

# Issue sync settings
sync_issues = true
auto_sync = true
sync_interval_minutes = 60
label_filter = ["bug", "enhancement", "task"]      # Only sync these labels
assignee_filter = "your-username"                  # Only sync assigned to you
milestone_filter = "current"                       # Only sync current milestone
state_filter = "open"                             # open, closed, all

# Task creation settings
create_tasks_from_issues = true
default_priority_mapping = {
    "critical" = 5,
    "high" = 4, 
    "medium" = 3,
    "low" = 2,
    "trivial" = 1
}
use_issue_labels_as_tags = true
set_due_date_from_milestone = true

# Commit enhancement
git_commit_enhancement = true
include_session_info = true
include_task_info = true
include_time_spent = true
commit_template = "[pomo-tui] {message}\n\nSession: {duration} on '{task}' ({progress})"

# Status updates
update_status = true
status_message = "🍅 In a focus session"
clear_status_on_break = true
```

### API Rate Limits

GitHub API has rate limits (5000 requests/hour for authenticated users):

- **Monitor usage:**
  ```bash
  pomo-tui integration github rate-limit
  ```

- **Optimize sync frequency:**
  ```toml
  [integrations.github]
  sync_interval_minutes = 120  # Reduce frequency
  batch_size = 50              # Fetch more issues per request
  ```

## Slack Integration

The Slack integration updates your status during focus sessions and break times.

### Setup

#### Step 1: Create Slack App

1. Go to [Slack API](https://api.slack.com/apps)
2. Click "Create New App" → "From scratch"
3. Enter app name (e.g., "pomo-tui") and select workspace
4. Go to "OAuth & Permissions" in sidebar

#### Step 2: Configure Permissions

Add these Bot Token Scopes:
- `users.profile:write` - Update user profile
- `users:write` - Update user information
- `chat:write` - Send messages (optional, for notifications)

#### Step 3: Install App

1. Click "Install to Workspace"
2. Authorize the app
3. Copy the "Bot User OAuth Token" (starts with `xoxb-`)

#### Step 4: Configure pomo-tui

**Interactive setup:**
```bash
pomo-tui integration slack setup
```

**Command line:**
```bash
pomo-tui integration slack setup --bot-token "xoxb-your-token"
```

**Configuration file:**
```toml
[integrations.slack]
bot_token = "xoxb-your-bot-token"
update_status = true
status_emoji = ":tomato:"
focus_message = "In a focus session"
break_message = "On a break"
```

#### Step 5: Test Integration
```bash
pomo-tui integration slack test
```

### Features

#### Automatic Status Updates

When enabled, pomo-tui automatically updates your Slack status:

- **Session starts:** "🍅 In a focus session" 
- **Break starts:** "☕ On a break"
- **Session ends:** Status cleared

#### Manual Status Control

```bash
# Set custom status
pomo-tui integration slack status "Deep work mode" --emoji ":brain:"

# Set status with expiration
pomo-tui integration slack status "In meeting" --expires 60  # 60 minutes

# Clear status
pomo-tui integration slack clear
```

#### Message Notifications (Optional)

Send messages to channels or DMs:

```bash
# Send to channel
pomo-tui integration slack message "#general" "Starting focus session!"

# Send DM to user
pomo-tui integration slack dm "@username" "Focus time!"
```

### Advanced Configuration

```toml
[integrations.slack]
# Authentication
bot_token = "xoxb-your-bot-token"
user_token = "xoxp-your-user-token"     # Optional, for more features

# Status settings
update_status = true
clear_status_on_break = true
status_expiry_minutes = 30              # Auto-clear after 30 min

# Status messages and emojis
focus_status = "🍅 In a focus session - do not disturb"
break_status = "☕ On a break"
focus_emoji = ":tomato:"
break_emoji = ":coffee:"

# Custom status per session type
[integrations.slack.status_templates]
pomodoro = "🍅 Pomodoro session: {task}"
short_break = "☕ Short break"
long_break = "🌿 Long break"

# Notification settings (requires chat:write scope)
send_notifications = false
notification_channel = "#productivity"
notify_on_session_complete = true
notify_on_day_complete = true
```

### Team Features

#### Shared Channels

Set up team productivity tracking:

```toml
[integrations.slack]
team_channel = "#productivity"
share_daily_stats = true
share_achievements = true
```

```bash
# Share today's stats
pomo-tui integration slack share-stats "#team"

# Share achievement
pomo-tui integration slack share-achievement "Completed 8 sessions today!" "#team"
```

#### Focus Rooms

Create focus rooms for team synchronization:

```bash
# Create focus room
pomo-tui integration slack create-room "Team Focus Time"

# Join existing focus room
pomo-tui integration slack join-room "focus-room-id"
```

## Git Integration

Enhance your git workflow with session context and statistics.

### Setup

```bash
pomo-tui preferences set git_commit_enhancement true
```

### Features

#### Commit Message Enhancement

**Basic enhancement:**
```bash
git commit -m "Fix login bug"
# Becomes: "Fix login bug [25min: Authentication fixes]"
```

**Detailed enhancement:**
```toml
[integrations.git]
enhancement_template = """
{original_message}

Session: {duration} working on "{task_title}"
Progress: {session_number}/{estimated_sessions}
Total time on task: {total_time}
"""
```

#### Branch-based Tasks

Create tasks from branch names:
```bash
# Create branch
git checkout -b feature/user-authentication

# Auto-create task
pomo-tui integration git create-task-from-branch
# Creates task: "User Authentication" with tag "feature"
```

#### Commit Statistics

Track commits per session:
```bash
# View commit stats
pomo-tui integration git stats

# Export git integration data
pomo-tui export --format json --include-git-stats
```

### Advanced Configuration

```toml
[integrations.git]
# Commit enhancement
enhancement_enabled = true
include_session_duration = true
include_task_context = true  
include_progress = true
enhancement_template = "{message} [{duration}: {task}]"

# Branch integration
auto_create_tasks_from_branches = true
branch_prefixes = ["feature/", "bug/", "hotfix/"]
ignore_branches = ["main", "master", "develop", "staging"]

# Statistics
track_commits_per_session = true
include_in_productivity_score = true
```

## Website Blocking

Block distracting websites during focus sessions.

### Setup

#### Requirements

- **macOS/Linux:** Requires `sudo` access to modify `/etc/hosts`
- **Windows:** Requires Administrator privileges

#### Configuration

```bash
# Enable website blocking
pomo-tui preferences set website_blocking true

# Add websites to block
pomo-tui preferences set blocked_websites "facebook.com,twitter.com,reddit.com,youtube.com"
```

### Advanced Setup

#### Sudoers Configuration (Linux/macOS)

For passwordless website blocking:

1. **Edit sudoers file:**
   ```bash
   sudo visudo
   ```

2. **Add this line** (replace `username`):
   ```
   username ALL=(ALL) NOPASSWD: /bin/cp /etc/hosts.pomo-backup /etc/hosts, /usr/bin/tee /etc/hosts
   ```

#### Custom Hosts File Location

```toml
[integrations.website_blocking]
hosts_file_path = "/etc/hosts"
backup_path = "/etc/hosts.pomo-backup"
blocked_ip = "127.0.0.1"              # IP to redirect to
```

### Features

#### Session-based Blocking

Websites are automatically blocked/unblocked:
- **Session starts:** Add blocked sites to hosts file
- **Session ends:** Restore original hosts file
- **Break starts:** Temporarily unblock (optional)

#### Safe Fallback

- Automatic backup of original hosts file
- Restoration on application crash/exit
- Manual restore command available

```bash
# Manual restore (if something goes wrong)
pomo-tui integration website-blocking restore

# Test blocking
pomo-tui integration website-blocking test
```

### Advanced Configuration

```toml
[integrations.website_blocking]
enabled = true
unblock_during_breaks = true            # Allow access during breaks
block_subdomains = true                 # Block *.facebook.com
backup_hosts_file = true                # Create backup before changes

# Blocked websites
blocked_sites = [
    "facebook.com",
    "twitter.com", 
    "reddit.com",
    "youtube.com",
    "instagram.com",
    "tiktok.com",
    "linkedin.com",   # Professional distraction
    "hacker-news.ycombinator.com"
]

# Time-based rules
[integrations.website_blocking.rules]
# Different rules for different times
work_hours = { time = "09:00-17:00", sites = ["facebook.com", "twitter.com"] }
deep_work = { sessions = ["pomodoro"], sites = ["all_social_media"] }

# Allowlist (never block these)
allowlist = [
    "work-domain.com",
    "documentation-site.com"
]
```

## Audio System Integration

Integrate with system audio for ambient sounds and notifications.

### Setup

#### Linux (ALSA/PulseAudio)

**Install dependencies:**
```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev pulseaudio-dev

# Fedora
sudo dnf install alsa-lib-devel pulseaudio-libs-devel

# Arch
sudo pacman -S alsa-lib pulseaudio
```

**Test audio:**
```bash
pomo-tui debug audio --test
```

#### macOS

Audio should work out of the box. If issues:
```bash
# Check audio devices
system_profiler SPAudioDataType

# Test with different backend
pomo-tui preferences set audio.backend coreaudio
```

#### Windows

Audio should work with default drivers. For issues:
```bash
# Test with different backend
pomo-tui preferences set audio.backend wasapi
```

### Features

#### Ambient Sounds

Built-in ambient sounds for better focus:
- `brown_noise` - Deep, rumbling background noise
- `white_noise` - Full spectrum static
- `pink_noise` - Balanced frequency noise  
- `rain` - Light rain sounds
- `forest` - Nature sounds with birds
- `ocean` - Ocean waves
- `coffee_shop` - Café ambiance
- `library` - Quiet library atmosphere

#### Notification Sounds

Session transition sounds:
- `bell` - Classic bell sound
- `chime` - Soft chime  
- `soft_bell` - Gentle bell
- `ding` - Simple notification
- `completion` - Achievement sound
- `gentle` - Very soft notification

### Advanced Configuration

```toml
[audio]
# General settings
enabled = true
volume = 0.7                            # Master volume (0.0-1.0)
backend = "auto"                        # auto, alsa, pulse, coreaudio, wasapi

# Ambient sounds
ambient_enabled = true
ambient_sound = "brown_noise"
ambient_volume = 0.5                    # Relative to master volume
fade_in_duration = 3                    # Seconds to fade in
fade_out_duration = 2                   # Seconds to fade out

# Notifications
notification_enabled = true
notification_sound = "soft_bell"
notification_volume = 0.8
session_start_sound = true
session_end_sound = true
break_start_sound = false
break_end_sound = true

# Custom audio files
custom_sounds_dir = "~/.local/share/pomo-tui/audio"
[audio.custom_sounds]
my_ambient = "custom/forest-rain.wav"
my_notification = "custom/tibetan-bell.wav"

# Audio device selection
[audio.devices]
output_device = "default"               # or specific device name
input_device = "none"                   # for future microphone features
```

### Custom Audio Files

Add your own audio files:

1. **Create audio directory:**
   ```bash
   mkdir -p ~/.local/share/pomo-tui/audio/custom
   ```

2. **Add audio files:**
   - Supported formats: WAV, MP3, OGG, FLAC
   - Recommended: WAV for best performance
   - Name files descriptively

3. **Update configuration:**
   ```toml
   [audio]
   ambient_sound = "custom/my-forest.wav"
   notification_sound = "custom/my-bell.wav"
   ```

4. **Test custom sounds:**
   ```bash
   pomo-tui debug audio --test-custom
   ```

## Custom Integrations

Create your own integrations using pomo-tui's plugin system and APIs.

### Plugin Architecture

```bash
# Plugin directory
~/.local/share/pomo-tui/plugins/

# Plugin structure
my-plugin/
├── plugin.toml              # Plugin metadata
├── hooks/                   # Event hooks
│   ├── session_start.sh
│   ├── session_end.sh
│   └── task_complete.sh
└── commands/                # Custom commands
    └── my-command.sh
```

### Event Hooks

#### Available Events

- `session_start` - Session begins
- `session_pause` - Session paused
- `session_resume` - Session resumed  
- `session_complete` - Session completed
- `session_abandon` - Session abandoned
- `break_start` - Break begins
- `break_end` - Break ends
- `task_create` - New task created
- `task_complete` - Task completed
- `day_complete` - Daily goal reached

#### Hook Example

**`~/.local/share/pomo-tui/plugins/notifications/hooks/session_complete.sh`:**
```bash
#!/bin/bash
# Receive session data via environment variables
# POMO_SESSION_DURATION, POMO_TASK_TITLE, POMO_SESSION_NUMBER, etc.

# Send desktop notification
notify-send "Pomodoro Complete!" \
    "Completed ${POMO_SESSION_DURATION}min session: ${POMO_TASK_TITLE}"

# Log to external system
curl -X POST "https://api.myservice.com/sessions" \
    -H "Content-Type: application/json" \
    -d "{
        \"duration\": \"${POMO_SESSION_DURATION}\",
        \"task\": \"${POMO_TASK_TITLE}\",
        \"timestamp\": \"$(date -Iseconds)\"
    }"
```

### Custom Commands

#### Command Example

**`~/.local/share/pomo-tui/plugins/todoist/commands/sync.sh`:**
```bash
#!/bin/bash
# Custom command: pomo-tui plugin todoist sync

TODOIST_TOKEN=${TODOIST_TOKEN:-$(pomo-tui preferences get todoist.token)}

# Fetch Todoist tasks
curl -X GET "https://api.todoist.com/rest/v2/tasks" \
    -H "Authorization: Bearer ${TODOIST_TOKEN}" \
    | jq '.[] | {title: .content, priority: (.priority | . + 1), due: .due.date}' \
    | while read task; do
        # Create pomo-tui task
        pomo-tui task new "$task"
    done
```

#### Plugin Metadata

**`~/.local/share/pomo-tui/plugins/todoist/plugin.toml`:**
```toml
[plugin]
name = "todoist"
version = "1.0.0"
author = "Your Name"
description = "Todoist integration for pomo-tui"

[commands.sync]
script = "commands/sync.sh"
description = "Sync tasks from Todoist"

[hooks]
session_complete = "hooks/update_todoist.sh"

[preferences]
token = { type = "string", description = "Todoist API token" }
project_id = { type = "string", description = "Default project ID" }
```

### API Integration

#### REST API (Future Feature)

pomo-tui will expose a REST API for external integrations:

```bash
# Start API server
pomo-tui serve --port 8080

# API endpoints
GET /api/v1/sessions/current      # Current session
POST /api/v1/sessions/start       # Start session
PUT /api/v1/sessions/pause        # Pause session
GET /api/v1/tasks                 # List tasks
POST /api/v1/tasks                # Create task
GET /api/v1/statistics            # Get statistics
```

#### Database Access

Direct database integration for advanced users:

```sql
-- SQLite database location
-- Linux: ~/.local/share/pomo-tui/pomo.db
-- macOS: ~/Library/Application Support/pomo-tui/pomo.db

-- Example: Custom analytics query
SELECT 
    DATE(start_time) as date,
    COUNT(*) as sessions,
    SUM(duration_minutes) as total_minutes,
    AVG(duration_minutes) as avg_duration
FROM sessions 
WHERE status = 'completed'
    AND start_time >= datetime('now', '-30 days')
GROUP BY DATE(start_time)
ORDER BY date;
```

## Troubleshooting

### Common Integration Issues

#### GitHub Issues

**Authentication fails:**
```bash
# Test token manually
curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user

# Check token scopes
curl -H "Authorization: token YOUR_TOKEN" -I https://api.github.com/user \
  | grep -i x-oauth-scopes

# Reset integration
pomo-tui integration github reset
```

**Rate limit exceeded:**
```bash
# Check rate limit status  
pomo-tui integration github rate-limit

# Reduce sync frequency
pomo-tui preferences set integrations.github.sync_interval_minutes 180
```

**Issues not syncing:**
```bash
# Test with verbose logging
RUST_LOG=debug pomo-tui integration github sync

# Check label filters
pomo-tui preferences show integrations.github.label_filter

# Manual issue fetch
curl -H "Authorization: token YOUR_TOKEN" \
  "https://api.github.com/repos/OWNER/REPO/issues?labels=bug"
```

#### Slack Issues

**Status not updating:**
```bash
# Test bot permissions
pomo-tui integration slack test

# Check bot scopes in Slack admin
# Ensure bot has users.profile:write scope

# Manual API test
curl -X POST https://slack.com/api/users.profile.set \
  -H "Authorization: Bearer YOUR_BOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"profile": {"status_text": "test"}}'
```

**Bot token vs user token:**
- Bot tokens (`xoxb-`) work for most features
- User tokens (`xoxp-`) required for some advanced features
- Use bot tokens unless specifically needed

#### Website Blocking Issues

**Permission denied:**
```bash
# Test with sudo
sudo pomo-tui session start

# Check hosts file permissions
ls -la /etc/hosts

# Check if backup exists
ls -la /etc/hosts.pomo-backup

# Manual restore
sudo cp /etc/hosts.pomo-backup /etc/hosts
```

**Blocking not working:**
```bash
# Test DNS resolution
nslookup facebook.com

# Check hosts file content
cat /etc/hosts | grep facebook

# Test with curl
curl -I facebook.com  # Should fail or redirect
```

**Windows-specific:**
```powershell
# Run as Administrator
# Check hosts file location
C:\Windows\System32\drivers\etc\hosts

# Flush DNS cache
ipconfig /flushdns
```

### Debug Commands

```bash
# Test all integrations
pomo-tui debug integrations --test-all

# Integration-specific debugging  
pomo-tui debug github --verbose
pomo-tui debug slack --check-permissions
pomo-tui debug website-blocking --dry-run

# Export integration logs
pomo-tui debug export-logs --include-integrations
```

### Getting Integration Help

1. **Check integration status:**
   ```bash
   pomo-tui integration status
   ```

2. **Review configuration:**
   ```bash
   pomo-tui preferences show integrations
   ```

3. **Test individual integrations:**
   ```bash
   pomo-tui integration github test
   pomo-tui integration slack test
   ```

4. **Enable debug logging:**
   ```bash
   RUST_LOG=debug pomo-tui integration github sync 2> github-debug.log
   ```

5. **Check system requirements:**
   ```bash
   pomo-tui debug system-requirements --integrations
   ```

For integration-specific issues, include debug logs and configuration (with tokens removed) when reporting issues.

---

This completes the comprehensive integration guide. For additional help, see the main [User Guide](user-guide.md) or visit our [GitHub repository](https://github.com/pomo-tui/pomo-tui).