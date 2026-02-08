use crate::config::Config;
use crate::state::jira::Jira;
use crate::state::service_status::ServiceStatus;
use crate::state::token_generator::TokenGenerator;
pub(crate) use crate::state::tools::Tool;
use crate::state::tools::ToolList;
use ratatui::widgets::ListState;

#[derive(Copy, Clone)]
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
