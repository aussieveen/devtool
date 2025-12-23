use ratatui::widgets::ListState;
pub(crate) use crate::state::focus::Focus;
use crate::state::tool_list::ToolList;
pub(crate) use crate::state::tool_state::ToolState;

#[derive(Debug)]
pub struct AppState {
    pub list: ToolList,
    pub tool: ToolState,
    pub focus: Focus
}

impl AppState {
    pub(crate) fn default() -> AppState {
        Self {
            list: ToolList {
                items: vec![
                    ToolState::Home.menu_entry(),
                    ToolState::DiffChecker.menu_entry(),
                    ToolState::TokenGenerator.menu_entry()
                ],
                state: ListState::default().with_selected(Some(0)),
            },
            tool: ToolState::Home,
            focus: Focus::List
        }
    }
}