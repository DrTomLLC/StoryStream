// crates/tui/src/ui/player.rs
//! Player view rendering

use crate::state::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Renders the player view
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Title/Artist
            Constraint::Length(3), // Progress bar
            Constraint::Length(5), // Time info
            Constraint::Length(7), // Controls
            Constraint::Min(0),    // Chapter info
        ])
        .split(area);

    render_now_playing(frame, chunks[0], state, theme);
    render_progress(frame, chunks[1], state, theme);
    render_time_info(frame, chunks[2], state, theme);
    render_controls(frame, chunks[3], state, theme);
    render_chapter_info(frame, chunks[4], state, theme);
}

/// Renders now playing information
fn render_now_playing(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &crate::theme::Theme,
) {
    let title = if let Some(ref file) = state.playback.current_file {
        file.clone()
    } else {
        "No audiobook loaded".to_string()
    };

    let text = vec![
        Line::from(Span::styled(
            "♪ Now Playing",
            theme.accent_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(title, theme.text_style())),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color())),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Renders progress bar
fn render_progress(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let progress = (state.playback.progress() * 100.0) as u16;
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("Progress"),
        )
        .gauge_style(theme.success_style())
        .percent(progress);

    frame.render_widget(gauge, area);
}

/// Renders time information
fn render_time_info(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let time_info = format!(
        "{} / {} ({}% complete)",
        state.playback.format_position(),
        state.playback.format_duration(),
        (state.playback.progress() * 100.0) as u16
    );

    let paragraph = Paragraph::new(time_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("Time"),
        )
        .alignment(Alignment::Center)
        .style(theme.highlight_style());

    frame.render_widget(paragraph, area);
}

/// Renders playback controls
fn render_controls(frame: &mut Frame, area: Rect, state: &AppState, theme: &crate::theme::Theme) {
    let status = if state.playback.is_playing {
        "▶ Playing"
    } else {
        "⏸ Paused"
    };

    let controls = vec![
        Line::from(Span::styled(
            status,
            Style::default()
                .fg(if state.playback.is_playing {
                    theme.playing
                } else {
                    theme.paused
                })
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Speed: ", theme.text_secondary_style()),
            Span::styled(
                format!("{:.1}x", state.playback.speed),
                theme.highlight_style(),
            ),
            Span::raw("  |  "),
            Span::styled("Volume: ", theme.text_secondary_style()),
            Span::styled(
                format!("{}%", (state.playback.volume * 100.0) as u8),
                theme.highlight_style(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Space: Play/Pause | ←/→: Seek | [/]: Speed | +/-: Volume",
            theme.text_secondary_style(),
        )),
    ];

    let paragraph = Paragraph::new(controls)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("Controls"),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Renders chapter information
fn render_chapter_info(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &crate::theme::Theme,
) {
    let chapter_info = if let Some(ch) = state.playback.chapter {
        format!("Chapter {} of ?", ch + 1)
    } else {
        "No chapters available".to_string()
    };

    let paragraph = Paragraph::new(vec![
        Line::from(Span::styled(chapter_info, theme.accent_style())),
        Line::from(""),
        Line::from(Span::styled(
            "n: Next Chapter | p: Previous Chapter",
            theme.text_secondary_style(),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color()))
            .title("Chapters"),
    )
    .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_render_compiles() {
        let state = AppState::new();
        let _ = state.playback.is_playing;
    }
}
