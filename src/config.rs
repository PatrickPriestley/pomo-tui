use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfig {
    pub jira: Option<JiraConfig>,
    pub obsidian: Option<ObsidianConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraConfig {
    pub base_url: String,
    pub email: String,
    pub api_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObsidianConfig {
    pub vault_path: String,
    pub daily_notes_folder: String,
    #[serde(default = "default_date_format")]
    pub date_format: String,
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

impl AppConfig {
    /// Load config from ~/.config/pomo-tui/config.toml.
    /// Returns Default (no integrations) on any failure — config is opt-in.
    pub fn load() -> Self {
        Self::load_from_path(Self::config_path()).unwrap_or_default()
    }

    fn load_from_path(path: Option<PathBuf>) -> Option<Self> {
        let path = path?;
        let content = fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    fn config_path() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?;
        path.push("pomo-tui");
        path.push("config.toml");
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_missing_file_returns_default() {
        let config = AppConfig::load_from_path(Some(PathBuf::from("/nonexistent/config.toml")));
        assert!(config.is_none());

        // load() should return default
        let default = AppConfig::default();
        assert!(default.jira.is_none());
    }

    #[test]
    fn test_parse_jira_config() {
        let toml_str = r#"
[jira]
base_url = "https://myorg.atlassian.net"
email = "user@myorg.com"
api_token = "secret123"
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        let jira = config.jira.unwrap();
        assert_eq!(jira.base_url, "https://myorg.atlassian.net");
        assert_eq!(jira.email, "user@myorg.com");
        assert_eq!(jira.api_token, "secret123");
    }

    #[test]
    fn test_parse_empty_config() {
        let config: AppConfig = toml::from_str("").unwrap();
        assert!(config.jira.is_none());
        assert!(config.obsidian.is_none());
    }

    #[test]
    fn test_parse_obsidian_config() {
        let toml_str = r#"
[obsidian]
vault_path = "/Users/pete/obsidian-vault"
daily_notes_folder = "Daily Notes"
date_format = "%Y-%m-%d"
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        let obsidian = config.obsidian.unwrap();
        assert_eq!(obsidian.vault_path, "/Users/pete/obsidian-vault");
        assert_eq!(obsidian.daily_notes_folder, "Daily Notes");
        assert_eq!(obsidian.date_format, "%Y-%m-%d");
    }

    #[test]
    fn test_default_date_format() {
        let toml_str = r#"
[obsidian]
vault_path = "/vault"
daily_notes_folder = "Daily"
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        let obsidian = config.obsidian.unwrap();
        assert_eq!(obsidian.date_format, "%Y-%m-%d");
    }
}
