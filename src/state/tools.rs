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

impl Tool {
    pub fn title(&self) -> &'static str {
        match self {
            Tool::ServiceStatus => "Service Status",
            Tool::TokenGenerator => "M2M Auth0 Token Generator",
            Tool::Jira => "My Jira Tickets",
        }
    }

    pub fn menu_entry(&self) -> &'static str {
        match self {
            Tool::ServiceStatus => "Service Status",
            Tool::TokenGenerator => "Token Generator",
            Tool::Jira => "Jira",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::Tool;
    use test_case::test_case;

    #[test_case(Tool::ServiceStatus, "Service Status")]
    #[test_case(Tool::TokenGenerator, "M2M Auth0 Token Generator")]
    #[test_case(Tool::Jira, "My Jira Tickets")]
    fn title_returns_expected(tool: Tool, expected: &str) {
        assert_eq!(tool.title(), expected);
    }

    #[test_case(Tool::ServiceStatus, "Service Status")]
    #[test_case(Tool::TokenGenerator, "Token Generator")]
    #[test_case(Tool::Jira, "Jira")]
    fn menu_entry_returns_expected(tool: Tool, expected: &str) {
        assert_eq!(tool.menu_entry(), expected);
    }
}
