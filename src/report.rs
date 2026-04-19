use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Utc};
use std::collections::HashMap;
use std::str::FromStr;

use crate::toggl::models::{Project, TimeEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoundingMode {
    #[default]
    Total,
    Entry,
}

impl RoundingMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Total => "per total",
            Self::Entry => "per entry",
        }
    }
}

impl FromStr for RoundingMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "total" | "totals" | "aggregate" => Ok(Self::Total),
            "entry" | "entries" | "per-entry" => Ok(Self::Entry),
            other => Err(anyhow::anyhow!(
                "invalid round mode '{other}', expected 'total' or 'entry'"
            )),
        }
    }
}

impl ReportPeriod {
    pub fn label(self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
        }
    }
}

impl FromStr for ReportPeriod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "daily" | "day" | "d" => Ok(Self::Daily),
            "weekly" | "week" | "w" => Ok(Self::Weekly),
            "monthly" | "month" | "m" => Ok(Self::Monthly),
            other => Err(anyhow::anyhow!(
                "invalid report period '{other}', expected 'daily', 'weekly', or 'monthly'"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectSummary {
    #[allow(dead_code)]
    pub project_id: Option<i64>,
    pub project_name: String,
    pub duration: i64,
    pub billable_duration: i64,
    pub non_billable_duration: i64,
}

#[derive(Debug, Clone)]
pub struct PeriodBucket {
    pub label: String,
    pub duration: i64,
    pub billable_duration: i64,
    pub non_billable_duration: i64,
    pub by_project: Vec<ProjectSummary>,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub period: ReportPeriod,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub entry_count: usize,
    pub total_duration: i64,
    pub billable_duration: i64,
    pub non_billable_duration: i64,
    pub by_project: Vec<ProjectSummary>,
    pub by_period: Vec<PeriodBucket>,
    pub round_minutes: Option<i64>,
    pub round_mode: RoundingMode,
}

fn bucket_key(start: DateTime<Utc>, period: ReportPeriod) -> (String, NaiveDate) {
    let local = start.with_timezone(&Local);
    match period {
        ReportPeriod::Daily => {
            let date = local.date_naive();
            (date.format("%Y-%m-%d").to_string(), date)
        }
        ReportPeriod::Weekly => {
            let weekday_idx = local.weekday().num_days_from_monday() as i64;
            let monday_date = local.date_naive() - Duration::days(weekday_idx);
            let label = format!(
                "{} (W{:02})",
                monday_date.format("%Y-%m-%d"),
                local.iso_week().week()
            );
            (label, monday_date)
        }
        ReportPeriod::Monthly => {
            let label = local.format("%Y-%m").to_string();
            let first = NaiveDate::from_ymd_opt(local.year(), local.month(), 1)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
            (label, first)
        }
    }
}

fn project_name(project_id: Option<i64>, projects: &HashMap<i64, Project>) -> String {
    match project_id {
        Some(id) => projects
            .get(&id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Project #{id}")),
        None => "(no project)".to_string(),
    }
}

fn aggregate_by_project(
    entries: &[(&TimeEntry, i64)],
    projects: &HashMap<i64, Project>,
) -> Vec<ProjectSummary> {
    let mut map: HashMap<Option<i64>, ProjectSummary> = HashMap::new();
    for (entry, dur) in entries {
        if *dur <= 0 {
            continue;
        }
        let summary = map
            .entry(entry.project_id)
            .or_insert_with(|| ProjectSummary {
                project_id: entry.project_id,
                project_name: project_name(entry.project_id, projects),
                duration: 0,
                billable_duration: 0,
                non_billable_duration: 0,
            });
        summary.duration += *dur;
        if entry.billable {
            summary.billable_duration += *dur;
        } else {
            summary.non_billable_duration += *dur;
        }
    }
    let mut out: Vec<ProjectSummary> = map.into_values().collect();
    out.sort_by(|a, b| {
        b.duration.cmp(&a.duration).then_with(|| {
            a.project_name
                .to_lowercase()
                .cmp(&b.project_name.to_lowercase())
        })
    });
    out
}

pub fn generate(
    entries: &[TimeEntry],
    projects: &[Project],
    period: ReportPeriod,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    round_minutes: Option<i64>,
    round_mode: RoundingMode,
) -> Report {
    let projects_map: HashMap<i64, Project> = projects.iter().map(|p| (p.id, p.clone())).collect();

    let duration_for = |raw: i64| -> i64 {
        match round_mode {
            RoundingMode::Entry => round_seconds_up(raw, round_minutes),
            RoundingMode::Total => raw,
        }
    };

    let valid: Vec<(&TimeEntry, i64)> = entries
        .iter()
        .filter(|e| e.duration > 0 && e.start >= start_date && e.start <= end_date)
        .map(|e| (e, duration_for(e.duration)))
        .collect();

    let total_duration: i64 = valid.iter().map(|(_, d)| *d).sum();
    let billable_duration: i64 = valid
        .iter()
        .filter(|(e, _)| e.billable)
        .map(|(_, d)| *d)
        .sum();
    let non_billable_duration = total_duration - billable_duration;

    let by_project = aggregate_by_project(&valid, &projects_map);

    type BucketEntries<'a> = (NaiveDate, Vec<(&'a TimeEntry, i64)>);
    let mut bucket_groups: HashMap<String, BucketEntries> = HashMap::new();
    for (entry, dur) in &valid {
        let (label, sort_key) = bucket_key(entry.start, period);
        bucket_groups
            .entry(label)
            .or_insert_with(|| (sort_key, Vec::new()))
            .1
            .push((entry, *dur));
    }

    let mut buckets_with_sort: Vec<(NaiveDate, PeriodBucket)> = bucket_groups
        .into_iter()
        .map(|(label, (sort_key, bucket_entries))| {
            let duration: i64 = bucket_entries.iter().map(|(_, d)| *d).sum();
            let bucket_billable: i64 = bucket_entries
                .iter()
                .filter(|(e, _)| e.billable)
                .map(|(_, d)| *d)
                .sum();
            let bucket_non_billable = duration - bucket_billable;
            let by_project = aggregate_by_project(&bucket_entries, &projects_map);
            (
                sort_key,
                PeriodBucket {
                    label,
                    duration,
                    billable_duration: bucket_billable,
                    non_billable_duration: bucket_non_billable,
                    by_project,
                },
            )
        })
        .collect();
    buckets_with_sort.sort_by_key(|(dt, _)| *dt);
    let by_period: Vec<PeriodBucket> = buckets_with_sort.into_iter().map(|(_, b)| b).collect();

    Report {
        period,
        start_date,
        end_date,
        entry_count: valid.len(),
        total_duration,
        billable_duration,
        non_billable_duration,
        by_project,
        by_period,
        round_minutes,
        round_mode,
    }
}

fn round_seconds_up(seconds: i64, round_minutes: Option<i64>) -> i64 {
    match round_minutes {
        Some(m) if m > 0 => {
            let step = m * 60;
            ((seconds as f64 / step as f64).ceil() as i64) * step
        }
        _ => seconds,
    }
}

fn format_hours(seconds: i64, round_minutes: Option<i64>) -> String {
    format!(
        "{:.2}h",
        round_seconds_up(seconds, round_minutes) as f64 / 3600.0
    )
}

fn pct(part: i64, total: i64) -> f64 {
    if total <= 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

pub fn print_text(report: &Report) {
    let start_local = report.start_date.with_timezone(&Local);
    let end_local = report.end_date.with_timezone(&Local);
    let display_round = if report.round_mode == RoundingMode::Total {
        report.round_minutes
    } else {
        None
    };
    let round_suffix = report
        .round_minutes
        .map(|m| format!(" (rounded up to {m} min, {})", report.round_mode.label()))
        .unwrap_or_default();

    println!(
        "\n{} Report — {} to {}{}",
        report.period.label(),
        start_local.format("%Y-%m-%d"),
        end_local.format("%Y-%m-%d"),
        round_suffix,
    );
    println!("{}", "─".repeat(70));

    if report.entry_count == 0 {
        println!("No time entries in the selected range.\n");
        return;
    }

    println!(
        "Total: {}  │  Billable: {} ({:.0}%)  │  Non-billable: {} ({:.0}%)  │  Entries: {}",
        format_hours(report.total_duration, display_round),
        format_hours(report.billable_duration, display_round),
        pct(report.billable_duration, report.total_duration),
        format_hours(report.non_billable_duration, display_round),
        pct(report.non_billable_duration, report.total_duration),
        report.entry_count,
    );

    println!("\nBy Project:");
    println!(
        "  {:<40} {:>10} {:>8} {:>10} {:>10}",
        "Project", "Total", "% Total", "Billable", "Non-bill."
    );
    println!("  {}", "-".repeat(82));
    for p in &report.by_project {
        println!(
            "  {:<40} {:>10} {:>7.0}% {:>10} {:>10}",
            truncate(&p.project_name, 40),
            format_hours(p.duration, display_round),
            pct(p.duration, report.total_duration),
            format_hours(p.billable_duration, display_round),
            format_hours(p.non_billable_duration, display_round),
        );
    }

    println!("\n{} Breakdown:", report.period.label());
    println!(
        "  {:<22} {:>10} {:>10} {:>10}",
        "Period", "Total", "Billable", "Non-bill."
    );
    println!("  {}", "-".repeat(58));
    for bucket in &report.by_period {
        println!(
            "  {:<22} {:>10} {:>10} {:>10}",
            bucket.label,
            format_hours(bucket.duration, display_round),
            format_hours(bucket.billable_duration, display_round),
            format_hours(bucket.non_billable_duration, display_round),
        );
        for p in bucket.by_project.iter().take(5) {
            println!(
                "      {:<36} {:>10} {:>7.0}%",
                truncate(&p.project_name, 36),
                format_hours(p.duration, display_round),
                pct(p.duration, bucket.duration),
            );
        }
        if bucket.by_project.len() > 5 {
            println!(
                "      ({} more project(s) hidden)",
                bucket.by_project.len() - 5
            );
        }
    }
    println!();
}

fn truncate(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn entry(
        id: i64,
        start: DateTime<Utc>,
        duration: i64,
        pid: Option<i64>,
        billable: bool,
    ) -> TimeEntry {
        TimeEntry {
            id,
            workspace_id: 1,
            project_id: pid,
            task_id: None,
            billable,
            start,
            stop: None,
            duration,
            description: None,
            tags: None,
            tag_ids: None,
            duronly: false,
            at: start,
            server_deleted_at: None,
            user_id: 1,
            uid: None,
            wid: None,
            pid: None,
        }
    }

    fn project(id: i64, name: &str) -> Project {
        Project {
            id,
            workspace_id: 1,
            client_id: None,
            name: name.to_string(),
            is_private: false,
            active: true,
            at: Utc::now(),
            created_at: Utc::now(),
            color: "#000000".to_string(),
            billable: None,
            template: None,
            auto_estimates: None,
            estimated_hours: None,
            rate: None,
            currency: None,
        }
    }

    #[test]
    fn daily_report_aggregates_by_day() {
        let d1 = Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2026, 4, 2, 10, 0, 0).unwrap();
        let entries = vec![
            entry(1, d1, 3600, Some(1), true),
            entry(2, d1, 1800, Some(1), false),
            entry(3, d2, 7200, Some(2), true),
        ];
        let projects = vec![project(1, "A"), project(2, "B")];
        let report = generate(
            &entries,
            &projects,
            ReportPeriod::Daily,
            d1,
            d2,
            None,
            RoundingMode::Total,
        );

        assert_eq!(report.total_duration, 3600 + 1800 + 7200);
        assert_eq!(report.billable_duration, 3600 + 7200);
        assert_eq!(report.non_billable_duration, 1800);
        assert_eq!(report.by_project.len(), 2);
        assert_eq!(report.by_project[0].project_name, "B");
        assert_eq!(report.by_period.len(), 2);
    }

    #[test]
    fn weekly_report_buckets_same_iso_week() {
        let wed = Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0).unwrap();
        let fri = Utc.with_ymd_and_hms(2026, 4, 3, 9, 0, 0).unwrap();
        let entries = vec![
            entry(1, wed, 3600, Some(1), true),
            entry(2, fri, 1800, Some(1), true),
        ];
        let projects = vec![project(1, "A")];
        let report = generate(
            &entries,
            &projects,
            ReportPeriod::Weekly,
            wed,
            fri,
            None,
            RoundingMode::Total,
        );
        assert_eq!(report.by_period.len(), 1);
        assert_eq!(report.by_period[0].duration, 5400);
    }

    #[test]
    fn zero_duration_entries_are_ignored() {
        let d = Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0).unwrap();
        let entries = vec![
            entry(1, d, 0, Some(1), true),
            entry(2, d, 3600, Some(1), true),
        ];
        let projects = vec![project(1, "A")];
        let report = generate(
            &entries,
            &projects,
            ReportPeriod::Daily,
            d,
            d,
            None,
            RoundingMode::Total,
        );
        assert_eq!(report.entry_count, 1);
        assert_eq!(report.total_duration, 3600);
    }

    #[test]
    fn rounding_ceils_to_next_interval() {
        assert_eq!(round_seconds_up(0, Some(15)), 0);
        assert_eq!(round_seconds_up(1, Some(15)), 900);
        assert_eq!(round_seconds_up(900, Some(15)), 900);
        assert_eq!(round_seconds_up(901, Some(15)), 1800);
        assert_eq!(round_seconds_up(3601, Some(30)), 5400);
        assert_eq!(round_seconds_up(3601, None), 3601);
        assert_eq!(round_seconds_up(3601, Some(0)), 3601);
    }

    #[test]
    fn period_parses_aliases() {
        assert_eq!(
            ReportPeriod::from_str("daily").unwrap(),
            ReportPeriod::Daily
        );
        assert_eq!(
            ReportPeriod::from_str("WEEK").unwrap(),
            ReportPeriod::Weekly
        );
        assert_eq!(ReportPeriod::from_str("m").unwrap(), ReportPeriod::Monthly);
        assert!(ReportPeriod::from_str("yearly").is_err());
    }

    #[test]
    fn round_mode_parses_aliases() {
        assert_eq!(
            RoundingMode::from_str("total").unwrap(),
            RoundingMode::Total
        );
        assert_eq!(
            RoundingMode::from_str("ENTRIES").unwrap(),
            RoundingMode::Entry
        );
        assert!(RoundingMode::from_str("bucket").is_err());
    }

    #[test]
    fn per_entry_rounding_differs_from_total_rounding() {
        let d = Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0).unwrap();
        // Two entries, 60s each = 120s raw. Total-round to 15min → 900s. Entry-round → 2 × 900s = 1800s.
        let entries = vec![
            entry(1, d, 60, Some(1), true),
            entry(2, d, 60, Some(1), true),
        ];
        let projects = vec![project(1, "A")];

        let totals_report = generate(
            &entries,
            &projects,
            ReportPeriod::Daily,
            d,
            d,
            Some(15),
            RoundingMode::Total,
        );
        // Raw aggregation; display-time rounding is applied by print_text.
        assert_eq!(totals_report.total_duration, 120);

        let entry_report = generate(
            &entries,
            &projects,
            ReportPeriod::Daily,
            d,
            d,
            Some(15),
            RoundingMode::Entry,
        );
        // Per-entry ceil to 15min: 60s → 900s each, sum = 1800s.
        assert_eq!(entry_report.total_duration, 1800);
        assert_eq!(entry_report.by_project[0].duration, 1800);
        assert_eq!(entry_report.by_period[0].duration, 1800);
    }
}
