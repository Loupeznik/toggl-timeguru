use anyhow::Result;
use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::toggl::models::{GroupedTimeEntry, TimeEntry};

pub struct App {
    pub time_entries: Vec<TimeEntry>,
    pub grouped_entries: Vec<GroupedTimeEntry>,
    pub list_state: ListState,
    pub should_quit: bool,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub show_grouped: bool,
    pub show_rounded: bool,
    pub round_minutes: Option<i64>,
}

impl App {
    pub fn new(
        time_entries: Vec<TimeEntry>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        round_minutes: Option<i64>,
    ) -> Self {
        let mut list_state = ListState::default();
        if !time_entries.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            time_entries,
            grouped_entries: Vec::new(),
            list_state,
            should_quit: false,
            start_date,
            end_date,
            show_grouped: false,
            show_rounded: true,
            round_minutes,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key);
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
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
            KeyCode::Char('g') => {
                self.toggle_grouping();
            }
            KeyCode::Char('r') => {
                self.toggle_rounding();
            }
            _ => {}
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

    fn toggle_rounding(&mut self) {
        self.show_rounded = !self.show_rounded;
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        self.render_header(f, chunks[0]);
        self.render_list(f, chunks[1]);
        self.render_footer(f, chunks[2]);
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

                    let content = Line::from(vec![
                        Span::styled(
                            format!("{:.2}h", hours),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" - "),
                        Span::raw(desc),
                        Span::styled(
                            format!(" ({} entries)", entry.entries.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]);

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
                        let rounded_duration = ((entry.duration as f64 / seconds_per_round as f64).ceil() as i64) * seconds_per_round;
                        rounded_duration as f64 / 3600.0
                    } else {
                        entry.duration as f64 / 3600.0
                    };

                    let content = Line::from(vec![
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
                        Span::raw(desc),
                    ]);

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

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let grouping_status = if self.show_grouped { "ON" } else { "OFF" };
        let rounding_status = if self.show_rounded { "ON" } else { "OFF" };

        let footer_text = format!(
            "q: Quit | ↑↓/jk: Navigate | g: Grouping ({}) | r: Rounding ({})",
            grouping_status, rounding_status
        );

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(footer, area);
    }
}
