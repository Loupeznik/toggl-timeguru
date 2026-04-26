use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode, header};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

use super::models::{Project, TimeEntry, Workspace};

#[derive(Debug, Clone)]
pub struct BulkUpdateOperation {
    pub op: String,
    pub path: String,
    pub value: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkUpdateResponse {
    pub success: Vec<i64>,
    pub failure: Vec<BulkUpdateFailure>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkUpdateFailure {
    pub id: i64,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub remaining: Option<u32>,
    pub resets_in: Option<u32>,
    pub last_updated: std::time::Instant,
}

impl Default for RateLimitInfo {
    fn default() -> Self {
        Self {
            remaining: None,
            resets_in: None,
            last_updated: std::time::Instant::now(),
        }
    }
}

pub struct TogglClient {
    client: Client,
    api_token: String,
    base_url: String,
    rate_limit_info: Arc<Mutex<RateLimitInfo>>,
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
            rate_limit_info: Arc::new(Mutex::new(RateLimitInfo::default())),
        })
    }

    fn auth_header(&self) -> String {
        let credentials = format!("{}:api_token", self.api_token);
        let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    fn extract_rate_limit_headers(&self, response: &reqwest::Response) {
        let remaining = response
            .headers()
            .get("X-Toggl-Quota-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let resets_in = response
            .headers()
            .get("X-Toggl-Quota-Resets-In")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        if let Ok(mut info) = self.rate_limit_info.lock() {
            info.remaining = remaining;
            info.resets_in = resets_in;
            info.last_updated = std::time::Instant::now();

            if let Some(r) = remaining {
                debug!("Rate limit remaining: {} requests", r);
                if r < 10 {
                    warn!("Rate limit low: {} requests remaining", r);
                }
                if r < 5 {
                    warn!(
                        "Rate limit critical: {} requests remaining, quota resets in {} seconds",
                        r,
                        resets_in.unwrap_or(0)
                    );
                }
            }
        }
    }

    pub fn get_rate_limit_info(&self) -> Option<RateLimitInfo> {
        self.rate_limit_info.lock().ok().map(|info| RateLimitInfo {
            remaining: info.remaining,
            resets_in: info.resets_in,
            last_updated: info.last_updated,
        })
    }

    async fn check_rate_limit_before_request(&self) -> Result<()> {
        if let Some(info) = self.get_rate_limit_info()
            && let Some(remaining) = info.remaining
        {
            if remaining == 0 {
                let wait_time = info.resets_in.unwrap_or(60);
                warn!(
                    "Rate limit exhausted, waiting {} seconds for quota reset",
                    wait_time
                );
                tokio::time::sleep(std::time::Duration::from_secs(wait_time as u64)).await;
            } else if remaining < 5 {
                debug!(
                    "Rate limit low ({}), adding 500ms delay to avoid exhaustion",
                    remaining
                );
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
        Ok(())
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

        self.extract_rate_limit_headers(&response);

        match response.status() {
            StatusCode::OK => {
                let user = response
                    .json::<serde_json::Value>()
                    .await
                    .context("Failed to parse user response")?;
                info!("Successfully fetched user information");
                if let Some(obj) = user.as_object() {
                    let safe_keys: Vec<&String> = obj
                        .keys()
                        .filter(|k| *k != "api_token" && *k != "api_token_last_four")
                        .collect();
                    debug!("User data keys: {:?}", safe_keys);
                }
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

            self.extract_rate_limit_headers(&response);

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
                    warn!("Rate limit hit (429), will retry if attempts remain");
                    last_error = Some(anyhow::anyhow!("Rate limit exceeded"));
                    continue;
                }
                StatusCode::PAYMENT_REQUIRED => {
                    let wait_time = if let Some(info) = self.get_rate_limit_info() {
                        info.resets_in.unwrap_or(60)
                    } else {
                        60
                    };
                    warn!(
                        "Quota exceeded (402), waiting {} seconds for reset",
                        wait_time
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(wait_time as u64)).await;
                    last_error = Some(anyhow::anyhow!("Quota exceeded, retrying"));
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

        self.extract_rate_limit_headers(&response);

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

        self.extract_rate_limit_headers(&response);

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

    pub async fn update_time_entry_project(
        &self,
        workspace_id: i64,
        entry_id: i64,
        project_id: Option<i64>,
    ) -> Result<TimeEntry> {
        self.check_rate_limit_before_request().await?;

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

        self.extract_rate_limit_headers(&response);

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

    #[allow(dead_code)]
    pub async fn update_time_entry_description(
        &self,
        workspace_id: i64,
        entry_id: i64,
        description: String,
    ) -> Result<TimeEntry> {
        self.check_rate_limit_before_request().await?;

        info!(
            "update_time_entry_description called: workspace={}, entry={}, description='{}'",
            workspace_id, entry_id, description
        );

        let url = format!(
            "{}/workspaces/{}/time_entries/{}",
            self.base_url, workspace_id, entry_id
        );

        debug!("API URL: {}", url);

        let mut body = serde_json::Map::new();
        body.insert(
            "description".to_string(),
            serde_json::Value::String(description.clone()),
        );

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

        self.extract_rate_limit_headers(&response);

        match response.status() {
            StatusCode::OK => {
                let updated_entry = response
                    .json::<TimeEntry>()
                    .await
                    .context("Failed to parse updated time entry")?;
                info!(
                    "Successfully updated time entry {} description to '{}'",
                    entry_id, description
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

    pub async fn start_time_entry(
        &self,
        workspace_id: i64,
        description: Option<String>,
    ) -> Result<TimeEntry> {
        self.check_rate_limit_before_request().await?;

        info!(
            "start_time_entry called: workspace={}, description={:?}",
            workspace_id, description
        );

        let url = format!("{}/workspaces/{}/time_entries", self.base_url, workspace_id);

        debug!("API URL: {}", url);

        let now = Utc::now();
        let mut body = serde_json::Map::new();
        body.insert(
            "workspace_id".to_string(),
            serde_json::Value::Number(workspace_id.into()),
        );
        body.insert(
            "start".to_string(),
            serde_json::Value::String(now.to_rfc3339()),
        );
        body.insert(
            "duration".to_string(),
            serde_json::Value::Number((-1).into()),
        );
        body.insert(
            "created_with".to_string(),
            serde_json::Value::String("toggl-timeguru".to_string()),
        );

        if let Some(desc) = description {
            body.insert("description".to_string(), serde_json::Value::String(desc));
        }

        debug!("Request body: {:?}", body);

        info!("Sending POST request to Toggl API...");

        let response = match self
            .client
            .post(&url)
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
                error!("Network error sending POST request: {}", e);
                return Err(anyhow::anyhow!("Network error: {}", e));
            }
        };

        self.extract_rate_limit_headers(&response);

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let time_entry = response
                    .json::<TimeEntry>()
                    .await
                    .context("Failed to parse time entry response")?;
                info!("Successfully started time entry with id {}", time_entry.id);
                Ok(time_entry)
            }
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                error!("Authentication failed while starting time entry");
                Err(anyhow::anyhow!(
                    "Authentication failed. Please check your API token."
                ))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                error!(
                    "Failed to start time entry - Status: {}, Error: {}",
                    status, error_text
                );
                Err(anyhow::anyhow!(
                    "Failed to start time entry. Status: {}, Error: {}",
                    status,
                    error_text
                ))
            }
        }
    }

    pub async fn stop_time_entry(&self, workspace_id: i64, entry_id: i64) -> Result<TimeEntry> {
        self.check_rate_limit_before_request().await?;

        info!(
            "stop_time_entry called: workspace={}, entry_id={}",
            workspace_id, entry_id
        );

        let url = format!(
            "{}/workspaces/{}/time_entries/{}/stop",
            self.base_url, workspace_id, entry_id
        );

        debug!("API URL: {}", url);

        info!("Sending PATCH request to Toggl API...");

        let response = match self
            .client
            .patch(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
        {
            Ok(resp) => {
                debug!("Received response from API");
                resp
            }
            Err(e) => {
                error!("Network error sending PATCH request: {}", e);
                return Err(anyhow::anyhow!("Network error: {}", e));
            }
        };

        self.extract_rate_limit_headers(&response);

        match response.status() {
            StatusCode::OK => {
                let time_entry = response
                    .json::<TimeEntry>()
                    .await
                    .context("Failed to parse time entry response")?;
                info!("Successfully stopped time entry with id {}", time_entry.id);
                Ok(time_entry)
            }
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                error!("Authentication failed while stopping time entry");
                Err(anyhow::anyhow!(
                    "Authentication failed. Please check your API token."
                ))
            }
            StatusCode::NOT_FOUND => {
                error!("Time entry {} not found", entry_id);
                Err(anyhow::anyhow!(
                    "Time entry {} not found. It may have already been stopped.",
                    entry_id
                ))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                error!(
                    "Failed to stop time entry - Status: {}, Error: {}",
                    status, error_text
                );
                Err(anyhow::anyhow!(
                    "Failed to stop time entry. Status: {}, Error: {}",
                    status,
                    error_text
                ))
            }
        }
    }

    pub async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>> {
        info!("get_current_time_entry called");

        let url = format!("{}/me/time_entries/current", self.base_url);

        debug!("API URL: {}", url);

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to send request to Toggl API")?;

        self.extract_rate_limit_headers(&response);

        match response.status() {
            StatusCode::OK => {
                let time_entry = response
                    .json::<Option<TimeEntry>>()
                    .await
                    .context("Failed to parse time entry response")?;

                if let Some(ref entry) = time_entry {
                    info!("Found running time entry with id {}", entry.id);
                } else {
                    info!("No running time entry found");
                }

                Ok(time_entry)
            }
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                error!("Authentication failed while getting current time entry");
                Err(anyhow::anyhow!(
                    "Authentication failed. Please check your API token."
                ))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                error!(
                    "Failed to get current time entry - Status: {}, Error: {}",
                    status, error_text
                );
                Err(anyhow::anyhow!(
                    "Failed to get current time entry. Status: {}, Error: {}",
                    status,
                    error_text
                ))
            }
        }
    }

    pub async fn bulk_update_time_entries(
        &self,
        workspace_id: i64,
        entry_ids: &[i64],
        operations: Vec<BulkUpdateOperation>,
    ) -> Result<BulkUpdateResponse> {
        if entry_ids.is_empty() {
            anyhow::bail!("Cannot update zero entries");
        }

        if entry_ids.len() > 100 {
            anyhow::bail!(
                "Cannot update more than 100 entries per request (got {})",
                entry_ids.len()
            );
        }

        let ids_str = entry_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let url = format!(
            "{}/workspaces/{}/time_entries/{}",
            self.base_url, workspace_id, ids_str
        );

        info!(
            "bulk_update_time_entries called: workspace={}, entry_count={}",
            workspace_id,
            entry_ids.len()
        );
        debug!("API URL: {}", url);

        let body: Vec<serde_json::Value> = operations
            .into_iter()
            .map(|op| {
                serde_json::json!({
                    "op": op.op,
                    "path": op.path,
                    "value": op.value
                })
            })
            .collect();

        debug!("Request body: {:?}", body);

        let max_retries = 3;
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 1..=max_retries {
            self.check_rate_limit_before_request().await?;

            info!(
                "Sending PATCH request to Toggl API... (attempt {}/{})",
                attempt, max_retries
            );

            let response = match self
                .client
                .patch(&url)
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
                    error!("Network error sending PATCH request: {}", e);
                    last_error = Some(anyhow::anyhow!("Network error: {}", e));
                    continue;
                }
            };

            self.extract_rate_limit_headers(&response);

            let status = response.status();

            match status {
                StatusCode::OK => {
                    let result = response
                        .json::<BulkUpdateResponse>()
                        .await
                        .context("Failed to parse bulk update response")?;
                    info!(
                        "Bulk update completed: {} succeeded, {} failed",
                        result.success.len(),
                        result.failure.len()
                    );
                    return Ok(result);
                }
                StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                    error!("Authentication failed during bulk update");
                    return Err(anyhow::anyhow!(
                        "Authentication failed. Please check your API token."
                    ));
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    warn!("Rate limit hit (429), will retry if attempts remain");
                    last_error = Some(anyhow::anyhow!("Rate limit exceeded"));
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                StatusCode::PAYMENT_REQUIRED => {
                    let wait_time = if let Some(info) = self.get_rate_limit_info() {
                        info.resets_in.unwrap_or(60)
                    } else {
                        60
                    };
                    warn!(
                        "Quota exceeded (402), waiting {} seconds for reset",
                        wait_time
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(wait_time as u64)).await;
                    last_error = Some(anyhow::anyhow!("Quota exceeded, retrying"));
                    continue;
                }
                StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT => {
                    warn!("Server error {}, will retry if attempts remain", status);
                    last_error = Some(anyhow::anyhow!("Server error: {}", status));
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                _ => {
                    let error_text = response.text().await.unwrap_or_default();
                    error!(
                        "Bulk update failed - Status: {}, Error: {}",
                        status, error_text
                    );
                    return Err(anyhow::anyhow!(
                        "Bulk update failed. Status: {}, Error: {}",
                        status,
                        error_text
                    ));
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("Bulk update failed after {} retries", max_retries)))
    }

    pub async fn bulk_assign_project(
        &self,
        workspace_id: i64,
        entry_ids: &[i64],
        project_id: Option<i64>,
    ) -> Result<BulkUpdateResponse> {
        let value = match project_id {
            Some(id) => serde_json::Value::Number(id.into()),
            None => serde_json::Value::Null,
        };

        let operations = vec![BulkUpdateOperation {
            op: "replace".to_string(),
            path: "/project_id".to_string(),
            value,
        }];

        info!(
            "bulk_assign_project called: {} entries, project_id={:?}",
            entry_ids.len(),
            project_id
        );

        self.bulk_update_time_entries(workspace_id, entry_ids, operations)
            .await
    }

    pub async fn bulk_update_descriptions(
        &self,
        workspace_id: i64,
        entry_ids: &[i64],
        description: String,
    ) -> Result<BulkUpdateResponse> {
        let operations = vec![BulkUpdateOperation {
            op: "replace".to_string(),
            path: "/description".to_string(),
            value: serde_json::Value::String(description.clone()),
        }];

        info!(
            "bulk_update_descriptions called: {} entries, description='{}'",
            entry_ids.len(),
            description
        );

        self.bulk_update_time_entries(workspace_id, entry_ids, operations)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use mockito::{Matcher, Server};

    async fn mock_client(server: &Server) -> TogglClient {
        let mut client = TogglClient::new("test_token".to_string()).unwrap();
        client.base_url = format!("{}/api/v9", server.url());
        client
    }

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

    #[test]
    fn test_rate_limit_info_default() {
        let info = RateLimitInfo::default();
        assert!(info.remaining.is_none());
        assert!(info.resets_in.is_none());
    }

    #[test]
    fn test_bulk_update_operation_creation() {
        let op = BulkUpdateOperation {
            op: "replace".to_string(),
            path: "/project_id".to_string(),
            value: serde_json::Value::Number(123.into()),
        };
        assert_eq!(op.op, "replace");
        assert_eq!(op.path, "/project_id");
    }

    #[tokio::test]
    async fn test_bulk_update_validates_empty_entries() {
        let client = TogglClient::new("test_token".to_string()).unwrap();
        let entry_ids: Vec<i64> = vec![];
        let operations = vec![BulkUpdateOperation {
            op: "replace".to_string(),
            path: "/project_id".to_string(),
            value: serde_json::Value::Number(123.into()),
        }];

        let result = client
            .bulk_update_time_entries(12345, &entry_ids, operations)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("zero entries"));
    }

    #[tokio::test]
    async fn test_bulk_update_validates_max_entries() {
        let client = TogglClient::new("test_token".to_string()).unwrap();
        let entry_ids: Vec<i64> = (1..=101).collect();
        let operations = vec![BulkUpdateOperation {
            op: "replace".to_string(),
            path: "/project_id".to_string(),
            value: serde_json::Value::Number(123.into()),
        }];

        let result = client
            .bulk_update_time_entries(12345, &entry_ids, operations)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("more than 100"));
    }

    #[test]
    fn test_get_rate_limit_info() {
        let client = TogglClient::new("test_token".to_string()).unwrap();
        let info = client.get_rate_limit_info();
        assert!(info.is_some());
        let info = info.unwrap();
        assert!(info.remaining.is_none());
        assert!(info.resets_in.is_none());
    }

    #[tokio::test]
    async fn test_mocked_rate_limit_headers_are_captured() {
        let mut server = Server::new_async().await;
        let client = mock_client(&server).await;
        let _mock = server
            .mock("GET", "/api/v9/me")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-Toggl-Quota-Remaining", "7")
            .with_header("X-Toggl-Quota-Resets-In", "42")
            .with_body(r#"{"id":123,"email":"user@example.com"}"#)
            .expect(1)
            .create_async()
            .await;

        let user = client.get_current_user().await.unwrap();
        assert_eq!(user["id"].as_i64(), Some(123));

        let info = client.get_rate_limit_info().unwrap();
        assert_eq!(info.remaining, Some(7));
        assert_eq!(info.resets_in, Some(42));
    }

    #[tokio::test]
    async fn test_mocked_rate_limit_response_returns_error() {
        let mut server = Server::new_async().await;
        let client = mock_client(&server).await;
        let _mock = server
            .mock("GET", "/api/v9/me/time_entries")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("start_date".into(), "2025-01-01".into()),
                Matcher::UrlEncoded("end_date".into(), "2025-01-02".into()),
            ]))
            .with_status(429)
            .with_header("X-Toggl-Quota-Remaining", "0")
            .with_header("X-Toggl-Quota-Resets-In", "30")
            .with_body("rate limited")
            .expect(1)
            .create_async()
            .await;

        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        let result = client.get_time_entries_with_retry(start, end, 1).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Rate limit exceeded")
        );

        let info = client.get_rate_limit_info().unwrap();
        assert_eq!(info.remaining, Some(0));
        assert_eq!(info.resets_in, Some(30));
    }
}
