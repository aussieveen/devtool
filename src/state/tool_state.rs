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

    pub fn content(&self) -> &'static str {
        match self {
            ToolState::Home => "This is the home page",
            ToolState::DiffChecker => "This is the PR Diff Checker",
            ToolState::TokenGenerator => "This is the token generator",
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