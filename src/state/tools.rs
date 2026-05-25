use ratatui::widgets::ListState;

pub struct ToolList {
    pub items: Vec<Tool>,
    pub list_state: ListState,
}

#[derive(Clone, PartialEq, Copy, Eq, Hash, Debug)]
pub enum Tool {
    ServiceStatus,
    TokenGenerator,
    Jira,
}
