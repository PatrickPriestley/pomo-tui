# Data Model: ADHD-Focused Pomodoro Terminal Application

**Date**: 2025-09-08
**Feature**: 001-build-a-terminal

## Entity Relationship Diagram

```
┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│    Task     │───┐   │   Session   │───┐   │    Break    │
└─────────────┘   │   └─────────────┘   │   └─────────────┘
                  │                      │
                  └──────────┬───────────┘
                             │
                      ┌──────▼──────┐
                      │  Statistics │
                      └─────────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────▼────────┐  ┌────────▼────────┐  ┌───────▼────────┐
│  Preferences   │  │  Integration    │  │   AudioFile   │
└────────────────┘  └─────────────────┘  └────────────────┘
```

## Core Entities

### Task
Represents a work item that can be selected for focus sessions.

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 0, -- 0=low, 1=medium, 2=high, 3=urgent
    estimated_pomodoros INTEGER DEFAULT 1,
    completed_pomodoros INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, active, completed, archived
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    github_issue_id TEXT, -- Optional link to GitHub issue
    jira_issue_key TEXT,  -- Optional link to Jira issue
    notes TEXT,
    tags TEXT -- JSON array of strings
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_priority ON tasks(priority DESC, created_at ASC);
```

**Validation Rules**:
- `title` must be 1-200 characters
- `priority` must be 0-3
- `estimated_pomodoros` must be positive if set
- `status` must be one of the defined values
- `completed_at` must be null unless status is 'completed'

### Session
Represents a pomodoro work period (typically 25 minutes).

```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    planned_duration INTEGER NOT NULL, -- seconds
    actual_duration INTEGER, -- seconds
    status TEXT NOT NULL DEFAULT 'active', -- active, paused, completed, abandoned
    pause_count INTEGER DEFAULT 0,
    total_pause_duration INTEGER DEFAULT 0, -- seconds
    notes TEXT,
    git_commits TEXT, -- JSON array of commit SHAs
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE INDEX idx_sessions_task ON sessions(task_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_date ON sessions(start_time);
```

**Validation Rules**:
- `planned_duration` must be positive (default 1500 seconds = 25 minutes)
- `actual_duration` cannot exceed planned_duration * 2
- `end_time` must be after `start_time`
- Session cannot be completed without `end_time`

### Break
Represents rest periods between sessions.

```sql
CREATE TABLE breaks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    after_session_id INTEGER NOT NULL,
    break_type TEXT NOT NULL, -- short, long
    planned_duration INTEGER NOT NULL, -- seconds
    actual_duration INTEGER, -- seconds
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    skipped BOOLEAN DEFAULT FALSE,
    activity_type TEXT, -- breathing, movement, rest, custom
    notes TEXT,
    FOREIGN KEY (after_session_id) REFERENCES sessions(id)
);

