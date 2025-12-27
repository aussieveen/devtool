use ratatui::widgets::ListState;
use crate::config::Config;
use crate::events::sender::EventSender;
use crate::state::diffchecker::DiffChecker;
pub(crate) use crate::state::focus::AppFocus;
use crate::state::token_generator::TokenGenerator;
use crate::state::tool_list::ToolList;
pub(crate) use crate::state::tool_state::Tool;

#[derive(Debug)]
pub struct AppState {
    pub list: ToolList,
    pub current_tool: Tool,
    pub diffchecker: DiffChecker,
    pub tokengenerator: TokenGenerator,
    pub focus: AppFocus
}

impl AppState {
    pub(crate) fn default(config: Config, event_sender: EventSender) -> AppState {
        Self {
            list: ToolList {
                items: vec![
                    Tool::Home.menu_entry(),
                    Tool::DiffChecker.menu_entry(),
                    Tool::TokenGenerator.menu_entry()
                ],
                list_state: ListState::default().with_selected(Some(0)),
            },
            current_tool: Tool::Home,
            diffchecker: DiffChecker::new(config.diffchecker, event_sender.clone()),
            tokengenerator: TokenGenerator::new(config.tokengenerator, event_sender.clone()),
            focus: AppFocus::List
        }
    }
}