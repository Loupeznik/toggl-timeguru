use chrono::{DateTime, TimeZone, Utc};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "toggl-timeguru")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Toggl API token")]
    pub api_token: Option<String>,

    #[arg(short = 'c', long, help = "Path to configuration file")]
    pub config: Option<String>,

    #[arg(short = 'v', long, help = "Enable verbose logging")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Configure the application (set API token, preferences)")]
    Config {
        #[arg(long, help = "Set Toggl API token")]
        set_token: Option<String>,

        #[arg(long, help = "Set default date range in days")]
        set_date_range: Option<i64>,

        #[arg(long, help = "Set rounding duration in minutes")]
        set_round_minutes: Option<i64>,

        #[arg(long, help = "Show current configuration")]
        show: bool,
    },

    #[command(about = "List time entries")]
    List {
        #[arg(short, long, help = "Start date (ISO 8601 format or YYYY-MM-DD)")]
        start: Option<String>,

        #[arg(short, long, help = "End date (ISO 8601 format or YYYY-MM-DD)")]
        end: Option<String>,

        #[arg(short, long, help = "Filter by project ID")]
        project: Option<i64>,

        #[arg(short = 't', long, help = "Filter by tag")]
        tag: Option<String>,

        #[arg(short = 'g', long, help = "Group entries by description")]
        group: bool,

        #[arg(long, help = "Use cached data (offline mode)")]
        offline: bool,
    },

    #[command(about = "Sync time entries from Toggl to local database")]
    Sync {
        #[arg(short, long, help = "Start date for sync")]
        start: Option<String>,

        #[arg(short, long, help = "End date for sync")]
        end: Option<String>,
    },

    #[command(about = "Interactive TUI mode")]
    Tui {
        #[arg(short, long, help = "Start date")]
        start: Option<String>,

        #[arg(short, long, help = "End date")]
        end: Option<String>,
    },

    #[command(about = "Delete application data (database and/or config)")]
    Clean {
        #[arg(long, help = "Delete all data (database + config)")]
        all: bool,

        #[arg(long, help = "Delete only the database")]
        data: bool,

        #[arg(long, help = "Delete only the configuration")]
        config: bool,

        #[arg(long, help = "Skip confirmation prompt")]
        confirm: bool,
    },

    #[command(about = "Export time entries to CSV format")]
    Export {
        #[arg(short, long, help = "Start date")]
        start: Option<String>,

        #[arg(short, long, help = "End date")]
        end: Option<String>,

        #[arg(short, long, help = "Output file path")]
        output: String,

        #[arg(long, help = "Include metadata header in export")]
        include_metadata: bool,

        #[arg(long, help = "Group entries by description")]
        group: bool,

        #[arg(long, help = "Group entries by description and day")]
        group_by_day: bool,
    },

    #[command(about = "Start or stop time tracking")]
    Track {
        #[command(subcommand)]
        action: TrackAction,
    },
}

#[derive(Subcommand)]
pub enum TrackAction {
    #[command(about = "Start a new time entry")]
    Start {
        #[arg(short, long, help = "Description for the time entry")]
        message: Option<String>,
    },

    #[command(about = "Stop the currently running time entry")]
    Stop,
}

impl Cli {
    pub fn parse_date(date_str: &str) -> anyhow::Result<DateTime<Utc>> {
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let naive_datetime = naive_date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
            return Ok(Utc.from_utc_datetime(&naive_datetime));
        }

        anyhow::bail!("Invalid date format. Use ISO 8601 (YYYY-MM-DDTHH:MM:SSZ) or YYYY-MM-DD")
    }
}
