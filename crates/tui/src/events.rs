// crates/tui/src/events.rs
//! Event handling for TUI

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::time::Duration;

/// Application events
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Tick event for updates
    Tick,
    /// Quit application
    Quit,
    /// Resize event
    Resize(u16, u16),
}

/// Event handler
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    /// Creates a new event handler
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    /// Polls for the next event
    pub fn next(&self) -> crate::error::TuiResult<AppEvent> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => {
                    // Check for quit keys
                    if key.code == KeyCode::Char('q')
                        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                    {
                        Ok(AppEvent::Quit)
                    } else {
                        Ok(AppEvent::Key(key))
                    }
                }
                CrosstermEvent::Mouse(mouse) => Ok(AppEvent::Mouse(mouse)),
                CrosstermEvent::Resize(w, h) => Ok(AppEvent::Resize(w, h)),
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}

/// Helper to check if mouse is in area
pub fn mouse_in_area(mouse_x: u16, mouse_y: u16, area: ratatui::layout::Rect) -> bool {
    mouse_x >= area.x
        && mouse_x < area.x + area.width
        && mouse_y >= area.y
        && mouse_y < area.y + area.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new(Duration::from_millis(100));
        assert_eq!(handler.tick_rate, Duration::from_millis(100));
    }

    #[test]
    fn test_event_handler_default() {
        let handler = EventHandler::default();
        assert_eq!(handler.tick_rate, Duration::from_millis(250));
    }

    #[test]
    fn test_app_event_equality() {
        let event1 = AppEvent::Tick;
        let event2 = AppEvent::Tick;
        assert_eq!(event1, event2);
    }

    #[test]
    fn test_app_event_quit() {
        let event = AppEvent::Quit;
        assert_eq!(event, AppEvent::Quit);
    }

    #[test]
    fn test_mouse_in_area() {
        use ratatui::layout::Rect;

        let area = Rect::new(10, 10, 20, 20);

        assert!(mouse_in_area(15, 15, area));
        assert!(mouse_in_area(10, 10, area));
        assert!(mouse_in_area(29, 29, area));
        assert!(!mouse_in_area(5, 15, area));
        assert!(!mouse_in_area(15, 5, area));
        assert!(!mouse_in_area(30, 15, area));
    }
}