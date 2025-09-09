-- Initial schema for pomo-tui ADHD-focused Pomodoro application

-- Tasks table: Core task management
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 3 CHECK (priority >= 1 AND priority <= 5),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'archived')),
    tags TEXT, -- JSON array of tags
    estimated_sessions INTEGER,
    completed_sessions INTEGER NOT NULL DEFAULT 0,
    project TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now')),
    started_at DATETIME,
    completed_at DATETIME,
    due_date DATE
);

-- Sessions table: Pomodoro sessions tracking
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    start_time DATETIME NOT NULL DEFAULT (datetime('now')),
    end_time DATETIME,
    duration_minutes INTEGER NOT NULL DEFAULT 25,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'abandoned')),
    session_type TEXT NOT NULL DEFAULT 'pomodoro' CHECK (session_type IN ('pomodoro', 'short_break', 'long_break')),
    paused_at DATETIME,
    resumed_at DATETIME,
    total_pause_time INTEGER DEFAULT 0, -- in seconds
    notes TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE
);

-- Breaks table: Break periods after sessions  
CREATE TABLE IF NOT EXISTS breaks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    after_session_id INTEGER NOT NULL,
    start_time DATETIME,
    end_time DATETIME,
    duration_minutes INTEGER NOT NULL DEFAULT 5,
    break_type TEXT NOT NULL DEFAULT 'short' CHECK (break_type IN ('short', 'long')),
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'active', 'completed', 'skipped')),
    FOREIGN KEY (after_session_id) REFERENCES sessions (id) ON DELETE CASCADE
);

-- Statistics table: Daily aggregated statistics
CREATE TABLE IF NOT EXISTS statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATE NOT NULL UNIQUE,
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
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Preferences table: User configuration (single row)
CREATE TABLE IF NOT EXISTS preferences (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1), -- Enforce single row
    session_duration INTEGER NOT NULL DEFAULT 25,
    short_break_duration INTEGER NOT NULL DEFAULT 5,
    long_break_duration INTEGER NOT NULL DEFAULT 15,
    sessions_until_long_break INTEGER NOT NULL DEFAULT 4,
    auto_start_breaks BOOLEAN NOT NULL DEFAULT 1,
    auto_start_sessions BOOLEAN NOT NULL DEFAULT 0,
    sound_enabled BOOLEAN NOT NULL DEFAULT 1,
    volume REAL NOT NULL DEFAULT 0.7,
    ambient_sound TEXT DEFAULT 'brown_noise',
    notification_sound TEXT DEFAULT 'soft_bell',
    visual_notifications BOOLEAN NOT NULL DEFAULT 1,
    desktop_notifications BOOLEAN NOT NULL DEFAULT 1,
    github_integration BOOLEAN NOT NULL DEFAULT 0,
    slack_integration BOOLEAN NOT NULL DEFAULT 0,
    git_commit_enhancement BOOLEAN NOT NULL DEFAULT 1,
    website_blocking BOOLEAN NOT NULL DEFAULT 0,
    theme TEXT NOT NULL DEFAULT 'dark',
    show_seconds BOOLEAN NOT NULL DEFAULT 1,
    show_session_count BOOLEAN NOT NULL DEFAULT 1,
    show_daily_goal BOOLEAN NOT NULL DEFAULT 1,
    daily_session_goal INTEGER NOT NULL DEFAULT 8,
    blocked_websites TEXT, -- JSON array of websites
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Insert default preferences
INSERT OR IGNORE INTO preferences (id) VALUES (1);