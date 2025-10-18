use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode, header};
use tracing::{debug, error, info, warn};

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

    pub async fn get_current_user(&self) -> Result<serde_json::Value> {
        let url = format!("{}/me", self.base_url);

        info!("Fetching current user information from Toggl API");
        debug!("API URL: {}", url);

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
                info!("Successfully fetched user information");
                debug!("User data: {:?}", user);
                Ok(user)
            }
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                error!("Authentication failed when fetching current user");
                anyhow::bail!("Authentication failed. Please check your API token.")
            }
            status => {
                error!("Unexpected response status when fetching user: {}", status);
                anyhow::bail!("Unexpected response status: {}", status)
            }
        }
    }

    pub async fn get_current_user_id(&self) -> Result<i64> {
        let user = self.get_current_user().await?;
        let user_id = user["id"]
            .as_i64()
            .context("Failed to extract user_id from API response")?;
        info!("Current user_id: {}", user_id);
        Ok(user_id)
    }

    pub async fn get_current_user_email(&self) -> Result<String> {
        let user = self.get_current_user().await?;
        let email = user["email"]
            .as_str()
            .context("Failed to extract email from API response")?
            .to_string();
        info!("Current user email: {}", email);
        Ok(email)
    }

    pub async fn get_time_entries(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>> {
        self.get_time_entries_with_retry(start_date, end_date, 3)
            .await
    }

    async fn get_time_entries_with_retry(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        max_retries: u32,
    ) -> Result<Vec<TimeEntry>> {
        let url = format!(
            "{}/me/time_entries?start_date={}&end_date={}",
            self.base_url,
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        );

        debug!("Fetching time entries from Toggl API: {}", url);
        info!(
            "Requesting time entries from {} to {}",
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        );

        let mut last_error = None;

        for attempt in 1..=max_retries {
            if attempt > 1 {
                let delay = std::time::Duration::from_secs(2_u64.pow(attempt - 1));
                warn!(
                    "Retrying API request (attempt {}/{}) after {:?}",
                    attempt, max_retries, delay
                );
                tokio::time::sleep(delay).await;
            }

            let response = match self
                .client
                .get(&url)
                .header(header::AUTHORIZATION, self.auth_header())
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Network error on attempt {}: {}", attempt, e);
                    last_error = Some(anyhow::anyhow!("Network error: {}", e));
                    continue;
                }
            };

            let status = response.status();
            debug!("API response status: {} (attempt {})", status, attempt);

            match status {
                StatusCode::OK => {
                    let entries = response
                        .json::<Vec<TimeEntry>>()
                        .await
                        .context("Failed to parse time entries")?;
                    info!("Successfully fetched {} time entries", entries.len());
                    debug!("Time entries: {:?}", entries);
                    return Ok(entries);
                }
                StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                    error!("Authentication failed with status: {}", status);
                    return Err(anyhow::anyhow!(
                        "Authentication failed. Please check your API token."
                    ));
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    warn!("Rate limit hit, will retry if attempts remain");
                    last_error = Some(anyhow::anyhow!("Rate limit exceeded"));
                    continue;
                }
                StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT => {
                    warn!("Server error {}, will retry if attempts remain", status);
                    last_error = Some(anyhow::anyhow!("Server error: {}", status));
                    continue;
                }
                _ => {
                    let error_text = response.text().await.unwrap_or_default();
                    error!(
                        "API request failed - Status: {}, Error: {}",
                        status, error_text
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to fetch time entries. Status: {}, Error: {}",
                        status,
                        error_text
                    ));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded")))
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

    #[allow(dead_code)]
    pub async fn update_time_entry_project(
        &self,
        workspace_id: i64,
        entry_id: i64,
        project_id: Option<i64>,
    ) -> Result<TimeEntry> {
        info!(
            "update_time_entry_project called: workspace={}, entry={}, project={:?}",
            workspace_id, entry_id, project_id
        );

        let url = format!(
            "{}/workspaces/{}/time_entries/{}",
            self.base_url, workspace_id, entry_id
        );

        debug!("API URL: {}", url);

        let mut body = serde_json::Map::new();
        if let Some(pid) = project_id {
            body.insert(
                "project_id".to_string(),
                serde_json::Value::Number(pid.into()),
            );
        } else {
            body.insert("project_id".to_string(), serde_json::Value::Null);
        }

        debug!("Request body: {:?}", body);

        info!("Sending PUT request to Toggl API...");

        let response = match self
            .client
            .put(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                debug!("Received response from API");
                resp
            }
            Err(e) => {
                error!("Network error sending PUT request: {}", e);
                return Err(anyhow::anyhow!("Network error: {}", e));
            }
        };

        match response.status() {
            StatusCode::OK => {
                let updated_entry = response
                    .json::<TimeEntry>()
                    .await
                    .context("Failed to parse updated time entry")?;
                info!(
                    "Successfully updated time entry {} project_id to {:?}",
                    entry_id, project_id
                );
                Ok(updated_entry)
            }
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                error!("Authentication failed while updating time entry");
                Err(anyhow::anyhow!(
                    "Authentication failed. Please check your API token."
                ))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                error!(
                    "Failed to update time entry - Status: {}, Error: {}",
                    status, error_text
                );
                Err(anyhow::anyhow!(
                    "Failed to update time entry. Status: {}, Error: {}",
                    status,
                    error_text
                ))
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
