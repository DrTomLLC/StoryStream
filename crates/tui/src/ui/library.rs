// crates/tui/src/ui/library.rs
//! Library view rendering

use crate::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Renders the library view
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    render_book_list(frame, chunks[0], state, theme);
    render_library_info(frame, chunks[1], state, theme);
}

/// Renders the book list
fn render_book_list(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    // Demo books for now
    let books = vec![
        "📖 Moby Dick by Herman Melville",
        "📖 Pride and Prejudice by Jane Austen",
        "📖 1984 by George Orwell",
        "📖 To Kill a Mockingbird by Harper Lee",
        "📖 The Great Gatsby by F. Scott Fitzgerald",
        "📖 War and Peace by Leo Tolstoy",
        "📖 The Catcher in the Rye by J.D. Salinger",
        "📖 Harry Potter by J.K. Rowling",
    ];

    let items: Vec<ListItem> = books
        .iter()
        .enumerate()
        .map(|(i, book)| {
            let style = if i == state.selected_item {
                theme.highlight_style()
            } else {
                theme.text_style()
            };

            ListItem::new(Line::from(Span::styled(*book, style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("📚 Library (↑/↓: Navigate | Enter: Play | /: Search | f: Favorite)"),
        )
        .style(theme.text_style());

    frame.render_widget(list, area);
}

/// Renders library information
fn render_library_info(frame: &mut Frame, area: Rect, _state: &AppState, theme: &crate::theme::Theme) {
    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Total: ", theme.text_secondary_style()),
            Span::styled("8 books", theme.highlight_style()),
            Span::raw("  |  "),
            Span::styled("Playing: ", theme.text_secondary_style()),
            Span::styled("None", theme.text_style()),
            Span::raw("  |  "),
            Span::styled("Last sync: ", theme.text_secondary_style()),
            Span::styled("Never", theme.text_style()),
        ]),
    ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("Info"),
        )
        .style(theme.text_style());

    frame.render_widget(info, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_render_compiles() {
        let state = AppState::new();
        let _ = state.selected_item;
    }
}