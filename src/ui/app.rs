use anyhow::Result;
use arboard::Clipboard;
use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::config::{PersistedFilter, ProjectSortMethod};
use crate::processor::TimeEntryFilter;
use crate::toggl::TogglClient;
use crate::toggl::models::{GroupedTimeEntry, Project, TimeEntry};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const PAGE_SIZE: usize = 10;
const POPUP_MARGIN: u16 = 10;
const POPUP_MAX_WIDTH: u16 = 80;
const POPUP_MAX_HEIGHT: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSection {
    Billable,
    Projects,
    Tags,
}

impl FilterSection {
    fn next(self) -> Self {
        match self {
            Self::Billable => Self::Projects,
            Self::Projects => Self::Tags,
            Self::Tags => Self::Billable,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Billable => Self::Tags,
            Self::Projects => Self::Billable,
            Self::Tags => Self::Projects,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Billable => "Billable",
            Self::Projects => "Projects",
            Self::Tags => "Tags",
        }
    }
}

fn sort_projects(projects: &mut [Project], method: ProjectSortMethod, usage: &HashMap<i64, usize>) {
    match method {
        ProjectSortMethod::Name => {
            projects.sort_by_cached_key(|p| p.name.to_lowercase());
        }
        ProjectSortMethod::Usage => {
            projects.sort_by_cached_key(|p| {
                let count = usage.get(&p.id).copied().unwrap_or(0);
                (std::cmp::Reverse(count), p.name.to_lowercase())
            });
        }
    }
}

