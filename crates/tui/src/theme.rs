// crates/tui/src/theme.rs
//! Theme system for customizable colors

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Available themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeType {
    /// Default dark theme
    Dark,
    /// Light theme
    Light,
    /// High contrast theme
    HighContrast,
    /// Solarized dark
    SolarizedDark,
    /// Solarized light
    SolarizedLight,
    /// Dracula theme
    Dracula,
    /// Nord theme
    Nord,
    /// Monokai theme
    Monokai,
}

impl Default for ThemeType {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeType {
    /// Returns all available themes
    pub fn all() -> Vec<ThemeType> {
        vec![
            ThemeType::Dark,
            ThemeType::Light,
            ThemeType::HighContrast,
            ThemeType::SolarizedDark,
            ThemeType::SolarizedLight,
            ThemeType::Dracula,
            ThemeType::Nord,
            ThemeType::Monokai,
        ]
    }

    /// Returns the theme name
    pub fn name(&self) -> &str {
        match self {
            ThemeType::Dark => "Dark",
            ThemeType::Light => "Light",
            ThemeType::HighContrast => "High Contrast",
            ThemeType::SolarizedDark => "Solarized Dark",
            ThemeType::SolarizedLight => "Solarized Light",
            ThemeType::Dracula => "Dracula",
            ThemeType::Nord => "Nord",
            ThemeType::Monokai => "Monokai",
        }
    }
}

/// Color theme
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme type
    pub theme_type: ThemeType,
    /// Primary text color
    pub text: Color,
    /// Secondary text color
    pub text_secondary: Color,
    /// Background color
    pub background: Color,
    /// Highlight/selection color
    pub highlight: Color,
    /// Accent color
    pub accent: Color,
    /// Success color (green)
    pub success: Color,
    /// Warning color (yellow)
    pub warning: Color,
    /// Error color (red)
    pub error: Color,
    /// Border color
    pub border: Color,
    /// Playing indicator
    pub playing: Color,
    /// Paused indicator
    pub paused: Color,
}

