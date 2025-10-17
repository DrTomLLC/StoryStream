// crates/tui/src/ui/search.rs
//! Search view rendering

use crate::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Renders the search view
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),     // Results
            Constraint::Length(3),  // Help
        ])
        .split(area);

    render_search_input(frame, chunks[0], state, theme);
    render_search_results(frame, chunks[1], state, theme);
    render_search_help(frame, chunks[2], theme);
}

/// Renders the search input
fn render_search_input(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let input = Paragraph::new(format!("🔍 {}_", state.search_query))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("Search"),
        )
        .style(theme.text_style());

    frame.render_widget(input, area);
}

/// Renders search results
fn render_search_results(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    // Demo results based on query
    let all_books = vec![
        ("📖 Moby Dick", "Herman Melville"),
        ("📖 Pride and Prejudice", "Jane Austen"),
        ("📖 1984", "George Orwell"),
        ("📖 To Kill a Mockingbird", "Harper Lee"),
        ("📖 The Great Gatsby", "F. Scott Fitzgerald"),
        ("📖 War and Peace", "Leo Tolstoy"),
        ("📖 The Catcher in the Rye", "J.D. Salinger"),
        ("📖 Harry Potter", "J.K. Rowling"),
    ];

    let filtered: Vec<_> = if state.search_query.is_empty() {
        all_books.clone()
    } else {
        all_books
            .iter()
            .filter(|(title, author)| {
                let query = state.search_query.to_lowercase();
                title.to_lowercase().contains(&query) || author.to_lowercase().contains(&query)
            })
            .copied()
            .collect()
    };

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, (title, author))| {
            let style = if i == state.selected_item {
                theme.highlight_style()
            } else {
                theme.text_style()
            };

            ListItem::new(vec![
                Line::from(Span::styled(*title, style)),
                Line::from(Span::styled(
                    format!("  by {}", author),
                    theme.text_secondary_style(),
                )),
            ])
        })
        .collect();

    let title = format!("Results ({} found)", filtered.len());
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title(title),
        )
        .style(theme.text_style());

    frame.render_widget(list, area);
}

/// Renders search help
fn render_search_help(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let help = Paragraph::new("Type to search | ↑/↓: Navigate | Enter: Play | Esc: Clear search")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color())),
        )
        .style(theme.text_secondary_style());

    frame.render_widget(help, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_render_compiles() {
        let state = AppState::new();
        let _ = state.search_query;
    }
}