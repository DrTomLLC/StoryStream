// crates/tui/src/ui/mod.rs
//! UI rendering modules

pub mod bookmarks;
pub mod help;
pub mod library;
pub mod player;
pub mod playlists;
pub mod search;
pub mod settings;
pub mod statistics;

use crate::{
    state::{AppState, View},
    theme::Theme,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

/// Renders the main UI
pub fn render(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Status bar
        ])
        .split(frame.area());

    render_tabs(frame, chunks[0], state, theme);
    render_content(frame, chunks[1], state, theme);
    render_status_bar(frame, chunks[2], state, theme);
}

/// Renders the tab bar
fn render_tabs(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let titles = vec![
        "Library",
        "Player",
        "Bookmarks",
        "Search",
        "Playlists",
        "Statistics",
        "Settings",
        "Help",
    ];
    let index = match state.view {
        View::Library => 0,
        View::Player => 1,
        View::Bookmarks => 2,
        View::Search => 3,
        View::Playlists => 4,
        View::Statistics => 5,
        View::Settings => 6,
        View::Help => 7,
        View::Plugin => 0,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("♪ StoryStream"),
        )
        .select(index)
        .style(theme.text_style())
        .highlight_style(theme.highlight_style());

    frame.render_widget(tabs, area);
}

/// Renders the current view content
fn render_content(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    match state.view {
        View::Library => library::render(frame, area, state, theme),
        View::Player => player::render(frame, area, state, theme),
        View::Bookmarks => bookmarks::render(frame, area, state, theme),
        View::Search => search::render(frame, area, state, theme),
        View::Playlists => playlists::render(frame, area, state, theme),
        View::Statistics => statistics::render(frame, area, state, theme),
        View::Settings => settings::render(frame, area, state, theme),
        View::Help => help::render(frame, area, state, theme),
        View::Plugin => {
            // Plugin rendering would go here
            library::render(frame, area, state, theme)
        }
    }
}

/// Renders the status bar
fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let status_text = if let Some(ref msg) = state.status_message {
        msg.clone()
    } else {
        format!(
            "q: Quit | h: Help | Tab: Switch | t: Theme ({}) | Mouse: Enabled",
            theme.theme_type.name()
        )
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            " ● ",
            Style::default().fg(if state.playback.is_playing {
                theme.playing
            } else {
                theme.paused
            }),
        ),
        Span::styled(status_text, theme.text_style()),
    ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color())),
        );

    frame.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_functions_exist() {
        // Just verify the functions compile
        let state = AppState::new();
        let _ = state.view;
    }
}