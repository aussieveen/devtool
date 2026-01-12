use crate::state::service_status::ServiceStatus;
use crate::state::token_generator::TokenGenerator;
use crate::ui::widgets::tools::*;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ToolList {
    pub items: Vec<&'static str>,
    pub list_state: ListState,
}

#[derive(Debug)]
pub enum Tool {
    Home,
    ServiceStatus,
    TokenGenerator,
    Jira
}

impl Tool {
    pub fn title(&self) -> &'static str {
        match self {
            Tool::Home => "Dev Tool",
            Tool::ServiceStatus => "Service Status",
            Tool::TokenGenerator => "M2M Auth0 Token Generator",
            Tool::Jira => "My Jira Tickets",
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        git_compare_state: &mut ServiceStatus,
        token_generator_state: &mut TokenGenerator,
    ) {
        match self {
            Tool::Home => home::render(frame, area),
            Tool::ServiceStatus => service_status::render(frame, area, git_compare_state),
            Tool::TokenGenerator => token_generator::render(frame, area, token_generator_state),
            Tool::Jira => {}
        }
    }

    pub fn menu_entry(&self) -> &'static str {
        match self {
            Tool::Home => "Home",
            Tool::ServiceStatus => "Service Status",
            Tool::TokenGenerator => "Token Generator",
            Tool::Jira => "Jira"
        }
    }
}
