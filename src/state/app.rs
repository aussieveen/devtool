use crate::config::model::Config;
use crate::popup::model::Popup;
use crate::state::config_editor::ConfigEditor;
use crate::state::log::LogState;
pub(crate) use crate::state::tools::Tool;
use crate::state::tools::ToolList;
use crate::tools::plugin::Plugin;
use ratatui::widgets::ListState;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AppFocus {
    List,
    Tool,
    Config,
    ToolConfig(Tool),
    JiraInput,
    Logs,
}

pub struct AppState {
    pub tool_list: ToolList,
    pub current_tool: Tool,
    pub focus: AppFocus,
    pub popup: Option<Popup>,
    pub config_editor: ConfigEditor,
    pub log: LogState,
}

impl AppState {
    pub(crate) fn new(config: &Config, plugins: &[Box<dyn Plugin>]) -> AppState {
        let config_editor = ConfigEditor::new(plugins, &config.features);
        let tool_list_items = config_editor.enabled_tools(plugins, config);
        let current_tool = tool_list_items
            .first()
            .copied()
            .unwrap_or(Tool::ServiceStatus);
        Self {
            tool_list: ToolList {
                items: tool_list_items,
                list_state: ListState::default().with_selected(Some(0)),
            },
            current_tool,
            focus: AppFocus::List,
            popup: None,
            config_editor,
            log: LogState::new(),
        }
    }

    pub fn has_popup(&self) -> bool {
        self.popup.is_some()
    }

    pub fn effective_focus(&self) -> AppFocus {
        if self.has_popup() {
            AppFocus::JiraInput
        } else {
            self.focus
        }
    }

