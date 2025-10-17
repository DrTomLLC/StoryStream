// crates/tui/src/ui/help.rs
//! Enhanced help view with detailed examples

use crate::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

/// Help sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpSection {
    General,
    Library,
    Player,
    Bookmarks,
    Search,
    Playlists,
    Statistics,
    Settings,
    KeyboardShortcuts,
    MouseControls,
    Examples,
}

impl HelpSection {
    pub fn all() -> Vec<HelpSection> {
        vec![
            HelpSection::General,
            HelpSection::Library,
            HelpSection::Player,
            HelpSection::Bookmarks,
            HelpSection::Search,
            HelpSection::Playlists,
            HelpSection::Statistics,
            HelpSection::Settings,
            HelpSection::KeyboardShortcuts,
            HelpSection::MouseControls,
            HelpSection::Examples,
        ]
    }

    pub fn title(&self) -> &str {
        match self {
            HelpSection::General => "General",
            HelpSection::Library => "Library",
            HelpSection::Player => "Player",
            HelpSection::Bookmarks => "Bookmarks",
            HelpSection::Search => "Search",
            HelpSection::Playlists => "Playlists",
            HelpSection::Statistics => "Statistics",
            HelpSection::Settings => "Settings",
            HelpSection::KeyboardShortcuts => "Keyboard",
            HelpSection::MouseControls => "Mouse",
            HelpSection::Examples => "Examples",
        }
    }
}

/// Renders the help view with sections
pub fn render(frame: &mut Frame, area: Rect, _state: &AppState, theme: &crate::theme::Theme) {
    // For now, show scrollable help with all sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    render_all_help(frame, chunks[0], theme);
}

