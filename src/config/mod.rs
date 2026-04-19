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
    #[serde(default)]
    pub project_sort_method: ProjectSortMethod,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReportFormat {
    Csv,
    Json,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectSortMethod {
    #[default]
    Name,
    Usage,
}

impl std::str::FromStr for ProjectSortMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "name" => Ok(Self::Name),
            "usage" => Ok(Self::Usage),
            other => Err(anyhow::anyhow!(
                "invalid project sort method '{other}', expected 'name' or 'usage'"
            )),
        }
    }
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
            project_sort_method: ProjectSortMethod::Name,
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
