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
        ((self.total_duration as f64 / seconds_per_round as f64).ceil() as i64) * seconds_per_round
    }

    pub fn rounded_hours(&self, round_to_minutes: i64) -> f64 {
        self.rounded_duration(round_to_minutes) as f64 / 3600.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_grouped_entry(duration_seconds: i64) -> GroupedTimeEntry {
        GroupedTimeEntry {
            description: Some("Test".to_string()),
            project_id: None,
            entries: vec![],
            total_duration: duration_seconds,
        }
    }

    #[test]
    fn test_rounding_quarter_hours_exact() {
        let entry = create_grouped_entry(900);
        assert_eq!(entry.rounded_duration(15), 900);
        assert_eq!(entry.rounded_hours(15), 0.25);

        let entry = create_grouped_entry(1800);
        assert_eq!(entry.rounded_duration(15), 1800);
        assert_eq!(entry.rounded_hours(15), 0.5);

        let entry = create_grouped_entry(2700);
        assert_eq!(entry.rounded_duration(15), 2700);
        assert_eq!(entry.rounded_hours(15), 0.75);

        let entry = create_grouped_entry(3600);
        assert_eq!(entry.rounded_duration(15), 3600);
        assert_eq!(entry.rounded_hours(15), 1.0);
    }

    #[test]
    fn test_rounding_up_to_next_quarter() {
        let entry = create_grouped_entry(1);
        assert_eq!(entry.rounded_duration(15), 900);
        assert_eq!(entry.rounded_hours(15), 0.25);

        let entry = create_grouped_entry(901);
        assert_eq!(entry.rounded_duration(15), 1800);
        assert_eq!(entry.rounded_hours(15), 0.5);

        let entry = create_grouped_entry(1801);
        assert_eq!(entry.rounded_duration(15), 2700);
        assert_eq!(entry.rounded_hours(15), 0.75);

        let entry = create_grouped_entry(3601);
        assert_eq!(entry.rounded_duration(15), 4500);
        assert_eq!(entry.rounded_hours(15), 1.25);
    }

    #[test]
    fn test_specific_user_cases() {
        let entry = create_grouped_entry(1332);
        assert_eq!(entry.rounded_duration(15), 1800);
        assert_eq!(entry.rounded_hours(15), 0.5);

        let entry = create_grouped_entry(4176);
        assert_eq!(entry.rounded_duration(15), 4500);
        assert_eq!(entry.rounded_hours(15), 1.25);
    }

    #[test]
    fn test_rounding_with_different_intervals() {
        let entry = create_grouped_entry(3600);
        assert_eq!(entry.rounded_duration(30), 3600);
        assert_eq!(entry.rounded_hours(30), 1.0);

        let entry = create_grouped_entry(3601);
        assert_eq!(entry.rounded_duration(30), 5400);
        assert_eq!(entry.rounded_hours(30), 1.5);

        let entry = create_grouped_entry(300);
        assert_eq!(entry.rounded_duration(5), 300);
        assert_eq!(entry.rounded_hours(5), 300.0 / 3600.0);

        let entry = create_grouped_entry(301);
        assert_eq!(entry.rounded_duration(5), 600);
        assert_eq!(entry.rounded_hours(5), 600.0 / 3600.0);
    }

    #[test]
    fn test_zero_duration() {
        let entry = create_grouped_entry(0);
        assert_eq!(entry.rounded_duration(15), 0);
        assert_eq!(entry.rounded_hours(15), 0.0);
    }

    #[test]
    fn test_total_hours_unrounded() {
        let entry = create_grouped_entry(1332);
        assert_eq!(entry.total_hours(), 0.37);

        let entry = create_grouped_entry(4176);
        assert_eq!(entry.total_hours(), 1.16);
    }
}
