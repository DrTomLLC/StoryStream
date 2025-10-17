// crates/tui/src/ui/statistics.rs
//! Statistics view rendering

use crate::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

/// Renders the statistics view
pub fn render(frame: &mut Frame, area: Rect, _state: &AppState, theme: &crate::theme::Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Overview
            Constraint::Length(10), // Listening stats
            Constraint::Min(0),     // Top books
        ])
        .split(area);

    render_overview(frame, chunks[0], theme);
    render_listening_stats(frame, chunks[1], theme);
    render_top_books(frame, chunks[2], theme);
}

/// Renders statistics overview
fn render_overview(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let stats = vec![
        Line::from(vec![
            Span::styled("📚 Total Books: ", theme.text_secondary_style()),
            Span::styled("156", theme.highlight_style()),
            Span::raw("  "),
            Span::styled("🎧 Hours Listened: ", theme.text_secondary_style()),
            Span::styled("342.5", theme.highlight_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("⭐ Favorites: ", theme.text_secondary_style()),
            Span::styled("23", theme.highlight_style()),
            Span::raw("  "),
            Span::styled("🏁 Finished: ", theme.text_secondary_style()),
            Span::styled("89", theme.highlight_style()),
            Span::raw("  "),
            Span::styled("📖 In Progress: ", theme.text_secondary_style()),
            Span::styled("12", theme.highlight_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📊 Average Rating: ", theme.text_secondary_style()),
            Span::styled("4.2/5.0", theme.highlight_style()),
            Span::raw("  "),
            Span::styled("🎯 Completion Rate: ", theme.text_secondary_style()),
            Span::styled("57%", theme.highlight_style()),
        ]),
    ];

    let paragraph = Paragraph::new(stats)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("📊 Overview"),
        )
        .style(theme.text_style());

    frame.render_widget(paragraph, area);
}

/// Renders listening statistics
fn render_listening_stats(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(area);

    // This week
    let gauge1 = Gauge::default()
        .block(Block::default().title("This Week: 12.5 hours"))
        .gauge_style(theme.success_style())
        .percent(75);
    frame.render_widget(gauge1, chunks[0]);

    // This month
    let gauge2 = Gauge::default()
        .block(Block::default().title("This Month: 42.3 hours"))
        .gauge_style(theme.accent_style())
        .percent(60);
    frame.render_widget(gauge2, chunks[1]);

    // This year
    let gauge3 = Gauge::default()
        .block(Block::default().title("This Year: 342.5 hours"))
        .gauge_style(theme.highlight_style())
        .percent(85);
    frame.render_widget(gauge3, chunks[2]);

    // All time
    let gauge4 = Gauge::default()
        .block(Block::default().title("All Time: 1,247.8 hours"))
        .gauge_style(Style::default().fg(theme.playing))
        .percent(100);
    frame.render_widget(gauge4, chunks[3]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_color()))
        .title("📈 Listening Time");
    frame.render_widget(block, area);
}

/// Renders top books
fn render_top_books(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let books = vec![
        ("1. 📖 Moby Dick", "Herman Melville", "23.5 hours"),
        ("2. 📖 War and Peace", "Leo Tolstoy", "61.2 hours"),
        ("3. 📖 Harry Potter Series", "J.K. Rowling", "117.3 hours"),
        ("4. 📖 The Lord of the Rings", "J.R.R. Tolkien", "54.8 hours"),
        ("5. 📖 Foundation Series", "Isaac Asimov", "42.1 hours"),
    ];

    let items: Vec<ListItem> = books
        .iter()
        .map(|(rank, title, time)| {
            ListItem::new(vec![
                Line::from(Span::styled(*rank, theme.highlight_style())),
                Line::from(Span::styled(*title, theme.text_style())),
                Line::from(Span::styled(
                    format!("  {} listened", time),
                    theme.text_secondary_style(),
                )),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("🏆 Most Listened"),
        )
        .style(theme.text_style());

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_render_compiles() {
        let state = AppState::new();
        let _ = state.view;
    }
}