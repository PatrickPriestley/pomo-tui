use std::io::Write;
use std::path::PathBuf;

use crate::config::ObsidianConfig;
use crate::persistence::CompletedSession;
use crate::utils::format_duration;

pub struct ObsidianClient {
    vault_path: PathBuf,
    daily_notes_folder: String,
    date_format: String,
}

impl ObsidianClient {
    pub fn new(config: &ObsidianConfig) -> Self {
        Self {
            vault_path: PathBuf::from(&config.vault_path),
            daily_notes_folder: config.daily_notes_folder.clone(),
            date_format: config.date_format.clone(),
        }
    }

    /// Appends a session summary line to today's daily note.
    /// Creates the file and parent directories if needed.
    pub fn append_session(&self, session: &CompletedSession) -> Result<(), std::io::Error> {
        let today = chrono::Local::now()
            .format(&self.date_format)
            .to_string();
        let file_path = self
            .vault_path
            .join(&self.daily_notes_folder)
            .join(format!("{}.md", today));

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let time = session
            .completed_at
            .with_timezone(&chrono::Local)
            .format("%H:%M");
        let duration = format_duration(session.duration_seconds);
        let task = session.task_label.as_deref().unwrap_or("(no label)");
        let status = if session.was_completed { "✓" } else { "✗" };

        let line = format!(
            "- 🍅 {} | {} {} | {} | {} interruptions\n",
            time, duration, status, task, session.interruption_count
        );

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        file.write_all(line.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_test_session() -> CompletedSession {
        let now = Utc::now();
        CompletedSession {
            started_at: now - chrono::Duration::minutes(25),
            completed_at: now,
            duration_seconds: 25 * 60,
            task_label: Some("BOSS-441: Fix login bug".to_string()),
            jira_ticket_key: Some("BOSS-441".to_string()),
            interruption_count: 2,
            was_completed: true,
        }
    }

    #[test]
    fn test_append_session_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = ObsidianConfig {
            vault_path: dir.path().to_string_lossy().to_string(),
            daily_notes_folder: "Daily Notes".to_string(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let client = ObsidianClient::new(&config);
        let session = make_test_session();

        client.append_session(&session).unwrap();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let file_path = dir.path().join("Daily Notes").join(format!("{}.md", today));
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("BOSS-441: Fix login bug"));
        assert!(content.contains("25m ✓"));
        assert!(content.contains("2 interruptions"));
    }

    #[test]
    fn test_append_session_appends_to_existing() {
        let dir = tempfile::tempdir().unwrap();
        let config = ObsidianConfig {
            vault_path: dir.path().to_string_lossy().to_string(),
            daily_notes_folder: "Daily Notes".to_string(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let client = ObsidianClient::new(&config);

        client.append_session(&make_test_session()).unwrap();
        client.append_session(&make_test_session()).unwrap();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let file_path = dir.path().join("Daily Notes").join(format!("{}.md", today));
        let content = std::fs::read_to_string(&file_path).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
    }
}
