use ratatui::Frame;
use ratatui::layout::Rect;
use crate::state::diffchecker::DiffChecker;
use crate::state::token_generator::TokenGenerator;
use crate::ui::widgets::tools::*;

#[derive(Debug)]
pub enum Tool {
    Home,
    DiffChecker,
    TokenGenerator,
}

impl Tool {
    pub fn title(&self) -> &'static str {
        match self {
            Tool::Home => "Home",
            Tool::DiffChecker => "PR Diff Checker",
            Tool::TokenGenerator => "M2M Auth0 Token Generator",
        }
    }

    pub fn render(&self,
                  frame: &mut Frame,
                  area: Rect,
                  diffchecker_state: &mut DiffChecker,
                  token_generator: &mut TokenGenerator
    ) {
        match self {
            Tool::Home => home::render(frame, area),
            Tool::DiffChecker => diffchecker::render(frame, area, diffchecker_state),
            Tool::TokenGenerator => token_generator::render(frame, area, token_generator),
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