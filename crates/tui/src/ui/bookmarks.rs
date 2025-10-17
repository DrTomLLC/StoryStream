// crates/tui/src/ui/bookmarks.rs
//! Bookmarks view rendering

use crate::state::AppState;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Renders the bookmarks view
pub fn render(frame: &mut Frame, area: Rect, _state: &AppState, theme: &crate::theme::Theme) {
    // Demo bookmarks
    let bookmarks = vec![
        "📌 00:15:32 - Call me Ishmael",
        "📌 01:23:45 - The whale appears",
        "📌 02:45:12 - Important quote",
    ];

    let items: Vec<ListItem> = bookmarks
        .iter()
        .map(|bookmark| ListItem::new(Line::from(Span::styled(*bookmark, theme.text_style()))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("🔖 Bookmarks (b: Add | d: Delete | Enter: Jump)"),
        )
        .style(theme.text_style());

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmarks_render_compiles() {
        let state = AppState::new();
        let _ = state.view;
    }
}