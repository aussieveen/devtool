use crate::config::model::Config;
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
}

impl AppState {
    pub(crate) fn new(config: &Config) -> AppState {
        Self {
            tool_list: ToolList {
                items: {
                    let mut items: Vec<Tool> = Vec::new();
                    items.push(Tool::Home);
                    items.push(Tool::ServiceStatus);
                    items.push(Tool::TokenGenerator);
                    if config.jira.is_some() {
                        items.push(Tool::Jira);
                    }

                    items
                },
                list_state: ListState::default().with_selected(Some(0)),
            },
            current_tool: Tool::Home,
            service_status: ServiceStatus::new(config.servicestatus.len()),
            token_generator: TokenGenerator::new(&config.tokengenerator.services),
            jira: Jira::new(),
            focus: AppFocus::List,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::model::{Auth0Config, Config, JiraConfig, TokenGenerator};
    use crate::state::app::AppState;

    #[test]
    fn new_adds_jira_item_when_jira_config_is_some() {
        let config = Config {
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
                email: "".to_string(),
                token: "".to_string(),
            }),
        };

        let app_state = AppState::new(&config);

        assert_eq!(app_state.tool_list.items.len(), 4);
    }

    #[test]
    fn new_skips_jira_item_when_jira_config_is_none() {
        let config = Config {
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
            jira: None,
        };

        let app_state = AppState::new(&config);

        assert_eq!(app_state.tool_list.items.len(), 3);
    }
}