/// Renders comprehensive help content
fn render_all_help(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let help_content = vec![
        // Header
        Line::from(vec![
            Span::styled(
                "═══════════════════════════════════════════════════════════════",
                theme.accent_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "        ♪ STORYSTREAM COMPLETE USER GUIDE ♪",
                theme.highlight_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "═══════════════════════════════════════════════════════════════",
                theme.accent_style(),
            ),
        ]),
        Line::from(""),

        // GENERAL
        section_header("1. GENERAL NAVIGATION", theme),
        Line::from(""),
        help_item("q / Ctrl+C", "Quit application", theme),
        help_item("Tab", "Switch between views (Library → Player → Bookmarks → ...)", theme),
        help_item("Shift+Tab", "Switch views in reverse", theme),
        help_item("h", "Show/hide this help screen", theme),
        help_item("t", "Cycle through color themes", theme),
        help_item("Esc", "Cancel current operation or go back", theme),
        Line::from(""),
        example_box("Example: Press Tab repeatedly to cycle through all views", theme),
        Line::from(""),

        // LIBRARY
        section_header("2. LIBRARY VIEW 📚", theme),
        Line::from(""),
        help_item("↑ / k", "Move selection up", theme),
        help_item("↓ / j", "Move selection down", theme),
        help_item("Enter", "Play selected audiobook", theme),
        help_item("s", "Sync library with other devices", theme),
        help_item("i", "Show detailed info about selected book", theme),
        help_item("f", "Toggle favorite status", theme),
        help_item("d", "Delete book (soft delete)", theme),
        help_item("/", "Open search (or switch to Search view)", theme),
        Line::from(""),
        example_box("Example: Use ↑/↓ to browse, Enter to start playing", theme),
        Line::from(""),

        // PLAYER
        section_header("3. PLAYER CONTROLS ▶️", theme),
        Line::from(""),
        subsection("Playback Control:", theme),
        help_item("Space", "Play/Pause toggle", theme),
        help_item("Enter", "Play (if paused)", theme),
        help_item("p", "Pause", theme),
        help_item(".", "Stop playback", theme),
        Line::from(""),
        subsection("Seeking:", theme),
        help_item("←", "Seek backward 10 seconds", theme),
        help_item("→", "Seek forward 10 seconds", theme),
        help_item("Shift+←", "Seek backward 30 seconds", theme),
        help_item("Shift+→", "Seek forward 30 seconds", theme),
        help_item("Home", "Jump to beginning", theme),
        help_item("End", "Jump to end", theme),
        Line::from(""),
        subsection("Speed Control:", theme),
        help_item("[", "Decrease speed by 0.1x (min: 0.5x)", theme),
        help_item("]", "Increase speed by 0.1x (max: 3.0x)", theme),
        help_item("Shift+[", "Set speed to 0.5x", theme),
        help_item("Shift+]", "Set speed to 3.0x", theme),
        help_item("\\", "Reset speed to 1.0x", theme),
        Line::from(""),
        subsection("Volume Control:", theme),
        help_item("+ / =", "Increase volume by 10%", theme),
        help_item("-", "Decrease volume by 10%", theme),
        help_item("0", "Mute/Unmute", theme),
        Line::from(""),
        subsection("Chapter Navigation:", theme),
        help_item("n", "Next chapter", theme),
        help_item("p / b", "Previous chapter", theme),
        help_item("Ctrl+n", "Skip to last chapter", theme),
        help_item("Ctrl+p", "Go to first chapter", theme),
        help_item("1-9", "Jump to chapter 1-9", theme),
        Line::from(""),
        example_box(
            "Example: Press Space to pause, then → → → to skip ahead 30s",
            theme,
        ),
        Line::from(""),

        // BOOKMARKS
        section_header("4. BOOKMARKS 🔖", theme),
        Line::from(""),
        help_item("b", "Add bookmark at current position", theme),
        help_item("Shift+B", "Add bookmark with custom note", theme),
        help_item("Enter", "Jump to selected bookmark", theme),
        help_item("d", "Delete selected bookmark", theme),
        help_item("e", "Edit bookmark note/title", theme),
        help_item("↑/↓", "Navigate bookmarks", theme),
        help_item("Ctrl+e", "Export bookmarks to file", theme),
        Line::from(""),
        example_box(
            "Example: While listening, press 'b' to bookmark important quotes",
            theme,
        ),
        Line::from(""),

        // SEARCH
        section_header("5. SEARCH 🔍", theme),
        Line::from(""),
        help_item("/", "Open search from any view", theme),
        help_item("Type text", "Search as you type", theme),
        help_item("↑/↓", "Navigate search results", theme),
        help_item("Enter", "Open selected result", theme),
        help_item("Esc", "Clear search and return", theme),
        help_item("Ctrl+f", "Focus search box", theme),
        Line::from(""),
        subsection("Search Examples:", theme),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("'tolkien'", theme.highlight_style()),
            Span::styled(" - Find all books by Tolkien", theme.text_style()),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("'1984'", theme.highlight_style()),
            Span::styled(" - Find book by title", theme.text_style()),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("'sci-fi'", theme.highlight_style()),
            Span::styled(" - Search in genres", theme.text_style()),
        ]),
        Line::from(""),

        // PLAYLISTS
        section_header("6. PLAYLISTS 📋", theme),
        Line::from(""),
        help_item("n", "Create new playlist", theme),
        help_item("a", "Add current book to playlist", theme),
        help_item("r", "Remove selected from playlist", theme),
        help_item("↑/↓", "Navigate playlists/items", theme),
        help_item("Enter", "Play playlist", theme),
        help_item("Shift+Enter", "Shuffle and play", theme),
        help_item("e", "Edit playlist", theme),
        help_item("d", "Delete playlist", theme),
        Line::from(""),
        example_box(
            "Example: Create a 'Bedtime Stories' playlist with calming books",
            theme,
        ),
        Line::from(""),

        // STATISTICS
        section_header("7. STATISTICS 📊", theme),
        Line::from(""),
        help_item("r", "Refresh statistics", theme),
        help_item("↑/↓", "Scroll through stats", theme),
        help_item("e", "Export stats to CSV", theme),
        Line::from(""),
        subsection("Statistics Include:", theme),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Total listening time and books completed"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Most listened books and authors"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Listening trends and patterns"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Completion rates and favorites"),
        ]),
        Line::from(""),

        // SETTINGS
        section_header("8. SETTINGS ⚙️", theme),
        Line::from(""),
        help_item("↑/↓", "Navigate settings", theme),
        help_item("Enter", "Edit selected setting", theme),
        help_item("Space", "Toggle boolean settings", theme),
        help_item("←/→", "Adjust numeric values", theme),
        help_item("t", "Cycle color themes", theme),
        help_item("r", "Reset all settings to defaults", theme),
        Line::from(""),
        subsection("Configurable Settings:", theme),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Default playback speed and volume"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Auto-save interval and resume behavior"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Library paths and scan settings"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Sync preferences and conflict resolution"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::raw("Theme and appearance"),
        ]),
        Line::from(""),

        // MOUSE CONTROLS
        section_header("9. MOUSE CONTROLS 🖱️", theme),
        Line::from(""),
        help_item("Click", "Select items in lists", theme),
        help_item("Double-click", "Activate/play selected item", theme),
        help_item("Right-click", "Open context menu", theme),
        help_item("Scroll wheel", "Scroll through lists", theme),
        help_item("Click on tabs", "Switch views", theme),
        help_item("Click progress bar", "Seek to position", theme),
        help_item("Drag progress bar", "Scrub through audio", theme),
        Line::from(""),
        example_box("Example: Click on a book in the library to select it", theme),
        Line::from(""),

        // THEMES
        section_header("10. COLOR THEMES 🎨", theme),
        Line::from(""),
        help_item("t", "Cycle to next theme", theme),
        Line::from(""),
        subsection("Available Themes:", theme),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("Dark", theme.highlight_style()),
            Span::raw(" - Classic dark theme (default)"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("Light", theme.highlight_style()),
            Span::raw(" - Light theme for daytime use"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("High Contrast", theme.highlight_style()),
            Span::raw(" - Maximum readability"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("Solarized Dark/Light", theme.highlight_style()),
            Span::raw(" - Popular Solarized themes"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("Dracula", theme.highlight_style()),
            Span::raw(" - Modern dark theme"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("Nord", theme.highlight_style()),
            Span::raw(" - Arctic-inspired theme"),
        ]),
        Line::from(vec![
            Span::styled("  • ", theme.text_secondary_style()),
            Span::styled("Monokai", theme.highlight_style()),
            Span::raw(" - Sublime Text classic"),
        ]),
        Line::from(""),

        // TIPS AND TRICKS
        section_header("11. TIPS & TRICKS 💡", theme),
        Line::from(""),
        Line::from(vec![
            Span::styled("Speed Listening:", theme.text_secondary_style().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Increase speed gradually to 1.5x-2.0x for efficient listening"),
        Line::from("  Perfect for catching up on backlogs!"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Bookmarking Strategy:", theme.text_secondary_style().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Press 'b' whenever you hear something interesting"),
        Line::from("  Use Shift+B to add detailed notes"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Smart Playlists:", theme.text_secondary_style().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Create mood-based playlists (Relaxing, Exciting, etc.)"),
        Line::from("  Use 'Recently Played' to resume your listening journey"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sync Across Devices:", theme.text_secondary_style().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Enable auto-sync in Settings"),
        Line::from("  Start on laptop, continue on phone seamlessly"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Night Listening:", theme.text_secondary_style().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Switch to a dark theme (press 't')"),
        Line::from("  Reduce volume gradually as you fall asleep"),
        Line::from(""),

        // EXAMPLES
        section_header("12. COMPLETE WORKFLOW EXAMPLES 📖", theme),
        Line::from(""),
        subsection("Example 1: Starting a New Book", theme),
        Line::from(vec![
            Span::styled("  1. ", theme.highlight_style()),
            Span::raw("Press Tab until you reach Library view"),
        ]),
        Line::from(vec![
            Span::styled("  2. ", theme.highlight_style()),
            Span::raw("Use ↑/↓ to browse your collection"),
        ]),
        Line::from(vec![
            Span::styled("  3. ", theme.highlight_style()),
            Span::raw("Press Enter to start playing"),
        ]),
        Line::from(vec![
            Span::styled("  4. ", theme.highlight_style()),
            Span::raw("Adjust speed with [ or ] if desired"),
        ]),
        Line::from(vec![
            Span::styled("  5. ", theme.highlight_style()),
            Span::raw("Press 'b' to bookmark important moments"),
        ]),
        Line::from(""),
        subsection("Example 2: Resuming Your Audiobook", theme),
        Line::from(vec![
            Span::styled("  1. ", theme.highlight_style()),
            Span::raw("Open StoryStream (position auto-saved)"),
        ]),
        Line::from(vec![
            Span::styled("  2. ", theme.highlight_style()),
            Span::raw("Go to Player view (Tab)"),
        ]),
        Line::from(vec![
            Span::styled("  3. ", theme.highlight_style()),
            Span::raw("Press Space to resume playback"),
        ]),
        Line::from(""),
        subsection("Example 3: Creating a Playlist", theme),
        Line::from(vec![
            Span::styled("  1. ", theme.highlight_style()),
            Span::raw("Switch to Playlists view"),
        ]),
        Line::from(vec![
            Span::styled("  2. ", theme.highlight_style()),
            Span::raw("Press 'n' to create new playlist"),
        ]),
        Line::from(vec![
            Span::styled("  3. ", theme.highlight_style()),
            Span::raw("Name it (e.g., 'Sci-Fi Favorites')"),
        ]),
        Line::from(vec![
            Span::styled("  4. ", theme.highlight_style()),
            Span::raw("Go to Library, select books, press 'a' to add"),
        ]),
        Line::from(vec![
            Span::styled("  5. ", theme.highlight_style()),
            Span::raw("Return to Playlists and press Enter to play"),
        ]),
        Line::from(""),
        subsection("Example 4: Searching Your Library", theme),
        Line::from(vec![
            Span::styled("  1. ", theme.highlight_style()),
            Span::raw("Press '/' from any view"),
        ]),
        Line::from(vec![
            Span::styled("  2. ", theme.highlight_style()),
            Span::raw("Type author name (e.g., 'tolkien')"),
        ]),
        Line::from(vec![
            Span::styled("  3. ", theme.highlight_style()),
            Span::raw("Use ↑/↓ to browse results"),
        ]),
        Line::from(vec![
            Span::styled("  4. ", theme.highlight_style()),
            Span::raw("Press Enter to play selected book"),
        ]),
        Line::from(""),
        subsection("Example 5: Customizing Your Experience", theme),
        Line::from(vec![
            Span::styled("  1. ", theme.highlight_style()),
            Span::raw("Go to Settings view"),
        ]),
        Line::from(vec![
            Span::styled("  2. ", theme.highlight_style()),
            Span::raw("Navigate to 'Default Speed'"),
        ]),
        Line::from(vec![
            Span::styled("  3. ", theme.highlight_style()),
            Span::raw("Use ←/→ to adjust (e.g., 1.25x)"),
        ]),
        Line::from(vec![
            Span::styled("  4. ", theme.highlight_style()),
            Span::raw("Press 't' to cycle themes until you find one you like"),
        ]),
        Line::from(vec![
            Span::styled("  5. ", theme.highlight_style()),
            Span::raw("Settings are automatically saved"),
        ]),
        Line::from(""),

        // TROUBLESHOOTING
        section_header("13. TROUBLESHOOTING 🔧", theme),
        Line::from(""),
        subsection("Problem: Controls don't respond", theme),
        Line::from("  → Ensure the terminal window has focus"),
        Line::from("  → Try pressing Esc to cancel any pending operation"),
        Line::from(""),
        subsection("Problem: Display looks wrong", theme),
        Line::from("  → Resize terminal window (minimum 80x24)"),
        Line::from("  → Try different theme (press 't')"),
        Line::from("  → Restart the application"),
        Line::from(""),
        subsection("Problem: Audio doesn't play", theme),
        Line::from("  → Check that the audio file exists"),
        Line::from("  → Verify volume isn't muted (press '+' to increase)"),
        Line::from("  → Check system audio settings"),
        Line::from(""),

        // FOOTER
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "═══════════════════════════════════════════════════════════════",
                theme.accent_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "       Press 'h' or Esc to close help · Press Tab to navigate",
                theme.text_secondary_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "═══════════════════════════════════════════════════════════════",
                theme.accent_style(),
            ),
        ]),
    ];

    let help = List::new(
        help_content
            .into_iter()
            .map(|line| ListItem::new(line))
            .collect::<Vec<_>>(),
    )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color()))
                .title("❓ Complete User Guide (Scroll with ↑/↓ or Mouse Wheel)"),
        );

    frame.render_widget(help, area);
}

fn section_header<'a>(text: &'a str, theme: &crate::theme::Theme) -> Line<'a> {
    Line::from(vec![Span::styled(
        text,
        theme.accent_style().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )])
}

fn subsection<'a>(text: &'a str, theme: &crate::theme::Theme) -> Line<'a> {
    Line::from(vec![Span::styled(
        text,
        theme.highlight_style().add_modifier(Modifier::BOLD),
    )])
}

fn help_item<'a>(key: &'a str, description: &'a str, theme: &crate::theme::Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{:20}", key),
            theme.highlight_style(),
        ),
        Span::styled(" → ", theme.text_secondary_style()),
        Span::styled(description, theme.text_style()),
    ])
}

fn example_box<'a>(text: &'a str, theme: &crate::theme::Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled("  💡 ", theme.warning_style()),
        Span::styled(text, theme.text_secondary_style().add_modifier(Modifier::ITALIC)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_render_compiles() {
        let state = AppState::new();
        let _ = state.view;
    }

    #[test]
    fn test_help_sections() {
        let sections = HelpSection::all();
        assert_eq!(sections.len(), 11);

        for section in sections {
            let _ = section.title();
        }
    }
}