CREATE INDEX idx_breaks_session ON breaks(after_session_id);
CREATE INDEX idx_breaks_type ON breaks(break_type);
```

**Validation Rules**:
- `break_type` must be 'short' or 'long'
- Short breaks: 300 seconds (5 min) default
- Long breaks: 900-1800 seconds (15-30 min) default
- Long break triggered after 4 consecutive sessions

### Preferences
User configuration and settings.

```sql
CREATE TABLE preferences (
    id INTEGER PRIMARY KEY DEFAULT 1, -- Single row table
    -- Timer Settings
    session_duration INTEGER DEFAULT 1500, -- 25 minutes in seconds
    short_break_duration INTEGER DEFAULT 300, -- 5 minutes
    long_break_duration INTEGER DEFAULT 900, -- 15 minutes
    sessions_before_long_break INTEGER DEFAULT 4,
    auto_start_breaks BOOLEAN DEFAULT TRUE,
    auto_start_sessions BOOLEAN DEFAULT FALSE,
    
    -- Sound Settings
    enable_sounds BOOLEAN DEFAULT TRUE,
    tick_sound_enabled BOOLEAN DEFAULT FALSE,
    completion_sound TEXT, -- path to audio file
    ambient_sound TEXT, -- brown_noise, white_noise, pink_noise, rain, none
    ambient_volume REAL DEFAULT 0.3, -- 0.0 to 1.0
    
    -- Focus Settings
    enable_website_blocking BOOLEAN DEFAULT FALSE,
    blocked_websites TEXT, -- JSON array of domains
    
    -- Integration Settings
    enable_git_integration BOOLEAN DEFAULT TRUE,
    enable_github_sync BOOLEAN DEFAULT FALSE,
    enable_jira_sync BOOLEAN DEFAULT FALSE,
    enable_slack_status BOOLEAN DEFAULT FALSE,
    slack_focus_status TEXT DEFAULT 'In focus mode 🍅',
    slack_break_status TEXT DEFAULT 'Taking a break',
    
    -- UI Settings
    theme TEXT DEFAULT 'dark', -- dark, light, high_contrast
    show_seconds BOOLEAN DEFAULT TRUE,
    show_progress_bar BOOLEAN DEFAULT TRUE,
    vim_mode_enabled BOOLEAN DEFAULT TRUE,
    
    -- Statistics Settings
    week_start_day INTEGER DEFAULT 1, -- 0=Sunday, 1=Monday
    daily_goal_sessions INTEGER DEFAULT 8,
    
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Statistics
Aggregated productivity data (materialized view updated periodically).

```sql
CREATE TABLE statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATE NOT NULL UNIQUE,
    completed_sessions INTEGER DEFAULT 0,
    abandoned_sessions INTEGER DEFAULT 0,
    total_focus_time INTEGER DEFAULT 0, -- seconds
    total_break_time INTEGER DEFAULT 0, -- seconds
    tasks_completed INTEGER DEFAULT 0,
    tasks_created INTEGER DEFAULT 0,
    average_session_duration REAL,
    longest_focus_streak INTEGER DEFAULT 0, -- consecutive sessions
    most_productive_hour INTEGER, -- 0-23
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_statistics_date ON statistics(date DESC);
```

### Integration
Stores credentials and configuration for external services.

```sql
CREATE TABLE integrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service TEXT NOT NULL UNIQUE, -- github, jira, slack
    enabled BOOLEAN DEFAULT FALSE,
    auth_type TEXT, -- api_key, oauth_token, personal_access_token
    encrypted_credentials TEXT, -- Encrypted JSON with service-specific auth
    config TEXT, -- JSON with service-specific settings
    last_sync TIMESTAMP,
    sync_status TEXT, -- success, failed, pending
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_integrations_service ON integrations(service);
```

### AudioFile
Manages audio assets for ambient sounds and notifications.

```sql
CREATE TABLE audio_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL, -- ambient, notification
    duration INTEGER, -- seconds, for ambient sounds
    size_bytes INTEGER,
    checksum TEXT, -- SHA256 for integrity
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audio_files_type ON audio_files(file_type);
```

## State Transitions

### Task States
```
pending → active → completed
    ↓        ↓         ↓
         archived ←─────┘
```

### Session States
```
active → paused → active
   ↓        ↓        ↓
   └──► completed ◄──┘
           ↓
      abandoned
```

### Break States
```
scheduled → active → completed
     ↓         ↓
     └─► skipped
```

## Data Integrity Rules

1. **Cascade Deletes**: 
   - Deleting a task deletes all associated sessions
   - Deleting a session deletes associated breaks

2. **Constraints**:
   - Only one session can be active at a time
   - Only one break can be active at a time
   - Cannot start a break without a completed session
   - Cannot complete a task with active sessions

3. **Audit Trail**:
   - All entities track created_at
   - Modifications track updated_at
   - Soft delete via status/archived flag

## Migration Strategy

### Initial Schema (v0.1.0)
All tables created as defined above.

### Future Migrations
- Use numbered migration files: `001_initial_schema.sql`, `002_add_field.sql`
- Include both UP and DOWN migrations
- Test rollback capability
- Version stored in `schema_version` table

## Performance Optimizations

1. **Indexes**: Created on all foreign keys and frequently queried columns
2. **Denormalization**: Statistics table for avoiding expensive aggregations
3. **JSON Fields**: Used sparingly for flexible data (tags, config)
4. **WAL Mode**: Enable for better concurrent access
5. **Vacuum**: Schedule periodic maintenance

## Export Formats

### JSON Export Schema
```json
{
  "version": "1.0.0",
  "exported_at": "2025-09-08T10:00:00Z",
  "tasks": [...],
  "sessions": [...],
  "statistics": {...}
}
```

### CSV Export
- Separate files for tasks, sessions, and daily statistics
- Headers included
- RFC 4180 compliant

### Markdown Report
- Daily/weekly summary with tables
- Task completion rates
- Focus time trends
- Top productive hours