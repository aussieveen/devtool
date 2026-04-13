use crate::config::model::Features;
use crate::state::app::Tool;
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
    /// All three tools are always shown in the config list. Jira's enabled state
    /// is independent of whether a jira config section exists — the config flag and
    /// the config section are separate concerns.
    pub fn new(features: &Features) -> Self {
        let items = vec![
            ConfigEditorItem {
                tool: Tool::ServiceStatus,
                enabled: features.service_status,
            },
            ConfigEditorItem {
                tool: Tool::TokenGenerator,
                enabled: features.token_generator,
            },
            ConfigEditorItem {
                tool: Tool::Jira,
                enabled: features.jira,
            },
        ];
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

    /// Returns the list of tools that should appear in the tool list, respecting
    /// the has_jira_config constraint.
    pub fn enabled_tools(&self, has_jira_config: bool) -> Vec<Tool> {
        self.items
            .iter()
            .filter(|i| i.enabled)
            .filter(|i| i.tool != Tool::Jira || has_jira_config)
            .map(|i| i.tool)
            .collect()
    }

    /// Sync the enabled state of each item from a `Features` value.
    pub fn sync_from_features(&mut self, features: &Features) {
        for item in &mut self.items {
            item.enabled = match item.tool {
                Tool::ServiceStatus => features.service_status,
                Tool::TokenGenerator => features.token_generator,
                Tool::Jira => features.jira,
            };
        }
    }

    /// Build a `Features` value from the current item state.
    pub fn to_features(&self) -> Features {
        let service_status = self
            .items
            .iter()
            .find(|i| i.tool == Tool::ServiceStatus)
            .map(|i| i.enabled)
            .unwrap_or(true);
        let token_generator = self
            .items
            .iter()
            .find(|i| i.tool == Tool::TokenGenerator)
            .map(|i| i.enabled)
            .unwrap_or(true);
        let jira = self
            .items
            .iter()
            .find(|i| i.tool == Tool::Jira)
            .map(|i| i.enabled)
            .unwrap_or(true);
        Features {
            service_status,
            token_generator,
            jira,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::Features;

    fn all_enabled() -> Features {
        Features::default()
    }

    fn make_editor(features: Features) -> ConfigEditor {
        ConfigEditor::new(&features)
    }

    #[test]
    fn new_creates_items_from_features() {
        let editor = make_editor(all_enabled());
        assert_eq!(editor.items.len(), 3);
        assert!(editor.items.iter().all(|i| i.enabled));
    }

    #[test]
    fn toggle_disables_selected_tool() {
        let mut editor = make_editor(all_enabled());
        editor.list_state.select(Some(0));
        let result = editor.toggle_selected();
        assert!(result.is_some());
        assert!(!editor.items[0].enabled);
    }

    #[test]
    fn toggle_allows_disabling_last_enabled_tool() {
        let features = Features {
            service_status: true,
            token_generator: false,
            jira: false,
        };
        let mut editor = make_editor(features);
        editor.list_state.select(Some(0)); // ServiceStatus — the only enabled tool
        let result = editor.toggle_selected();
        assert!(result.is_some());
        assert!(!editor.items[0].enabled); // now all disabled
    }

    #[test]
    fn toggle_re_enables_tool() {
        let features = Features {
            service_status: false,
            token_generator: true,
            jira: true,
        };
        let mut editor = make_editor(features);
        editor.list_state.select(Some(0));
        let result = editor.toggle_selected();
        assert_eq!(result, Some((Tool::ServiceStatus, true)));
        assert!(editor.items[0].enabled);
    }

    #[test]
    fn enabled_tools_excludes_jira_when_no_jira_config() {
        let editor = make_editor(all_enabled());
        let tools = editor.enabled_tools(false);
        assert!(!tools.contains(&Tool::Jira));
        assert!(tools.contains(&Tool::ServiceStatus));
        assert!(tools.contains(&Tool::TokenGenerator));
    }

    #[test]
    fn enabled_tools_includes_jira_when_config_present() {
        let editor = make_editor(all_enabled());
        let tools = editor.enabled_tools(true);
        assert!(tools.contains(&Tool::Jira));
    }

    #[test]
    fn to_features_reflects_current_state() {
        let mut editor = make_editor(all_enabled());
        editor.items[2].enabled = false; // Jira
        let features = editor.to_features();
        assert!(features.service_status);
        assert!(features.token_generator);
        assert!(!features.jira);
    }
}
