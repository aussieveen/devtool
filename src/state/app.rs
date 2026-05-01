use crate::config::model::Config;
use crate::popup::model::Popup;
use crate::state::config_editor::ConfigEditor;
use crate::state::jira::Jira;
use crate::state::jira_config::JiraConfigEditor;
use crate::state::log::LogState;
use crate::state::service_status::ServiceStatus;
use crate::state::service_status_config::ServiceStatusConfigEditor;
use crate::state::token_generator::TokenGenerator;
use crate::state::token_generator_config::TokenGeneratorConfigEditor;
pub(crate) use crate::state::tools::Tool;
use crate::state::tools::ToolList;
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
    pub service_status: ServiceStatus,
    pub token_generator: TokenGenerator,
    pub jira: Jira,
    pub focus: AppFocus,
    pub popup: Option<Popup>,
    pub config_editor: ConfigEditor,
    pub service_status_config_editor: ServiceStatusConfigEditor,
    pub token_generator_config_editor: TokenGeneratorConfigEditor,
    pub jira_config_editor: JiraConfigEditor,
    pub log: LogState,
}

impl AppState {
    pub(crate) fn new(config: &Config) -> AppState {
        Self::build(config, Jira::new())
    }

    pub(crate) fn build(config: &Config, jira: Jira) -> AppState {
        let has_jira_config = config.jira.is_some();
        let config_editor = ConfigEditor::new(&config.features);
        let tool_list_items = config_editor.enabled_tools(has_jira_config);
        Self {
            tool_list: ToolList {
                items: tool_list_items,
                list_state: ListState::default().with_selected(Some(0)),
            },
            current_tool: Tool::ServiceStatus,
            service_status: ServiceStatus::new(config.servicestatus.len()),
            token_generator: TokenGenerator::new(&config.tokengenerator.services),
            jira,
            focus: AppFocus::List,
            popup: None,
            config_editor,
            service_status_config_editor: ServiceStatusConfigEditor::new(),
            token_generator_config_editor: TokenGeneratorConfigEditor::new(),
            jira_config_editor: JiraConfigEditor::new(),
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
    pub fn rebuild_tool_list(&mut self, has_jira_config: bool) {
        let new_items = self.config_editor.enabled_tools(has_jira_config);
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
    use crate::config::model::{Auth0Config, Config, JiraConfig, TokenGenerator};
    use crate::persistence::persister::JiraFile;
    use crate::popup::model::Popup;
    use crate::state::app::{AppState, Tool};
    use crate::state::jira::Jira;
    use crate::ui::widgets::popup::Type;
    use tempfile::TempDir;

    fn test_jira() -> Jira {
        let dir = TempDir::new().unwrap();
        Jira::new_empty(JiraFile::new_from_path(dir.path().join("test.yaml")))
    }

    fn test_config() -> Config {
        Config {
            servicestatus: vec![],
            tokengenerator: TokenGenerator {
                auth0: Auth0Config {
                    local: "".to_string(),
                    staging: "".to_string(),
                    preproduction: "".to_string(),
                    production: "".to_string(),
                },
                services: vec![],
            },
            jira: Some(JiraConfig {
                url: "".to_string(),
                email: "".to_string(),
                token: "".to_string(),
            }),
            features: crate::config::model::Features::default(),
        }
    }

    #[test]
    fn new_adds_jira_item_when_jira_config_is_some() {
        let app_state = AppState::build(&test_config(), test_jira());

        assert_eq!(app_state.tool_list.items.len(), 3);
    }

    #[test]
    fn new_skips_jira_item_when_jira_config_is_none() {
        let mut config = test_config();
        config.jira = None;

        let app_state = AppState::build(&config, test_jira());

        assert_eq!(app_state.tool_list.items.len(), 2);
    }

    #[test]
    fn focus_is_jira_input_when_error_set() {
        let mut app_state = AppState::build(&test_config(), test_jira());
        app_state.popup = Some(Popup {
            popup_type: Type::Error,
            title: "".to_string(),
            parts: vec![],
            actions: vec![],
        });

        assert_eq!(app_state.effective_focus(), AppFocus::JiraInput);
    }

    #[test]
    fn focus_is_app_state_focus_when_error_set() {
        let app_state = AppState::build(&test_config(), test_jira());

        assert_eq!(app_state.effective_focus(), AppFocus::List);
    }

    #[test]
    fn rebuild_tool_list_stays_on_current_tool_when_still_enabled() {
        let mut state = AppState::build(&test_config(), test_jira());
        // Start on TokenGenerator (index 1)
        state.tool_list.list_state.select(Some(1));
        state.current_tool = Tool::TokenGenerator;
        // Disable Jira; TokenGenerator stays enabled
        state.config_editor.items[2].enabled = false;

        state.rebuild_tool_list(true);

        // TokenGenerator should still be selected
        assert_eq!(state.current_tool, Tool::TokenGenerator);
        assert_eq!(state.tool_list.list_state.selected(), Some(1));
    }

    #[test]
    fn rebuild_tool_list_moves_up_when_current_tool_disabled() {
        let mut state = AppState::build(&test_config(), test_jira());
        // Start on Jira (index 2)
        state.tool_list.list_state.select(Some(2));
        state.current_tool = Tool::Jira;
        // Disable Jira
        state.config_editor.items[2].enabled = false;

        state.rebuild_tool_list(true);

        // Should move up to index 1 (TokenGenerator)
        assert_eq!(state.tool_list.list_state.selected(), Some(1));
        assert_eq!(state.current_tool, Tool::TokenGenerator);
    }

    #[test]
    fn rebuild_tool_list_selects_first_when_list_was_empty() {
        let mut state = AppState::build(&test_config(), test_jira());
        // Simulate all disabled
        state.tool_list.items = vec![];
        state.tool_list.list_state.select(None);
        // Re-enable everything
        state
            .config_editor
            .items
            .iter_mut()
            .for_each(|i| i.enabled = true);

        state.rebuild_tool_list(true);

        assert_eq!(state.tool_list.list_state.selected(), Some(0));
    }
}
