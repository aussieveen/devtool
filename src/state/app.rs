use crate::config::model::Config;
use crate::error::model::Error;
use crate::state::jira::Jira;
use crate::state::service_status::ServiceStatus;
use crate::state::token_generator::TokenGenerator;
pub(crate) use crate::state::tools::Tool;
use crate::state::tools::ToolList;
use ratatui::widgets::ListState;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AppFocus {
    List,
    Tool,
    PopUp,
}

pub struct AppState {
    pub tool_list: ToolList,
    pub current_tool: Tool,
    pub service_status: ServiceStatus,
    pub token_generator: TokenGenerator,
    pub jira: Jira,
    pub focus: AppFocus,
    pub error: Option<Error>,
}

impl AppState {
    pub(crate) fn new(config: &Config) -> AppState {
        Self::build(config, Jira::new())
    }

    fn build(config: &Config, jira: Jira) -> AppState {
        Self {
            tool_list: ToolList {
                items: {
                    let mut items = vec![Tool::ServiceStatus, Tool::TokenGenerator];
                    if config.jira.is_some() { items.push(Tool::Jira); }
                    items
                },
                list_state: ListState::default().with_selected(Some(0)),
            },
            current_tool: Tool::ServiceStatus,
            service_status: ServiceStatus::new(config.servicestatus.len()),
            token_generator: TokenGenerator::new(&config.tokengenerator.services),
            jira,
            focus: AppFocus::List,
            error: None,
        }
    }

    pub fn effective_focus(&self) -> AppFocus {
        if self.error.is_some() {
            AppFocus::PopUp
        } else {
            self.focus
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::AppFocus;
    use crate::config::model::{Auth0Config, Config, JiraConfig, TokenGenerator};
    use crate::error::model::Error;
    use crate::persistence::persister::JiraFile;
    use crate::state::app::AppState;
    use crate::state::jira::Jira;
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
    fn focus_is_popup_when_error_set() {
        let mut app_state = AppState::build(&test_config(), test_jira());
        app_state.error = Some(Error {
            title: "".to_string(),
            originating_event: "".to_string(),
            tool: "".to_string(),
            description: "".to_string(),
        });

        assert_eq!(app_state.effective_focus(), AppFocus::PopUp);
    }

    #[test]
    fn focus_is_app_state_focus_when_error_set() {
        let app_state = AppState::build(&test_config(), test_jira());

        assert_eq!(app_state.effective_focus(), AppFocus::List);
    }
}
