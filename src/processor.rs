use crate::toggl::models::{GroupedTimeEntry, Project, TimeEntry};
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
pub fn filter_by_client(
    entries: Vec<TimeEntry>,
    client_id: i64,
    projects: &[Project],
) -> Vec<TimeEntry> {
    let project_ids: Vec<i64> = projects
        .iter()
        .filter(|p| p.client_id == Some(client_id))
        .map(|p| p.id)
        .collect();

    entries
        .into_iter()
        .filter(|e| {
            if let Some(pid) = e.project_id {
                project_ids.contains(&pid)
            } else {
                false
            }
        })
        .collect()
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TimeEntryFilter {
    pub project_id: Option<i64>,
    pub tag: Option<String>,
    pub client_id: Option<i64>,
    pub billable_only: bool,
}

#[allow(dead_code)]
impl TimeEntryFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_project(mut self, project_id: i64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn with_client(mut self, client_id: i64) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn with_billable_only(mut self) -> Self {
        self.billable_only = true;
        self
    }

    pub fn apply(&self, mut entries: Vec<TimeEntry>, projects: &[Project]) -> Vec<TimeEntry> {
        if let Some(project_id) = self.project_id {
            entries = filter_by_project(entries, project_id);
        }

        if let Some(ref tag) = self.tag {
            entries = filter_by_tag(entries, tag);
        }

        if let Some(client_id) = self.client_id {
            entries = filter_by_client(entries, client_id, projects);
        }

        if self.billable_only {
            entries.retain(|e| e.billable);
        }

        entries
    }
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

    fn create_test_project(id: i64, client_id: Option<i64>) -> crate::toggl::models::Project {
        crate::toggl::models::Project {
            id,
            workspace_id: 1,
            client_id,
            name: format!("Project {}", id),
            is_private: false,
            active: true,
            at: Utc::now(),
            created_at: Utc::now(),
            color: "#000000".to_string(),
            billable: Some(false),
            template: None,
            auto_estimates: None,
            estimated_hours: None,
            rate: None,
            currency: None,
        }
    }

    #[test]
    fn test_filter_by_client() {
        let projects = vec![
            create_test_project(1, Some(100)),
            create_test_project(2, Some(200)),
            create_test_project(3, Some(100)),
        ];

        let entries = vec![
            create_test_entry(1, "Task A", 3600, Some(1)),
            create_test_entry(2, "Task B", 1800, Some(2)),
            create_test_entry(3, "Task C", 7200, Some(3)),
        ];

        let filtered = filter_by_client(entries, 100, &projects);

        assert_eq!(filtered.len(), 2);
        assert!(
            filtered
                .iter()
                .all(|e| e.project_id == Some(1) || e.project_id == Some(3))
        );
    }

    #[test]
    fn test_combined_filters() {
        let projects = vec![
            create_test_project(1, Some(100)),
            create_test_project(2, Some(200)),
        ];

        let mut entry1 = create_test_entry(1, "Task A", 3600, Some(1));
        entry1.tags = Some(vec!["urgent".to_string()]);

        let mut entry2 = create_test_entry(2, "Task B", 1800, Some(1));
        entry2.tags = Some(vec!["normal".to_string()]);

        let mut entry3 = create_test_entry(3, "Task C", 7200, Some(2));
        entry3.tags = Some(vec!["urgent".to_string()]);

        let entries = vec![entry1, entry2, entry3];

        let filter = TimeEntryFilter::new()
            .with_client(100)
            .with_tag("urgent".to_string());

        let filtered = filter.apply(entries, &projects);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut entry1 = create_test_entry(1, "Task A", 3600, Some(1));
        entry1.tags = Some(vec!["urgent".to_string(), "bug".to_string()]);

        let mut entry2 = create_test_entry(2, "Task B", 1800, Some(1));
        entry2.tags = Some(vec!["feature".to_string()]);

        let entries = vec![entry1, entry2];

        let filtered = filter_by_tag(entries, "urgent");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_billable_filter() {
        let mut entry1 = create_test_entry(1, "Task A", 3600, Some(1));
        entry1.billable = true;

        let entry2 = create_test_entry(2, "Task B", 1800, Some(1));
        // entry2.billable = false (default)

        let entries = vec![entry1, entry2];

        let filter = TimeEntryFilter::new().with_billable_only();
        let filtered = filter.apply(entries, &[]);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
        assert!(filtered[0].billable);
    }
}
