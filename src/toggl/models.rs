use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: i64,
    pub workspace_id: i64,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub billable: bool,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
    pub duration: i64,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub tag_ids: Option<Vec<i64>>,
    pub duronly: bool,
    pub at: DateTime<Utc>,
    pub server_deleted_at: Option<DateTime<Utc>>,
    pub user_id: i64,
    pub uid: Option<i64>,
    pub wid: Option<i64>,
    pub pid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub workspace_id: i64,
    pub client_id: Option<i64>,
    pub name: String,
    pub is_private: bool,
    pub active: bool,
    pub at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub color: String,
    pub billable: Option<bool>,
    pub template: Option<bool>,
    pub auto_estimates: Option<bool>,
    pub estimated_hours: Option<i64>,
    pub rate: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub premium: bool,
    pub admin: bool,
    pub default_hourly_rate: Option<f64>,
    pub default_currency: String,
    pub only_admins_may_create_projects: bool,
    pub only_admins_see_billable_rates: bool,
    pub rounding: i32,
    pub rounding_minutes: i32,
    pub at: DateTime<Utc>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GroupedTimeEntry {
    pub description: Option<String>,
    #[allow(dead_code)]
    pub project_id: Option<i64>,
    pub entries: Vec<TimeEntry>,
    pub total_duration: i64,
}

impl GroupedTimeEntry {
    pub fn total_hours(&self) -> f64 {
        self.total_duration as f64 / 3600.0
    }

    pub fn rounded_duration(&self, round_to_minutes: i64) -> i64 {
        let seconds_per_round = round_to_minutes * 60;
        ((self.total_duration as f64 / seconds_per_round as f64).round() as i64) * seconds_per_round
    }

    pub fn rounded_hours(&self, round_to_minutes: i64) -> f64 {
        self.rounded_duration(round_to_minutes) as f64 / 3600.0
    }
}
