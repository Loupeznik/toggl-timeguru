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

use cli::{Cli, Commands, TrackAction};
use config::Config;
use db::Database;
use processor::{
    filter_by_project, filter_by_tag, group_by_description, group_by_description_and_day,
};
use toggl::TogglClient;
use ui::App;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    init_tracing(cli.verbose);

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("========================================");
        tracing::error!("PANIC OCCURRED!");
        tracing::error!("Panic info: {}", panic_info);
        if let Some(location) = panic_info.location() {
            tracing::error!(
                "Panic location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            tracing::error!("Panic message: {}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            tracing::error!("Panic message: {}", s);
        }
        tracing::error!("========================================");
        eprintln!("\n\nAPPLICATION CRASHED! Check log file for details.");
        eprintln!(
            "Log location: {}/app.log",
            std::env::temp_dir().join("toggl-timeguru").display()
        );
    }));

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

            Commands::Clean {
                all,
                data,
                config,
                confirm,
            } => handle_clean(all, data, config, confirm).await?,

            Commands::Export {
                start,
                end,
                output,
                include_metadata,
                group,
                group_by_day,
            } => handle_export(start, end, output, include_metadata, group, group_by_day).await?,

            Commands::Track { action } => handle_track(action, cli.api_token).await?,
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
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    let default_level = if verbose { "debug" } else { "info" };

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    let log_dir = std::env::temp_dir().join("toggl-timeguru");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir.clone(), "app.log");

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false);

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr.with_max_level(tracing::Level::ERROR))
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stderr_layer)
        .init();

    tracing::info!("========================================");
    tracing::info!("Toggl TimeGuru starting");
    tracing::info!("Log file location: {}/app.log", log_dir.display());
    tracing::info!("Tracing initialized with level: {}", default_level);
    tracing::info!("========================================");
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
        db.get_time_entries(start_date, end_date, config.current_user_id)?
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
    let mut config = Config::load()?;
    let api_token = get_api_token(cli_api_token, &config)?;
    let client = TogglClient::new(api_token)?;
    let db = Database::new(None)?;

    let user_id = client.get_current_user_id().await?;
    let user_email = client.get_current_user_email().await?;

    if config.current_user_id.is_none() {
        config.current_user_id = Some(user_id);
        config.current_user_email = Some(user_email.clone());
        config.save()?;
        println!("Configured for user: {}", user_email);
    } else if config.current_user_id != Some(user_id) {
        println!("Switching to new user account: {}", user_email);
        println!("Previous data will not be visible.");
        println!("Use 'toggl-timeguru clean --data' to remove old data if needed.");
        config.current_user_id = Some(user_id);
        config.current_user_email = Some(user_email);
        config.save()?;
    }

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

    println!("Syncing projects and workspaces...");

    let workspaces = client.get_workspaces().await?;
    let mut total_projects = 0;

    for workspace in workspaces {
        let projects = client.get_projects(workspace.id).await?;
        let project_count = db.save_projects(&projects)?;
        total_projects += project_count;
    }

    println!("Successfully synced {} projects", total_projects);

    Ok(())
}

