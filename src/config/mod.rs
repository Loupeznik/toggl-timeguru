use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_date_range_days: i64,
    pub preferred_report_format: ReportFormat,
    pub api_token_encrypted: Option<Vec<u8>>,
    pub round_duration_minutes: Option<i64>,
    pub current_user_id: Option<i64>,
    pub current_user_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReportFormat {
    Csv,
    Json,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_date_range_days: 7,
            preferred_report_format: ReportFormat::Csv,
            api_token_encrypted: None,
            round_duration_minutes: Some(15),
            current_user_id: None,
            current_user_email: None,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Ok(confy::load("toggl-timeguru", "config")?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        confy::store("toggl-timeguru", "config", self)?;
        Ok(())
    }

    pub fn default_date_range(&self) -> Duration {
        Duration::days(self.default_date_range_days)
    }
}
