use crate::config::Config;
use crate::events::sender::EventSender;
use crate::state::service_status::ServiceStatus;
use crate::state::token_generator::TokenGenerator;
pub(crate) use crate::state::tools::Tool;
use crate::state::tools::ToolList;
use ratatui::widgets::ListState;
use crate::state::jira::Jira;

#[derive(Debug, Copy, Clone)]
pub enum AppFocus {
    List,
    Tool,
}

#[derive(Debug)]
pub struct AppState {
    pub tool_list: ToolList,
    pub current_tool: Tool,
    pub git_compare: ServiceStatus,
    pub token_generator: TokenGenerator,
    pub jira: Option<Jira>,
    pub focus: AppFocus,
}

impl AppState {
    pub(crate) fn new(config: Config, event_sender: EventSender) -> AppState {
        Self {
            tool_list: ToolList {
                items: {
                    let mut items: Vec<&'static str> = Vec::new();
                    items.push(Tool::Home.menu_entry());
                    items.push(Tool::ServiceStatus.menu_entry());
                    items.push(Tool::TokenGenerator.menu_entry());
                    if let Some(_) = config.jira {
                        items.push(Tool::Jira.menu_entry());
                    }

                    items
                },
                list_state: ListState::default().with_selected(Some(0)),
            },
            current_tool: Tool::Home,
            git_compare: ServiceStatus::new(config.servicestatus, event_sender.clone()),
            token_generator: TokenGenerator::new(config.tokengenerator, event_sender.clone()),
            jira: config.jira.map(Jira::new),
            focus: AppFocus::List,
        }
    }
}
