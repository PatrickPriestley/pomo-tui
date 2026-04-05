use std::path::PathBuf;

use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use rusqlite::{params, Connection};

use super::migrations;
use super::models::{CompletedSession, StoredSession};

pub struct SessionStore {
    conn: Connection,
}

impl SessionStore {
    /// Opens or creates the database at the given path. Runs migrations.
    pub fn open(db_path: PathBuf) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::InvalidPath(
                    format!("Failed to create directory {}: {}", parent.display(), e).into(),
                )
            })?;
        }

        let conn = Connection::open(&db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        migrations::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    /// Opens an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        migrations::run_migrations(&conn)?;
        Ok(Self { conn })
    }

    /// Records a completed session. Returns the row id.
    pub fn record_session(&self, session: &CompletedSession) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO sessions (started_at, completed_at, duration_seconds, task_label, jira_ticket_key, interruption_count, was_completed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.started_at.to_rfc3339(),
                session.completed_at.to_rfc3339(),
                session.duration_seconds,
                session.task_label,
                session.jira_ticket_key,
                session.interruption_count,
                session.was_completed as i32,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Returns sessions completed today in the user's local timezone.
    pub fn sessions_today(&self) -> Result<Vec<StoredSession>, rusqlite::Error> {
        // Calculate today's start/end in UTC based on local timezone
        let today_local = Local::now().date_naive();
        let start_of_day = today_local
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end_of_day = today_local
            .and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap());

        let start_utc = Local
            .from_local_datetime(&start_of_day)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let end_utc = Local
            .from_local_datetime(&end_of_day)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, completed_at, duration_seconds, task_label, jira_ticket_key, interruption_count, was_completed
             FROM sessions
             WHERE completed_at >= ?1 AND completed_at <= ?2
             ORDER BY completed_at ASC",
        )?;

        let rows = stmt.query_map(
            params![start_utc.to_rfc3339(), end_utc.to_rfc3339()],
            |row| {
                let started_at_str: String = row.get(1)?;
                let completed_at_str: String = row.get(2)?;
                let was_completed_int: i32 = row.get(7)?;

                Ok(StoredSession {
                    id: row.get(0)?,
                    started_at: DateTime::parse_from_rfc3339(&started_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    completed_at: DateTime::parse_from_rfc3339(&completed_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    duration_seconds: row.get(3)?,
                    task_label: row.get(4)?,
                    jira_ticket_key: row.get(5)?,
                    interruption_count: row.get(6)?,
                    was_completed: was_completed_int != 0,
                })
            },
        )?;

        rows.collect()
    }

    /// Returns total session count.
    pub fn total_session_count(&self) -> Result<u64, rusqlite::Error> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM sessions",
            [],
            |row| row.get::<_, u64>(0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_session(was_completed: bool) -> CompletedSession {
        let now = Utc::now();
        CompletedSession {
            started_at: now - Duration::minutes(25),
            completed_at: now,
            duration_seconds: 25 * 60,
            task_label: None,
            jira_ticket_key: None,
            interruption_count: 0,
            was_completed,
        }
    }

    #[test]
    fn test_open_in_memory() {
        let store = SessionStore::open_in_memory();
        assert!(store.is_ok());
    }

    #[test]
    fn test_record_and_count() {
        let store = SessionStore::open_in_memory().unwrap();
        let session = make_session(true);

        let id = store.record_session(&session).unwrap();
        assert_eq!(id, 1);
        assert_eq!(store.total_session_count().unwrap(), 1);
    }

    #[test]
    fn test_record_multiple_sessions() {
        let store = SessionStore::open_in_memory().unwrap();

        for _ in 0..3 {
            store.record_session(&make_session(true)).unwrap();
        }

        assert_eq!(store.total_session_count().unwrap(), 3);
    }

    #[test]
    fn test_was_completed_flag() {
        let store = SessionStore::open_in_memory().unwrap();

        store.record_session(&make_session(true)).unwrap();
        store.record_session(&make_session(false)).unwrap();

        let sessions = store.sessions_today().unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions[0].was_completed);
        assert!(!sessions[1].was_completed);
    }

    #[test]
    fn test_nullable_fields() {
        let store = SessionStore::open_in_memory().unwrap();
        let now = Utc::now();

        let session = CompletedSession {
            started_at: now - Duration::minutes(25),
            completed_at: now,
            duration_seconds: 25 * 60,
            task_label: Some("Working on feature X".to_string()),
            jira_ticket_key: Some("BOSS-441".to_string()),
            interruption_count: 3,
            was_completed: true,
        };
        store.record_session(&session).unwrap();

        // Also record one with None values
        store.record_session(&make_session(true)).unwrap();

        let sessions = store.sessions_today().unwrap();
        assert_eq!(sessions.len(), 2);

        assert_eq!(sessions[0].task_label.as_deref(), Some("Working on feature X"));
        assert_eq!(sessions[0].jira_ticket_key.as_deref(), Some("BOSS-441"));
        assert_eq!(sessions[0].interruption_count, 3);

        assert!(sessions[1].task_label.is_none());
        assert!(sessions[1].jira_ticket_key.is_none());
        assert_eq!(sessions[1].interruption_count, 0);
    }

    #[test]
    fn test_sessions_today_filters_correctly() {
        let store = SessionStore::open_in_memory().unwrap();
        let now = Utc::now();

        // Record a session "today"
        store.record_session(&make_session(true)).unwrap();

        // Record a session from "yesterday" by inserting directly
        let yesterday = now - Duration::days(1);
        store.conn.execute(
            "INSERT INTO sessions (started_at, completed_at, duration_seconds, interruption_count, was_completed)
             VALUES (?1, ?2, ?3, 0, 1)",
            params![
                (yesterday - Duration::minutes(25)).to_rfc3339(),
                yesterday.to_rfc3339(),
                25 * 60,
            ],
        ).unwrap();

        assert_eq!(store.total_session_count().unwrap(), 2);

        let today_sessions = store.sessions_today().unwrap();
        assert_eq!(today_sessions.len(), 1);
    }
}
