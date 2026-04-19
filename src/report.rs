use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use std::collections::HashMap;
use std::str::FromStr;

use crate::toggl::models::{Project, TimeEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
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
}

fn bucket_key(start: DateTime<Utc>, period: ReportPeriod) -> (String, DateTime<Utc>) {
    let local = start;
    match period {
        ReportPeriod::Daily => {
            let date = local.date_naive();
            (date.format("%Y-%m-%d").to_string(), local)
        }
        ReportPeriod::Weekly => {
            let weekday_idx = local.weekday().num_days_from_monday() as i64;
            let monday_date = local.date_naive() - Duration::days(weekday_idx);
            let label = format!(
                "{} (W{:02})",
                monday_date.format("%Y-%m-%d"),
                local.iso_week().week()
            );
            let sort_dt = DateTime::<Utc>::from_naive_utc_and_offset(
                monday_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| {
                    NaiveDate::from_ymd_opt(1970, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                }),
                Utc,
            );
            (label, sort_dt)
        }
        ReportPeriod::Monthly => {
            let label = local.format("%Y-%m").to_string();
            let sort_dt = DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDate::from_ymd_opt(local.year(), local.month(), 1)
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                Utc,
            );
            (label, sort_dt)
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
    entries: &[&TimeEntry],
    projects: &HashMap<i64, Project>,
) -> Vec<ProjectSummary> {
    let mut map: HashMap<Option<i64>, ProjectSummary> = HashMap::new();
    for entry in entries {
        if entry.duration <= 0 {
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
        summary.duration += entry.duration;
        if entry.billable {
            summary.billable_duration += entry.duration;
        } else {
            summary.non_billable_duration += entry.duration;
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
) -> Report {
    let projects_map: HashMap<i64, Project> = projects.iter().map(|p| (p.id, p.clone())).collect();

    let valid: Vec<&TimeEntry> = entries.iter().filter(|e| e.duration > 0).collect();

    let total_duration: i64 = valid.iter().map(|e| e.duration).sum();
    let billable_duration: i64 = valid
        .iter()
        .filter(|e| e.billable)
        .map(|e| e.duration)
        .sum();
    let non_billable_duration = total_duration - billable_duration;

    let by_project = aggregate_by_project(&valid, &projects_map);

    let mut bucket_groups: HashMap<String, (DateTime<Utc>, Vec<&TimeEntry>)> = HashMap::new();
    for entry in &valid {
        let (label, sort_dt) = bucket_key(entry.start, period);
        bucket_groups
            .entry(label)
            .or_insert_with(|| (sort_dt, Vec::new()))
            .1
            .push(entry);
    }

    let mut buckets_with_sort: Vec<(DateTime<Utc>, PeriodBucket)> = bucket_groups
        .into_iter()
        .map(|(label, (sort_dt, bucket_entries))| {
            let duration: i64 = bucket_entries.iter().map(|e| e.duration).sum();
            let bucket_billable: i64 = bucket_entries
                .iter()
                .filter(|e| e.billable)
                .map(|e| e.duration)
                .sum();
            let bucket_non_billable = duration - bucket_billable;
            let by_project = aggregate_by_project(&bucket_entries, &projects_map);
            (
                sort_dt,
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
    }
}

fn format_hours(seconds: i64) -> String {
    format!("{:.2}h", seconds as f64 / 3600.0)
}

fn pct(part: i64, total: i64) -> f64 {
    if total <= 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

pub fn print_text(report: &Report) {
    println!(
        "\n{} Report — {} to {}",
        report.period.label(),
        report.start_date.format("%Y-%m-%d"),
        report.end_date.format("%Y-%m-%d"),
    );
    println!("{}", "─".repeat(70));

    if report.entry_count == 0 {
        println!("No time entries in the selected range.\n");
        return;
    }

    println!(
        "Total: {}  │  Billable: {} ({:.0}%)  │  Non-billable: {} ({:.0}%)  │  Entries: {}",
        format_hours(report.total_duration),
        format_hours(report.billable_duration),
        pct(report.billable_duration, report.total_duration),
        format_hours(report.non_billable_duration),
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
            format_hours(p.duration),
            pct(p.duration, report.total_duration),
            format_hours(p.billable_duration),
            format_hours(p.non_billable_duration),
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
            format_hours(bucket.duration),
            format_hours(bucket.billable_duration),
            format_hours(bucket.non_billable_duration),
        );
        for p in bucket.by_project.iter().take(5) {
            println!(
                "      {:<36} {:>10} {:>7.0}%",
                truncate(&p.project_name, 36),
                format_hours(p.duration),
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
        let report = generate(&entries, &projects, ReportPeriod::Daily, d1, d2);

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
        let report = generate(&entries, &projects, ReportPeriod::Weekly, wed, fri);
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
        let report = generate(&entries, &projects, ReportPeriod::Daily, d, d);
        assert_eq!(report.entry_count, 1);
        assert_eq!(report.total_duration, 3600);
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
}