pub struct App {
    pub time_entries: Vec<TimeEntry>,
    pub grouped_entries: Vec<GroupedTimeEntry>,
    pub all_entries: Vec<TimeEntry>,
    pub list_state: ListState,
    pub should_quit: bool,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub show_grouped: bool,
    pub group_by_day: bool,
    pub sort_by_date: bool,
    pub show_rounded: bool,
    pub round_minutes: Option<i64>,
    pub projects: HashMap<i64, Project>,
    pub show_filter_panel: bool,
    pub filter_section: FilterSection,
    pub filter_projects_state: ListState,
    pub filter_tags_state: ListState,
    pub available_tags: Vec<String>,
    pub active_filter: TimeEntryFilter,
    pub clipboard_message: Option<String>,
    pub show_project_selector: bool,
    pub project_selector_state: ListState,
    pub project_search_query: String,
    pub filtered_projects: Vec<Project>,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub show_edit_modal: bool,
    pub edit_input: String,
    pub edit_cursor: usize,
    pub edit_entry_ids: Vec<i64>,
    pub client: Option<Arc<TogglClient>>,
    pub runtime_handle: Option<tokio::runtime::Handle>,
    pub current_user_email: Option<String>,
    pub db: Arc<crate::db::Database>,
    pub project_usage: HashMap<i64, usize>,
    pub project_usage_total: usize,
    pub project_usage_window_start: DateTime<Utc>,
    pub project_sort_method: ProjectSortMethod,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time_entries: Vec<TimeEntry>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        round_minutes: Option<i64>,
        projects: Vec<Project>,
        client: Option<Arc<TogglClient>>,
        runtime_handle: Option<tokio::runtime::Handle>,
        current_user_email: Option<String>,
        db: Arc<crate::db::Database>,
        project_usage: HashMap<i64, usize>,
        project_usage_window_start: DateTime<Utc>,
        project_sort_method: ProjectSortMethod,
        saved_filter: PersistedFilter,
    ) -> Self {
        let projects_map: HashMap<i64, Project> =
            projects.iter().map(|p| (p.id, p.clone())).collect();
        let project_usage_total: usize = project_usage.values().sum();
        let mut filtered_projects = projects.clone();
        sort_projects(&mut filtered_projects, project_sort_method, &project_usage);

        let all_entries = time_entries.clone();

        let mut project_selector_state = ListState::default();
        if !filtered_projects.is_empty() {
            project_selector_state.select(Some(0));
        }

        let available_tags_set: HashSet<String> = all_entries
            .iter()
            .filter_map(|e| e.tags.as_ref())
            .flatten()
            .map(|t| t.to_lowercase())
            .collect();
        let mut available_tags: Vec<String> = available_tags_set.iter().cloned().collect();
        available_tags.sort();

        let mut active_filter = TimeEntryFilter::new();
        for pid in saved_filter.project_ids {
            if projects_map.contains_key(&pid) {
                active_filter.project_ids.insert(pid);
            }
        }
        for tag in saved_filter.tags {
            let lower = tag.to_lowercase();
            if available_tags_set.contains(&lower) {
                active_filter.tags.insert(lower);
            }
        }
        active_filter.billable_only = saved_filter.billable_only;

        let projects_vec: Vec<Project> = projects_map.values().cloned().collect();
        let filtered_entries = active_filter.apply(all_entries.clone(), &projects_vec);

        let mut list_state = ListState::default();
        if !filtered_entries.is_empty() {
            list_state.select(Some(0));
        }

        let mut filter_projects_state = ListState::default();
        if !filtered_projects.is_empty() {
            filter_projects_state.select(Some(0));
        }
        let mut filter_tags_state = ListState::default();
        if !available_tags.is_empty() {
            filter_tags_state.select(Some(0));
        }

        Self {
            time_entries: filtered_entries,
            grouped_entries: Vec::new(),
            all_entries,
            list_state,
            should_quit: false,
            start_date,
            end_date,
            show_grouped: false,
            group_by_day: false,
            sort_by_date: false,
            show_rounded: true,
            round_minutes,
            projects: projects_map,
            show_filter_panel: false,
            filter_section: FilterSection::Billable,
            filter_projects_state,
            filter_tags_state,
            available_tags,
            active_filter,
            clipboard_message: None,
            show_project_selector: false,
            project_selector_state,
            project_search_query: String::new(),
            filtered_projects,
            status_message: None,
            error_message: None,
            show_edit_modal: false,
            edit_input: String::new(),
            edit_cursor: 0,
            edit_entry_ids: Vec::new(),
            client,
            runtime_handle,
            current_user_email,
            db,
            project_usage,
            project_usage_total,
            project_usage_window_start,
            project_sort_method,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key_event(key);
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if self.error_message.is_some() {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.error_message = None;
                }
                _ => {}
            }
            return;
        }

        if self.show_edit_modal {
            match key.code {
                KeyCode::Enter => {
                    self.save_edited_description();
                }
                KeyCode::Esc => {
                    self.show_edit_modal = false;
                    self.edit_input.clear();
                    self.edit_cursor = 0;
                    self.edit_entry_ids.clear();
                }
                KeyCode::Char(c) => {
                    self.edit_insert_char(c);
                }
                KeyCode::Backspace => {
                    self.edit_backspace();
                }
                KeyCode::Delete => {
                    self.edit_delete();
                }
                KeyCode::Left => {
                    self.edit_cursor = self.edit_cursor.saturating_sub(1);
                }
                KeyCode::Right => {
                    let char_count = self.edit_input.chars().count();
                    if self.edit_cursor < char_count {
                        self.edit_cursor += 1;
                    }
                }
                KeyCode::Home => {
                    self.edit_cursor = 0;
                }
                KeyCode::End => {
                    self.edit_cursor = self.edit_input.chars().count();
                }
                _ => {}
            }
            return;
        }

        if self.show_project_selector {
            match key.code {
                KeyCode::Esc | KeyCode::Char('p') => {
                    self.show_project_selector = false;
                    self.project_search_query.clear();
                    self.reset_filtered_projects();
                }
                KeyCode::Enter => {
                    self.assign_project_to_entry();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_project();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous_project();
                }
                KeyCode::PageDown => {
                    self.page_down_project();
                }
                KeyCode::PageUp => {
                    self.page_up_project();
                }
                KeyCode::Home => {
                    self.goto_first_project();
                }
                KeyCode::End => {
                    self.goto_last_project();
                }
                KeyCode::Char('/') => {
                    self.start_project_search();
                }
                KeyCode::Char(c) if !self.project_search_query.is_empty() => {
                    self.project_search_query.push(c);
                    self.filter_projects();
                }
                KeyCode::Backspace if !self.project_search_query.is_empty() => {
                    self.project_search_query.pop();
                    self.filter_projects();
                }
                KeyCode::Char(c) if c.is_alphanumeric() => {
                    self.jump_to_project_by_char(c);
                }
                _ => {}
            }
        } else if self.show_filter_panel {
            match key.code {
                KeyCode::Esc | KeyCode::Char('f') => {
                    self.show_filter_panel = false;
                }
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                    self.filter_section = self.filter_section.next();
                }
                KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                    self.filter_section = self.filter_section.prev();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.filter_section_next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.filter_section_previous();
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.toggle_filter_selection();
                }
                KeyCode::Char('b') => {
                    self.toggle_billable_filter();
                }
                KeyCode::Char('c') => {
                    self.clear_filters();
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_quit = true;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_item();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous_item();
                }
                KeyCode::PageDown => {
                    self.page_down();
                }
                KeyCode::PageUp => {
                    self.page_up();
                }
                KeyCode::Home => {
                    self.goto_first();
                }
                KeyCode::End => {
                    self.goto_last();
                }
                KeyCode::Char('g') => {
                    self.toggle_grouping();
                }
                KeyCode::Char('d') => {
                    self.toggle_day_grouping();
                }
                KeyCode::Char('s') => {
                    self.toggle_sort_by_date();
                }
                KeyCode::Char('r') => {
                    self.toggle_rounding();
                }
                KeyCode::Char('f') => {
                    self.toggle_filter_panel();
                }
                KeyCode::Char('c') if self.active_filter.is_active() => {
                    self.clear_filters();
                    self.status_message = Some("Filters cleared".to_string());
                }
                KeyCode::Char('y') => {
                    self.copy_to_clipboard();
                }
                KeyCode::Char('p') => {
                    self.toggle_project_selector();
                }
                KeyCode::Char('e') => {
                    self.open_edit_modal();
                }
                _ => {}
            }
        }
    }

    fn next_item(&mut self) {
        let len = if self.show_grouped {
            self.grouped_entries.len()
        } else {
            self.time_entries.len()
        };

        if len == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous_item(&mut self) {
        let len = if self.show_grouped {
            self.grouped_entries.len()
        } else {
            self.time_entries.len()
        };

        if len == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_grouping(&mut self) {
        self.show_grouped = !self.show_grouped;
        self.list_state.select(Some(0));
    }

    fn toggle_day_grouping(&mut self) {
        self.group_by_day = !self.group_by_day;
        self.recompute_grouped_entries();
        self.list_state.select(Some(0));
    }

    fn recompute_grouped_entries(&mut self) {
        use crate::processor::{group_by_description, group_by_description_and_day};

        self.grouped_entries = if self.group_by_day {
            group_by_description_and_day(self.time_entries.clone())
        } else {
            group_by_description(self.time_entries.clone())
        };
    }

    fn sort_entries(&mut self) {
        if self.sort_by_date {
            self.time_entries.sort_by_key(|a| a.start);
        }
    }

    fn toggle_rounding(&mut self) {
        self.show_rounded = !self.show_rounded;
    }

    fn toggle_sort_by_date(&mut self) {
        self.sort_by_date = !self.sort_by_date;
        if self.sort_by_date {
            self.time_entries.sort_by_key(|a| a.start);
        } else {
            let projects_vec: Vec<_> = self.projects.values().cloned().collect();
            self.time_entries = self
                .active_filter
                .apply(self.all_entries.clone(), &projects_vec);
        }
        self.recompute_grouped_entries();
        self.list_state.select(Some(0));
    }

    fn toggle_filter_panel(&mut self) {
        self.show_filter_panel = !self.show_filter_panel;
    }

    fn toggle_project_selector(&mut self) {
        self.show_project_selector = !self.show_project_selector;
    }

    fn open_edit_modal(&mut self) {
        if self.show_grouped {
            if let Some(selected_idx) = self.list_state.selected()
                && let Some(grouped_entry) = self.grouped_entries.get(selected_idx)
            {
                self.edit_input = grouped_entry.description.clone().unwrap_or_default();
                self.edit_cursor = self.edit_input.chars().count();
                self.edit_entry_ids = grouped_entry.entries.iter().map(|e| e.id).collect();
                self.show_edit_modal = true;
            }
        } else if let Some(selected_idx) = self.list_state.selected()
            && let Some(entry) = self.time_entries.get(selected_idx)
        {
            self.edit_input = entry.description.clone().unwrap_or_default();
            self.edit_cursor = self.edit_input.chars().count();
            self.edit_entry_ids = vec![entry.id];
            self.show_edit_modal = true;
        }
    }

    fn save_edited_description(&mut self) {
        if self.edit_entry_ids.is_empty() {
            self.error_message = Some("Cannot save: no entry selected".to_string());
            self.show_edit_modal = false;
            return;
        }

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.error_message = Some("API client not available".to_string());
                self.show_edit_modal = false;
                return;
            }
        };

        let handle = match &self.runtime_handle {
            Some(h) => h.clone(),
            None => {
                self.error_message = Some("Runtime handle not available".to_string());
                self.show_edit_modal = false;
                return;
            }
        };

        let db = self.db.clone();
        let entry_ids = self.edit_entry_ids.clone();
        let new_description = self.edit_input.clone();

        let entries_to_update: Vec<(i64, i64)> = self
            .all_entries
            .iter()
            .filter(|e| entry_ids.contains(&e.id))
            .map(|e| (e.workspace_id, e.id))
            .collect();

        if entries_to_update.is_empty() {
            self.error_message = Some("Could not find entries to update".to_string());
            self.show_edit_modal = false;
            return;
        }

        if let Some(rate_limit_info) = client.get_rate_limit_info()
            && let Some(remaining) = rate_limit_info.remaining
        {
            if remaining == 0 {
                self.error_message = Some(format!(
                    "API rate limit exhausted. Please wait {} seconds and try again.",
                    rate_limit_info.resets_in.unwrap_or(60)
                ));
                self.show_edit_modal = false;
                return;
            } else if remaining < 5 {
                self.status_message = Some(format!(
                    "Warning: Only {} API requests remaining",
                    remaining
                ));
            }
        }

        self.show_edit_modal = false;
        self.edit_input.clear();
        self.edit_cursor = 0;
        self.edit_entry_ids.clear();

        tracing::info!(
            "Using bulk API to update description for {} entries",
            entries_to_update.len()
        );

        let workspace_id = entries_to_update[0].0;
        let entry_ids: Vec<i64> = entries_to_update.iter().map(|(_, id)| *id).collect();

        let chunks: Vec<Vec<i64>> = entry_ids.chunks(100).map(|chunk| chunk.to_vec()).collect();

        let mut success_count = 0;
        let mut fail_count = 0;
        let mut error_occurred = false;
        let mut successful_ids: HashSet<i64> = HashSet::new();

        for chunk in chunks {
            tracing::debug!("Processing chunk of {} entries", chunk.len());

            let (tx, rx) = std::sync::mpsc::channel();
            let client_clone = client.clone();
            let chunk_clone = chunk.clone();
            let desc_clone = new_description.clone();

            handle.spawn(async move {
                let result = client_clone
                    .bulk_update_descriptions(workspace_id, &chunk_clone, desc_clone)
                    .await;
                let _ = tx.send(result);
            });

            self.status_message = Some("Updating description...".to_string());

            match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                Ok(Ok(bulk_result)) => {
                    tracing::debug!(
                        "Bulk description update completed: {} succeeded, {} failed",
                        bulk_result.success.len(),
                        bulk_result.failure.len()
                    );

                    for entry_id in &bulk_result.success {
                        successful_ids.insert(*entry_id);

                        if let Some(time_entry) =
                            self.time_entries.iter_mut().find(|e| e.id == *entry_id)
                        {
                            time_entry.description = Some(new_description.clone());
                        }

                        if let Some(all_entry) =
                            self.all_entries.iter_mut().find(|e| e.id == *entry_id)
                        {
                            all_entry.description = Some(new_description.clone());
                        }

                        if let Err(e) =
                            db.update_time_entry_description(*entry_id, new_description.clone())
                        {
                            tracing::error!(
                                "Failed to update description in database for entry {}: {}",
                                entry_id,
                                e
                            );
                        } else {
                            tracing::debug!(
                                "Successfully updated description in database for entry {}",
                                entry_id
                            );
                        }
                    }

                    for failure in &bulk_result.failure {
                        tracing::error!(
                            "Failed to update entry {}: {}",
                            failure.id,
                            failure.message
                        );
                    }

                    success_count += bulk_result.success.len();
                    fail_count += bulk_result.failure.len();
                }
                Ok(Err(e)) => {
                    tracing::error!("API error during bulk description update: {}", e);
                    fail_count += chunk.len();
                    let error_msg = e.to_string();
                    if error_msg.contains("Rate limit") || error_msg.contains("429") {
                        self.error_message = Some(
                            "API rate limit exceeded. Please wait a few minutes and try again."
                                .to_string(),
                        );
                    } else if error_msg.contains("Quota") || error_msg.contains("402") {
                        self.error_message = Some(
                            "API quota exceeded. Please wait for quota reset and try again."
                                .to_string(),
                        );
                    } else {
                        self.error_message = Some(format!("Failed to update description: {}", e));
                    }
                    error_occurred = true;
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    tracing::warn!("API request timed out (likely due to rate limiting)");
                    fail_count += chunk.len();
                    self.error_message = Some(
                        "Update timed out (API rate limit hit). The operation may still complete in the background. Please wait and refresh.".to_string(),
                    );
                    error_occurred = true;
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    tracing::error!("Channel disconnected while waiting for API result");
                    fail_count += chunk.len();
                    self.error_message =
                        Some("Lost connection to API task. Please try again.".to_string());
                    error_occurred = true;
                    break;
                }
            }
        }

        if !error_occurred {
            if fail_count == 0 {
                self.status_message = Some(format!(
                    "Successfully updated description for {} entr{}",
                    success_count,
                    if success_count == 1 { "y" } else { "ies" }
                ));
            } else {
                self.status_message = Some(format!(
                    "Updated description for {}/{} entries ({} failed)",
                    success_count,
                    success_count + fail_count,
                    fail_count
                ));
            }

            for entry in self.grouped_entries.iter_mut() {
                if entry.entries.iter().any(|e| successful_ids.contains(&e.id)) {
                    entry.description = Some(new_description.clone());
                }
            }
        }
    }

    fn toggle_billable_filter(&mut self) {
        self.active_filter.billable_only = !self.active_filter.billable_only;
        self.apply_filters();
    }

    fn clear_filters(&mut self) {
        self.active_filter = TimeEntryFilter::new();
        self.apply_filters();
    }

    fn filter_section_len(&self) -> usize {
        match self.filter_section {
            FilterSection::Billable => 0,
            FilterSection::Projects => self.filtered_projects.len(),
            FilterSection::Tags => self.available_tags.len(),
        }
    }

    fn filter_section_state(&mut self) -> Option<&mut ListState> {
        match self.filter_section {
            FilterSection::Billable => None,
            FilterSection::Projects => Some(&mut self.filter_projects_state),
            FilterSection::Tags => Some(&mut self.filter_tags_state),
        }
    }

    fn filter_section_next(&mut self) {
        let len = self.filter_section_len();
        if len == 0 {
            return;
        }
        if let Some(state) = self.filter_section_state() {
            let i = state.selected().map(|i| (i + 1) % len).unwrap_or(0);
            state.select(Some(i));
        }
    }

    fn filter_section_previous(&mut self) {
        let len = self.filter_section_len();
        if len == 0 {
            return;
        }
        if let Some(state) = self.filter_section_state() {
            let i = state
                .selected()
                .map(|i| if i == 0 { len - 1 } else { i - 1 })
                .unwrap_or(0);
            state.select(Some(i));
        }
    }

    fn toggle_filter_selection(&mut self) {
        match self.filter_section {
            FilterSection::Billable => {
                self.toggle_billable_filter();
            }
            FilterSection::Projects => {
                if let Some(idx) = self.filter_projects_state.selected()
                    && let Some(project) = self.filtered_projects.get(idx)
                {
                    let pid = project.id;
                    if self.active_filter.project_ids.contains(&pid) {
                        self.active_filter.project_ids.remove(&pid);
                    } else {
                        self.active_filter.project_ids.insert(pid);
                    }
                    self.apply_filters();
                }
            }
            FilterSection::Tags => {
                if let Some(idx) = self.filter_tags_state.selected()
                    && let Some(tag) = self.available_tags.get(idx).cloned()
                {
                    if self.active_filter.tags.contains(&tag) {
                        self.active_filter.tags.remove(&tag);
                    } else {
                        self.active_filter.tags.insert(tag);
                    }
                    self.apply_filters();
                }
            }
        }
    }

    pub fn persisted_filter(&self) -> PersistedFilter {
        let mut project_ids: Vec<i64> = self.active_filter.project_ids.iter().copied().collect();
        project_ids.sort();
        let mut tags: Vec<String> = self.active_filter.tags.iter().cloned().collect();
        tags.sort();
        PersistedFilter {
            project_ids,
            tags,
            billable_only: self.active_filter.billable_only,
        }
    }

    fn apply_filters(&mut self) {
        let projects_vec: Vec<_> = self.projects.values().cloned().collect();
        self.time_entries = self
            .active_filter
            .apply(self.all_entries.clone(), &projects_vec);
        self.sort_entries();
        self.recompute_grouped_entries();
        self.list_state.select(if self.time_entries.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    fn page_down(&mut self) {
        let len = if self.show_grouped {
            self.grouped_entries.len()
        } else {
            self.time_entries.len()
        };

        if len == 0 {
            return;
        }

        let page_size = PAGE_SIZE;
        let i = match self.list_state.selected() {
            Some(i) => {
                let new_pos = i + page_size;
                if new_pos >= len { len - 1 } else { new_pos }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn page_up(&mut self) {
        let len = if self.show_grouped {
            self.grouped_entries.len()
        } else {
            self.time_entries.len()
        };

        if len == 0 {
            return;
        }

        let page_size = PAGE_SIZE;
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(page_size),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn goto_first(&mut self) {
        let len = if self.show_grouped {
            self.grouped_entries.len()
        } else {
            self.time_entries.len()
        };

        if len > 0 {
            self.list_state.select(Some(0));
        }
    }

    fn goto_last(&mut self) {
        let len = if self.show_grouped {
            self.grouped_entries.len()
        } else {
            self.time_entries.len()
        };

        if len > 0 {
            self.list_state.select(Some(len - 1));
        }
    }

    fn copy_to_clipboard(&mut self) {
        let description = if self.show_grouped {
            self.list_state.selected().and_then(|i| {
                self.grouped_entries
                    .get(i)
                    .and_then(|entry| entry.description.clone())
            })
        } else {
            self.list_state.selected().and_then(|i| {
                self.time_entries
                    .get(i)
                    .and_then(|entry| entry.description.clone())
            })
        };

        if let Some(desc) = description {
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if clipboard.set_text(&desc).is_ok() {
                        self.clipboard_message = Some(format!("Copied: {}", desc));
                    } else {
                        self.clipboard_message = Some("Failed to copy to clipboard".to_string());
                    }
                }
                Err(_) => {
                    self.clipboard_message = Some("Clipboard unavailable".to_string());
                }
            }
        } else {
            self.clipboard_message = Some("No description to copy".to_string());
        }
    }

    fn next_project(&mut self) {
        let len = self.filtered_projects.len();
        if len == 0 {
            return;
        }

        let i = match self.project_selector_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.project_selector_state.select(Some(i));
    }

    fn previous_project(&mut self) {
        let len = self.filtered_projects.len();
        if len == 0 {
            return;
        }

        let i = match self.project_selector_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.project_selector_state.select(Some(i));
    }

    fn page_down_project(&mut self) {
        let len = self.filtered_projects.len();
        if len == 0 {
            return;
        }

        let page_size = PAGE_SIZE;
        let i = match self.project_selector_state.selected() {
            Some(i) => {
                let new_pos = i + page_size;
                if new_pos >= len { len - 1 } else { new_pos }
            }
            None => 0,
        };
        self.project_selector_state.select(Some(i));
    }

    fn page_up_project(&mut self) {
        let len = self.filtered_projects.len();
        if len == 0 {
            return;
        }

        let page_size = PAGE_SIZE;
        let i = match self.project_selector_state.selected() {
            Some(i) => i.saturating_sub(page_size),
            None => 0,
        };
        self.project_selector_state.select(Some(i));
    }

    fn goto_first_project(&mut self) {
        if !self.filtered_projects.is_empty() {
            self.project_selector_state.select(Some(0));
        }
    }

    fn goto_last_project(&mut self) {
        let len = self.filtered_projects.len();
        if len > 0 {
            self.project_selector_state.select(Some(len - 1));
        }
    }

    fn jump_to_project_by_char(&mut self, c: char) {
        let target = c.to_ascii_lowercase();
        let matches_target =
            |p: &Project| p.name.chars().next().map(|c| c.to_ascii_lowercase()) == Some(target);

        let start = self
            .project_selector_state
            .selected()
            .map(|i| i + 1)
            .unwrap_or(0);
        let len = self.filtered_projects.len();
        if len == 0 {
            return;
        }

        let next = (0..len)
            .map(|offset| (start + offset) % len)
            .find(|&i| matches_target(&self.filtered_projects[i]));

        if let Some(idx) = next {
            self.project_selector_state.select(Some(idx));
        }
    }

    fn start_project_search(&mut self) {
        self.project_search_query.push('/');
    }

    fn filter_projects(&mut self) {
        let query = self
            .project_search_query
            .trim_start_matches('/')
            .to_lowercase();

        if query.is_empty() {
            self.reset_filtered_projects();
            return;
        }

        let all_projects: Vec<_> = self.projects.values().cloned().collect();
        self.filtered_projects = all_projects
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&query))
            .collect();

        sort_projects(
            &mut self.filtered_projects,
            self.project_sort_method,
            &self.project_usage,
        );

        if !self.filtered_projects.is_empty() {
            self.project_selector_state.select(Some(0));
        } else {
            self.project_selector_state.select(None);
        }
    }

    fn reset_filtered_projects(&mut self) {
        self.filtered_projects = self.projects.values().cloned().collect();
        sort_projects(
            &mut self.filtered_projects,
            self.project_sort_method,
            &self.project_usage,
        );

        if !self.filtered_projects.is_empty() {
            self.project_selector_state.select(Some(0));
        }
    }

    fn adjust_usage_for_reassign(
        &mut self,
        start: DateTime<Utc>,
        old_pid: Option<i64>,
        new_pid: Option<i64>,
    ) {
        if old_pid == new_pid {
            return;
        }
        if start < self.project_usage_window_start || start > Utc::now() {
            return;
        }
        if let Some(old) = old_pid
            && let Some(count) = self.project_usage.get_mut(&old)
        {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.project_usage.remove(&old);
            }
            self.project_usage_total = self.project_usage_total.saturating_sub(1);
        }
        if let Some(new) = new_pid {
            *self.project_usage.entry(new).or_insert(0) += 1;
            self.project_usage_total += 1;
        }
    }

    fn assign_project_to_entry(&mut self) {
        tracing::info!("assign_project_to_entry called");

        let selected_project_idx = match self.project_selector_state.selected() {
            Some(idx) => {
                tracing::debug!("Selected project index: {}", idx);
                idx
            }
            None => {
                tracing::warn!("No project selected");
                self.status_message = Some("No project selected".to_string());
                return;
            }
        };

        let selected_project = match self.filtered_projects.get(selected_project_idx) {
            Some(project) => {
                tracing::debug!("Selected project: {} (id: {})", project.name, project.id);
                project
            }
            None => {
                tracing::error!("Invalid project selection index: {}", selected_project_idx);
                self.status_message = Some("Invalid project selection".to_string());
                return;
            }
        };

        let project_id = selected_project.id;
        let project_name = selected_project.name.clone();

        let selected_entry_idx = match self.list_state.selected() {
            Some(idx) => {
                tracing::debug!("Selected entry index: {}", idx);
                idx
            }
            None => {
                tracing::warn!("No time entry selected");
                self.status_message = Some("No time entry selected".to_string());
                return;
            }
        };

        let client = match &self.client {
            Some(c) => {
                tracing::debug!("API client available");
                c.clone()
            }
            None => {
                tracing::error!("API client not available");
                self.status_message = Some("API client not available".to_string());
                return;
            }
        };

        let handle = match &self.runtime_handle {
            Some(h) => {
                tracing::debug!("Runtime handle available");
                h.clone()
            }
            None => {
                tracing::error!("Runtime handle not available");
                self.status_message = Some("Runtime not available".to_string());
                return;
            }
        };

        if self.show_grouped {
            tracing::info!("Batch assignment for grouped entry");
            let grouped_entry = match self.grouped_entries.get(selected_entry_idx) {
                Some(e) => {
                    tracing::debug!(
                        "Grouped entry contains {} individual entries",
                        e.entries.len()
                    );
                    e
                }
                None => {
                    tracing::error!("Invalid grouped entry selection");
                    self.status_message = Some("Invalid entry selection".to_string());
                    return;
                }
            };

            if let Some(rate_limit_info) = client.get_rate_limit_info()
                && let Some(remaining) = rate_limit_info.remaining
            {
                if remaining == 0 {
                    self.error_message = Some(format!(
                        "API rate limit exhausted. Please wait {} seconds and try again.",
                        rate_limit_info.resets_in.unwrap_or(60)
                    ));
                    self.show_project_selector = false;
                    self.project_search_query.clear();
                    self.reset_filtered_projects();
                    return;
                } else if remaining < 5 {
                    self.status_message = Some(format!(
                        "Warning: Only {} API requests remaining",
                        remaining
                    ));
                }
            }

            let total_entries = grouped_entry.entries.len();
            let entry_ids: Vec<i64> = grouped_entry.entries.iter().map(|e| e.id).collect();
            let workspace_id = grouped_entry.entries[0].workspace_id;

            tracing::info!(
                "Using bulk API to assign project {} to {} entries in workspace {}",
                project_id,
                entry_ids.len(),
                workspace_id
            );

            let chunks: Vec<Vec<i64>> = entry_ids.chunks(100).map(|chunk| chunk.to_vec()).collect();

            let mut success_count = 0;
            let mut fail_count = 0;

            for chunk in chunks {
                tracing::debug!("Processing chunk of {} entries", chunk.len());

                let (tx, rx) = std::sync::mpsc::channel();
                let client_clone = client.clone();
                let chunk_clone = chunk.clone();

                handle.spawn(async move {
                    let result = client_clone
                        .bulk_assign_project(workspace_id, &chunk_clone, Some(project_id))
                        .await;
                    let _ = tx.send(result);
                });

                self.status_message = Some("Assigning project...".to_string());

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(bulk_result)) => {
                        tracing::debug!(
                            "Bulk update completed: {} succeeded, {} failed",
                            bulk_result.success.len(),
                            bulk_result.failure.len()
                        );

                        for entry_id in &bulk_result.success {
                            let prior = self
                                .all_entries
                                .iter()
                                .find(|e| e.id == *entry_id)
                                .map(|e| (e.start, e.project_id));

                            if let Some(time_entry) =
                                self.time_entries.iter_mut().find(|e| e.id == *entry_id)
                            {
                                time_entry.project_id = Some(project_id);
                            }

                            if let Some(all_entry) =
                                self.all_entries.iter_mut().find(|e| e.id == *entry_id)
                            {
                                all_entry.project_id = Some(project_id);
                            }

                            if let Some((start, old_pid)) = prior {
                                self.adjust_usage_for_reassign(start, old_pid, Some(project_id));
                            }

                            if let Err(e) = self
                                .db
                                .update_time_entry_project(*entry_id, Some(project_id))
                            {
                                tracing::error!(
                                    "Failed to update project in database for entry {}: {}",
                                    entry_id,
                                    e
                                );
                            } else {
                                tracing::debug!(
                                    "Successfully updated project in database for entry {}",
                                    entry_id
                                );
                            }
                        }

                        for failure in &bulk_result.failure {
                            tracing::error!(
                                "Failed to update entry {}: {}",
                                failure.id,
                                failure.message
                            );
                        }

                        success_count += bulk_result.success.len();
                        fail_count += bulk_result.failure.len();
                    }
                    Ok(Err(e)) => {
                        tracing::error!("API error during bulk assignment: {}", e);
                        fail_count += chunk.len();
                        let error_msg = e.to_string();
                        if error_msg.contains("Rate limit") || error_msg.contains("429") {
                            self.error_message = Some(
                                "API rate limit exceeded. Please wait a few minutes and try again."
                                    .to_string(),
                            );
                        } else if error_msg.contains("Quota") || error_msg.contains("402") {
                            self.error_message = Some(
                                "API quota exceeded. Please wait for quota reset and try again."
                                    .to_string(),
                            );
                        } else {
                            self.error_message = Some(format!("Failed to assign project: {}", e));
                        }
                        break;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        tracing::warn!(
                            "Project assignment timed out (likely due to rate limiting)"
                        );
                        fail_count += chunk.len();
                        self.error_message = Some(
                            "Assignment timed out (API rate limit hit). The operation may still complete in the background. Please wait and refresh.".to_string(),
                        );
                        break;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        tracing::error!("Channel disconnected during project assignment");
                        fail_count += chunk.len();
                        self.error_message =
                            Some("Lost connection to API task. Please try again.".to_string());
                        break;
                    }
                }
            }

            tracing::info!(
                "Batch assignment complete: {} succeeded, {} failed out of {}",
                success_count,
                fail_count,
                total_entries
            );

            if fail_count == 0 {
                self.status_message = Some(format!(
                    "Assigned {} to {} entries",
                    project_name, success_count
                ));
            } else {
                self.status_message = Some(format!(
                    "Assigned {} to {}/{} entries ({} failed)",
                    project_name, success_count, total_entries, fail_count
                ));
            }

            self.recompute_grouped_entries();
            self.show_project_selector = false;
            self.project_search_query.clear();
            self.reset_filtered_projects();
        } else {
            tracing::info!("Single entry assignment");
            let entry = match self.time_entries.get(selected_entry_idx) {
                Some(e) => {
                    tracing::debug!(
                        "Assigning project {} to entry {} in workspace {}",
                        project_id,
                        e.id,
                        e.workspace_id
                    );
                    e
                }
                None => {
                    tracing::error!("Invalid entry selection");
                    self.status_message = Some("Invalid entry selection".to_string());
                    return;
                }
            };

            let entry_id = entry.id;
            let workspace_id = entry.workspace_id;

            tracing::debug!("Spawning async task for single entry {}", entry_id);

            let (tx, rx) = std::sync::mpsc::channel();
            let client_clone = client.clone();

            handle.spawn(async move {
                let result = client_clone
                    .update_time_entry_project(workspace_id, entry_id, Some(project_id))
                    .await;
                let _ = tx.send(result);
            });

            match rx.recv() {
                Ok(Ok(_updated_entry)) => {
                    tracing::info!("Successfully assigned project to entry {}", entry_id);

                    let prior = self
                        .all_entries
                        .iter()
                        .find(|e| e.id == entry_id)
                        .map(|e| (e.start, e.project_id));

                    if let Some(entry_mut) = self.time_entries.get_mut(selected_entry_idx) {
                        entry_mut.project_id = Some(project_id);
                    }

                    if let Some(all_entry) = self.all_entries.iter_mut().find(|e| e.id == entry_id)
                    {
                        all_entry.project_id = Some(project_id);
                    }

                    if let Some((start, old_pid)) = prior {
                        self.adjust_usage_for_reassign(start, old_pid, Some(project_id));
                    }

                    if let Err(e) = self
                        .db
                        .update_time_entry_project(entry_id, Some(project_id))
                    {
                        tracing::error!(
                            "Failed to update project in database for entry {}: {}",
                            entry_id,
                            e
                        );
                        self.status_message = Some(format!(
                            "Assigned project: {}, but failed to save to database: {}",
                            project_name, e
                        ));
                    } else {
                        tracing::debug!(
                            "Successfully updated project in database for entry {}",
                            entry_id
                        );
                        self.status_message = Some(format!("Assigned project: {}", project_name));
                    }

                    self.show_project_selector = false;
                    self.project_search_query.clear();
                    self.reset_filtered_projects();
                }
                Ok(Err(e)) => {
                    tracing::error!("API error: {}", e);
                    self.error_message = Some(format!("Failed to assign project: {}", e));
                }
                Err(e) => {
                    tracing::error!("Channel error while waiting for API result: {}", e);
                    self.error_message = Some(format!("Error communicating with API task: {}", e));
                }
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        if self.show_project_selector {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(12),
                    Constraint::Length(4),
                ])
                .split(f.area());

            self.render_header(f, chunks[0]);
            self.render_list(f, chunks[1]);
            self.render_project_selector_panel(f, chunks[2]);
            self.render_footer(f, chunks[3]);
        } else if self.show_filter_panel {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(14),
                    Constraint::Length(4),
                ])
                .split(f.area());

            self.render_header(f, chunks[0]);
            self.render_list(f, chunks[1]);
            self.render_filter_panel(f, chunks[2]);
            self.render_footer(f, chunks[3]);
        } else {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(4),
                ])
                .split(f.area());

            self.render_header(f, chunks[0]);
            self.render_list(f, chunks[1]);
            self.render_footer(f, chunks[2]);
        }

        if self.error_message.is_some() {
            self.render_error_popup(f);
        }

        if self.show_edit_modal {
            self.render_edit_modal(f);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = if let Some(ref email) = self.current_user_email {
            format!(
                "Toggl TimeGuru - {} to {} [{}]",
                self.start_date.format("%Y-%m-%d"),
                self.end_date.format("%Y-%m-%d"),
                email
            )
        } else {
            format!(
                "Toggl TimeGuru - {} to {}",
                self.start_date.format("%Y-%m-%d"),
                self.end_date.format("%Y-%m-%d")
            )
        };

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(header, area);
    }

    fn parse_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6
            && let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            )
        {
            return Color::Rgb(r, g, b);
        }
        Color::White
    }

    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = if self.show_grouped {
            self.grouped_entries
                .iter()
                .map(|entry| {
                    let desc = entry
                        .description
                        .clone()
                        .unwrap_or_else(|| "(No description)".to_string());
                    let hours = if let Some(round_to_minutes) = self.round_minutes
                        && self.show_rounded
                    {
                        entry.rounded_hours(round_to_minutes)
                    } else {
                        entry.total_hours()
                    };

                    let mut spans = vec![];

                    if self.group_by_day
                        && let Some(date) = entry.date
                    {
                        spans.push(Span::styled(
                            date.format("%Y-%m-%d").to_string(),
                            Style::default().fg(Color::Yellow),
                        ));
                        spans.push(Span::raw(" - "));
                    }

                    spans.push(Span::styled(
                        format!("{:.2}h", hours),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::raw(" - "));

                    if let Some(project_id) = entry.project_id
                        && let Some(project) = self.projects.get(&project_id)
                    {
                        let color = Self::parse_color(&project.color);
                        spans.push(Span::styled(
                            format!("[{}] ", project.name),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ));
                    }

                    spans.push(Span::raw(desc));
                    spans.push(Span::styled(
                        format!(" ({} entries)", entry.entries.len()),
                        Style::default().fg(Color::DarkGray),
                    ));

                    let content = Line::from(spans);
                    ListItem::new(content)
                })
                .collect()
        } else {
            self.time_entries
                .iter()
                .map(|entry| {
                    let desc = entry
                        .description
                        .clone()
                        .unwrap_or_else(|| "(No description)".to_string());

                    let duration_hours = if let Some(round_to_minutes) = self.round_minutes
                        && self.show_rounded
                    {
                        let seconds_per_round = round_to_minutes * 60;
                        let rounded_duration = ((entry.duration as f64 / seconds_per_round as f64)
                            .ceil() as i64)
                            * seconds_per_round;
                        rounded_duration as f64 / 3600.0
                    } else {
                        entry.duration as f64 / 3600.0
                    };

                    let mut spans = vec![
                        Span::styled(
                            entry.start.format("%Y-%m-%d %H:%M").to_string(),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::raw(" - "),
                        Span::styled(
                            format!("{:.2}h", duration_hours),
                            Style::default().fg(Color::Green),
                        ),
                        Span::raw(" - "),
                    ];

                    if let Some(project_id) = entry.project_id
                        && let Some(project) = self.projects.get(&project_id)
                    {
                        let color = Self::parse_color(&project.color);
                        spans.push(Span::styled(
                            format!("[{}] ", project.name),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ));
                    }

                    spans.push(Span::raw(desc));

                    let content = Line::from(spans);
                    ListItem::new(content)
                })
                .collect()
        };

        let title = if self.show_grouped {
            "Time Entries (Grouped)"
        } else {
            "Time Entries"
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_filter_panel(&mut self, f: &mut Frame, area: Rect) {
        let title = format!("Filters — {} active", self.active_filter.active_count());
        let outer = Block::default().borders(Borders::ALL).title(title);
        let inner = outer.inner(area);
        f.render_widget(outer, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let header_spans: Vec<Span> = [
            FilterSection::Billable,
            FilterSection::Projects,
            FilterSection::Tags,
        ]
        .iter()
        .enumerate()
        .flat_map(|(i, section)| {
            let active = *section == self.filter_section;
            let count_hint = match section {
                FilterSection::Billable => {
                    if self.active_filter.billable_only {
                        " (on)"
                    } else {
                        ""
                    }
                }
                FilterSection::Projects => {
                    if self.active_filter.project_ids.is_empty() {
                        ""
                    } else {
                        " ●"
                    }
                }
                FilterSection::Tags => {
                    if self.active_filter.tags.is_empty() {
                        ""
                    } else {
                        " ●"
                    }
                }
            };
            let label = format!("[{}{}]", section.label(), count_hint);
            let style = if active {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let mut v = vec![Span::styled(label, style)];
            if i < 2 {
                v.push(Span::raw(" "));
            }
            v
        })
        .collect();

        let header = Paragraph::new(Line::from(header_spans));
        f.render_widget(header, rows[0]);

        let help_line = Line::from(vec![Span::styled(
            "Tab/←→: Section  │  ↑↓/jk: Move  │  Enter/Space: Toggle  │  b: Billable  │  c: Clear  │  f/Esc: Close",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]);
        f.render_widget(Paragraph::new(help_line), rows[2]);

        match self.filter_section {
            FilterSection::Billable => {
                let (billable_label, billable_style) = if self.active_filter.billable_only {
                    (
                        "Billable only: ON",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ("Billable only: OFF", Style::default().fg(Color::Gray))
                };
                let body = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(Span::styled(billable_label, billable_style)),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press Enter/Space (or 'b') to toggle.",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]);
                f.render_widget(body, rows[1]);
            }
            FilterSection::Projects => {
                let items: Vec<ListItem> = self
                    .filtered_projects
                    .iter()
                    .map(|p| {
                        let selected = self.active_filter.project_ids.contains(&p.id);
                        let mark = if selected { "[x]" } else { "[ ]" };
                        let color = Self::parse_color(&p.color);
                        ListItem::new(Line::from(vec![
                            Span::raw(mark),
                            Span::raw(" "),
                            Span::styled(
                                p.name.clone(),
                                Style::default().fg(color).add_modifier(Modifier::BOLD),
                            ),
                        ]))
                    })
                    .collect();
                let list = List::new(items)
                    .highlight_style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, rows[1], &mut self.filter_projects_state);
            }
            FilterSection::Tags => {
                if self.available_tags.is_empty() {
                    let body = Paragraph::new(Line::from(Span::styled(
                        "No tags found in the loaded entries.",
                        Style::default().fg(Color::DarkGray),
                    )));
                    f.render_widget(body, rows[1]);
                } else {
                    let items: Vec<ListItem> = self
                        .available_tags
                        .iter()
                        .map(|t| {
                            let selected = self.active_filter.tags.contains(t);
                            let mark = if selected { "[x]" } else { "[ ]" };
                            ListItem::new(Line::from(vec![
                                Span::raw(mark),
                                Span::raw(" "),
                                Span::styled(t.clone(), Style::default().fg(Color::Cyan)),
                            ]))
                        })
                        .collect();
                    let list = List::new(items)
                        .highlight_style(
                            Style::default()
                                .bg(Color::DarkGray)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol("> ");
                    f.render_stateful_widget(list, rows[1], &mut self.filter_tags_state);
                }
            }
        }
    }

    fn render_project_selector_panel(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let project_items: Vec<ListItem> = self
            .filtered_projects
            .iter()
            .map(|project| {
                let color = Self::parse_color(&project.color);
                let mut spans = vec![
                    Span::styled(
                        format!("[{}]", project.name),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        if project.active { "Active" } else { "Archived" },
                        if project.active {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                ];

                let count = self.project_usage.get(&project.id).copied().unwrap_or(0);
                if count > 0 {
                    let pct = if self.project_usage_total > 0 {
                        (count as f64 / self.project_usage_total as f64) * 100.0
                    } else {
                        0.0
                    };
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        format!("· {} entries ({:.0}%)", count, pct),
                        Style::default().fg(Color::Gray),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let sort_label = match self.project_sort_method {
            ProjectSortMethod::Name => "name",
            ProjectSortMethod::Usage => "usage (30d)",
        };
        let project_list = List::new(project_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Select Project to Assign — sorted by {sort_label}")),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(project_list, chunks[0], &mut self.project_selector_state);

        let mut help_spans = vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow)),
            Span::raw(
                "↑↓/jk: Navigate  │  0-9/a-z: Jump  │  /: Search  │  Enter: Select  │  p/Esc: Cancel",
            ),
        ];

        if !self.project_search_query.is_empty() {
            help_spans.push(Span::raw("  │  "));
            help_spans.push(Span::styled("Search: ", Style::default().fg(Color::Cyan)));
            help_spans.push(Span::styled(
                &self.project_search_query,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let help_text = Line::from(help_spans);

        let help_para = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(help_para, chunks[1]);
    }

    fn rate_limit_footer_text(&self) -> Option<String> {
        let info = self.client.as_ref()?.get_rate_limit_info()?;
        let remaining = info.remaining?;
        match info.resets_in {
            Some(resets_in) => Some(format!("{remaining} req left, resets in {resets_in}s")),
            None => Some(format!("{remaining} req left")),
        }
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let grouping_status = if self.show_grouped { "ON" } else { "OFF" };
        let day_grouping_status = if self.group_by_day { "ON" } else { "OFF" };
        let sort_status = if self.sort_by_date { "ON" } else { "OFF" };
        let rounding_status = if self.show_rounded { "ON" } else { "OFF" };
        let rate_limit_indicator = self.rate_limit_footer_text();
        let filter_indicator = if self.active_filter.is_active() {
            let mut parts: Vec<String> = Vec::new();
            if self.active_filter.billable_only {
                parts.push("billable".to_string());
            }
            if !self.active_filter.project_ids.is_empty() {
                parts.push(format!(
                    "{} project(s)",
                    self.active_filter.project_ids.len()
                ));
            }
            if !self.active_filter.tags.is_empty() {
                parts.push(format!("{} tag(s)", self.active_filter.tags.len()));
            }
            format!(" [FILTERED: {}]", parts.join(", "))
        } else {
            String::new()
        };

        let len = if self.show_grouped {
            self.grouped_entries.len()
        } else {
            self.time_entries.len()
        };

        let selected_pos = self.list_state.selected().map(|i| i + 1).unwrap_or(0);

        let mut footer_lines = vec![
            Line::from(vec![
                Span::styled("Navigation: ", Style::default().fg(Color::Yellow)),
                Span::raw("↑↓/jk "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("PgUp/PgDn "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("Home/End "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Toggles: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("g:Group({}) ", grouping_status)),
                Span::raw(format!("d:Day({}) ", day_grouping_status)),
                Span::raw(format!("s:Sort({}) ", sort_status)),
                Span::raw(format!("r:Round({}) ", rounding_status)),
                Span::raw("f:Filter "),
                Span::raw("c:ClearFilters "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("p:Project "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("y:Copy "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("e:Edit "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("q/Esc:Quit"),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Entry {}/{}", selected_pos, len)),
                Span::styled(
                    filter_indicator.clone(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("Date Range: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!(
                    "{} to {}",
                    self.start_date.format("%Y-%m-%d"),
                    self.end_date.format("%Y-%m-%d")
                )),
            ]),
        ];

        if let Some(rate_limit) = rate_limit_indicator {
            footer_lines[1]
                .spans
                .push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
            footer_lines[1]
                .spans
                .push(Span::styled("API: ", Style::default().fg(Color::Cyan)));
            footer_lines[1].spans.push(Span::raw(rate_limit));
        }

        if let Some(ref msg) = self.clipboard_message {
            footer_lines.push(Line::from(vec![
                Span::styled("Clipboard: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    msg,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        if let Some(ref msg) = self.status_message {
            footer_lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    msg,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        let footer = Paragraph::new(footer_lines)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Help"));

        f.render_widget(footer, area);
    }

    fn render_error_popup(&self, f: &mut Frame) {
        if let Some(ref error_msg) = self.error_message {
            let area = f.area();
            let popup_width = area.width.saturating_sub(POPUP_MARGIN).min(POPUP_MAX_WIDTH);
            let popup_height = area
                .height
                .saturating_sub(POPUP_MARGIN)
                .min(POPUP_MAX_HEIGHT);

            let popup_area = Rect {
                x: (area.width.saturating_sub(popup_width)) / 2,
                y: (area.height.saturating_sub(popup_height)) / 2,
                width: popup_width,
                height: popup_height,
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::Black))
                .title("Error")
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

            let inner_area = block.inner(popup_area);

            let text = vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    error_msg.as_str(),
                    Style::default().fg(Color::White),
                )]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press Enter or Esc to close",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )]),
            ];

            f.render_widget(Clear, popup_area);
            f.render_widget(block, popup_area);

            let paragraph = Paragraph::new(text)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .style(Style::default().bg(Color::Black));

            f.render_widget(paragraph, inner_area);
        }
    }

    fn edit_char_byte_index(&self, char_index: usize) -> usize {
        self.edit_input
            .char_indices()
            .nth(char_index)
            .map(|(byte, _)| byte)
            .unwrap_or(self.edit_input.len())
    }

    fn edit_insert_char(&mut self, c: char) {
        let byte_pos = self.edit_char_byte_index(self.edit_cursor);
        self.edit_input.insert(byte_pos, c);
        self.edit_cursor += 1;
    }

    fn edit_backspace(&mut self) {
        if self.edit_cursor == 0 {
            return;
        }
        let start = self.edit_char_byte_index(self.edit_cursor - 1);
        let end = self.edit_char_byte_index(self.edit_cursor);
        self.edit_input.replace_range(start..end, "");
        self.edit_cursor -= 1;
    }

    fn edit_delete(&mut self) {
        let char_count = self.edit_input.chars().count();
        if self.edit_cursor >= char_count {
            return;
        }
        let start = self.edit_char_byte_index(self.edit_cursor);
        let end = self.edit_char_byte_index(self.edit_cursor + 1);
        self.edit_input.replace_range(start..end, "");
    }

    fn render_edit_modal(&self, f: &mut Frame) {
        if !self.show_edit_modal {
            return;
        }

        let area = f.area();
        let popup_width = area.width.saturating_sub(POPUP_MARGIN).min(60);
        let popup_height = 7;

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black))
            .title("Edit Description")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        let inner_area = block.inner(popup_area);

        let text_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let cursor_on_char_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);
        let cursor_at_end_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::SLOW_BLINK);

        let chars: Vec<char> = self.edit_input.chars().collect();
        let cursor_pos = self.edit_cursor.min(chars.len());
        let before: String = chars[..cursor_pos].iter().collect();
        let input_line: Line = if cursor_pos < chars.len() {
            let cursor_char = chars[cursor_pos].to_string();
            let after: String = chars[cursor_pos + 1..].iter().collect();
            Line::from(vec![
                Span::styled(before, text_style),
                Span::styled(cursor_char, cursor_on_char_style),
                Span::styled(after, text_style),
            ])
        } else {
            Line::from(vec![
                Span::styled(before, text_style),
                Span::styled(" ", cursor_at_end_style),
            ])
        };

        let text = vec![
            Line::from(""),
            input_line,
            Line::from(""),
            Line::from(vec![Span::styled(
                "Enter: Save  │  ←/→: Move  │  Del/Backspace: Erase  │  Esc: Cancel",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )]),
        ];

        f.render_widget(Clear, popup_area);
        f.render_widget(block, popup_area);

        let paragraph = Paragraph::new(text)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(Style::default().bg(Color::Black));

        f.render_widget(paragraph, inner_area);
    }
}
