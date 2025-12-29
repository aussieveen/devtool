use crate::state::git_compare::GitCompare;
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
    GitCompare,
    TokenGenerator,
}

impl Tool {
    pub fn title(&self) -> &'static str {
        match self {
            Tool::Home => "Dev Tool",
            Tool::GitCompare => "Git Compare",
            Tool::TokenGenerator => "M2M Auth0 Token Generator",
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        git_compare_state: &mut GitCompare,
        token_generator_state: &mut TokenGenerator,
    ) {
        match self {
            Tool::Home => home::render(frame, area),
            Tool::GitCompare => git_compare::render(frame, area, git_compare_state),
            Tool::TokenGenerator => token_generator::render(frame, area, token_generator_state),
        }
    }

    pub fn menu_entry(&self) -> &'static str {
        match self {
            Tool::Home => "Home",
            Tool::GitCompare => "Git Compare",
            Tool::TokenGenerator => "Token Generator",
        }
    }
}
