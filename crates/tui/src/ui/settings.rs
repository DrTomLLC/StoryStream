// crates/tui/src/ui/settings.rs

use crate::state::AppState;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Renders the settings view
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    // Pre-format dynamic strings to avoid lifetime issues
    let appearance_text = "🎨 Appearance (Press 't' to cycle)".to_string();
    let theme_text = format!("  └─ Theme: {}", state.theme.name());

    let settings = vec![
        "⚙️  Audio Settings",
        "  └─ Default Volume: 100%",
        "  └─ Default Speed: 1.0x",
        "",
        "📁 Library Settings",
        "  └─ Auto-scan: Enabled",
        "  └─ Library Paths: ~/Audiobooks",
        "",
        "🔄 Sync Settings",
        "  └─ Auto-sync: Disabled",
        "  └─ Conflict Resolution: Use Newest",
        "",
        appearance_text.as_str(),
        theme_text.as_str(),
    ];

    let items: Vec<ListItem> = settings
        .iter()
        .map(|setting| ListItem::new(Line::from(*setting)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("⚙️  Settings (↑/↓: Navigate | Enter: Edit | t: Change Theme)"),
        )
        .style(theme.text_style());

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_render_compiles() {
        let state = AppState::new();
        let _ = state.view;
    }
}