    /// Rebuild the tool list from the config editor state.
    /// - If the current tool is still enabled, stay on it.
    /// - If it was disabled, move selection up one.
    /// - If the list becomes empty, clear selection.
    /// - If the list was empty and now has items, select the first.
    pub fn rebuild_tool_list(&mut self, plugins: &[Box<dyn Plugin>], config: &Config) {
        let new_items = self.config_editor.enabled_tools(plugins, config);
        if new_items.is_empty() {
            self.tool_list.items = new_items;
            self.tool_list.list_state.select(None);
        } else {
            // Try to keep the currently active tool selected.
            let new_idx = if let Some(pos) = new_items.iter().position(|t| *t == self.current_tool)
            {
                pos
            } else {
                // Tool was disabled — move up one from the previous selection.
                let prev = self.tool_list.list_state.selected().unwrap_or(0);
                prev.saturating_sub(1).min(new_items.len() - 1)
            };
            self.tool_list.items = new_items;
            self.tool_list.list_state.select(Some(new_idx));
            if let Some(tool) = self.tool_list.items.get(new_idx) {
                self.current_tool = *tool;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::AppFocus;
    use crate::client::auth_zero::api::ImmediateAuthZeroApi;
    use crate::client::healthcheck::api::ImmediateHealthcheckApi;
    use crate::client::jira::api::ImmediateJiraApi;
    use crate::config::model::{
        Auth0Config, Config, Credentials, JiraConfig, ServiceConfig, ServiceStatusConfig,
        TokenGenerator,
    };
    use crate::environment::Environment;
    use crate::popup::model::Popup;
    use crate::state::app::{AppState, Tool};
    use crate::tools::plugin::{create_plugins, Plugin};
    use crate::ui::widgets::popup::Type;
    use std::sync::Arc;

    fn test_config() -> Config {
        Config {
            servicestatus: vec![ServiceStatusConfig {
                name: "svc".into(),
                staging: "".into(),
                preproduction: "".into(),
                production: "".into(),
                repo: "".into(),
            }],
            tokengenerator: TokenGenerator {
                auth0: Auth0Config::default(),
                services: vec![ServiceConfig {
                    name: "svc".into(),
                    audience: "".into(),
                    credentials: vec![Credentials {
                        env: Environment::Staging,
                        client_id: "".into(),
                        client_secret: "".into(),
                    }],
                }],
            },
            jira: Some(JiraConfig {
                url: "".to_string(),
                email: "".to_string(),
                token: "".to_string(),
            }),
            features: crate::config::model::Features::default(),
        }
    }

    fn test_plugins(config: &Config) -> Vec<Box<dyn Plugin>> {
        create_plugins(
            config,
            Arc::new(ImmediateAuthZeroApi::new()),
            Arc::new(ImmediateJiraApi::new()),
            Arc::new(ImmediateHealthcheckApi::new()),
        )
    }

    #[test]
    fn new_includes_jira_when_jira_config_present() {
        let config = test_config();
        let plugins = test_plugins(&config);
        let app_state = AppState::new(&config, &plugins);
        assert!(app_state.tool_list.items.contains(&Tool::Jira));
    }

    #[test]
    fn new_excludes_jira_when_jira_config_absent() {
        let mut config = test_config();
        config.jira = None;
        let plugins = test_plugins(&config);
        let app_state = AppState::new(&config, &plugins);
        assert!(!app_state.tool_list.items.contains(&Tool::Jira));
    }

    #[test]
    fn focus_is_jira_input_when_error_set() {
        let config = test_config();
        let plugins = test_plugins(&config);
        let mut app_state = AppState::new(&config, &plugins);
        app_state.popup = Some(Popup {
            popup_type: Type::Error,
            title: "".to_string(),
            parts: vec![],
            actions: vec![],
        });
        assert_eq!(app_state.effective_focus(), AppFocus::JiraInput);
    }

    #[test]
    fn focus_is_app_state_focus_when_no_popup() {
        let config = test_config();
        let plugins = test_plugins(&config);
        let app_state = AppState::new(&config, &plugins);
        assert_eq!(app_state.effective_focus(), AppFocus::List);
    }

    #[test]
    fn rebuild_tool_list_stays_on_current_tool_when_still_enabled() {
        let config = test_config();
        let plugins = test_plugins(&config);
        let mut state = AppState::new(&config, &plugins);
        // Move to TokenGenerator (index 1, since all three tools have min config)
        state.tool_list.list_state.select(Some(1));
        state.current_tool = Tool::TokenGenerator;
        // Disable Jira; TokenGenerator stays enabled
        if let Some(jira_item) = state
            .config_editor
            .items
            .iter_mut()
            .find(|i| i.tool == Tool::Jira)
        {
            jira_item.enabled = false;
        }

        state.rebuild_tool_list(&plugins, &config);

        assert_eq!(state.current_tool, Tool::TokenGenerator);
    }

    #[test]
    fn rebuild_tool_list_moves_up_when_current_tool_disabled() {
        let config = test_config();
        let plugins = test_plugins(&config);
        let mut state = AppState::new(&config, &plugins);
        // Find Jira's position and select it
        let jira_idx = state
            .tool_list
            .items
            .iter()
            .position(|t| *t == Tool::Jira)
            .expect("jira should be in list");
        state.tool_list.list_state.select(Some(jira_idx));
        state.current_tool = Tool::Jira;
        if let Some(jira_item) = state
            .config_editor
            .items
            .iter_mut()
            .find(|i| i.tool == Tool::Jira)
        {
            jira_item.enabled = false;
        }

        state.rebuild_tool_list(&plugins, &config);

        assert_ne!(state.current_tool, Tool::Jira);
    }

    #[test]
    fn rebuild_tool_list_selects_first_when_list_was_empty() {
        let config = test_config();
        let plugins = test_plugins(&config);
        let mut state = AppState::new(&config, &plugins);
        // Simulate all disabled
        state.tool_list.items = vec![];
        state.tool_list.list_state.select(None);
        // Re-enable everything
        state
            .config_editor
            .items
            .iter_mut()
            .for_each(|i| i.enabled = true);

        state.rebuild_tool_list(&plugins, &config);

        assert_eq!(state.tool_list.list_state.selected(), Some(0));
    }
}
