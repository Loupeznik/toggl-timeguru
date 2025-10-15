use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

#[allow(dead_code)]
pub fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[allow(dead_code)]
pub fn status_line(message: &str, style: Style) -> Line<'_> {
    Line::from(vec![Span::styled(message, style)])
}

#[allow(dead_code)]
pub fn loading_indicator() -> Line<'static> {
    Line::from(vec![
        Span::styled("Loading", Style::default().fg(Color::Yellow)),
        Span::raw("..."),
    ])
}
