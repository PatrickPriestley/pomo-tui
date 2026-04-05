use crate::config::JiraConfig;
use serde::Deserialize;

#[derive(Clone)]
pub struct JiraClient {
    base_url: String,
    email: String,
    api_token: String,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct JiraIssue {
    fields: JiraFields,
}

#[derive(Deserialize)]
struct JiraFields {
    summary: String,
}

impl JiraClient {
    pub fn new(config: &JiraConfig) -> Self {
        Self {
            base_url: config.base_url.trim_end_matches('/').to_string(),
            email: config.email.clone(),
            api_token: config.api_token.clone(),
            http: reqwest::Client::new(),
        }
    }

    /// Fetches issue summary from Jira. Returns None on any failure.
    pub async fn fetch_issue_summary(&self, ticket_key: &str) -> Option<String> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=summary",
            self.base_url, ticket_key
        );

        let response = self
            .http
            .get(&url)
            .basic_auth(&self.email, Some(&self.api_token))
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let issue: JiraIssue = response.json().await.ok()?;
        Some(issue.fields.summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_client_creation() {
        let config = JiraConfig {
            base_url: "https://myorg.atlassian.net".to_string(),
            email: "user@myorg.com".to_string(),
            api_token: "secret".to_string(),
        };
        let client = JiraClient::new(&config);
        assert_eq!(client.base_url, "https://myorg.atlassian.net");
    }

    #[test]
    fn test_trailing_slash_stripped() {
        let config = JiraConfig {
            base_url: "https://myorg.atlassian.net/".to_string(),
            email: "user@myorg.com".to_string(),
            api_token: "secret".to_string(),
        };
        let client = JiraClient::new(&config);
        assert_eq!(client.base_url, "https://myorg.atlassian.net");
    }
}
