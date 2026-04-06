use chrono::{DateTime, NaiveDate, Utc};

/// A completed Pomodoro session ready to be persisted.
#[derive(Debug, Clone)]
pub struct CompletedSession {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_seconds: u32,
    pub task_label: Option<String>,
    pub jira_ticket_key: Option<String>,
    pub interruption_count: u32,
    /// true = timer expired naturally, false = session was skipped
    pub was_completed: bool,
}

/// An interruption logged during a Pomodoro session.
#[derive(Debug, Clone)]
pub struct StoredInterruption {
    pub id: i64,
    pub session_started_at: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
    pub label: String,
}

/// A session retrieved from the database (includes row id).
#[derive(Debug, Clone)]
pub struct StoredSession {
    pub id: i64,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_seconds: u32,
    pub task_label: Option<String>,
    pub jira_ticket_key: Option<String>,
    pub interruption_count: u32,
    pub was_completed: bool,
}

/// Aggregated stats for a single day (used in weekly view).
#[derive(Debug, Clone)]
pub struct DaySummary {
    pub date: NaiveDate,
    pub total_seconds: u32,
    pub session_count: u32,
    pub completed_count: u32,
    pub interruption_count: u32,
}
