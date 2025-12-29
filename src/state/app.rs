use ratatui::widgets::ListState;
use crate::config::Config;
use crate::events::sender::EventSender;
use crate::state::diff_checker::DiffChecker;
use crate::state::token_generator::TokenGenerator;
pub(crate) use crate::state::tools::Tool;
use crate::state::tools::ToolList;

#[derive(Debug, Copy)]
#[derive(Clone)]
pub enum AppFocus {
    List,
    Tool
}

#[derive(Debug)]
pub struct AppState {
    pub tool_list: ToolList,
    pub current_tool: Tool,
    pub diff_checker: DiffChecker,
    pub token_generator: TokenGenerator,
    pub focus: AppFocus
}

impl AppState {
    pub(crate) fn new(config: Config, event_sender: EventSender) -> AppState {
        Self {
            tool_list: ToolList {
                items: vec![
                    Tool::Home.menu_entry(),
                    Tool::DiffChecker.menu_entry(),
                    Tool::TokenGenerator.menu_entry()
                ],
                list_state: ListState::default().with_selected(Some(0)),
            },
            current_tool: Tool::Home,
            diff_checker: DiffChecker::new(config.diffchecker, event_sender.clone()),
            token_generator: TokenGenerator::new(config.tokengenerator, event_sender.clone()),
            focus: AppFocus::List
        }
    }
}