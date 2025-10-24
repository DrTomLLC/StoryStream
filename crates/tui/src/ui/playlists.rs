// crates/tui/src/ui/playlists.rs
//! Playlists view rendering

use crate::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Renders the playlists view
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_playlist_list(frame, chunks[0], state, theme);
    render_playlist_items(frame, chunks[1], state, theme);
}

/// Renders the playlist list
fn render_playlist_list(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &crate::theme::Theme,
) {
    let playlists = vec![
        ("🎵 Recently Played", "8 books"),
        ("⭐ Favorites", "12 books"),
        ("📚 Currently Reading", "3 books"),
        ("🏁 Finished", "45 books"),
        ("🎧 Podcasts", "6 series"),
        ("🆕 New Arrivals", "4 books"),
    ];

    let items: Vec<ListItem> = playlists
        .iter()
        .enumerate()
        .map(|(i, (name, count))| {
            let style = if i == state.selected_item {
                theme.highlight_style()
            } else {
                theme.text_style()
            };

            ListItem::new(vec![
                Line::from(Span::styled(*name, style)),
                Line::from(Span::styled(
                    format!("  {}", count),
                    theme.text_secondary_style(),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("📋 Playlists"),
        )
        .style(theme.text_style());

    frame.render_widget(list, area);
}

/// Renders playlist items
fn render_playlist_items(
    frame: &mut Frame,
    area: Rect,
    _state: &AppState,
    theme: &crate::theme::Theme,
) {
    let books = vec![
        "📖 Moby Dick",
        "📖 Pride and Prejudice",
        "📖 1984",
        "📖 To Kill a Mockingbird",
        "📖 The Great Gatsby",
    ];

    let items: Vec<ListItem> = books
        .iter()
        .map(|book| ListItem::new(Line::from(Span::styled(*book, theme.text_style()))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("📚 Recently Played (↑/↓: Navigate | Enter: Play | n: New playlist)"),
        )
        .style(theme.text_style());

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlists_render_compiles() {
        let state = AppState::new();
        let _ = state.view;
    }
}
