use ratatui::widgets::ListState;
use crate::config::Config;
use crate::state::diffchecker::DiffChecker;
pub(crate) use crate::state::focus::Focus;
use crate::state::tool_list::ToolList;
pub(crate) use crate::state::tool_state::Tool;

#[derive(Debug)]
pub struct AppState {
    pub list: ToolList,
    pub tool: Tool,
    pub diff_checker: DiffChecker,
    pub focus: Focus
}

impl AppState {
    pub(crate) fn default(config: Config) -> AppState {
        Self {
            list: ToolList {
                items: vec![
                    Tool::Home.menu_entry(),
                    Tool::DiffChecker.menu_entry(),
                    Tool::TokenGenerator.menu_entry()
                ],
                state: ListState::default().with_selected(Some(0)),
            },
            tool: Tool::Home,
            diff_checker: DiffChecker::new(config.diffchecker),
            focus: Focus::List
        }
    }
}