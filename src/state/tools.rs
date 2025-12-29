use crate::state::diff_checker::DiffChecker;
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
    DiffChecker,
    TokenGenerator,
}

impl Tool {
    pub fn title(&self) -> &'static str {
        match self {
            Tool::Home => "Dev Tool",
            Tool::DiffChecker => "PR Diff Checker",
            Tool::TokenGenerator => "M2M Auth0 Token Generator",
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        diff_checker_state: &mut DiffChecker,
        token_generator_state: &mut TokenGenerator,
    ) {
        match self {
            Tool::Home => home::render(frame, area),
            Tool::DiffChecker => diff_checker::render(frame, area, diff_checker_state),
            Tool::TokenGenerator => token_generator::render(frame, area, token_generator_state),
        }
    }

    pub fn menu_entry(&self) -> &'static str {
        match self {
            Tool::Home => "Home",
            Tool::DiffChecker => "Diff Checker",
            Tool::TokenGenerator => "Token Generator",
        }
    }
}
