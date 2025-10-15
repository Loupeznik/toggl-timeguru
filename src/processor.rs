use crate::toggl::models::{GroupedTimeEntry, TimeEntry};
use std::collections::HashMap;

pub fn group_by_description(entries: Vec<TimeEntry>) -> Vec<GroupedTimeEntry> {
    let mut groups: HashMap<(Option<String>, Option<i64>), Vec<TimeEntry>> = HashMap::new();

    for entry in entries {
        let key = (entry.description.clone(), entry.project_id);
        groups.entry(key).or_default().push(entry);
    }

    let mut grouped: Vec<GroupedTimeEntry> = groups
        .into_iter()
        .map(|((description, project_id), entries)| {
            let total_duration: i64 = entries.iter().map(|e| e.duration).sum();

            GroupedTimeEntry {
                description,
                project_id,
                entries,
                total_duration,
            }
        })
        .collect();

    grouped.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));

    grouped
}

pub fn filter_by_project(entries: Vec<TimeEntry>, project_id: i64) -> Vec<TimeEntry> {
    entries
        .into_iter()
        .filter(|e| e.project_id == Some(project_id))
        .collect()
}

pub fn filter_by_tag(entries: Vec<TimeEntry>, tag: &str) -> Vec<TimeEntry> {
    entries
        .into_iter()
        .filter(|e| {
            if let Some(tags) = &e.tags {
                tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
            } else {
                false
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn calculate_total_duration(entries: &[TimeEntry]) -> i64 {
    entries.iter().map(|e| e.duration).sum()
}

#[allow(dead_code)]
pub fn calculate_billable_duration(entries: &[TimeEntry]) -> i64 {
    entries
        .iter()
        .filter(|e| e.billable)
        .map(|e| e.duration)
        .sum()
}

#[allow(dead_code)]
pub fn calculate_non_billable_duration(entries: &[TimeEntry]) -> i64 {
    entries
        .iter()
        .filter(|e| !e.billable)
        .map(|e| e.duration)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_entry(
        id: i64,
        description: &str,
        duration: i64,
        project_id: Option<i64>,
    ) -> TimeEntry {
        TimeEntry {
            id,
            workspace_id: 1,
            project_id,
            task_id: None,
            billable: false,
            start: Utc::now(),
            stop: Some(Utc::now()),
            duration,
            description: Some(description.to_string()),
            tags: None,
            tag_ids: None,
            duronly: false,
            at: Utc::now(),
            server_deleted_at: None,
            user_id: 1,
            uid: None,
            wid: None,
            pid: None,
        }
    }

    #[test]
    fn test_group_by_description() {
        let entries = vec![
            create_test_entry(1, "Task A", 3600, Some(1)),
            create_test_entry(2, "Task A", 1800, Some(1)),
            create_test_entry(3, "Task B", 7200, Some(2)),
        ];

        let grouped = group_by_description(entries);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].total_duration, 7200);
        assert_eq!(grouped[1].total_duration, 5400);
    }

    #[test]
    fn test_filter_by_project() {
        let entries = vec![
            create_test_entry(1, "Task A", 3600, Some(1)),
            create_test_entry(2, "Task B", 1800, Some(2)),
            create_test_entry(3, "Task C", 7200, Some(1)),
        ];

        let filtered = filter_by_project(entries, 1);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.project_id == Some(1)));
    }

    #[test]
    fn test_calculate_total_duration() {
        let entries = vec![
            create_test_entry(1, "Task A", 3600, Some(1)),
            create_test_entry(2, "Task B", 1800, Some(1)),
        ];

        let total = calculate_total_duration(&entries);

        assert_eq!(total, 5400);
    }
}
