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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::processor::TimeEntryFilter;
use crate::toggl::TogglClient;
use crate::toggl::models::{GroupedTimeEntry, Project, TimeEntry};
use std::collections::HashMap;
use std::sync::Arc;

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
    pub active_filter: TimeEntryFilter,
    pub clipboard_message: Option<String>,
    pub show_project_selector: bool,
    #[allow(dead_code)]
    pub project_selector_state: ListState,
    #[allow(dead_code)]
    pub project_search_query: String,
    #[allow(dead_code)]
    pub filtered_projects: Vec<Project>,
    #[allow(dead_code)]
    pub status_message: Option<String>,
    pub client: Option<Arc<TogglClient>>,
    pub runtime_handle: Option<tokio::runtime::Handle>,
}

impl App {
    pub fn new(
        time_entries: Vec<TimeEntry>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        round_minutes: Option<i64>,
        projects: Vec<Project>,
        client: Option<Arc<TogglClient>>,
        runtime_handle: Option<tokio::runtime::Handle>,
    ) -> Self {
        let mut list_state = ListState::default();
        if !time_entries.is_empty() {
            list_state.select(Some(0));
        }

        let projects_map: HashMap<i64, Project> =
            projects.iter().map(|p| (p.id, p.clone())).collect();
        let mut filtered_projects = projects.clone();
        filtered_projects.sort_by(|a, b| a.name.cmp(&b.name));

        let all_entries = time_entries.clone();

        let mut project_selector_state = ListState::default();
        if !filtered_projects.is_empty() {
            project_selector_state.select(Some(0));
        }

        Self {
            time_entries,
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
            active_filter: TimeEntryFilter::new(),
            clipboard_message: None,
            show_project_selector: false,
            project_selector_state,
            project_search_query: String::new(),
            filtered_projects,
            status_message: None,
            client,
            runtime_handle,
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
                _ => {}
            }
        } else if self.show_filter_panel {
            match key.code {
                KeyCode::Esc | KeyCode::Char('f') => {
                    self.show_filter_panel = false;
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
                KeyCode::Char('y') => {
                    self.copy_to_clipboard();
                }
                KeyCode::Char('p') => {
                    self.toggle_project_selector();
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
            self.time_entries.sort_by(|a, b| a.start.cmp(&b.start));
        }
    }

    fn toggle_rounding(&mut self) {
        self.show_rounded = !self.show_rounded;
    }

    fn toggle_sort_by_date(&mut self) {
        self.sort_by_date = !self.sort_by_date;
        if self.sort_by_date {
            self.time_entries.sort_by(|a, b| a.start.cmp(&b.start));
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

    fn toggle_billable_filter(&mut self) {
        self.active_filter = if self.active_filter.billable_only {
            TimeEntryFilter::new()
        } else {
            TimeEntryFilter::new().with_billable_only()
        };
        self.apply_filters();
    }

    fn clear_filters(&mut self) {
        self.active_filter = TimeEntryFilter::new();
        self.apply_filters();
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

        let page_size = 10;
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

        let page_size = 10;
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

        let page_size = 10;
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

        let page_size = 10;
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

        self.filtered_projects.sort_by(|a, b| a.name.cmp(&b.name));

        if !self.filtered_projects.is_empty() {
            self.project_selector_state.select(Some(0));
        } else {
            self.project_selector_state.select(None);
        }
    }

    fn reset_filtered_projects(&mut self) {
        self.filtered_projects = self.projects.values().cloned().collect();
        self.filtered_projects.sort_by(|a, b| a.name.cmp(&b.name));

        if !self.filtered_projects.is_empty() {
            self.project_selector_state.select(Some(0));
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

            let mut success_count = 0;
            let mut fail_count = 0;
            let total_entries = grouped_entry.entries.len();

            for entry in &grouped_entry.entries {
                tracing::debug!(
                    "Assigning project {} to entry {} in workspace {}",
                    project_id,
                    entry.id,
                    entry.workspace_id
                );

                tracing::debug!("About to call handle.block_on for entry {}", entry.id);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handle.block_on(client.update_time_entry_project(
                        entry.workspace_id,
                        entry.id,
                        Some(project_id),
                    ))
                }));

                match result {
                    Ok(Ok(_)) => {
                        tracing::debug!("Successfully assigned project to entry {}", entry.id);
                        success_count += 1;

                        if let Some(time_entry) =
                            self.time_entries.iter_mut().find(|e| e.id == entry.id)
                        {
                            time_entry.project_id = Some(project_id);
                        }

                        if let Some(all_entry) =
                            self.all_entries.iter_mut().find(|e| e.id == entry.id)
                        {
                            all_entry.project_id = Some(project_id);
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!("API error assigning project to entry {}: {}", entry.id, e);
                        fail_count += 1;
                    }
                    Err(panic_err) => {
                        tracing::error!(
                            "PANIC occurred while assigning project to entry {}",
                            entry.id
                        );
                        if let Some(s) = panic_err.downcast_ref::<&str>() {
                            tracing::error!("Panic message: {}", s);
                        } else if let Some(s) = panic_err.downcast_ref::<String>() {
                            tracing::error!("Panic message: {}", s);
                        }
                        fail_count += 1;
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

            tracing::debug!(
                "About to call handle.block_on for single entry {}",
                entry_id
            );
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle.block_on(client.update_time_entry_project(
                    workspace_id,
                    entry_id,
                    Some(project_id),
                ))
            }));

            match result {
                Ok(Ok(_updated_entry)) => {
                    tracing::info!("Successfully assigned project to entry {}", entry_id);

                    if let Some(entry_mut) = self.time_entries.get_mut(selected_entry_idx) {
                        entry_mut.project_id = Some(project_id);
                    }

                    if let Some(all_entry) = self.all_entries.iter_mut().find(|e| e.id == entry_id)
                    {
                        all_entry.project_id = Some(project_id);
                    }

                    self.status_message = Some(format!("Assigned project: {}", project_name));
                    self.show_project_selector = false;
                    self.project_search_query.clear();
                    self.reset_filtered_projects();
                }
                Ok(Err(e)) => {
                    tracing::error!("API error: {}", e);
                    self.status_message = Some(format!("Failed to assign project: {}", e));
                }
                Err(panic_err) => {
                    tracing::error!("PANIC occurred while assigning project");
                    if let Some(s) = panic_err.downcast_ref::<&str>() {
                        tracing::error!("Panic message: {}", s);
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        tracing::error!("Panic message: {}", s);
                    }
                    self.status_message =
                        Some("Crashed while assigning project - check logs".to_string());
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
                    Constraint::Length(8),
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
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = format!(
            "Toggl TimeGuru - {} to {}",
            self.start_date.format("%Y-%m-%d"),
            self.end_date.format("%Y-%m-%d")
        );

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
                    let hours = if self.show_rounded && self.round_minutes.is_some() {
                        entry.rounded_hours(self.round_minutes.unwrap())
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

                    let duration_hours = if self.show_rounded && self.round_minutes.is_some() {
                        let round_to_minutes = self.round_minutes.unwrap();
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

    fn render_filter_panel(&self, f: &mut Frame, area: Rect) {
        let billable_status = if self.active_filter.billable_only {
            "ACTIVE"
        } else {
            "OFF"
        };

        let filter_lines = vec![
            Line::from(vec![Span::styled(
                "Active Filters:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("  Billable Only: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    billable_status,
                    if self.active_filter.billable_only {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Filter Controls:",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![Span::raw(
                "  b: Toggle Billable Only  │  c: Clear All Filters  │  f/Esc: Close Panel",
            )]),
        ];

        let panel = Paragraph::new(filter_lines)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Filters"));

        f.render_widget(panel, area);
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
                let spans = vec![
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
                ListItem::new(Line::from(spans))
            })
            .collect();

        let project_list = List::new(project_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select Project to Assign"),
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
            Span::raw("↑↓/jk: Navigate  │  /: Search  │  Enter: Select  │  p/Esc: Cancel"),
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

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let grouping_status = if self.show_grouped { "ON" } else { "OFF" };
        let day_grouping_status = if self.group_by_day { "ON" } else { "OFF" };
        let sort_status = if self.sort_by_date { "ON" } else { "OFF" };
        let rounding_status = if self.show_rounded { "ON" } else { "OFF" };
        let filter_indicator = if self.active_filter.billable_only {
            " [FILTERED]"
        } else {
            ""
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
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("p:Project "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("y:Copy "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw("q/Esc:Quit"),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Entry {}/{}", selected_pos, len)),
                Span::styled(
                    filter_indicator,
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
}
