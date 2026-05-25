use crate::config::model::{Config, Features};
use crate::state::app::Tool;
use crate::tools::plugin::Plugin;
use ratatui::widgets::ListState;

pub struct ConfigEditorItem {
    pub tool: Tool,
    pub enabled: bool,
}

pub struct ConfigEditor {
    pub items: Vec<ConfigEditorItem>,
    pub list_state: ListState,
}

impl ConfigEditor {
    pub fn new(plugins: &[Box<dyn Plugin>], features: &Features) -> Self {
        let items = plugins
            .iter()
            .map(|p| ConfigEditorItem {
                tool: p.id(),
                enabled: p.is_enabled(features),
            })
            .collect();
        Self {
            items,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    /// Toggle the currently selected item. Returns the updated (Tool, enabled) pair,
    /// or None if nothing is selected.
    pub fn toggle_selected(&mut self) -> Option<(Tool, bool)> {
        let idx = self.list_state.selected()?;
        let item = self.items.get(idx)?;
        let currently_enabled = item.enabled;
        let tool = item.tool;

        let item = self.items.get_mut(idx)?;
        item.enabled = !currently_enabled;
        Some((tool, item.enabled))
    }

    /// Returns the list of tools that should appear in the tool list: enabled by
    /// the user AND with a minimum viable config.
    pub fn enabled_tools(&self, plugins: &[Box<dyn Plugin>], config: &Config) -> Vec<Tool> {
        self.items
            .iter()
            .filter(|i| i.enabled)
            .filter(|i| {
                plugins
                    .iter()
                    .find(|p| p.id() == i.tool)
                    .is_some_and(|p| p.has_min_config(config))
            })
            .map(|i| i.tool)
            .collect()
    }

    /// Sync the enabled state of each item from a `Features` value.
    pub fn sync_from_features(&mut self, plugins: &[Box<dyn Plugin>], features: &Features) {
        for item in &mut self.items {
            if let Some(p) = plugins.iter().find(|p| p.id() == item.tool) {
                item.enabled = p.is_enabled(features);
            }
        }
    }

    /// Build a `Features` value from the current item state.
    pub fn to_features(&self, plugins: &[Box<dyn Plugin>]) -> Features {
        let mut features = Features::default();
        for item in &self.items {
            if let Some(p) = plugins.iter().find(|p| p.id() == item.tool) {
                p.apply_feature_flag(&mut features, item.enabled);
            }
        }
        features
    }
}
