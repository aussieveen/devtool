use crate::config::Config;
use crate::state::jira::Jira;
use crate::state::service_status::ServiceStatus;
use crate::state::token_generator::TokenGenerator;
use crate::ui::widgets::tools::*;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ToolList {
    pub items: Vec<Tool>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub enum Tool {
    Home,
    ServiceStatus,
    TokenGenerator,
    Jira,
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
        config: &Config,
        service_status_state: &mut ServiceStatus,
        token_generator_state: &mut TokenGenerator,
        jira_state: &mut Jira,
    ) {
        match self {
            Tool::Home => home::render(frame, area),
            Tool::ServiceStatus => {
                service_status::render(frame, area, service_status_state, &config.servicestatus)
            }
            Tool::TokenGenerator => token_generator::render(
                frame,
                area,
                token_generator_state,
                &config.tokengenerator.services,
            ),
            Tool::Jira => jira::render(frame, area, jira_state),
        }
    }

    pub fn menu_entry(&self) -> &'static str {
        match self {
            Tool::Home => "Home",
            Tool::ServiceStatus => "Service Status",
            Tool::TokenGenerator => "Token Generator",
            Tool::Jira => "Jira",
        }
    }
}
