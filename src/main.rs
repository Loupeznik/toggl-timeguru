mod cli;
mod config;
mod db;
mod processor;
mod toggl;
mod ui;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cli::{Cli, Commands};
use config::Config;
use db::Database;
use processor::{filter_by_project, filter_by_tag, group_by_description};
use toggl::TogglClient;
use ui::App;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    init_tracing(cli.verbose);

    if let Some(command) = cli.command {
        match command {
            Commands::Config {
                set_token,
                set_date_range,
                set_round_minutes,
                show,
            } => handle_config(set_token, set_date_range, set_round_minutes, show).await?,

            Commands::List {
                start,
                end,
                project,
                tag,
                group,
                offline,
            } => handle_list(start, end, project, tag, group, offline, cli.api_token).await?,

            Commands::Sync { start, end } => handle_sync(start, end, cli.api_token).await?,

            Commands::Tui { start, end } => handle_tui(start, end, cli.api_token).await?,
        }
    } else {
        println!("Toggl TimeGuru - Use --help for usage information");
        println!("\nQuick start:");
        println!("  1. Set your API token: toggl-timeguru config --set-token YOUR_TOKEN");
        println!("  2. Sync your time entries: toggl-timeguru sync");
        println!("  3. View entries: toggl-timeguru tui");
    }

    Ok(())
}

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        tracing_subscriber::EnvFilter::new("debug")
    } else {
        tracing_subscriber::EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn handle_config(
    set_token: Option<String>,
    set_date_range: Option<i64>,
    set_round_minutes: Option<i64>,
    show: bool,
) -> Result<()> {
    let mut config = Config::load()?;

    if let Some(token) = set_token {
        config.api_token_encrypted = Some(token.into_bytes());
        config.save()?;
        println!("API token saved successfully");
    }

    if let Some(days) = set_date_range {
        config.default_date_range_days = days;
        config.save()?;
        println!("Default date range set to {} days", days);
    }

    if let Some(minutes) = set_round_minutes {
        config.round_duration_minutes = Some(minutes);
        config.save()?;
        println!("Rounding duration set to {} minutes", minutes);
    }

    if show {
        println!("\nCurrent Configuration:");
        println!(
            "  Default date range: {} days",
            config.default_date_range_days
        );
        println!("  Report format: {:?}", config.preferred_report_format);
        println!(
            "  Round duration: {:?} minutes",
            config.round_duration_minutes
        );
        println!(
            "  API token configured: {}",
            config.api_token_encrypted.is_some()
        );
    }

    Ok(())
}

async fn handle_list(
    start: Option<String>,
    end: Option<String>,
    project: Option<i64>,
    tag: Option<String>,
    group: bool,
    offline: bool,
    cli_api_token: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let db = Database::new(None)?;

    let end_date = if let Some(end_str) = end {
        Cli::parse_date(&end_str)?
    } else {
        Utc::now()
    };

    let start_date = if let Some(start_str) = start {
        Cli::parse_date(&start_str)?
    } else {
        end_date - config.default_date_range()
    };

    let mut entries = if offline {
        db.get_time_entries(start_date, end_date)?
    } else {
        let api_token = get_api_token(cli_api_token, &config)?;
        let client = TogglClient::new(api_token)?;

        let entries = client.get_time_entries(start_date, end_date).await?;
        db.save_time_entries(&entries)?;
        db.update_sync_metadata("time_entries", entries.last().map(|e| e.id))?;

        entries
    };

    if let Some(project_id) = project {
        entries = filter_by_project(entries, project_id);
    }

    if let Some(tag_name) = tag {
        entries = filter_by_tag(entries, &tag_name);
    }

    if group {
        let grouped = group_by_description(entries);
        println!("\nGrouped Time Entries ({} groups):", grouped.len());
        println!("{:<60} {:>10} {:>10}", "Description", "Duration", "Entries");
        println!("{}", "-".repeat(82));

        for entry in grouped {
            let desc = entry
                .description
                .clone()
                .unwrap_or_else(|| "(No description)".to_string());
            let hours = if let Some(round_min) = config.round_duration_minutes {
                entry.rounded_hours(round_min)
            } else {
                entry.total_hours()
            };

            println!(
                "{:<60} {:>9.2}h {:>10}",
                truncate(&desc, 60),
                hours,
                entry.entries.len()
            );
        }
    } else {
        println!("\nTime Entries ({}):", entries.len());
        println!("{:<20} {:<60} {:>10}", "Date", "Description", "Duration");
        println!("{}", "-".repeat(92));

        for entry in entries {
            let desc = entry
                .description
                .unwrap_or_else(|| "(No description)".to_string());
            let hours = entry.duration as f64 / 3600.0;

            println!(
                "{:<20} {:<60} {:>9.2}h",
                entry.start.format("%Y-%m-%d %H:%M"),
                truncate(&desc, 60),
                hours
            );
        }
    }

    Ok(())
}

async fn handle_sync(
    start: Option<String>,
    end: Option<String>,
    cli_api_token: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let api_token = get_api_token(cli_api_token, &config)?;
    let client = TogglClient::new(api_token)?;
    let db = Database::new(None)?;

    let end_date = if let Some(end_str) = end {
        Cli::parse_date(&end_str)?
    } else {
        Utc::now()
    };

    let start_date = if let Some(start_str) = start {
        Cli::parse_date(&start_str)?
    } else {
        end_date - Duration::days(90)
    };

    println!(
        "Syncing time entries from {} to {}...",
        start_date.format("%Y-%m-%d"),
        end_date.format("%Y-%m-%d")
    );

    let entries = client.get_time_entries(start_date, end_date).await?;
    let count = db.save_time_entries(&entries)?;
    db.update_sync_metadata("time_entries", entries.last().map(|e| e.id))?;

    println!("Successfully synced {} time entries", count);

    Ok(())
}

async fn handle_tui(
    start: Option<String>,
    end: Option<String>,
    _cli_api_token: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let db = Database::new(None)?;

    let end_date = if let Some(end_str) = end {
        Cli::parse_date(&end_str)?
    } else {
        Utc::now()
    };

    let start_date = if let Some(start_str) = start {
        Cli::parse_date(&start_str)?
    } else {
        end_date - config.default_date_range()
    };

    let entries = db
        .get_time_entries(start_date, end_date)
        .context("Failed to load time entries. Try running 'sync' first.")?;

    if entries.is_empty() {
        println!("No time entries found. Run 'toggl-timeguru sync' first to download your data.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(entries, start_date, end_date, config.round_duration_minutes);
    let grouped = group_by_description(app.time_entries.clone());
    app.grouped_entries = grouped;

    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn get_api_token(cli_token: Option<String>, config: &Config) -> Result<String> {
    if let Some(token) = cli_token {
        return Ok(token);
    }

    if let Some(encrypted) = &config.api_token_encrypted {
        return String::from_utf8(encrypted.clone()).context("Failed to decode API token");
    }

    anyhow::bail!(
        "No API token provided. Set it with: toggl-timeguru config --set-token YOUR_TOKEN"
    )
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