impl Theme {
    /// Creates a new theme
    pub fn new(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Dark => Self::dark(),
            ThemeType::Light => Self::light(),
            ThemeType::HighContrast => Self::high_contrast(),
            ThemeType::SolarizedDark => Self::solarized_dark(),
            ThemeType::SolarizedLight => Self::solarized_light(),
            ThemeType::Dracula => Self::dracula(),
            ThemeType::Nord => Self::nord(),
            ThemeType::Monokai => Self::monokai(),
        }
    }

    /// Dark theme (default)
    fn dark() -> Self {
        Self {
            theme_type: ThemeType::Dark,
            text: Color::White,
            text_secondary: Color::Gray,
            background: Color::Black,
            highlight: Color::Yellow,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            border: Color::Gray,
            playing: Color::Green,
            paused: Color::Red,
        }
    }

    /// Light theme
    fn light() -> Self {
        Self {
            theme_type: ThemeType::Light,
            text: Color::Black,
            text_secondary: Color::DarkGray,
            background: Color::White,
            highlight: Color::Blue,
            accent: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            border: Color::DarkGray,
            playing: Color::Green,
            paused: Color::Red,
        }
    }

    /// High contrast theme
    fn high_contrast() -> Self {
        Self {
            theme_type: ThemeType::HighContrast,
            text: Color::White,
            text_secondary: Color::LightYellow,
            background: Color::Black,
            highlight: Color::LightYellow,
            accent: Color::LightCyan,
            success: Color::LightGreen,
            warning: Color::LightYellow,
            error: Color::LightRed,
            border: Color::White,
            playing: Color::LightGreen,
            paused: Color::LightRed,
        }
    }

    /// Solarized Dark theme
    fn solarized_dark() -> Self {
        Self {
            theme_type: ThemeType::SolarizedDark,
            text: Color::Rgb(131, 148, 150),          // base0
            text_secondary: Color::Rgb(88, 110, 117), // base01
            background: Color::Rgb(0, 43, 54),        // base03
            highlight: Color::Rgb(181, 137, 0),       // yellow
            accent: Color::Rgb(38, 139, 210),         // blue
            success: Color::Rgb(133, 153, 0),         // green
            warning: Color::Rgb(181, 137, 0),         // yellow
            error: Color::Rgb(220, 50, 47),           // red
            border: Color::Rgb(7, 54, 66),            // base02
            playing: Color::Rgb(133, 153, 0),         // green
            paused: Color::Rgb(220, 50, 47),          // red
        }
    }

    /// Solarized Light theme
    fn solarized_light() -> Self {
        Self {
            theme_type: ThemeType::SolarizedLight,
            text: Color::Rgb(101, 123, 131),           // base00
            text_secondary: Color::Rgb(147, 161, 161), // base1
            background: Color::Rgb(253, 246, 227),     // base3
            highlight: Color::Rgb(181, 137, 0),        // yellow
            accent: Color::Rgb(38, 139, 210),          // blue
            success: Color::Rgb(133, 153, 0),          // green
            warning: Color::Rgb(181, 137, 0),          // yellow
            error: Color::Rgb(220, 50, 47),            // red
            border: Color::Rgb(238, 232, 213),         // base2
            playing: Color::Rgb(133, 153, 0),          // green
            paused: Color::Rgb(220, 50, 47),           // red
        }
    }

    /// Dracula theme
    fn dracula() -> Self {
        Self {
            theme_type: ThemeType::Dracula,
            text: Color::Rgb(248, 248, 242),          // foreground
            text_secondary: Color::Rgb(98, 114, 164), // comment
            background: Color::Rgb(40, 42, 54),       // background
            highlight: Color::Rgb(255, 121, 198),     // pink
            accent: Color::Rgb(189, 147, 249),        // purple
            success: Color::Rgb(80, 250, 123),        // green
            warning: Color::Rgb(241, 250, 140),       // yellow
            error: Color::Rgb(255, 85, 85),           // red
            border: Color::Rgb(68, 71, 90),           // current line
            playing: Color::Rgb(80, 250, 123),        // green
            paused: Color::Rgb(255, 85, 85),          // red
        }
    }

    /// Nord theme
    fn nord() -> Self {
        Self {
            theme_type: ThemeType::Nord,
            text: Color::Rgb(216, 222, 233),           // nord4
            text_secondary: Color::Rgb(143, 157, 179), // nord3
            background: Color::Rgb(46, 52, 64),        // nord0
            highlight: Color::Rgb(136, 192, 208),      // nord8
            accent: Color::Rgb(129, 161, 193),         // nord9
            success: Color::Rgb(163, 190, 140),        // nord14
            warning: Color::Rgb(235, 203, 139),        // nord13
            error: Color::Rgb(191, 97, 106),           // nord11
            border: Color::Rgb(59, 66, 82),            // nord1
            playing: Color::Rgb(163, 190, 140),        // nord14
            paused: Color::Rgb(191, 97, 106),          // nord11
        }
    }

    /// Monokai theme
    fn monokai() -> Self {
        Self {
            theme_type: ThemeType::Monokai,
            text: Color::Rgb(248, 248, 240),          // foreground
            text_secondary: Color::Rgb(117, 113, 94), // comment
            background: Color::Rgb(39, 40, 34),       // background
            highlight: Color::Rgb(249, 38, 114),      // pink
            accent: Color::Rgb(102, 217, 239),        // cyan
            success: Color::Rgb(166, 226, 46),        // green
            warning: Color::Rgb(230, 219, 116),       // yellow
            error: Color::Rgb(249, 38, 114),          // pink
            border: Color::Rgb(73, 72, 62),           // selection
            playing: Color::Rgb(166, 226, 46),        // green
            paused: Color::Rgb(249, 38, 114),         // pink
        }
    }

    /// Returns base text style
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    /// Returns secondary text style
    pub fn text_secondary_style(&self) -> Style {
        Style::default().fg(self.text_secondary)
    }

    /// Returns highlighted style
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns accent style
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Returns success style
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Returns warning style
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Returns error style
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Returns border color
    pub fn border_color(&self) -> Color {
        self.border
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::new(ThemeType::Dark);
        assert_eq!(theme.theme_type, ThemeType::Dark);
    }

    #[test]
    fn test_all_themes() {
        for theme_type in ThemeType::all() {
            let theme = Theme::new(theme_type);
            assert_eq!(theme.theme_type, theme_type);
        }
    }

    #[test]
    fn test_theme_names() {
        assert_eq!(ThemeType::Dark.name(), "Dark");
        assert_eq!(ThemeType::Dracula.name(), "Dracula");
    }

    #[test]
    fn test_theme_styles() {
        let theme = Theme::default();
        let _ = theme.text_style();
        let _ = theme.highlight_style();
        let _ = theme.border_color();
    }
}
