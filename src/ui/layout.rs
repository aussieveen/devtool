// ui/layout.rs
use crate::state::app::AppFocus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Areas {
    pub tools_list: Rect,
    pub config_list: Rect,
    pub logs_list: Rect,
    pub content: Rect,
    pub footer: Rect,
}

pub fn main(area: Rect, focus: AppFocus) -> Areas {
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(outer_chunks[0]);

    // Collapsed height: just border + title line (3 rows).
    // Active panel gets Min(0) to fill remaining space.
    const COLLAPSED: u16 = 3;

    let (tools_c, config_c, logs_c) = match focus {
        AppFocus::List => (
            Constraint::Min(0),
            Constraint::Length(COLLAPSED),
            Constraint::Length(COLLAPSED),
        ),
        AppFocus::Config => (
            Constraint::Length(COLLAPSED),
            Constraint::Min(0),
            Constraint::Length(COLLAPSED),
        ),
        AppFocus::Logs => (
            Constraint::Length(COLLAPSED),
            Constraint::Length(COLLAPSED),
            Constraint::Min(0),
        ),
        // ToolConfig / Tool / JiraInput: content area owns the space, all panels collapse
        AppFocus::ToolConfig(_) | AppFocus::Tool | AppFocus::JiraInput => (
            Constraint::Length(COLLAPSED),
            Constraint::Length(COLLAPSED),
            Constraint::Length(COLLAPSED),
        ),
    };

    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([tools_c, config_c, logs_c])
        .split(chunks[0]);

    Areas {
        tools_list: sidebar[0],
        config_list: sidebar[1],
        logs_list: sidebar[2],
        content: chunks[1],
        footer: outer_chunks[1],
    }
}
