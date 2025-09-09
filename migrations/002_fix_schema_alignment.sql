-- Fix schema alignment with Rust struct definitions

-- Drop existing tables to recreate with correct schema
DROP TABLE IF EXISTS statistics;
DROP TABLE IF EXISTS breaks;  
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS preferences;

-- Tasks table: Matches Task struct exactly
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 0 CHECK (priority >= 0 AND priority <= 3),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'completed', 'archived')),
    tags TEXT, -- JSON array of tags
    estimated_sessions INTEGER,
    completed_sessions INTEGER NOT NULL DEFAULT 0,
    project TEXT,
    created_at TEXT NOT NULL, -- SQLite TEXT for DateTime<Utc>
    updated_at TEXT NOT NULL, -- SQLite TEXT for DateTime<Utc>
    started_at TEXT,          -- SQLite TEXT for Option<DateTime<Utc>>
    completed_at TEXT,        -- SQLite TEXT for Option<DateTime<Utc>>
    due_date TEXT             -- SQLite TEXT for Option<NaiveDate>
);

-- Sessions table: Matches Session struct exactly
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    start_time TEXT NOT NULL,        -- SQLite TEXT for DateTime<Utc>
    end_time TEXT,                   -- SQLite TEXT for Option<DateTime<Utc>>
    duration_minutes INTEGER NOT NULL DEFAULT 25,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'abandoned')),
    session_type TEXT NOT NULL DEFAULT 'pomodoro',
    paused_at TEXT,                  -- SQLite TEXT for Option<DateTime<Utc>>
    resumed_at TEXT,                 -- SQLite TEXT for Option<DateTime<Utc>>
    total_pause_time INTEGER DEFAULT 0, -- in seconds
    notes TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE
);

-- Preferences table: Matches Preferences struct exactly
CREATE TABLE preferences (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    pomodoro_duration_minutes INTEGER NOT NULL DEFAULT 25,
    short_break_duration_minutes INTEGER NOT NULL DEFAULT 5,
    long_break_duration_minutes INTEGER NOT NULL DEFAULT 15,
    sessions_until_long_break INTEGER NOT NULL DEFAULT 4,
    auto_start_breaks BOOLEAN NOT NULL DEFAULT 0,
    auto_start_sessions BOOLEAN NOT NULL DEFAULT 0,
    sound_enabled BOOLEAN NOT NULL DEFAULT 1,
    sound_volume INTEGER NOT NULL DEFAULT 70,
    notification_enabled BOOLEAN NOT NULL DEFAULT 1,
    theme TEXT NOT NULL DEFAULT 'default',
    task_list_view_mode TEXT NOT NULL DEFAULT 'list',
    show_session_progress BOOLEAN NOT NULL DEFAULT 1,
    show_break_suggestions BOOLEAN NOT NULL DEFAULT 1,
    github_integration_enabled BOOLEAN NOT NULL DEFAULT 0,
    github_token TEXT,
    jira_integration_enabled BOOLEAN NOT NULL DEFAULT 0,
    jira_url TEXT,
    jira_username TEXT,
    jira_token TEXT,
    daily_goal_sessions INTEGER,
    weekly_goal_sessions INTEGER,
    created_at TEXT NOT NULL,        -- SQLite TEXT for DateTime<Utc>
    updated_at TEXT NOT NULL         -- SQLite TEXT for DateTime<Utc>
);

-- Breaks table: For break periods after sessions  
CREATE TABLE breaks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    after_session_id INTEGER NOT NULL,
    start_time TEXT,                 -- SQLite TEXT for Option<DateTime<Utc>>
    end_time TEXT,                   -- SQLite TEXT for Option<DateTime<Utc>>
    duration_minutes INTEGER NOT NULL DEFAULT 5,
    break_type TEXT NOT NULL DEFAULT 'short' CHECK (break_type IN ('short', 'long')),
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'active', 'completed', 'skipped')),
    FOREIGN KEY (after_session_id) REFERENCES sessions (id) ON DELETE CASCADE
);

-- Statistics table: Daily aggregated statistics
CREATE TABLE statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,      -- SQLite TEXT for NaiveDate
    completed_sessions INTEGER NOT NULL DEFAULT 0,
    completed_breaks INTEGER NOT NULL DEFAULT 0,
    abandoned_sessions INTEGER NOT NULL DEFAULT 0,
    total_focus_time_minutes INTEGER NOT NULL DEFAULT 0,
    total_break_time_minutes INTEGER NOT NULL DEFAULT 0,
    productivity_score REAL NOT NULL DEFAULT 0.0,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    average_session_length REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL,        -- SQLite TEXT for DateTime<Utc>
    updated_at TEXT NOT NULL         -- SQLite TEXT for DateTime<Utc>
);

-- Insert default preferences
INSERT INTO preferences (
    id,
    pomodoro_duration_minutes,
    short_break_duration_minutes,
    long_break_duration_minutes,
    sessions_until_long_break,
    auto_start_breaks,
    auto_start_sessions,
    sound_enabled,
    sound_volume,
    notification_enabled,
    theme,
    task_list_view_mode,
    show_session_progress,
    show_break_suggestions,
    github_integration_enabled,
    jira_integration_enabled,
    daily_goal_sessions,
    weekly_goal_sessions,
    created_at,
    updated_at
) VALUES (
    1,
    25,
    5,
    15,
    4,
    0,
    0,
    1,
    70,
    1,
    'default',
    'list',
    1,
    1,
    0,
    0,
    8,
    40,
    datetime('now'),
    datetime('now')
);