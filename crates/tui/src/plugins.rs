// crates/tui/src/plugins.rs
//! Plugin system for custom views

use crate::{error::TuiResult, state::AppState};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{layout::Rect, Frame};
use std::collections::HashMap;

/// Plugin trait for custom views
pub trait Plugin: Send + Sync {
    /// Returns the plugin name
    fn name(&self) -> &str;

    /// Returns the plugin description
    fn description(&self) -> &str;

    /// Renders the plugin view
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState);

    /// Handles key events
    fn handle_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        state: &mut AppState,
    ) -> TuiResult<()>;

    /// Called when the view becomes active
    fn on_activate(&mut self, _state: &mut AppState) -> TuiResult<()> {
        Ok(())
    }

    /// Called when the view becomes inactive
    fn on_deactivate(&mut self, _state: &mut AppState) -> TuiResult<()> {
        Ok(())
    }
}

/// Plugin manager
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    active_plugin: Option<String>,
}

impl PluginManager {
    /// Creates a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            active_plugin: None,
        }
    }

    /// Registers a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    /// Gets a plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Gets a mutable plugin by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut (dyn Plugin + '_)> {
        Some(&mut **self.plugins.get_mut(name)?)
    }

    /// Activates a plugin
    pub fn activate(&mut self, name: &str, state: &mut AppState) -> TuiResult<()> {
        if let Some(current) = &self.active_plugin {
            if let Some(plugin) = self.plugins.get_mut(current) {
                plugin.on_deactivate(state)?;
            }
        }

        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.on_activate(state)?;
            self.active_plugin = Some(name.to_string());
        }

        Ok(())
    }

    /// Gets the active plugin
    pub fn active(&self) -> Option<&dyn Plugin> {
        self.active_plugin.as_ref().and_then(|name| self.get(name))
    }

    /// Gets the active plugin mutably
    pub fn active_mut(&mut self) -> Option<&mut dyn Plugin> {
        let name = self.active_plugin.clone();
        name.and_then(|n| self.get_mut(&n))
    }

    /// Lists all plugin names
    pub fn list(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Test plugin"
        }

        fn render(&self, _frame: &mut Frame, _area: Rect, _state: &AppState) {}

        fn handle_key(
            &mut self,
            _code: KeyCode,
            _modifiers: KeyModifiers,
            _state: &mut AppState,
        ) -> TuiResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin {
            name: "test".to_string(),
        });

        manager.register(plugin);
        assert_eq!(manager.list().len(), 1);
        assert!(manager.get("test").is_some());
    }

    #[test]
    fn test_plugin_activation() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin {
            name: "test".to_string(),
        });

        manager.register(plugin);

        let mut state = AppState::new();
        manager.activate("test", &mut state).unwrap();

        assert!(manager.active().is_some());
    }
}