async fn handle_tui(
    start: Option<String>,
    end: Option<String>,
    cli_api_token: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let db = std::sync::Arc::new(Database::new(None)?);

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
        .get_time_entries(start_date, end_date, config.current_user_id)
        .context("Failed to load time entries. Try running 'sync' first.")?;

    if entries.is_empty() {
        println!("No time entries found. Run 'toggl-timeguru sync' first to download your data.");
        return Ok(());
    }

    let projects = db.get_projects().unwrap_or_default();

    let client = match get_api_token(cli_api_token, &config) {
        Ok(token) => match TogglClient::new(token) {
            Ok(c) => Some(std::sync::Arc::new(c)),
            Err(_) => None,
        },
        Err(_) => None,
    };

    let runtime_handle = Some(tokio::runtime::Handle::current());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(
        entries,
        start_date,
        end_date,
        config.round_duration_minutes,
        projects,
        client,
        runtime_handle,
        config.current_user_email.clone(),
        db,
    );
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

async fn handle_clean(all: bool, data: bool, config: bool, confirm: bool) -> Result<()> {
    use std::io::{self, Write};

    let delete_data = all || data;
    let delete_config = all || config;

    if !delete_data && !delete_config {
        println!("Please specify what to delete:");
        println!("  --all      Delete both database and config");
        println!("  --data     Delete only the database");
        println!("  --config   Delete only the configuration");
        return Ok(());
    }

    let db_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("toggl-timeguru")
        .join("timeguru.db");

    let config_path = confy::get_configuration_file_path("toggl-timeguru", "config")
        .unwrap_or_else(|_| std::path::PathBuf::from("~/.config/toggl-timeguru/config.toml"));

    println!("\nThe following will be deleted:");
    if delete_data {
        println!("  Database: {}", db_path.display());
    }
    if delete_config {
        println!("  Config:   {}", config_path.display());
    }

    if !confirm {
        print!("\nAre you sure you want to continue? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }
    }

    let mut deleted_items = Vec::new();
    let mut errors = Vec::new();

    if delete_data && db_path.exists() {
        match std::fs::remove_file(&db_path) {
            Ok(_) => {
                deleted_items.push(format!("Database: {}", db_path.display()));
                let parent_dir = db_path.parent();
                if let Some(dir) = parent_dir
                    && dir
                        .read_dir()
                        .map(|mut d| d.next().is_none())
                        .unwrap_or(false)
                {
                    let _ = std::fs::remove_dir(dir);
                }
            }
            Err(e) => errors.push(format!("Failed to delete database: {}", e)),
        }
    } else if delete_data {
        println!("Database not found at {}", db_path.display());
    }

    if delete_config && config_path.exists() {
        match std::fs::remove_file(&config_path) {
            Ok(_) => {
                deleted_items.push(format!("Config: {}", config_path.display()));
                let parent_dir = config_path.parent();
                if let Some(dir) = parent_dir
                    && dir
                        .read_dir()
                        .map(|mut d| d.next().is_none())
                        .unwrap_or(false)
                {
                    let _ = std::fs::remove_dir(dir);
                }
            }
            Err(e) => errors.push(format!("Failed to delete config: {}", e)),
        }
    } else if delete_config {
        println!("Config not found at {}", config_path.display());
    }

    if !deleted_items.is_empty() {
        println!("\nSuccessfully deleted:");
        for item in deleted_items {
            println!("  ✓ {}", item);
        }
    }

    if !errors.is_empty() {
        println!("\nErrors:");
        for error in errors {
            println!("  ✗ {}", error);
        }
        anyhow::bail!("Failed to delete some items");
    }

    println!("\nCleanup complete!");
    Ok(())
}

async fn handle_export(
    start: Option<String>,
    end: Option<String>,
    output: String,
    include_metadata: bool,
    group: bool,
    group_by_day: bool,
) -> Result<()> {
    use std::fs::File;

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

    let entries = db.get_time_entries(start_date, end_date, config.current_user_id)?;

    if entries.is_empty() {
        println!("No time entries found for the specified date range.");
        return Ok(());
    }

    let file = File::create(&output)
        .with_context(|| format!("Failed to create output file: {}", output))?;
    let mut wtr = csv::Writer::from_writer(file);

    let max_metadata_cols = 6;

    if include_metadata {
        let mut row = vec![String::new(); max_metadata_cols];
        row[0] = "# Toggl TimeGuru Export".to_string();
        wtr.write_record(&row)?;

        row.fill(String::new());
        row[0] = format!(
            "# Date Range: {} to {}",
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        );
        wtr.write_record(&row)?;

        row.fill(String::new());
        row[0] = format!("# Total Entries: {}", entries.len());
        wtr.write_record(&row)?;

        if let Some(user_email) = &config.current_user_email {
            row.fill(String::new());
            row[0] = format!("# User: {}", user_email);
            wtr.write_record(&row)?;
        }

        row.fill(String::new());
        wtr.write_record(&row)?;
    }

    let projects = db.get_projects().unwrap_or_default();
    let project_map: std::collections::HashMap<i64, String> =
        projects.into_iter().map(|p| (p.id, p.name)).collect();

    if group || group_by_day {
        let grouped = if group_by_day {
            group_by_description_and_day(entries)
        } else {
            group_by_description(entries)
        };

        if group_by_day {
            wtr.write_record([
                "Date",
                "Description",
                "Project",
                "Duration (hours)",
                "Entry Count",
                "Billable",
            ])?;
        } else {
            wtr.write_record([
                "Description",
                "Project",
                "Duration (hours)",
                "Entry Count",
                "Billable",
            ])?;
        }

        for entry in grouped {
            let desc = entry
                .description
                .clone()
                .unwrap_or_else(|| "(No description)".to_string());
            let project_name = entry
                .project_id
                .and_then(|pid| project_map.get(&pid).cloned())
                .unwrap_or_else(String::new);
            let hours = if let Some(round_min) = config.round_duration_minutes {
                entry.rounded_hours(round_min)
            } else {
                entry.total_hours()
            };
            let billable = if entry.entries.iter().all(|e| e.billable) {
                "Yes"
            } else if entry.entries.iter().all(|e| !e.billable) {
                "No"
            } else {
                "Mixed"
            };

            if group_by_day {
                let date_str = entry
                    .date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(String::new);
                wtr.write_record([
                    &date_str,
                    &desc,
                    &project_name,
                    &format!("{:.2}", hours),
                    &entry.entries.len().to_string(),
                    billable,
                ])?;
            } else {
                wtr.write_record([
                    &desc,
                    &project_name,
                    &format!("{:.2}", hours),
                    &entry.entries.len().to_string(),
                    billable,
                ])?;
            }
        }
    } else {
        wtr.write_record([
            "Date",
            "Time",
            "Description",
            "Project",
            "Duration (hours)",
            "Billable",
        ])?;

        for entry in entries {
            let desc = entry
                .description
                .unwrap_or_else(|| "(No description)".to_string());
            let project_name = entry
                .project_id
                .and_then(|pid| project_map.get(&pid).cloned())
                .unwrap_or_else(String::new);
            let hours = entry.duration as f64 / 3600.0;
            let billable = if entry.billable { "Yes" } else { "No" };

            wtr.write_record([
                &entry.start.format("%Y-%m-%d").to_string(),
                &entry.start.format("%H:%M").to_string(),
                &desc,
                &project_name,
                &format!("{:.2}", hours),
                billable,
            ])?;
        }
    }

    wtr.flush()?;
    println!("Successfully exported to: {}", output);
    Ok(())
}

async fn handle_track(action: TrackAction, cli_api_token: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let api_token = get_api_token(cli_api_token, &config)?;
    let client = TogglClient::new(api_token)?;

    let workspaces = client.get_workspaces().await?;
    let workspace_id = workspaces
        .first()
        .ok_or_else(|| anyhow::anyhow!("No workspace found for your account"))?
        .id;

    match action {
        TrackAction::Start { message } => {
            println!("Starting time tracking...");

            let time_entry = client
                .start_time_entry(workspace_id, message.clone())
                .await?;

            println!("✓ Time tracking started successfully!");
            if let Some(desc) = time_entry.description {
                println!("  Description: {}", desc);
            } else {
                println!("  Description: (No description)");
            }
            println!(
                "  Started at: {}",
                time_entry.start.format("%Y-%m-%d %H:%M:%S")
            );
            println!("  Entry ID: {}", time_entry.id);
        }

        TrackAction::Stop => {
            println!("Stopping time tracking...");

            let current_entry = client.get_current_time_entry(workspace_id).await?;

            if let Some(entry) = current_entry {
                let stopped_entry = client.stop_time_entry(workspace_id, entry.id).await?;

                println!("✓ Time tracking stopped successfully!");
                if let Some(desc) = stopped_entry.description {
                    println!("  Description: {}", desc);
                } else {
                    println!("  Description: (No description)");
                }
                println!(
                    "  Started at: {}",
                    stopped_entry.start.format("%Y-%m-%d %H:%M:%S")
                );
                if let Some(stop) = stopped_entry.stop {
                    println!("  Stopped at: {}", stop.format("%Y-%m-%d %H:%M:%S"));
                }
                let duration_hours = stopped_entry.duration as f64 / 3600.0;
                println!("  Duration: {:.2}h", duration_hours);
            } else {
                println!("No time entry is currently running.");
            }
        }
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
