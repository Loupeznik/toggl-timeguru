use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode, header};

use super::models::{Project, TimeEntry, Workspace};

pub struct TogglClient {
    client: Client,
    api_token: String,
    base_url: String,
}

impl TogglClient {
    pub fn new(api_token: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_token,
            base_url: "https://api.track.toggl.com/api/v9".to_string(),
        })
    }

    fn auth_header(&self) -> String {
        let credentials = format!("{}:api_token", self.api_token);
        let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    #[allow(dead_code)]
    pub async fn get_current_user(&self) -> Result<serde_json::Value> {
        let url = format!("{}/me", self.base_url);

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to send request to Toggl API")?;

        match response.status() {
            StatusCode::OK => {
                let user = response
                    .json::<serde_json::Value>()
                    .await
                    .context("Failed to parse user response")?;
                Ok(user)
            }
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                anyhow::bail!("Authentication failed. Please check your API token.")
            }
            status => {
                anyhow::bail!("Unexpected response status: {}", status)
            }
        }
    }

    pub async fn get_time_entries(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>> {
        let url = format!(
            "{}/me/time_entries?start_date={}&end_date={}",
            self.base_url,
            start_date.to_rfc3339(),
            end_date.to_rfc3339()
        );

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to fetch time entries")?;

        match response.status() {
            StatusCode::OK => {
                let entries = response
                    .json::<Vec<TimeEntry>>()
                    .await
                    .context("Failed to parse time entries")?;
                Ok(entries)
            }
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                anyhow::bail!("Authentication failed. Please check your API token.")
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                anyhow::bail!(
                    "Failed to fetch time entries. Status: {}, Error: {}",
                    status,
                    error_text
                )
            }
        }
    }

    #[allow(dead_code)]
    pub async fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        let url = format!("{}/workspaces", self.base_url);

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to fetch workspaces")?;

        match response.status() {
            StatusCode::OK => {
                let workspaces = response
                    .json::<Vec<Workspace>>()
                    .await
                    .context("Failed to parse workspaces")?;
                Ok(workspaces)
            }
            status => {
                anyhow::bail!("Failed to fetch workspaces. Status: {}", status)
            }
        }
    }

    #[allow(dead_code)]
    pub async fn get_projects(&self, workspace_id: i64) -> Result<Vec<Project>> {
        let url = format!("{}/workspaces/{}/projects", self.base_url, workspace_id);

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to fetch projects")?;

        match response.status() {
            StatusCode::OK => {
                let projects = response
                    .json::<Vec<Project>>()
                    .await
                    .context("Failed to parse projects")?;
                Ok(projects)
            }
            status => {
                anyhow::bail!("Failed to fetch projects. Status: {}", status)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TogglClient::new("test_token".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_auth_header() {
        let client = TogglClient::new("test_token".to_string()).unwrap();
        let auth = client.auth_header();
        assert!(auth.starts_with("Basic "));
    }
}
