use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ToolList {
    pub items: Vec<&'static str>,
    pub list_state: ListState
}