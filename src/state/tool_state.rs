use ratatui::Frame;
use ratatui::layout::Rect;
use crate::ui::widgets::tools::*;

#[derive(Debug)]
pub enum ToolState {
    Home,
    DiffChecker,
    TokenGenerator,
}

impl ToolState {
    pub fn title(&self) -> &'static str {
        match self {
            ToolState::Home => "Home",
            ToolState::DiffChecker => "PR Diff Checker",
            ToolState::TokenGenerator => "M2M Auth0 Token Generator",
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match self {
            ToolState::Home => home::render(frame, area),
            ToolState::DiffChecker => diff_checker::render(frame,area),
            ToolState::TokenGenerator => token_generator::render(frame, area)
        }
    }

    pub fn menu_entry(&self) -> &'static str {
        match self {
            ToolState::Home => "Home",
            ToolState::DiffChecker => "Diff Checker",
            ToolState::TokenGenerator => "Token Generator",
        }
    }
}